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

/// Execute a builtin function, optionally redirecting stdout to a file.
/// Uses a writer — either stdout or a File — so no unsafe needed.
pub fn execute_builtin(
    func: crate::builtin::BuiltinFn,
    args: &[String],
    redirect_stdout: Option<(&str, bool)>,
) {
    match redirect_stdout {
        Some((target, append)) => {
            match open_redirect(target, append) {
                Ok(mut file) => func(args, &mut file),
                Err(e) => eprintln!("redirect: cannot open '{}': {}", target, e),
            }
        }
        None => func(args, &mut std::io::stdout()),
    }
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
