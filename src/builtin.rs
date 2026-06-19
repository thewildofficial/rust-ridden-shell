use std::io::Write;
use std::collections::HashMap;

pub type BuiltinFn = fn(&[String], &mut dyn Write);

pub fn get_dispatch_table() -> HashMap<&'static str, BuiltinFn> {
    let mut map: HashMap<&'static str, BuiltinFn> = HashMap::new();
    map.insert("exit", exit as BuiltinFn);
    map.insert("echo", echo as BuiltinFn);
    map.insert("type", type_cmd as BuiltinFn);
    map.insert("pwd", pwd as BuiltinFn);
    map.insert("cd", cd as BuiltinFn);
    map
}

pub fn is_builtin(name: &str) -> bool {
    get_dispatch_table().contains_key(name)
}

pub fn exit(args: &[String], _w: &mut dyn Write) {
    let code: i32 = args.first()
        .and_then(|s: &String| s.parse::<i32>().ok())
        .unwrap_or(0);
    std::process::exit(code);
}

pub fn echo(args: &[String], w: &mut dyn Write) {
    writeln!(w, "{}", args.join(" ")).unwrap();
}

pub fn type_cmd(args: &[String], w: &mut dyn Write) {
    let target: &String = &args[0];
    if is_builtin(target) {
        writeln!(w, "{} is a shell builtin", target).unwrap();
    } else if let Some(path) = crate::helpers::find_executable(target) {
        writeln!(w, "{} is {}", target, path).unwrap();
    } else {
        eprintln!("{}: not found", target);
    }
}

pub fn pwd(_args: &[String], w: &mut dyn Write) {
    match std::env::current_dir() {
        Ok(path) => writeln!(w, "{}", path.display()).unwrap(),
        Err(e) => eprintln!("pwd: error: {}", e),
    }
}

pub fn cd(args: &[String], _w: &mut dyn Write) {
    let target: &str = args.first().map(|s: &String| s.as_str()).unwrap_or("~");
    let path: String = if target == "~" {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        target.to_string()
    };
    if let Err(_) = std::env::set_current_dir(&path) {
        eprintln!("cd: {}: No such file or directory", target);
    }
}
