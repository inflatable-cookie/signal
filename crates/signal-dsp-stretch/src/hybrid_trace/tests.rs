use super::*;

fn stable_sine(frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|index| (std::f32::consts::TAU * 17.0 * index as f32 / 1_024.0).sin() * 0.5)
        .collect()
}

#[test]
fn hybrid_trace_is_deterministic_and_does_not_mix_audio() {
    let input = stable_sine(8_192);
    let output = stable_sine(12_288);

    let first = build_hybrid_trace(&input, &output, 1.5);
    let repeated = build_hybrid_trace(&input, &output, 1.5);

    assert_eq!(first, repeated);
    assert_eq!(first.output_frames, output.len());
}

#[test]
fn hybrid_trace_qualifies_stable_expansion_as_tonal_after_hold() {
    let input = stable_sine(8_192);
    let output = stable_sine(12_288);
    let trace = build_hybrid_trace(&input, &output, 1.5);

    assert!(trace
        .frames
        .iter()
        .any(|frame| frame.owner == StretchHybridOwner::Tonal));
    assert!(trace
        .frames
        .iter()
        .filter(|frame| {
            frame.source_frame < DEFAULT_WINDOW_SIZE / 2
                || frame.source_frame + DEFAULT_WINDOW_SIZE / 2 >= input.len()
        })
        .all(|frame| frame.owner == StretchHybridOwner::Mixed));
}

#[test]
fn hybrid_trace_guards_sudden_attack_with_transient_owner() {
    let mut input = stable_sine(8_192);
    input[..4_096].fill(0.0);
    let output = vec![0.0; 12_288];
    let trace = build_hybrid_trace(&input, &output, 1.5);
    let transient_frames = trace
        .frames
        .iter()
        .filter(|frame| frame.owner == StretchHybridOwner::Transient)
        .count();

    assert!(transient_frames >= TRANSIENT_PREROLL_FRAMES + TRANSIENT_POSTROLL_FRAMES + 1);
}

#[test]
fn hybrid_trace_identity_stays_on_current_owner() {
    let input = stable_sine(8_192);
    let trace = build_hybrid_trace(&input, &input, 1.0);

    assert!(trace
        .frames
        .iter()
        .all(|frame| frame.owner == StretchHybridOwner::Mixed));
    assert!(trace.transitions.is_empty());
}

#[test]
fn hybrid_trace_compression_never_selects_tonal_owner() {
    let input = stable_sine(8_192);
    let output = stable_sine(6_144);
    let trace = build_hybrid_trace(&input, &output, 0.75);

    assert!(trace
        .frames
        .iter()
        .all(|frame| frame.owner != StretchHybridOwner::Tonal));
}

#[test]
fn hybrid_transition_search_is_bounded_and_avoids_transient_interior() {
    let mut input = stable_sine(8_192);
    input[..4_096].fill(0.0);
    let output = stable_sine(12_288);
    let trace = build_hybrid_trace(&input, &output, 1.5);
    let search_bound = (COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP as f64 * 1.5).round() as i64;

    assert!(!trace.transitions.is_empty());
    assert!(trace
        .transitions
        .iter()
        .all(|transition| transition.search_offset_frames.abs() <= search_bound));
    assert!(trace.transitions.iter().all(|transition| {
        nearest_owner(&trace.frames, transition.scheduled_output_frame)
            != StretchHybridOwner::Transient
    }));
}
