use super::MeterSuppressionProfile;
use crate::rhythm_policy::*;
use signal_analysis::Confidence;

pub fn push_cause(
    slots: &mut [Option<MeterContinuityCause>; 3],
    count: &mut usize,
    cause: MeterContinuityCause,
) {
    if slots.iter().flatten().any(|existing| *existing == cause) {
        return;
    }
    if *count < slots.len() {
        slots[*count] = Some(cause);
        *count += 1;
    }
}

#[derive(Clone, Copy)]
pub struct MeterContinuityCauseInputs {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub trigger: MeterContinuityTrigger,
    pub suppression_profile: MeterSuppressionProfile,
    pub tempo_ambiguity: Confidence,
    pub phase_displaced: bool,
    pub stage_index: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityStageContext {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub stage_index: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityArcInputs {
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub current: MeterContinuityHistory,
    pub refresh: MeterContinuityTransition,
    pub first_decay: MeterContinuityTransition,
    pub final_decay: MeterContinuityTransition,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityPlanInputs {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
}

#[derive(Clone, Copy)]
pub struct MeterContinuityInputs<'a> {
    pub estimate: Option<&'a MeterEstimate>,
    pub suppression_profile: MeterSuppressionProfile,
    pub confidence: Confidence,
    pub tempo_ambiguity: Confidence,
    pub bpm: f32,
    pub beat_positions_seconds: &'a [f32],
}
