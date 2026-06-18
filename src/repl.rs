
use std::io::Write;
pub fn repl() {
    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<String> = trimmed.split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        let cmd = &parts[0];
        let args = &parts[1..];
        // builtin command logic
        if crate::builtin::is_builtin(cmd) {
            match cmd.as_str() {
                "exit" => crate::builtin::exit(args),
                "echo" => crate::builtin::echo(args),
                "type" => crate::builtin::type_cmd(args),
                _ => unreachable!(),
            }
        // find command, if found execute, else print error
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if let Err(e) = crate::executor::execute_command(&path, cmd, args) {
                eprintln!("Error executing command: {}", e);
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}