pub fn unescape(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => {
                    chars.next();
                    result.push('\n');
                }
                Some('t') => {
                    chars.next();
                    result.push('\t');
                }
                Some('r') => {
                    chars.next();
                    result.push('\r');
                }
                Some('0') => {
                    chars.next();
                    result.push('\0');
                }
                Some('\\') => {
                    chars.next();
                    result.push('\\');
                }
                Some('"') => {
                    chars.next();
                    result.push('"');
                }
                Some('\'') => {
                    chars.next();
                    result.push('\'');
                }
                _ => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
