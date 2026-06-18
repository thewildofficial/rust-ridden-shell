#[allow(unused_imports)]
use std::io::{self, Write};
mod builtin;
mod helpers;
mod repl;
mod executor;
fn main() {
    repl::repl();
}
