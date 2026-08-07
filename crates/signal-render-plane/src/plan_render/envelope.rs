/// Sample a sorted `(frame, value)` automation envelope at `frame`: linear
/// interpolation between breakpoints, clamped to the first/last values
/// outside the span. Binary search + arithmetic only — audio-thread safe.
#[inline]
pub(crate) fn sample_envelope(points: &[(u64, f32)], frame: u64) -> f32 {
    match points.binary_search_by(|(point_frame, _)| point_frame.cmp(&frame)) {
        Ok(index) => points[index].1,
        Err(0) => points[0].1,
        Err(index) if index == points.len() => points[points.len() - 1].1,
        Err(index) => {
            let (start_frame, start_value) = points[index - 1];
            let (end_frame, end_value) = points[index];
            let span = (end_frame - start_frame).max(1) as f64;
            let progress = (frame - start_frame) as f64 / span;
            start_value + (end_value - start_value) * progress as f32
        }
    }
}

/// Window gain for a frame inside a clip: per side, an explicit equal-power
/// fade when requested, otherwise the linear edge declick.
///
/// Start side with `fade_in_frames > 0`: `sin(π/2 · p/F)` over positions
/// `p = frame - start_frames ∈ [0, F)` (the first frame is exactly 0 — that
/// exactness is what makes an overlapped fade-out/fade-in pair sum to unity
/// POWER at every frame: the two quarter-wave arguments are complementary).
/// End side with `fade_out_frames > 0`: the mirror, `sin(π/2 · r/F)` over
/// `r = end_frames - frame ∈ (0, F]`. A requested fade REPLACES the declick
/// on its side; a zero-fade side keeps the declick ramp, byte-identical to
/// the historical behavior. The sides combine via `min`, which reproduces
/// the historical declick-only expression exactly and is inert wherever the
/// spans do not overlap.
///
/// Pure function of the frame position — stateless, so fades are correct
/// across block boundaries, seeks into the middle of a fade, and plan swaps.
#[inline]
pub(crate) fn clip_window_gain(
    frame: u64,
    start_frames: u64,
    end_frames: u64,
    edge_fade_frames: u64,
    fade_in_frames: u64,
    fade_out_frames: u64,
) -> f32 {
    let in_gain = if fade_in_frames > 0 {
        let position = frame - start_frames;
        if position < fade_in_frames {
            (std::f32::consts::FRAC_PI_2 * position as f32 / fade_in_frames as f32).sin()
        } else {
            1.0
        }
    } else if edge_fade_frames > 0 {
        ((frame - start_frames + 1) as f32 / edge_fade_frames as f32).min(1.0)
    } else {
        1.0
    };
    let out_gain = if fade_out_frames > 0 {
        let remaining = end_frames - frame;
        if remaining < fade_out_frames {
            (std::f32::consts::FRAC_PI_2 * remaining as f32 / fade_out_frames as f32).sin()
        } else {
            1.0
        }
    } else if edge_fade_frames > 0 {
        ((end_frames - frame) as f32 / edge_fade_frames as f32).min(1.0)
    } else {
        1.0
    };
    in_gain.min(out_gain)
}
