use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn execute_command(path: &str, cmd_name: &str, args: &[String]) -> Result<(), std::io::Error> {
    let mut cmd = Command::new(path);
    // set argv[0] to the bare command name, not the resolved path
    cmd.arg0(cmd_name);
    cmd.args(args);
    let mut child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}
