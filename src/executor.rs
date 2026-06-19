use std::os::unix::process::CommandExt;

pub fn execute_command(path: &str, cmd_name: &str, args: &[String]) -> Result<(), std::io::Error> {
    let mut cmd: std::process::Command = std::process::Command::new(path);
    // set argv[0] to the bare command name, not the resolved path
    cmd.arg0(cmd_name);
    cmd.args(args);
    let mut child: std::process::Child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}
