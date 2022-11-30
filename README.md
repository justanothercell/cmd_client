# Rust Cmd Client
Simple user interface for rust command line based applications.

This interface  handles input on a different thread. 
This allows for simultaneous in- and output. 
It supports basic control for arrow/other special keys.

Tested on windows and should also work on linux. 
If not, feel free to contact me and I will fix it.

**ATTENTION:** This will most likely not work in your IDE terminal, use
the windows default cmd.

### Run [Example](./examples)
`cargo run --example simple_client`

### Use In Project
```toml
cmd_client = { it = "https://github.com/DragonFIghter603", version = "0.1.0"}
```