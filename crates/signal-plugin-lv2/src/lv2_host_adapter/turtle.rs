//! Handwritten Turtle-subset parser for LV2 manifests (packet g11.033
//! decision 6: no lilv/serd/RDF dependencies).
//!
//! # Subset boundaries
//!
//! Supported syntax — the shapes real LV2 bundle TTL uses:
//! - `@prefix` directives (a prefix table, applied inline);
//! - triple statements with `;` predicate continuations and `,` object
//!   lists (including a trailing `;` before the closing `.`);
//! - the `a` keyword as `rdf:type`;
//! - anonymous blank-node property lists (`[ ... ]`, nesting allowed) as
//!   objects — the LV2 `lv2:port` shape — plus labeled `_:name` nodes;
//! - `<...>` IRIs (absolute or bundle-relative) and `prefix:name` forms;
//! - string literals with `\"`/`\\`/`\n`/`\r`/`\t`/`\uXXXX`/`\UXXXXXXXX`
//!   escapes, with optional (ignored) `@lang` tags and `^^datatype`
//!   annotations;
//! - integer, decimal, and exponent-form numeric literals; `true`/`false`;
//! - `#` comments outside literals.
//!
//! Deliberately NOT a general RDF store. Anything outside the subset —
//! `@base`, SPARQL-style `PREFIX`, collections `( ... )`, triple-quoted
//! long strings, blank-node subjects, unknown prefixes — returns a parse
//! error which discovery surfaces as a `MalformedManifest` diagnostic.
//! The parser never panics on malformed input and never silently
//! misparses a construct it does not support.

use std::collections::BTreeMap;

/// One RDF term in the parsed subset.
#[derive(Clone, Debug, PartialEq)]
pub enum TurtleTerm {
    /// An IRI (already prefix-expanded; may be relative to the document).
    Iri(String),
    /// An anonymous or labeled blank node, identified per-document.
    Blank(usize),
    /// A string literal (language tags / datatypes parsed then dropped).
    Literal(String),
    /// A numeric literal (integer, decimal, or exponent form).
    Number(f64),
    /// A boolean literal.
    Bool(bool),
}

impl TurtleTerm {
    /// The IRI text when this term is an IRI.
    pub fn as_iri(&self) -> Option<&str> {
        match self {
            Self::Iri(iri) => Some(iri.as_str()),
            _ => None,
        }
    }

    /// The literal text when this term is a string literal.
    pub fn as_literal(&self) -> Option<&str> {
        match self {
            Self::Literal(text) => Some(text.as_str()),
            _ => None,
        }
    }

    /// The numeric value when this term is a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// One parsed subject–predicate–object triple.
#[derive(Clone, Debug, PartialEq)]
pub struct TurtleTriple {
    /// Subject term (IRI or blank node).
    pub subject: TurtleTerm,
    /// Predicate IRI (`a` expands to `rdf:type`).
    pub predicate: String,
    /// Object term.
    pub object: TurtleTerm,
}

/// A parsed Turtle document: the flat triple list plus query helpers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurtleDocument {
    /// All triples in document order.
    pub triples: Vec<TurtleTriple>,
}

/// `rdf:type`, which the `a` keyword expands to.
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

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

    /// Merge another document's triples into this one, remapping the other
    /// document's blank-node ids so they stay distinct.
    pub fn merge(&mut self, other: &TurtleDocument) {
        let offset = self.next_blank_id();
        for triple in &other.triples {
            self.triples.push(TurtleTriple {
                subject: remap_blank(&triple.subject, offset),
                predicate: triple.predicate.clone(),
                object: remap_blank(&triple.object, offset),
            });
        }
    }

    fn next_blank_id(&self) -> usize {
        self.triples
            .iter()
            .flat_map(|triple| [&triple.subject, &triple.object])
            .filter_map(|term| match term {
                TurtleTerm::Blank(id) => Some(*id + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    /// All objects of (`subject`, `predicate`) triples, in document order.
    pub fn objects<'doc>(
        &'doc self,
        subject: &TurtleTerm,
        predicate: &str,
    ) -> impl Iterator<Item = &'doc TurtleTerm> + 'doc {
        let subject = subject.clone();
        let predicate = predicate.to_string();
        self.triples.iter().filter_map(move |triple| {
            (triple.subject == subject && triple.predicate == predicate).then_some(&triple.object)
        })
    }

    /// First object of (`subject`, `predicate`), if any.
    pub fn object(&self, subject: &TurtleTerm, predicate: &str) -> Option<&TurtleTerm> {
        self.objects(subject, predicate).next()
    }

    /// All IRI subjects carrying an `rdf:type` of `type_iri`.
    pub fn iri_subjects_of_type(&self, type_iri: &str) -> Vec<String> {
        let mut subjects = Vec::new();
        for triple in &self.triples {
            if triple.predicate == RDF_TYPE
                && triple.object == TurtleTerm::Iri(type_iri.to_string())
            {
                if let TurtleTerm::Iri(subject) = &triple.subject {
                    if !subjects.contains(subject) {
                        subjects.push(subject.clone());
                    }
                }
            }
        }
        subjects
    }

    /// Whether `subject` carries an `rdf:type` of `type_iri`.
    pub fn has_type(&self, subject: &TurtleTerm, type_iri: &str) -> bool {
        self.objects(subject, RDF_TYPE)
            .any(|object| object.as_iri() == Some(type_iri))
    }
}

fn remap_blank(term: &TurtleTerm, offset: usize) -> TurtleTerm {
    match term {
        TurtleTerm::Blank(id) => TurtleTerm::Blank(id + offset),
        other => other.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    const LV2: &str = "http://lv2plug.in/ns/lv2core#";

    fn iri(text: &str) -> TurtleTerm {
        TurtleTerm::Iri(text.to_string())
    }

    #[test]
    fn parses_prefixes_continuations_and_object_lists() {
        let doc = TurtleDocument::parse(
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             @prefix doap: <http://usefulinc.com/ns/doap#> .\n\
             # a comment\n\
             <http://example.com/p> a lv2:Plugin ;\n\
                 doap:name \"Example\" ;\n\
                 lv2:optionalFeature lv2:hardRTCapable , <http://x#y> ;\n\
                 .\n",
        )
        .expect("subset document parses");
        let subject = iri("http://example.com/p");
        assert!(doc.has_type(&subject, &format!("{LV2}Plugin")));
        assert_eq!(
            doc.object(&subject, "http://usefulinc.com/ns/doap#name"),
            Some(&TurtleTerm::Literal("Example".into())),
        );
        let features: Vec<_> = doc
            .objects(&subject, &format!("{LV2}optionalFeature"))
            .collect();
        assert_eq!(
            features,
            vec![&iri(&format!("{LV2}hardRTCapable")), &iri("http://x#y")],
        );
    }

    #[test]
    fn parses_blank_node_port_lists_with_numbers() {
        let doc = TurtleDocument::parse(
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <http://example.com/p>\n\
                 lv2:port [\n\
                     a lv2:AudioPort , lv2:InputPort ;\n\
                     lv2:index 0 ;\n\
                     lv2:symbol \"in_l\" ;\n\
                 ] , [\n\
                     a lv2:ControlPort , lv2:InputPort ;\n\
                     lv2:index 1 ;\n\
                     lv2:default 0.5 ;\n\
                     lv2:minimum -1.0 ;\n\
                     lv2:maximum 1e1 ;\n\
                 ] .\n",
        )
        .expect("port shape parses");
        let subject = iri("http://example.com/p");
        let ports: Vec<_> = doc.objects(&subject, &format!("{LV2}port")).collect();
        assert_eq!(ports.len(), 2);
        let control = ports[1].clone();
        assert!(doc.has_type(&control, &format!("{LV2}ControlPort")));
        assert_eq!(
            doc.object(&control, &format!("{LV2}index"))
                .and_then(TurtleTerm::as_number),
            Some(1.0),
        );
        assert_eq!(
            doc.object(&control, &format!("{LV2}default"))
                .and_then(TurtleTerm::as_number),
            Some(0.5),
        );
        assert_eq!(
            doc.object(&control, &format!("{LV2}minimum"))
                .and_then(TurtleTerm::as_number),
            Some(-1.0),
        );
        assert_eq!(
            doc.object(&control, &format!("{LV2}maximum"))
                .and_then(TurtleTerm::as_number),
            Some(10.0),
        );
    }

    #[test]
    fn parses_string_escapes_language_tags_and_datatypes() {
        let doc = TurtleDocument::parse(
            "@prefix doap: <http://usefulinc.com/ns/doap#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             <http://e/p> doap:name \"Say \\\"hi\\\"\\n\"@en ;\n\
                 doap:shortdesc \"typed\"^^xsd:string .\n",
        )
        .expect("literal forms parse");
        let subject = iri("http://e/p");
        assert_eq!(
            doc.object(&subject, "http://usefulinc.com/ns/doap#name")
                .and_then(TurtleTerm::as_literal),
            Some("Say \"hi\"\n"),
        );
        assert_eq!(
            doc.object(&subject, "http://usefulinc.com/ns/doap#shortdesc")
                .and_then(TurtleTerm::as_literal),
            Some("typed"),
        );
    }

    #[test]
    fn parses_booleans_and_labeled_blank_nodes() {
        let doc = TurtleDocument::parse(
            "@prefix ex: <http://e#> .\n\
             _:b ex:flag true .\n\
             _:b ex:other false .\n",
        )
        .expect("labeled blanks parse");
        assert_eq!(doc.triples.len(), 2);
        assert_eq!(doc.triples[0].subject, doc.triples[1].subject);
        assert_eq!(doc.triples[0].object, TurtleTerm::Bool(true));
        assert_eq!(doc.triples[1].object, TurtleTerm::Bool(false));
    }

    #[test]
    fn rejects_constructs_outside_the_subset_without_panicking() {
        let rejected = [
            "@base <http://example.com/> .",
            "PREFIX ex: <http://e#>\n<http://p> ex:x 1 .",
            "@prefix ex: <http://e#> .\n<http://p> ex:list ( 1 2 ) .",
            "@prefix ex: <http://e#> .\n<http://p> ex:name \"\"\"long\"\"\" .",
            "@prefix ex: <http://e#> .\n[ ex:x 1 ] ex:y 2 .",
            "<http://p> unknown:pred 1 .",
            "@prefix ex: <http://e#> .\n<http://p> ex:x \"unterminated .",
            "@prefix ex: <http://e#> .\n<http://p> ex:x 1",
        ];
        for source in rejected {
            assert!(
                TurtleDocument::parse(source).is_err(),
                "should reject: {source}",
            );
        }
    }

    #[test]
    fn merge_keeps_blank_nodes_distinct() {
        let mut left =
            TurtleDocument::parse("@prefix ex: <http://e#> .\n<http://a> ex:port [ ex:index 0 ] .")
                .expect("left parses");
        let right =
            TurtleDocument::parse("@prefix ex: <http://e#> .\n<http://b> ex:port [ ex:index 1 ] .")
                .expect("right parses");
        left.merge(&right);
        let a_port = left
            .object(&iri("http://a"), "http://e#port")
            .cloned()
            .expect("left port");
        let b_port = left
            .object(&iri("http://b"), "http://e#port")
            .cloned()
            .expect("right port");
        assert_ne!(a_port, b_port, "blank ids must not collide across merges");
        assert_eq!(
            left.object(&b_port, "http://e#index")
                .and_then(TurtleTerm::as_number),
            Some(1.0),
        );
    }
}
