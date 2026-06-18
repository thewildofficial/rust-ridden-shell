use std::{os::unix::fs::PermissionsExt};

pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo","type"];

pub fn exit(args: &[String]) {
    let code = args.first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    std::process::exit(code);
}

// find_executable: search for the command in the directories listed in PATH.
fn find_executable(target: &str) -> Option<String> {
    for dir in std::env::var("PATH").unwrap_or_default().split(':') {
        if dir.is_empty() {
            continue;
        }
        let path = std::path::Path::new(dir).join(target);
        // "If the file exists but lacks execute permissions, skip it and continue to the next directory.". this is an additional check we need to do
        if let Ok(metadata) = std::fs::metadata(&path) {
            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }
    None
}

pub fn type_cmd(args: &[String]) {
    // get the target
    let target = &args[0];

    // Check if the command is a builtin command (like exit or echo). If it is, report it as a builtin (<command> is a shell builtin) and stop.
    if is_builtin(&target) {

        // {target} is a shell builtin
        println!("{} is a shell builtin", target);
    } 
    /* condition two,

    If the command is not a builtin, your shell must go through every directory in PATH. For each directory:
    
        Check if a file with the command name exists.
        Check if the file has execute permissions.
        If the file exists and has execute permissions, print <command> is <full_path> and stop.
        If the file exists but lacks execute permissions, skip it and continue to the next directory.
        */
        else if let Some(path) = find_executable(&target) {
            println!("{} is {}", target, path);
        } else {
            // invalid_command: not found
            // If no executable is found in any directory, print <command>: not found.
            eprintln!("{}: not found", target);
        }
    }


pub fn echo(args: &[String]) {
    // args already has the command name stripped, so just join and print
    println!("{}", args.join(" "));
}

// is builtin function
// param: name
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_COMMANDS.contains(&name)
}