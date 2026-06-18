use std::{ process::Command};

pub fn execute_command(path: &str, args: &[String]) -> Result<(), std::io::Error> {
    // get the command
    let mut cmd = Command::new(path);
    cmd.args(args);
    //spawn command 
    let mut status = cmd.spawn()?;
    // wait for command to finish
    status.wait()?;
    Ok(())
}