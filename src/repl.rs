use std::io::Write;

pub fn repl() {
    let dispatch: std::collections::HashMap<&str, crate::builtin::BuiltinFn> =
        crate::builtin::get_dispatch_table();
    let mut job_manager: crate::jobs::JobManager = crate::jobs::JobManager::new();

    loop {
        // Reap finished jobs before showing prompt
        let reaped: Vec<u32> = job_manager.reap_finished();
        for id in &reaped {
            if let Some(job) = job_manager.all_sorted().iter().find(|j| j.id == *id) {
                let latest: Option<u32> = job_manager.latest_id();
                let second_latest: Option<u32> = job_manager.second_latest_id();
                let marker: &str = if Some(*id) == latest {
                    "+"
                } else if Some(*id) == second_latest {
                    "-"
                } else {
                    " "
                };
                eprintln!("[{}]{}  Done                    {}", job.id, marker, job.command);
            }
        }
        // Remove done jobs from the manager
        job_manager.remove_done();
        // Recycle job numbers if no jobs remain
        job_manager.recycle_id();

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

        // Check for pipe operator |
        let pipe_positions: Vec<usize> = tokens.iter().enumerate()
            .filter(|(_, t)| *t == "|")
            .map(|(i, _)| i)
            .collect();

        if !pipe_positions.is_empty() {
            // Handle pipeline: split tokens at each | and execute
            let mut segments: Vec<&[String]> = Vec::new();
            let mut start: usize = 0;
            for &pos in &pipe_positions {
                segments.push(&tokens[start..pos]);
                start = pos + 1;
            }
            segments.push(&tokens[start..]);

            crate::executor::execute_pipeline(&segments);
            continue;
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
            if cmd == "jobs" {
                let jobs: Vec<&crate::jobs::Job> = job_manager.all_sorted();
                let latest: Option<u32> = job_manager.latest_id();
                let second_latest: Option<u32> = job_manager.second_latest_id();
                for job in &jobs {
                    let status_str: &str = match job.status {
                        crate::jobs::JobStatus::Running => "Running",
                        crate::jobs::JobStatus::Done => "Done",
                    };
                    let marker: &str = if Some(job.id) == latest {
                        "+"
                    } else if Some(job.id) == second_latest {
                        "-"
                    } else {
                        " "
                    };
                    writeln!(
                        std::io::stdout(),
                        "[{}]{} {:<24} {}",
                        job.id,
                        marker,
                        status_str,
                        job.command
                    )
                    .unwrap();
                }
            } else {
                crate::executor::execute_builtin(*func, args, stdout_redirect_info, stderr_redirect_info);
            }
        } else if let Some(path) = crate::helpers::find_executable(cmd) {
            if is_background {
                let pid: u32 = crate::executor::execute_background(
                    &path,
                    cmd,
                    args,
                    stdout_redirect_info,
                    stderr_redirect_info,
                );
                let job_id: u32 = job_manager.add(pid, trimmed.to_string());
                println!("[{}] {}", job_id, pid);
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
