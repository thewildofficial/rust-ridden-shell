// a single file with all the builtins. each builtin is a function. you add a new one by writing a function and adding it to a match statement. clean, flat, easy.


 /* The exit Builtin

The exit builtin terminates the shell. Builtin commands are handled directly by the shell without starting a new process.

When your shell receives the exit command, it should terminate immediately.
 */

pub fn exit(code: i32) {
    // take the exit code, default to 0 if not provided
    let code = code.max(0);
    
    std::process::exit(code);
}
