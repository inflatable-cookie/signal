pub(crate) struct MeterDecision {
    pub estimate: Option<MeterEstimate>,
    pub ambiguity: RhythmStructureAmbiguitySummary,
}

use crate::rhythm_policy::*;

mod meter_state_infer;
pub(crate) use meter_state_infer::infer_meter;
