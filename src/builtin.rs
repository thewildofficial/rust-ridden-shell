pub fn exit(args: &[String]) {
    let code = args.first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    std::process::exit(code);
}

pub fn echo(args: &[String]) {
    // args already has the command name stripped, so just join and print
    println!("{}", args.join(" "));
}