#![feature(fn_traits)]

use std::io::{stdout, Write};
use std::{thread};
use std::mem::swap;
use std::process::exit;
use std::sync::{Arc, Mutex, MutexGuard};
use getch::Getch;
use terminal_size::{Height, terminal_size, Width};

type ThreadCmdClient = Arc<Mutex<CmdClient>>;

pub struct CmdClient {
    prompt: String,
    input: String,
    cursor: usize
}

trait ClientSeal {
    fn seal(&self) -> MutexGuard<CmdClient>;
}

impl ClientSeal for ThreadCmdClient {
    fn seal(&self) -> MutexGuard<CmdClient> {
        self.lock().unwrap()
    }
}

#[derive(Clone)]
pub struct CmdClientHandle {
    client: ThreadCmdClient,
}

impl CmdClientHandle {
    pub fn writeln(&self, line: &str){
        self.client.seal().writeln(line)
    }

    fn prompt_input(&self) {
        let getch = Getch::new();
        {
            let mut c = self.client.seal();
            c.input = String::new();
            c.refresh_input();
        }
        let _ = stdout().flush();
        loop {
            if let Ok(c) = getch.getch() {
                let ch = c as char;
                let mut cl = self.client.seal();
                match c {
                    3 => exit(0), // ^C
                    8 => { // backspace
                        if cl.input.len() > 0 && cl.cursor > 0 {
                            let cursor = cl.cursor;
                            let _ = cl.input.remove(cursor - 1);
                            cl.cursor = usize::min(cl.cursor - 1, cl.input.len());
                        }
                    },
                    13 => break, // \n
                    224 => if let Ok(c) = getch.getch() { // control key (arrow keys, etc)
                        match c as char {
                        'H' => (), // arrow up
                        'P' => (), // arrow down
                        'K' if cl.cursor > 0 => cl.cursor -= 1, // arrow left
                        'M' if cl.cursor < cl.input.len() => cl.cursor += 1, // arrow right
                        'G' => cl.cursor = 0, // pos1
                        'O' => cl.cursor = cl.input.len(), // end
                        'S' if cl.input.len() > 0 && cl.cursor < cl.input.len() => { // delete
                            let cursor = cl.cursor;
                            let _ = cl.input.remove(cursor);
                        },
                        _ => () // unsupported
                    }}
                    _ if ch.is_ascii() => { // ascii printable character
                        let cursor = cl.cursor;
                        cl.input.insert(cursor, ch);
                        cl.cursor += 1;
                    },
                    _ => () // unsupported
                }
                cl.refresh_input();
            }
        }
    }
}

impl CmdClient {
    pub fn start<T: 'static+Send>(prompt: &str, handler_args: T, input_handler: fn(&str, &T, &CmdClientHandle)) -> CmdClientHandle{
        let client = CmdClientHandle {
            client: Arc::new(Mutex::new(Self {prompt: prompt.to_string(), input: String::new(), cursor: 0}))
        };
        let input_client = client.clone();
        thread::spawn(move ||{
            loop {
                input_client.prompt_input();
                let input = {
                    let mut c = input_client.client.seal();
                    c.cursor = 0;
                    let mut input = String::new();
                    swap(&mut input, &mut c.input);
                    input
                };
                input_handler.call((&input, &handler_args, &input_client));
            }
        });

        client
    }

    pub fn term_size() -> (u16, u16){
        if let Some((Width(w), Height(h))) = terminal_size() {
            (w, h)
        } else {
            panic!("\rUnable to get terminal size. Please use different terminal");
        }
    }

    fn refresh_input(&self){
        let (w, _) = Self::term_size();
        print!("\r{}", " ".repeat(w as usize));
        print!("\r{}{}", self.prompt, self.input);
        print!("\r{}{}", self.prompt, unsafe {self.input.as_str().get_unchecked(0..self.cursor)});
        let _ = stdout().flush();
    }

    pub(crate) fn writeln(&self, line: &str) {
        let (w, _) = Self::term_size();
        print!("\r{}", " ".repeat(w as usize));
        println!("\r{:1$}", line, w as usize);
        self.refresh_input();
    }
}