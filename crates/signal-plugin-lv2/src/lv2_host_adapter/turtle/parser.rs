//! Turtle-subset parser.

use std::collections::BTreeMap;

use super::document::{TurtleDocument, TurtleTerm, TurtleTriple, RDF_TYPE};

impl TurtleDocument {
    /// Parse `text` against the documented subset. Errors carry a
    /// human-readable detail with a line number.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parser = Parser::new(text);
        parser.parse_document()?;
        Ok(Self {
            triples: parser.triples,
        })
    }
}

// ── Parser ──────────────────────────────────────────────────────────────────

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    line: usize,
    prefixes: BTreeMap<String, String>,
    labeled_blanks: BTreeMap<String, usize>,
    next_blank: usize,
    triples: Vec<TurtleTriple>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            position: 0,
            line: 1,
            prefixes: BTreeMap::new(),
            labeled_blanks: BTreeMap::new(),
            next_blank: 0,
            triples: Vec::new(),
        }
    }

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

    fn parse_document(&mut self) -> Result<(), String> {
        loop {
            self.skip_trivia();
            let Some(byte) = self.peek() else {
                return Ok(());
            };
            if byte == b'@' {
                self.parse_directive()?;
            } else {
                self.parse_statement()?;
            }
        }
    }

    fn parse_directive(&mut self) -> Result<(), String> {
        // Consume '@' + keyword.
        self.bump();
        let keyword = self.take_name_chars();
        if keyword != "prefix" {
            return Err(self.error(format!(
                "unsupported directive @{keyword} (only @prefix is in the subset)"
            )));
        }
        self.skip_trivia();
        let prefix = self.take_name_chars();
        self.expect(b':', "':' after prefix name")?;
        self.skip_trivia();
        let iri = self.parse_iri_ref()?;
        self.expect(b'.', "'.' ending @prefix directive")?;
        self.prefixes.insert(prefix, iri);
        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), String> {
        let subject = self.parse_subject()?;
        self.parse_predicate_object_list(&subject)?;
        self.expect(b'.', "'.' ending statement")?;
        Ok(())
    }

    fn parse_subject(&mut self) -> Result<TurtleTerm, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => Ok(TurtleTerm::Iri(self.parse_iri_ref()?)),
            Some(b'_') if self.peek_at(1) == Some(b':') => Ok(self.parse_labeled_blank()),
            Some(b'[') => Err(self
                .error("blank-node subjects are outside the supported Turtle subset".to_string())),
            Some(b'(') => {
                Err(self.error("collections are outside the supported Turtle subset".to_string()))
            }
            Some(_) => Ok(TurtleTerm::Iri(self.parse_prefixed_name()?)),
            None => Err(self.error("unexpected end of input reading subject")),
        }
    }

    fn parse_predicate_object_list(&mut self, subject: &TurtleTerm) -> Result<(), String> {
        loop {
            let predicate = self.parse_predicate()?;
            loop {
                let object = self.parse_object()?;
                self.triples.push(TurtleTriple {
                    subject: subject.clone(),
                    predicate: predicate.clone(),
                    object,
                });
                self.skip_trivia();
                if self.peek() == Some(b',') {
                    self.bump();
                } else {
                    break;
                }
            }
            self.skip_trivia();
            if self.peek() == Some(b';') {
                self.bump();
                self.skip_trivia();
                // Trailing ';' before '.' or ']' is legal Turtle.
                match self.peek() {
                    Some(b'.') | Some(b']') | None => return Ok(()),
                    Some(b';') => {
                        // Consecutive semicolons collapse.
                        continue;
                    }
                    _ => continue,
                }
            }
            return Ok(());
        }
    }

    fn parse_predicate(&mut self) -> Result<String, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => self.parse_iri_ref(),
            Some(b'a')
                if !self
                    .peek_at(1)
                    .is_some_and(|byte| is_name_byte(byte) || byte == b':') =>
            {
                self.bump();
                Ok(RDF_TYPE.to_string())
            }
            Some(_) => self.parse_prefixed_name(),
            None => Err(self.error("unexpected end of input reading predicate")),
        }
    }

    fn parse_object(&mut self) -> Result<TurtleTerm, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => Ok(TurtleTerm::Iri(self.parse_iri_ref()?)),
            Some(b'"') => self.parse_string_literal(),
            Some(b'[') => self.parse_blank_property_list(),
            Some(b'(') => {
                Err(self.error("collections are outside the supported Turtle subset".to_string()))
            }
            Some(b'_') if self.peek_at(1) == Some(b':') => Ok(self.parse_labeled_blank()),
            Some(byte) if byte == b'+' || byte == b'-' || byte.is_ascii_digit() => {
                self.parse_number()
            }
            Some(_) => {
                // Bareword: boolean or a prefixed name.
                let checkpoint = self.position;
                let word = self.take_name_chars();
                match word.as_str() {
                    "true" if self.peek() != Some(b':') => Ok(TurtleTerm::Bool(true)),
                    "false" if self.peek() != Some(b':') => Ok(TurtleTerm::Bool(false)),
                    _ => {
                        self.position = checkpoint;
                        Ok(TurtleTerm::Iri(self.parse_prefixed_name()?))
                    }
                }
            }
            None => Err(self.error("unexpected end of input reading object")),
        }
    }

    fn parse_blank_property_list(&mut self) -> Result<TurtleTerm, String> {
        self.expect(b'[', "'['")?;
        let blank = TurtleTerm::Blank(self.next_blank);
        self.next_blank += 1;
        self.skip_trivia();
        if self.peek() != Some(b']') {
            self.parse_predicate_object_list(&blank)?;
        }
        self.expect(b']', "']' closing blank node")?;
        Ok(blank)
    }

    fn parse_labeled_blank(&mut self) -> TurtleTerm {
        // Consume `_:`.
        self.bump();
        self.bump();
        let label = self.take_name_chars();
        let next = self.next_blank;
        let id = *self.labeled_blanks.entry(label).or_insert_with(|| next);
        if id == next {
            self.next_blank += 1;
        }
        TurtleTerm::Blank(id)
    }

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
