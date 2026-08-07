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

mod document;
mod parser;

pub use document::{TurtleDocument, TurtleTerm, TurtleTriple, RDF_TYPE};

#[cfg(test)]
mod tests;
