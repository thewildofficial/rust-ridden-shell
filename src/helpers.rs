use std::os::unix::fs::PermissionsExt;

#[derive(PartialEq)]
enum ParseState {
    Normal,
    SingleQuote,
    DoubleQuote,
    BackslashNormal,
    BackslashDoubleQuote,
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current: String = String::new();
    let mut state: ParseState = ParseState::Normal;
    let mut chars: std::iter::Peekable<std::str::Chars<'_>> = input.chars().peekable();

    while let Some(c) = chars.next() {
        match state {
            ParseState::BackslashNormal => {
                // outside quotes: backslash escapes ANY next character
                current.push(c);
                state = ParseState::Normal;
            }
            ParseState::BackslashDoubleQuote => {
                // inside double quotes: backslash only escapes " and \
                match c {
                    '"' | '\\' => {
                        current.push(c);
                        state = ParseState::DoubleQuote;
                    }
                    _ => {
                        // backslash is literal, char is literal
                        current.push('\\');
                        current.push(c);
                        state = ParseState::DoubleQuote;
                    }
                }
            }
            ParseState::SingleQuote => {
                if c == '\'' {
                    state = ParseState::Normal;
                } else {
                    current.push(c);
                }
            }
            ParseState::DoubleQuote => {
                match c {
                    '"' => state = ParseState::Normal,
                    '\\' => state = ParseState::BackslashDoubleQuote,
                    _ => current.push(c),
                }
            }
            ParseState::Normal => {
                match c {
                    '\\' => state = ParseState::BackslashNormal,
                    '\'' => state = ParseState::SingleQuote,
                    '"' => state = ParseState::DoubleQuote,
                    '>' => {
                        // Check if current ends with a digit (for 1>, 2> etc.)
                        let digit: Option<char> = if current.ends_with(|c: char| c.is_ascii_digit()) {
                            Some(current.pop().unwrap())
                        } else {
                            None
                        };

                        // Peek ahead for >> (append mode)
                        let is_append: bool = chars.peek() == Some(&'>');
                        if is_append {
                            chars.next(); // consume the second >
                        }

                        let op: String = match (digit, is_append) {
                            (Some(d), true)  => format!("{}{}", d, ">>"),
                            (Some(d), false) => format!("{}{}", d, ">"),
                            (None, true)     => ">>".to_string(),
                            (None, false)    => ">".to_string(),
                        };

                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        tokens.push(op);
                    }
                    ' ' | '\t' => {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                    }
                    _ => current.push(c),
                }
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Parse tokens into (command_args, stdout_redirect, stderr_redirect).
/// Each redirect is (target_filename, is_append).
/// Scans for `>`, `1>`, `>>`, `1>>`, `2>`, `2>>` in the token list.
pub fn parse_redirections(
    tokens: &[String],
) -> (Vec<String>, Option<(String, bool)>, Option<(String, bool)>) {
    let mut stdout_target: Option<(String, bool)> = None;
    let mut stderr_target: Option<(String, bool)> = None;
    let mut cmd_end: usize = tokens.len();

    for (i, token) in tokens.iter().enumerate() {
        if token == ">" || token == "1>" {
            cmd_end = i;
            stdout_target = tokens.get(i + 1).cloned().map(|t| (t, false));
            break;
        }
        if token == ">>" || token == "1>>" {
            cmd_end = i;
            stdout_target = tokens.get(i + 1).cloned().map(|t| (t, true));
            break;
        }
        if token == "2>" {
            cmd_end = i;
            stderr_target = tokens.get(i + 1).cloned().map(|t| (t, false));
            break;
        }
        if token == "2>>" {
            cmd_end = i;
            stderr_target = tokens.get(i + 1).cloned().map(|t| (t, true));
            break;
        }
    }

    let cmd_args: Vec<String> = tokens[..cmd_end].to_vec();
    (cmd_args, stdout_target, stderr_target)
}

pub fn find_executable(target: &str) -> Option<String> {
    let path_var: String = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let path: std::path::PathBuf = std::path::Path::new(dir).join(target);
        // "If the file exists but lacks execute permissions, skip it and continue to the next directory.". this is an additional check we need to do
        if let Ok(metadata) = std::fs::metadata(&path)
            && metadata.is_file()
            && metadata.permissions().mode() & 0o111 != 0 {
                return Some(path.to_string_lossy().into_owned());
            }
    }
    None
}
