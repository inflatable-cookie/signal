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
}

include!("lexer.rs");
include!("statements.rs");
include!("literals.rs");
