//! Stretch benchmark corpus, measurement, comparison, and report surfaces.

mod compare;
mod measure;
mod report;
mod synthetic;
mod types;

pub use compare::*;
pub use measure::*;
pub use report::*;
pub use synthetic::{
    generate_synthetic_stretch_audio, synthetic_stretch_corpus_cases, StretchSyntheticAudio,
};
pub use types::*;

#[cfg(test)]
mod a18_crossover_smear;
