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
        let (cmd_tokens, stdout_redirect, stderr_redirect): (
            Vec<String>,
            Option<(String, bool)>,
            Option<(String, bool)>,
        ) = crate::helpers::parse_redirections(&tokens);

        if cmd_tokens.is_empty() {
            continue;
        }

        let cmd: &String = &cmd_tokens[0];
        let args: &[String] = &cmd_tokens[1..];
        let stdout_redirect_info: Option<(&str, bool)> =
            stdout_redirect.as_ref().map(|(t, append)| (t.as_str(), *append));
        let _stderr_redirect_info: Option<(&str, bool)> =
            stderr_redirect.as_ref().map(|(t, append)| (t.as_str(), *append));

        if let Some(func) = dispatch.get(cmd.as_str()) {
            crate::executor::execute_builtin(*func, args, stdout_redirect_info);
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if let Err(e) = crate::executor::execute_external(
                &path,
                cmd,
                args,
                stdout_redirect_info,
                _stderr_redirect_info,
            ) {
                eprintln!("Error executing command: {}", e);
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}
