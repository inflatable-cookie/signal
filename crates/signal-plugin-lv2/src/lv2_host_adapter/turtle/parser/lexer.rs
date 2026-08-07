impl<'a> Parser<'a> {
    fn error(&self, detail: impl std::fmt::Display) -> String {
        format!("line {}: {detail}", self.line)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.position + offset).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        if byte == b'\n' {
            self.line += 1;
        }
        Some(byte)
    }

    /// Skip whitespace and `#` comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(byte) if byte.is_ascii_whitespace() => {
                    self.bump();
                }
                Some(b'#') => {
                    while let Some(byte) = self.peek() {
                        if byte == b'\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn expect(&mut self, byte: u8, what: &str) -> Result<(), String> {
        self.skip_trivia();
        if self.peek() == Some(byte) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(format!(
                "expected {what} ({:?})",
                char::from(self.peek().unwrap_or(0)),
            )))
        }
    }

    /// Prefix / directive / bareword characters.
    fn take_name_chars(&mut self) -> String {
        let mut name = String::new();
        while let Some(byte) = self.peek() {
            if is_name_byte(byte) {
                name.push(byte as char);
                self.bump();
            } else {
                break;
            }
        }
        name
    }

    /// Local-part characters: name bytes plus interior dots (a dot only
    /// counts when followed by another local character, so a statement
    /// terminator never merges into a name).
    fn take_local_name_chars(&mut self) -> String {
        let mut name = String::new();
        while let Some(byte) = self.peek() {
            if is_name_byte(byte) {
                name.push(byte as char);
                self.bump();
            } else if byte == b'.' && self.peek_at(1).is_some_and(is_name_byte) {
                name.push('.');
                self.bump();
            } else {
                break;
            }
        }
        name
    }
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn utf8_width(byte: u8) -> usize {
    if byte >= 0xF0 {
        4
    } else if byte >= 0xE0 {
        3
    } else if byte >= 0xC0 {
        2
    } else {
        1
    }
}
