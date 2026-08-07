use signal_dsp::PolyphaseInterpolationTable;

/// Per-frame interpolated source read shared by the `Samples` and `Stream`
/// paths: polyphase windowed-sinc when rate-converted, clamped lerp at 1:1.
/// `fetch(source_frame, channel)` returns the source sample or `None` when
/// that frame is unavailable (off the buffer for samples, not currently
/// held for streams). Returns `None` — render silence, and for streams
/// count an underrun — when the center frame itself is unavailable;
/// unavailable outer sinc taps just contribute zero, matching the buffer
/// path's edge behavior. Arithmetic and accumulation order are identical to
/// the historical `Samples` implementation (golden-hash stable).
#[inline]
pub(crate) fn interpolate_source_frame(
    fetch: &impl Fn(i64, usize) -> Option<f32>,
    source_index: u64,
    fraction: f64,
    table: Option<&PolyphaseInterpolationTable>,
    channel: usize,
) -> Option<f32> {
    match table {
        // Rate conversion: polyphase windowed-sinc tap dot product (table
        // reads only — no allocation, no transcendentals).
        Some(table) => {
            fetch(source_index as i64, channel)?;
            let row = table.phase_row(fraction);
            let first = table.first_tap_offset();
            let mut acc = 0.0f32;
            for (tap, coefficient) in row.iter().enumerate() {
                let tap_index = source_index as i64 + first as i64 + tap as i64;
                if let Some(value) = fetch(tap_index, channel) {
                    acc += value * coefficient;
                }
            }
            Some(acc)
        }
        // 1:1 playback: direct read with last-frame clamp (`fetch` wraps
        // instead when the source loops).
        None => {
            let a = fetch(source_index as i64, channel)?;
            let b = fetch(source_index as i64 + 1, channel).unwrap_or(a);
            Some(a + (b - a) * fraction as f32)
        }
    }
}
