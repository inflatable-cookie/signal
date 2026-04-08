use signal_analysis::Confidence;

#[derive(Clone, Copy)]
pub struct TempoStateScopeContext {
    pub boundary_pressure: Confidence,
    pub tempo_ambiguity: Confidence,
    pub base_confidence: f32,
    pub localized_edge_scope: bool,
    pub core_stable_scope: bool,
    pub mid_track_unstable_scope: bool,
    pub strong_integer_anchor: bool,
}

impl TempoStateScopeContext {
    pub fn localized_edge_horizons(self) -> (usize, usize, usize, usize, f32) {
        if self.boundary_pressure.0 >= 0.20 {
            (10, 6, 12, 18, 0.60)
        } else {
            (12, 8, 14, 20, 0.64)
        }
    }
}
