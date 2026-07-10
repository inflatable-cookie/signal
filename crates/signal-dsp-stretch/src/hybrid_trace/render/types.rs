use signal_primitives::Sample;

use crate::{StretchHybridTrace, StretchHybridTransitionTrace};

/// Reason a report-only hybrid ownership span stayed on the current path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchHybridTransitionRejection {
    /// The outgoing and incoming branch windows were not sufficiently aligned.
    LowCorrelation,
    /// Correlation-aware transition gain exceeded the frozen bound.
    ExcessNormalization,
    /// The ownership span was too short to contain both frozen crossfades.
    SpanTooShort,
}

/// Report-only decision for one mixed/branch transition boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchHybridTransitionDecision {
    /// Scheduled transition inherited from the classifier trace.
    pub transition: StretchHybridTransitionTrace,
    /// Measured zero-lag outgoing/incoming correlation.
    pub correlation: f64,
    /// Maximum correlation-aware normalization gain required by the crossfade.
    pub max_normalization_gain_db: f64,
    /// Whether this boundary was applied to candidate audio.
    pub applied: bool,
    /// Rejection reason when the boundary stayed on the current path.
    pub rejection: Option<StretchHybridTransitionRejection>,
}

/// Report-only fixed-ratio mono structural-hybrid render and evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHybridRender {
    /// Candidate mono samples. Production routing never consumes this field.
    pub samples: Vec<Sample>,
    /// Frozen classifier and transition schedule used by the candidate.
    pub trace: StretchHybridTrace,
    /// Correlation and normalization decisions for attempted ownership spans.
    pub transition_decisions: Vec<StretchHybridTransitionDecision>,
    /// Number of transient or tonal spans applied to candidate audio.
    pub applied_span_count: usize,
    /// Number of spans kept entirely on the current path.
    pub rejected_span_count: usize,
}
