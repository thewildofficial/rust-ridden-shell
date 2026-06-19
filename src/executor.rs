use std::os::unix::process::CommandExt;

/// Execute an external command, optionally redirecting stdout/stderr to files.
pub fn execute_external(
    path: &str,
    cmd_name: &str,
    args: &[String],
    redirect_stdout: Option<&str>,
    redirect_stderr: Option<&str>,
) -> Result<(), std::io::Error> {
    let mut cmd: std::process::Command = std::process::Command::new(path);
    cmd.arg0(cmd_name);
    cmd.args(args);

    if let Some(target) = redirect_stdout {
        let file: std::fs::File = std::fs::File::create(target)?;
        cmd.stdout(std::process::Stdio::from(file));
    }

    if let Some(target) = redirect_stderr {
        let file: std::fs::File = std::fs::File::create(target)?;
        cmd.stderr(std::process::Stdio::from(file));
    }

    let mut child: std::process::Child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}

/// Execute a builtin function, optionally redirecting stdout to a file.
/// Uses a writer — either stdout or a File — so no unsafe needed.
pub fn execute_builtin(
    func: crate::builtin::BuiltinFn,
    args: &[String],
    redirect_stdout: Option<&str>,
) {
    match redirect_stdout {
        Some(target) => {
            match std::fs::File::create(target) {
                Ok(mut file) => func(args, &mut file),
                Err(e) => eprintln!("redirect: cannot open '{}': {}", target, e),
            }
        }
        None => func(args, &mut std::io::stdout()),
    }
}
