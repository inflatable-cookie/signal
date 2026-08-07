use super::support::*;
use super::*;

#[test]
fn degree_frequency_derivation_is_bit_identical_to_the_u8_pitch_formula() {
    // g12.034 widening compatibility pin: a degree with no pitch intent
    // must derive the EXACT bits the pre-widening
    // `440 * 2^((pitch - 69) / 12)` path produced for every u8 pitch.
    for pitch in 0u8..=127 {
        let old = 440.0 * f64::powf(2.0, (f64::from(pitch) - 69.0) / 12.0);
        let new = note(0, 1, i32::from(pitch), 1.0).frequency_hz();
        assert_eq!(old.to_bits(), new.to_bits(), "diverged at pitch {pitch}");
    }
}

#[test]
fn pitch_intent_overrides_or_offsets_the_degree_frequency() {
    let mut absolute = note(0, 1, 69, 1.0);
    absolute.pitch_intent = Some(RenderPitchIntent::FrequencyHz(432.0));
    assert_eq!(absolute.frequency_hz(), 432.0);

    let mut offset = note(0, 1, 69, 1.0);
    offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(1200.0));
    assert!((offset.frequency_hz() - 880.0).abs() < 1e-9);

    let mut zero_offset = note(0, 1, 69, 1.0);
    zero_offset.pitch_intent = Some(RenderPitchIntent::CentsOffset(0.0));
    assert_eq!(zero_offset.frequency_hz(), 440.0);
}
#[test]
fn note_clips_render_at_the_note_frequency() {
    // A4 (pitch 69) sustained for one second: past the attack and clip
    // edge fade, the output must be a 440 Hz sine at the velocity
    // amplitude. Quadrature projection at 440 Hz recovers the amplitude
    // and the residual bounds everything that is not that sine.
    let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let left = render_notes_left(&spec, 0, 24_000);

    let start = 4_800usize; // Past attack (144 frames) and edge fade.
    let count = 14_400usize; // Whole periods of 440 at 48k every 300.
    let mut in_phase = 0.0f64;
    let mut quadrature = 0.0f64;
    for index in 0..count {
        let n = (start + index) as f64;
        let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
        let sample = f64::from(left[start + index]);
        in_phase += sample * angle.sin();
        quadrature += sample * angle.cos();
    }
    let amplitude = 2.0 * (in_phase * in_phase + quadrature * quadrature).sqrt() / count as f64;
    assert!(
        (amplitude - 1.0).abs() < 0.01,
        "440 Hz amplitude read {amplitude}",
    );
    // Residual after removing the projected 440 Hz component: > 60 dB
    // below the tone (proof the output is that sine, not something else).
    let sine_gain = 2.0 * in_phase / count as f64;
    let cosine_gain = 2.0 * quadrature / count as f64;
    let mut error = 0.0f64;
    let mut power = 0.0f64;
    for index in 0..count {
        let n = (start + index) as f64;
        let angle = std::f64::consts::TAU * 440.0 * n / 48_000.0;
        let expected = sine_gain * angle.sin() + cosine_gain * angle.cos();
        let actual = f64::from(left[start + index]);
        error += (actual - expected) * (actual - expected);
        power += expected * expected;
    }
    let snr = 10.0 * (power / error.max(1e-30)).log10();
    assert!(snr > 60.0, "note tone SNR {snr:.1} dB");
}

#[test]
fn note_envelope_is_silent_before_attacks_and_releases() {
    // Note at frame 4_800, 4_800 frames long: silence before the start,
    // ramping attack (3 ms = 144 frames), full velocity through the
    // sustain, and a 40 ms release tail that ends in exact silence.
    let buffer = note_buffer(vec![note(4_800, 4_800, 69, 1.0)]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let left = render_notes_left(&spec, 0, 24_000);

    assert!(
        left[..4_800].iter().all(|sample| *sample == 0.0),
        "audio before the note start",
    );
    let peak =
        |range: std::ops::Range<usize>| left[range].iter().fold(0.0f32, |max, s| max.max(s.abs()));
    // Attack: the first 72 frames stay under the half-ramped level.
    assert!(peak(4_800..4_872) < 0.55, "attack did not ramp");
    // Sustain: full velocity once the attack completes.
    let sustain_peak = peak(5_200..9_000);
    assert!(
        (sustain_peak - 1.0).abs() < 0.05,
        "sustain peak {sustain_peak}",
    );
    // Release: decaying after the note end...
    let release_start = 4_800 + 4_800;
    let release_end = release_start + 1_920; // 40 ms at 48 kHz.
    assert!(peak(release_start + 960..release_end) < 0.6, "release flat");
    // ...and exactly silent once the tail ends.
    assert!(
        left[release_end + 1..].iter().all(|sample| *sample == 0.0),
        "audio after the release tail",
    );
}

#[test]
fn chords_render_as_the_sum_of_their_notes() {
    let pitches = [60i32, 64, 67];
    let chord = note_buffer(pitches.iter().map(|p| note(0, 24_000, *p, 0.3)).collect());
    let chord_left = render_notes_left(&notes_spec(&chord, 0, u64::MAX), 0, 12_000);

    let mut summed = vec![0.0f32; 12_000];
    for pitch in pitches {
        let single = note_buffer(vec![note(0, 24_000, pitch, 0.3)]);
        let left = render_notes_left(&notes_spec(&single, 0, u64::MAX), 0, 12_000);
        for (accumulator, sample) in summed.iter_mut().zip(left.iter()) {
            *accumulator += *sample;
        }
    }
    for (index, (chord_sample, sum_sample)) in chord_left.iter().zip(summed.iter()).enumerate() {
        assert!(
            (chord_sample - sum_sample).abs() < 1e-5,
            "chord diverged from note sum at {index}: {chord_sample} vs {sum_sample}",
        );
    }
}

#[test]
fn seeking_into_a_sustained_note_reproduces_played_through_samples() {
    // Statelessness proof: rendering from a mid-note seek must produce
    // the SAME bits as playing through from zero — there is no voice
    // state whose absence a seek could expose.
    let buffer = note_buffer(vec![
        note(0, 48_000, 57, 0.8),
        note(6_000, 30_000, 64, 0.6),
        note(12_000, 12_000, 72, 1.0),
    ]);
    let spec = notes_spec(&buffer, 0, u64::MAX);
    let played_through = render_notes_left(&spec, 0, 48_000);
    let seeked = render_notes_left(&spec, 24_000, 4_800);
    for (index, (seek_sample, through_sample)) in seeked
        .iter()
        .zip(played_through[24_000..24_000 + 4_800].iter())
        .enumerate()
    {
        assert_eq!(
            seek_sample.to_bits(),
            through_sample.to_bits(),
            "seek diverged from play-through at offset {index}",
        );
    }
}

#[test]
fn note_polyphony_caps_at_the_limit_keeping_earliest_started() {
    // 33 simultaneous notes: the render must equal the first 32 alone
    // (the 33rd — latest in sorted order — is skipped), and dropping to
    // 31 must change the output (the cap has teeth).
    let make = |count: usize| {
        note_buffer(
            (0..count)
                .map(|index| note(index as u64, 24_000, 40 + index as i32, 0.02))
                .collect(),
        )
    };
    let render =
        |count: usize| render_notes_left(&notes_spec(&make(count), 0, u64::MAX), 0, 12_000);
    let with_33 = render(33);
    let with_32 = render(32);
    let with_31 = render(31);
    assert_eq!(
        with_33
            .iter()
            .zip(with_32.iter())
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count(),
        0,
        "the 33rd simultaneous note leaked past the polyphony cap",
    );
    assert!(
        with_32
            .iter()
            .zip(with_31.iter())
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "32nd note should be audible (cap test has no teeth)",
    );
}

#[test]
fn unsorted_note_buffers_are_rejected_at_compile() {
    let (mut controller, _executor) = render_plane();
    let buffer = note_buffer(vec![note(1_000, 100, 60, 1.0), note(0, 100, 62, 1.0)]);
    let error = controller
        .install_plan(&notes_spec(&buffer, 0, u64::MAX))
        .unwrap_err();
    assert!(error.message.contains("sorted"), "{}", error.message);
}

#[test]
fn note_buffers_compare_by_pointer_for_cheap_spec_equality() {
    let notes: Arc<[RenderNote]> = vec![note(0, 100, 60, 1.0)].into();
    let a = RenderNoteBuffer {
        notes: Arc::clone(&notes),
    };
    let b = RenderNoteBuffer { notes };
    let c = note_buffer(vec![note(0, 100, 60, 1.0)]);
    assert_eq!(a, b);
    assert_ne!(a, c);
    // Spec equality follows buffer equality: idempotent recompiles.
    assert_eq!(notes_spec(&a, 0, 100), notes_spec(&b, 0, 100));
    assert_ne!(notes_spec(&a, 0, 100), notes_spec(&c, 0, 100));
}

#[test]
fn note_clip_windows_gate_notes_on_the_stream_clock() {
    // Clip windowed [1_000, 2_000): a clip-relative note at 0 sounds at
    // stream frame 1_000, and nothing sounds past the window end even
    // though the note's tail extends beyond it.
    let buffer = note_buffer(vec![note(0, 48_000, 69, 1.0)]);
    let spec = notes_spec(&buffer, 1_000, 2_000);
    let left = render_notes_left(&spec, 0, 4_000);
    assert!(left[..1_000].iter().all(|sample| *sample == 0.0));
    assert!(
        left[1_100..1_900].iter().any(|sample| sample.abs() > 0.5),
        "windowed note inaudible",
    );
    assert!(left[2_000..].iter().all(|sample| *sample == 0.0));
}
