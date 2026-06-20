use std::io::Write;

pub fn repl() {
    let dispatch: std::collections::HashMap<&str, crate::builtin::BuiltinFn> =
        crate::builtin::get_dispatch_table();
    let mut next_job_id: u32 = 1;

    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut input: String = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let trimmed: &str = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut tokens: Vec<String> = crate::helpers::tokenize(trimmed);

        // Check for background operator &
        let is_background: bool = tokens.last().map(|t| t == "&").unwrap_or(false);
        if is_background {
            tokens.pop();
        }

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
        let stderr_redirect_info: Option<(&str, bool)> =
            stderr_redirect.as_ref().map(|(t, append)| (t.as_str(), *append));

        if let Some(func) = dispatch.get(cmd.as_str()) {
            crate::executor::execute_builtin(*func, args, stdout_redirect_info, stderr_redirect_info);
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if is_background {
                let job_id: u32 = next_job_id;
                next_job_id += 1;
                crate::executor::execute_background(&path, cmd, args, stdout_redirect_info, stderr_redirect_info, job_id);
            } else if let Err(e) = crate::executor::execute_external(
                &path,
                cmd,
                args,
                stdout_redirect_info,
                stderr_redirect_info,
            ) {
                eprintln!("Error executing command: {}", e);
            }
        } else {
            println!("{}: command not found", trimmed);
        }
    }
}
