# rust-ridden-shell

a shell. in rust. for a java class.

## why

my OS professor wanted us to build a shell. i don't speak java. so i built it in rust instead. zero unsafe. 845 lines. 100/100 on the test suite.

## what it does

- **navigation** — `cd`, `pwd`, `ls`, all that
- **quoting** — single quotes, double quotes, backslash. handles `echo 'hello   world'` like a real shell
- **redirection** — `>`, `>>`, `1>`, `2>`, `1>>`, `2>>`. stdout and stderr go where you tell them
- **background jobs** — `sleep 10 &`. `jobs` lists them. reaps them when they finish. recycles job IDs like a real OS
- **pipelines** — `cat | head -n 3 | wc`. multi-command. builtins in pipelines. streaming via OS pipes, no buffer-everything-into-memory nonsense

## what it doesn't do

- autocompletion (lazy)
- history (ctrl+r is for people who don't use tmux)
- parameter expansion (no `$HOME`, no `$PATH`. you want env vars? use bash)

## the numbers

| metric | value |
|--------|-------|
| lines of rust | 845 |
| unsafe blocks | 0 |
| binary size | 641K (unstripped) |
| test score | 100/100 |
| java used | 0% |
| time spent learning rust enums | too much |
| time spent explaining to professor why rust | even more |

## the architecture

```
main.rs → repl.rs → helpers.rs (tokenizer)
                  → executor.rs (external commands, pipelines, background jobs)
                  → builtin.rs (echo, exit, type, pwd, cd, jobs)
                  → jobs.rs (job manager, reaping)
```

state machine tokenizer using a flat enum (`Normal`, `SingleQuote`, `DoubleQuote`, `BackslashNormal`, `BackslashDoubleQuote`). writer injection for builtin output instead of `dup2` (that's how we stay at zero unsafe). `os_pipe` crate for streaming pipelines. `waitpid(WNOHANG)` + `/proc` fallback for job reaping.

## the real question

is it good? idk. it passes the tests. it doesn't crash (usually). it's written in safe rust. the binary is 641K which is honestly embarrassing for a shell but rust's stdlib is fat and i'm not gonna strip it because that's effort.

is it efficient? it's a shell. it forks and execs. the bottleneck is the OS, not my code.

is it safe? zero unsafe blocks. the borrow checker is my therapist.

would i use it as my daily driver? absolutely not. but it should get me an A* (inshallah).

## license

do whatever you want. it's a school project.
