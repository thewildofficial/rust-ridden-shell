use std::os::unix::process::CommandExt;

/// Execute an external command, optionally redirecting stdout to a file.
pub fn execute_external(
    path: &str,
    cmd_name: &str,
    args: &[String],
    redirect_stdout: Option<&str>,
) -> Result<(), std::io::Error> {
    let mut cmd: std::process::Command = std::process::Command::new(path);
    cmd.arg0(cmd_name);
    cmd.args(args);

    if let Some(target) = redirect_stdout {
        let file: std::fs::File = std::fs::File::create(target)?;
        cmd.stdout(std::process::Stdio::from(file));
    }

    let mut child: std::process::Child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}

/// Execute a builtin function, optionally redirecting stdout to a file.
/// Uses dup2 to redirect the shell's own stdout temporarily.
pub fn execute_builtin(
    func: crate::builtin::BuiltinFn,
    args: &[String],
    redirect_stdout: Option<&str>,
) {
    if let Some(target) = redirect_stdout {
        // Save current stdout (fd 1) by duplicating it
        let saved_stdout: i32 = unsafe { libc::dup(1) };
        if saved_stdout < 0 {
            eprintln!("redirect: failed to save stdout");
            return;
        }

        // Open the target file for writing (create/truncate)
        let file_fd: i32 = unsafe {
            let c_str: std::ffi::CString =
                std::ffi::CString::new(target).unwrap_or_else(|_| {
                    std::ffi::CString::new("").unwrap()
                });
            libc::open(
                c_str.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
                0o644,
            )
        };
        if file_fd < 0 {
            eprintln!("redirect: cannot open '{}'", target);
            unsafe { libc::close(saved_stdout); }
            return;
        }

        // Redirect fd 1 to the file
        unsafe {
            libc::dup2(file_fd, 1);
            libc::close(file_fd);
        }

        // Run the builtin — its println!() goes to the file
        func(args);

        // Restore stdout
        unsafe {
            libc::dup2(saved_stdout, 1);
            libc::close(saved_stdout);
        }
    } else {
        func(args);
    }
}
