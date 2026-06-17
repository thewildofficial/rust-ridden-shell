#[allow(unused_imports)]
use std::io::{self, Write};
mod builtin;
mod repl;
fn main() {
    repl::repl();
}
