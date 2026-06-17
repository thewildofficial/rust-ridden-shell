pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo","type"];

pub fn exit(args: &[String]) {
    let code = args.first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    std::process::exit(code);
}

pub fn type_cmd(args: &[String]) {
    // get the target
    let target = &args[0];
    if is_builtin(&target) {

        // {target} is a shell builtin
        println!("{} is a shell builtin", target);
    } else {
        // invalid_command: not found
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