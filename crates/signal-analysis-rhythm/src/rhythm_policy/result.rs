use signal_analysis::Confidence;

use super::{MeterEstimate, RhythmStructureAmbiguitySummary};
use crate::{TempoCandidate, TempoDiagnostics, TempoInterpretation};

/// Full output of a single [`BeatTracker`](crate::BeatTracker) analysis pass.
#[derive(Clone, Debug, PartialEq)]
pub struct BeatAnalysisResult {
    /// Primary tempo estimate in beats per minute.
    pub bpm: f32,
    /// Overall confidence in the tempo and beat grid.
    pub confidence: Confidence,
    /// Beat onset times in seconds, in ascending order.
    pub beat_positions_seconds: Vec<f32>,
    /// Normalised onset-strength envelope used for tempo and beat estimation.
    pub onset_envelope: Vec<f32>,
    /// Ranked list of competing tempo hypotheses.
    pub tempo_candidates: Vec<TempoCandidate>,
    /// Detailed per-beat and windowed tempo stability diagnostics.
    pub tempo_diagnostics: TempoDiagnostics,
    /// High-level tempo trust level and snap recommendation.
    pub tempo_interpretation: TempoInterpretation,
    /// Confidence that a strong alternative tempo exists (higher = more ambiguous).
    pub tempo_ambiguity: Confidence,
    /// Meter estimate (beats per bar and downbeat positions), if detected.
    pub meter: Option<MeterEstimate>,
    /// Summary of any competing or ambiguous meter/downbeat hypotheses.
    pub structure_ambiguity: RhythmStructureAmbiguitySummary,
}
