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

/// Execute a builtin function, optionally redirecting stdout/stderr to files.
/// Uses dup2 to redirect the shell's own fds temporarily.
pub fn execute_builtin(
    func: crate::builtin::BuiltinFn,
    args: &[String],
    redirect_stdout: Option<&str>,
    redirect_stderr: Option<&str>,
) {
    // Save current stdout (fd 1) and stderr (fd 2)
    let saved_stdout: i32;
    let saved_stderr: i32;

    if redirect_stdout.is_some() {
        saved_stdout = unsafe { libc::dup(1) };
        if saved_stdout < 0 {
            eprintln!("redirect: failed to save stdout");
            return;
        }

        let file_fd: i32 = open_file(redirect_stdout.unwrap());
        if file_fd < 0 {
            unsafe { libc::close(saved_stdout); }
            return;
        }

        unsafe {
            libc::dup2(file_fd, 1);
            libc::close(file_fd);
        }
    } else {
        saved_stdout = -1;
    }

    if redirect_stderr.is_some() {
        saved_stderr = unsafe { libc::dup(2) };
        if saved_stderr < 0 {
            if saved_stdout >= 0 { unsafe { libc::dup2(saved_stdout, 1); libc::close(saved_stdout); } }
            eprintln!("redirect: failed to save stderr");
            return;
        }

        let file_fd: i32 = open_file(redirect_stderr.unwrap());
        if file_fd < 0 {
            unsafe { libc::close(saved_stderr); }
            if saved_stdout >= 0 { unsafe { libc::dup2(saved_stdout, 1); libc::close(saved_stdout); } }
            return;
        }

        unsafe {
            libc::dup2(file_fd, 2);
            libc::close(file_fd);
        }
    } else {
        saved_stderr = -1;
    }

    // Run the builtin — println!() and eprintln!() go to the redirected fds
    func(args);

    // Restore stdout
    if saved_stdout >= 0 {
        unsafe {
            libc::dup2(saved_stdout, 1);
            libc::close(saved_stdout);
        }
    }

    // Restore stderr
    if saved_stderr >= 0 {
        unsafe {
            libc::dup2(saved_stderr, 2);
            libc::close(saved_stderr);
        }
    }
}

fn open_file(target: &str) -> i32 {
    unsafe {
        let c_str: std::ffi::CString =
            std::ffi::CString::new(target).unwrap_or_else(|_| {
                std::ffi::CString::new("").unwrap()
            });
        libc::open(
            c_str.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            0o644,
        )
    }
}