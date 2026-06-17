
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

        if crate::builtin::is_builtin(cmd) {
            match cmd.as_str() {
                "exit" => crate::builtin::exit(args),
                "echo" => crate::builtin::echo(args),
                "type" => crate::builtin::type_cmd(args),
                _ => unreachable!(),
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}