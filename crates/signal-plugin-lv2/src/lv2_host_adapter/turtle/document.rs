//! Turtle term/document model for the LV2 manifest subset.

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

pub(crate) fn remap_blank(term: &TurtleTerm, offset: usize) -> TurtleTerm {
    match term {
        TurtleTerm::Blank(id) => TurtleTerm::Blank(id + offset),
        other => other.clone(),
    }
}
