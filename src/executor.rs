use std::os::unix::process::CommandExt;

/// Execute an external command, optionally redirecting stdout/stderr to files.
/// Each redirect is (target_filename, is_append).
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

/// Execute an external command in the background.
/// Spawns the process, prints [job_id] pid, and returns immediately.
pub fn execute_background(
    path: &str,
    cmd_name: &str,
    args: &[String],
    redirect_stdout: Option<(&str, bool)>,
    redirect_stderr: Option<(&str, bool)>,
    job_id: u32,
) {
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
        Ok(child) => {
            println!("[{}] {}", job_id, child.id());
            // Don't wait — that's the whole point of background
        }
        Err(e) => {
            eprintln!("Error starting background job: {}", e);
        }
    }
}

/// Execute a builtin function, optionally redirecting stdout/stderr to files.
/// Uses writers — no unsafe needed.
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
