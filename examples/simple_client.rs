use cmd_client::CmdClient;

fn main() {
    let cmd = CmdClient::start("> ", (), |input, _args, cmd| {
        cmd.writeln(&format!("you wrote: {}", input))
    });

    cmd.writeln("Welcome!");
    cmd.writeln("Write something!");
    loop {
        // run forever
    }
}