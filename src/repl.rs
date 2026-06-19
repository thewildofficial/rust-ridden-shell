use std::io::Write;

pub fn repl() {
    let dispatch: std::collections::HashMap<&str, crate::builtin::BuiltinFn> =
        crate::builtin::get_dispatch_table();

    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut input: String = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed: &str = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tokens: Vec<String> = crate::helpers::tokenize(trimmed);
        let (cmd_tokens, redirect_target): (Vec<String>, Option<String>) =
            crate::helpers::parse_redirections(&tokens);

        if cmd_tokens.is_empty() {
            continue;
        }

        let cmd: &String = &cmd_tokens[0];
        let args: &[String] = &cmd_tokens[1..];
        let redirect: Option<&str> = redirect_target.as_deref();

        if let Some(func) = dispatch.get(cmd.as_str()) {
            crate::executor::execute_builtin(*func, args, redirect);
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if let Err(e) = crate::executor::execute_external(&path, cmd, args, redirect) {
                eprintln!("Error executing command: {}", e);
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}
