#![feature(fn_traits)]

use std::io::{stdout, Write};
use std::{thread};
use std::mem::swap;
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use getch::Getch;
use terminal_size::{Height, terminal_size, Width};

type ThreadCmdClient = Arc<Mutex<CmdClient>>;

pub struct CmdClient {
    prompt: String,
    input: String
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
                if c == 3 { // ^c
                    exit(0);
                }
                else if c == 8 { // delete
                    let mut c = self.client.seal();
                    c.input.pop();
                    c.refresh_input();
                }
                else if c == 13 { // \n
                    break
                }
                // is printable?
                else if ch.is_ascii() {
                    let mut c = self.client.seal();
                    c.input.push(ch);
                    c.refresh_input();
                }
            }
        }
    }
}

impl CmdClient {
    pub fn start<T: 'static+Send>(prompt: &str, handler_args: T, input_handler: fn(&str, &T, &CmdClientHandle)) -> CmdClientHandle{
        let client = CmdClientHandle {
            client: Arc::new(Mutex::new(Self {prompt: prompt.to_string(), input: String::new()}))
        };
        let input_client = client.clone();
        thread::spawn(move ||{
            loop {
                input_client.prompt_input();
                let input = {
                    let mut c = input_client.client.seal();
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
        let _ = stdout().flush();
    }

    pub(crate) fn writeln(&self, line: &str) {
        let (w, _) = Self::term_size();
        print!("\r{}", " ".repeat(w as usize));
        println!("\r{:1$}", line, w as usize);
        self.refresh_input();
    }
}