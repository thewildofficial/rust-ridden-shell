use std::io::Write;

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current.push(c);
            }
        } else {
            match c {
                '\'' => in_single_quote = true,
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

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

        let parts = tokenize(trimmed);

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
