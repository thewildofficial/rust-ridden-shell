use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

/// Execute a pipeline of commands connected by pipes.
pub fn execute_pipeline(segments: &[&[String]]) {
    if segments.is_empty() {
        return;
    }

    let n: usize = segments.len();

    // For dual-command pipelines
    if n == 2 {
        let cmd0: &str = &segments[0][0];
        let cmd1: &str = &segments[1][0];
        let args0: &[String] = &segments[0][1..];
        let args1: &[String] = &segments[1][1..];

        let is_ext0: bool = crate::helpers::find_executable(cmd0).is_some();
        let is_ext1: bool = crate::helpers::find_executable(cmd1).is_some();

        // external | external — use real OS pipe
        if is_ext0 && is_ext1 {
            let path0: String = crate::helpers::find_executable(cmd0).unwrap();
            let path1: String = crate::helpers::find_executable(cmd1).unwrap();

            // Create a real OS pipe
            let (reader, writer) = match os_pipe::pipe() {
                Ok(p) => p,
                Err(e) => { eprintln!("pipe error: {}", e); return; }
            };

            let mut child0: Command = Command::new(&path0);
            child0.arg0(cmd0);
            child0.args(args0);
            child0.stdout(Stdio::from(writer));
            child0.stderr(Stdio::inherit());

            let mut child1: Command = Command::new(&path1);
            child1.arg0(cmd1);
            child1.args(args1);
            child1.stdin(Stdio::from(reader));
            child1.stdout(Stdio::inherit());
            child1.stderr(Stdio::inherit());

            let mut c0: std::process::Child = match child0.spawn() {
                Ok(c) => c,
                Err(e) => { eprintln!("pipeline error: {}", e); return; }
            };
            let mut c1: std::process::Child = match child1.spawn() {
                Ok(c) => c,
                Err(e) => { eprintln!("pipeline error: {}", e); return; }
            };

            // Wait for both — c1 will finish when c0's pipe closes
            let _ = c0.wait();
            let _ = c1.wait();
            return;
        }

        // builtin | external
        if crate::builtin::is_builtin(cmd0) && is_ext1 {
            let path1: String = crate::helpers::find_executable(cmd1).unwrap();

            let mut buf: Vec<u8> = Vec::new();
            if let Some(func) = crate::builtin::get_dispatch_table().get(cmd0) {
                let mut stderr_writer: Box<dyn std::io::Write> = Box::new(std::io::stderr());
                func(args0, &mut buf, &mut *stderr_writer);
            }

            let mut child1: Command = Command::new(&path1);
            child1.arg0(cmd1);
            child1.args(args1);
            child1.stdin(Stdio::piped());
            child1.stdout(Stdio::inherit());
            child1.stderr(Stdio::inherit());

            if let Ok(mut c1) = child1.spawn() {
                if let Some(mut stdin) = c1.stdin.take() {
                    let _ = stdin.write_all(&buf);
                }
                let _ = c1.wait();
            }
            return;
        }

        // external | builtin
        if is_ext0 && crate::builtin::is_builtin(cmd1) {
            let path0: String = crate::helpers::find_executable(cmd0).unwrap();

            let mut child0: Command = Command::new(&path0);
            child0.arg0(cmd0);
            child0.args(args0);
            child0.stdout(Stdio::piped());
            child0.stderr(Stdio::inherit());

            if let Ok(mut c0) = child0.spawn() {
                let mut buf: Vec<u8> = Vec::new();
                if let Some(mut stdout) = c0.stdout.take() {
                    let _ = stdout.read_to_end(&mut buf);
                }
                let _ = c0.wait();

                let output: String = String::from_utf8_lossy(&buf).to_string();
                let piped_args: Vec<String> = output.lines().map(|s| s.to_string()).collect();

                if let Some(func) = crate::builtin::get_dispatch_table().get(cmd1) {
                    let mut stdout_writer: Box<dyn std::io::Write> = Box::new(std::io::stdout());
                    let mut stderr_writer: Box<dyn std::io::Write> = Box::new(std::io::stderr());
                    func(&piped_args, &mut *stdout_writer, &mut *stderr_writer);
                }
            }
            return;
        }
    }

    // Multi-command pipeline (3+ commands) — all external
    if n >= 3 {
        let all_external: bool = segments
            .iter()
            .all(|s| crate::helpers::find_executable(&s[0]).is_some());

        if all_external {
            // Chain: create pipes between each pair of commands
            let mut children: Vec<std::process::Child> = Vec::new();
            let mut prev_read: Option<os_pipe::PipeReader> = None;

            for (i, segment) in segments.iter().enumerate() {
                let cmd_name: &str = &segment[0];
                let args: &[String] = &segment[1..];
                let path: String = crate::helpers::find_executable(cmd_name).unwrap();

                let mut cmd: Command = Command::new(&path);
                cmd.arg0(cmd_name);
                cmd.args(args);

                // Create pipe for this command's output (except last)
                if i < n - 1 {
                    let (reader, writer) = match os_pipe::pipe() {
                        Ok(p) => p,
                        Err(e) => { eprintln!("pipe error: {}", e); return; }
                    };

                    // Set stdin from previous pipe
                    if let Some(ref prev) = prev_read {
                        cmd.stdin(Stdio::from(prev.try_clone().unwrap()));
                    }

                    cmd.stdout(Stdio::from(writer));
                    cmd.stderr(Stdio::inherit());

                    match cmd.spawn() {
                        Ok(c) => {
                            children.push(c);
                            prev_read = Some(reader);
                        }
                        Err(e) => { eprintln!("pipeline error: {}", e); return; }
                    }
                } else {
                    // Last command
                    if let Some(ref prev) = prev_read {
                        cmd.stdin(Stdio::from(prev.try_clone().unwrap()));
                    }
                    cmd.stdout(Stdio::inherit());
                    cmd.stderr(Stdio::inherit());

                    match cmd.spawn() {
                        Ok(c) => children.push(c),
                        Err(e) => { eprintln!("pipeline error: {}", e); return; }
                    }
                }
            }

            // Wait for all children
            for mut c in children {
                let _ = c.wait();
            }
            return;
        }
    }

    // Fallback: run the last command
    let last: &[String] = segments[n - 1];
    let cmd_name: &str = &last[0];
    let args: &[String] = &last[1..];

    if let Some(func) = crate::builtin::get_dispatch_table().get(cmd_name) {
        let mut stdout_writer: Box<dyn std::io::Write> = Box::new(std::io::stdout());
        let mut stderr_writer: Box<dyn std::io::Write> = Box::new(std::io::stderr());
        func(args, &mut *stdout_writer, &mut *stderr_writer);
    } else if let Some(path) = crate::helpers::find_executable(cmd_name) {
        let mut cmd: Command = Command::new(&path);
        cmd.arg0(cmd_name);
        cmd.args(args);
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        if let Ok(mut child) = cmd.spawn() {
            let _ = child.wait();
        }
    }
}

/// Execute an external command, optionally redirecting stdout/stderr to files.
pub fn execute_external(
    path: &str,
    cmd_name: &str,
    args: &[String],
    redirect_stdout: Option<(&str, bool)>,
    redirect_stderr: Option<(&str, bool)>,
) -> Result<(), std::io::Error> {
    let mut cmd: std::process::Command = std::process::Command::new(path);
    cmd.arg0(cmd_name);
    cmd.args(args);

    if let Some((target, append)) = redirect_stdout {
        let file: std::fs::File = open_redirect(target, append)?;
        cmd.stdout(std::process::Stdio::from(file));
    }

    if let Some((target, append)) = redirect_stderr {
        let file: std::fs::File = open_redirect(target, append)?;
        cmd.stderr(std::process::Stdio::from(file));
    }

    let mut child: std::process::Child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}

/// Execute an external command in the background. Returns the PID.
pub fn execute_background(
    path: &str,
    cmd_name: &str,
    args: &[String],
    redirect_stdout: Option<(&str, bool)>,
    redirect_stderr: Option<(&str, bool)>,
) -> u32 {
    let mut cmd: std::process::Command = std::process::Command::new(path);
    cmd.arg0(cmd_name);
    cmd.args(args);

    if let Some((target, append)) = redirect_stdout {
        if let Ok(file) = open_redirect(target, append) {
            cmd.stdout(std::process::Stdio::from(file));
        }
    }

    if let Some((target, append)) = redirect_stderr {
        if let Ok(file) = open_redirect(target, append) {
            cmd.stderr(std::process::Stdio::from(file));
        }
    }

    match cmd.spawn() {
        Ok(child) => child.id(),
        Err(e) => {
            eprintln!("Error starting background job: {}", e);
            0
        }
    }
}

/// Execute a builtin function, optionally redirecting stdout/stderr to files.
pub fn execute_builtin(
    func: crate::builtin::BuiltinFn,
    args: &[String],
    redirect_stdout: Option<(&str, bool)>,
    redirect_stderr: Option<(&str, bool)>,
) {
    let mut stdout_writer: Box<dyn std::io::Write> = match redirect_stdout {
        Some((target, append)) => match open_redirect(target, append) {
            Ok(file) => Box::new(file),
            Err(e) => {
                eprintln!("redirect: cannot open '{}': {}", target, e);
                Box::new(std::io::stdout())
            }
        },
        None => Box::new(std::io::stdout()),
    };

    let mut stderr_writer: Box<dyn std::io::Write> = match redirect_stderr {
        Some((target, append)) => match open_redirect(target, append) {
            Ok(file) => Box::new(file),
            Err(e) => {
                eprintln!("redirect: cannot open '{}': {}", target, e);
                Box::new(std::io::stderr())
            }
        },
        None => Box::new(std::io::stderr()),
    };

    func(args, &mut *stdout_writer, &mut *stderr_writer);
}

fn open_redirect(target: &str, append: bool) -> Result<std::fs::File, std::io::Error> {
    use std::fs::OpenOptions;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!append)
        .append(append)
        .open(target)
}
