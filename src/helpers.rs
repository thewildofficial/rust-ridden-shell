use std::os::unix::fs::PermissionsExt;

#[derive(PartialEq)]
enum ParseState {
    Normal,
    SingleQuote,
    DoubleQuote,
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut state = ParseState::Normal;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match state {
            ParseState::SingleQuote => {
                if c == '\'' {
                    state = ParseState::Normal;
                } else {
                    current.push(c);
                }
            }
            ParseState::DoubleQuote => {
                if c == '"' {
                    state = ParseState::Normal;
                } else {
                    current.push(c);
                }
            }
            ParseState::Normal => {
                match c {
                    '\'' => state = ParseState::SingleQuote,
                    '"' => state = ParseState::DoubleQuote,
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

pub fn find_executable(target: &str) -> Option<String> {
    for dir in std::env::var("PATH").unwrap_or_default().split(':') {
        if dir.is_empty() {
            continue;
        }
        let path = std::path::Path::new(dir).join(target);
        // "If the file exists but lacks execute permissions, skip it and continue to the next directory.". this is an additional check we need to do
        if let Ok(metadata) = std::fs::metadata(&path)
            && metadata.is_file()
            && metadata.permissions().mode() & 0o111 != 0 {
                return Some(path.to_string_lossy().into_owned());
            }
    }
    None
}
