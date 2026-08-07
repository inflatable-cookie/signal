/// Wrap a phase into `-PI..PI` by rounding.
///
/// See the crate-root `wrap_phase` for why this second implementation is
/// retained: the two forms are not bit-equivalent, so unifying them changes
/// rendered output and needs its own evidence.
pub(crate) fn wrap_phase(phase: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    phase - tau * (phase / tau).round()
}
