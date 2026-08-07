impl<'a> Parser<'a> {
    fn parse_iri_ref(&mut self) -> Result<String, String> {
        self.expect(b'<', "'<' opening IRI")?;
        let mut iri = String::new();
        loop {
            match self.bump() {
                Some(b'>') => return Ok(iri),
                Some(byte) if byte.is_ascii_whitespace() => {
                    return Err(self.error("whitespace inside IRI reference"));
                }
                Some(byte) => iri.push(byte as char),
                None => return Err(self.error("unterminated IRI reference")),
            }
        }
    }

    fn parse_prefixed_name(&mut self) -> Result<String, String> {
        let prefix = self.take_name_chars();
        if self.peek() != Some(b':') {
            return Err(self.error(format!(
                "expected ':' in prefixed name after {prefix:?} (bare words are outside the subset)"
            )));
        }
        self.bump();
        let local = self.take_local_name_chars();
        let Some(base) = self.prefixes.get(&prefix) else {
            return Err(self.error(format!("unknown prefix {prefix:?}")));
        };
        Ok(format!("{base}{local}"))
    }

    fn parse_string_literal(&mut self) -> Result<TurtleTerm, String> {
        // Reject triple-quoted long strings up front (outside the subset).
        if self.peek_at(1) == Some(b'"') && self.peek_at(2) == Some(b'"') {
            return Err(self.error("triple-quoted strings are outside the supported Turtle subset"));
        }
        self.expect(b'"', "'\"' opening string")?;
        let mut value = String::new();
        loop {
            match self.bump() {
                Some(b'"') => break,
                Some(b'\\') => {
                    let escaped = self
                        .bump()
                        .ok_or_else(|| self.error("unterminated escape"))?;
                    match escaped {
                        b'"' => value.push('"'),
                        b'\\' => value.push('\\'),
                        b'n' => value.push('\n'),
                        b'r' => value.push('\r'),
                        b't' => value.push('\t'),
                        b'u' => value.push(self.parse_unicode_escape(4)?),
                        b'U' => value.push(self.parse_unicode_escape(8)?),
                        other => {
                            return Err(self.error(format!(
                                "unsupported string escape \\{}",
                                char::from(other)
                            )));
                        }
                    }
                }
                Some(b'\n') => return Err(self.error("unterminated string literal")),
                Some(byte) => {
                    // Re-assemble UTF-8 bytes verbatim.
                    let start = self.position - 1;
                    let width = utf8_width(byte);
                    for _ in 1..width {
                        self.bump();
                    }
                    let slice = &self.bytes[start..self.position];
                    value.push_str(
                        std::str::from_utf8(slice)
                            .map_err(|_| self.error("invalid UTF-8 in string literal"))?,
                    );
                }
                None => return Err(self.error("unterminated string literal")),
            }
        }
        // Optional language tag or datatype annotation (parsed, dropped).
        if self.peek() == Some(b'@') {
            self.bump();
            while self
                .peek()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                self.bump();
            }
        } else if self.peek() == Some(b'^') && self.peek_at(1) == Some(b'^') {
            self.bump();
            self.bump();
            self.skip_trivia();
            match self.peek() {
                Some(b'<') => {
                    self.parse_iri_ref()?;
                }
                _ => {
                    self.parse_prefixed_name()?;
                }
            }
        }
        Ok(TurtleTerm::Literal(value))
    }

    fn parse_unicode_escape(&mut self, digits: usize) -> Result<char, String> {
        let mut code = 0u32;
        for _ in 0..digits {
            let byte = self
                .bump()
                .ok_or_else(|| self.error("unterminated unicode escape"))?;
            let digit = (byte as char)
                .to_digit(16)
                .ok_or_else(|| self.error("invalid unicode escape digit"))?;
            code = code * 16 + digit;
        }
        char::from_u32(code).ok_or_else(|| self.error("invalid unicode escape code point"))
    }

    fn parse_number(&mut self) -> Result<TurtleTerm, String> {
        let start = self.position;
        if matches!(self.peek(), Some(b'+') | Some(b'-')) {
            self.bump();
        }
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.bump();
        }
        // A '.' joins the number only when a digit follows — otherwise it is
        // the statement terminator.
        if self.peek() == Some(b'.') && self.peek_at(1).is_some_and(|byte| byte.is_ascii_digit()) {
            self.bump();
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.bump();
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.bump();
            }
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.bump();
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| self.error("invalid number"))?;
        text.parse::<f64>()
            .map(TurtleTerm::Number)
            .map_err(|_| self.error(format!("invalid numeric literal {text:?}")))
    }
}
