use std::io::Write;

pub fn repl() {
    let dispatch = crate::builtin::get_dispatch_table();

    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts = crate::helpers::tokenize(trimmed);

        let cmd = &parts[0];
        let args = &parts[1..];

        if let Some(func) = dispatch.get(cmd.as_str()) {
            func(args);
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if let Err(e) = crate::executor::execute_command(&path, cmd, args) {
                eprintln!("Error executing command: {}", e);
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}
