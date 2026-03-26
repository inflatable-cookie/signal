use super::*;

#[test]
fn refine_bpm_from_beats_ignores_terminal_outlier_intervals() {
    let stable_interval_frames = 46.875;
    let intervals = [
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames,
        stable_interval_frames * 1.23,
        stable_interval_frames * 0.84,
        stable_interval_frames * 1.32,
        stable_interval_frames,
    ];
    let mut beat_frames = Vec::with_capacity(intervals.len() + 1);
    let mut current = 0.0;
    beat_frames.push(current);
    for interval in intervals {
        current += interval;
        beat_frames.push(current);
    }

    let refined = super::refine_bpm_from_beats(127.97321, &beat_frames, SampleRate(48_000), 512);

    assert!((refined - 128.0).abs() < 0.05, "refined bpm {}", refined);
}
