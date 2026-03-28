fn add_click(samples: &mut [f32], index: usize, amplitude: f32) {
    for offset in 0..CLICK_LENGTH {
        if let Some(sample) = samples.get_mut(index + offset) {
            *sample += amplitude * (1.0 - offset as f32 / CLICK_LENGTH as f32);
        }
    }
}

fn add_tone_burst(
    samples: &mut [f32],
    sample_rate: u32,
    index: usize,
    frequencies: &[f32],
    amplitude: f32,
) {
    for offset in 0..TONE_BURST_LENGTH {
        let Some(sample) = samples.get_mut(index + offset) else {
            break;
        };
        let t = offset as f32 / sample_rate as f32;
        let envelope = (1.0 - offset as f32 / TONE_BURST_LENGTH as f32).max(0.0);
        let tone = frequencies
            .iter()
            .copied()
            .map(|frequency| (core::f32::consts::TAU * frequency * t).sin())
            .sum::<f32>();
        *sample += amplitude * envelope * tone / frequencies.len().max(1) as f32;
    }
}

fn click_track(sample_rate: u32, bpm: f32, seconds: f32) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let mut samples = vec![0.0; frames];
    let interval = (60.0 / bpm * sample_rate as f32).round() as usize;

    let mut index = 0usize;
    while index < frames {
        add_click(&mut samples, index, 1.0);
        index = index.saturating_add(interval.max(1));
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn grid_click_track(
    sample_rate: u32,
    bpm: f32,
    steps_per_beat: usize,
    seconds: f32,
    pattern: &[f32],
    swing_ratio: Option<f32>,
) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let beat_frames = 60.0 / bpm * sample_rate as f32;
    let subdivision_frames = beat_frames / steps_per_beat.max(1) as f32;
    let mut samples = vec![0.0; frames];
    let total_steps = ((seconds * bpm / 60.0) * steps_per_beat as f32).ceil() as usize;

    for step in 0..total_steps {
        let amplitude = pattern[step % pattern.len()];
        if amplitude <= 0.0 {
            continue;
        }

        let beat_index = step / steps_per_beat.max(1);
        let step_in_beat = step % steps_per_beat.max(1);
        let offset_frames = if steps_per_beat == 2 {
            match (step_in_beat, swing_ratio) {
                (0, _) => 0.0,
                (1, Some(ratio)) => beat_frames * ratio.clamp(0.5, 0.85),
                _ => subdivision_frames,
            }
        } else {
            step_in_beat as f32 * subdivision_frames
        };
        let index = (beat_index as f32 * beat_frames + offset_frames).round() as usize;
        add_click(&mut samples, index, amplitude);
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn beat_sequence_track(
    sample_rate: u32,
    bpm: f32,
    beat_amplitudes: &[f32],
    tone_events: &[(usize, &'static [f32], f32)],
) -> AudioBuffer {
    let beat_frames = (60.0 / bpm * sample_rate as f32).round() as usize;
    let frames = beat_frames
        .saturating_mul(beat_amplitudes.len())
        .saturating_add(TONE_BURST_LENGTH);
    let mut samples = vec![0.0; frames];

    for (beat_index, amplitude) in beat_amplitudes.iter().copied().enumerate() {
        if amplitude > 0.0 {
            add_click(
                &mut samples,
                beat_index.saturating_mul(beat_frames),
                amplitude,
            );
        }
    }

    for (beat_index, frequencies, amplitude) in tone_events {
        add_tone_burst(
            &mut samples,
            sample_rate,
            beat_index.saturating_mul(beat_frames),
            frequencies,
            *amplitude,
        );
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn push_four_four_groove(
    beats: &mut Vec<f32>,
    tone_events: &mut Vec<(usize, &'static [f32], f32)>,
    start_beat: usize,
    section: GrooveSection,
) {
    for bar in 0..section.bars {
        let beat_pattern = section
            .bar_patterns
            .and_then(|patterns| patterns.get(bar).copied())
            .unwrap_or(section.beat_pattern);
        let is_dropout_bar = section.dropout_bars.contains(&bar);

        for (beat_in_bar, pattern_value) in beat_pattern.iter().copied().enumerate() {
            let beat_index = start_beat + bar * 4 + beat_in_bar;
            let beat_amplitude = if is_dropout_bar {
                0.35 * pattern_value
            } else {
                pattern_value
            };
            beats.push(beat_amplitude);

            if !is_dropout_bar {
                tone_events.push((beat_index, KICK_TONES, 0.18 * beat_amplitude));
                if beat_in_bar == 1 || beat_in_bar == 3 {
                    tone_events.push((beat_index, SNARE_TONES, 0.28));
                } else {
                    tone_events.push((beat_index, HAT_TONES, 0.12));
                }
            } else if beat_in_bar == 3 {
                tone_events.push((beat_index, HAT_TONES, 0.08));
            }
        }

        let bar_chord = section
            .bar_chords
            .and_then(|plan| plan.get(bar).copied())
            .or_else(|| {
                if bar % section.chord_every_bars == 0 {
                    Some(
                        section.chord_cycle
                            [(bar / section.chord_every_bars) % section.chord_cycle.len()],
                    )
                } else {
                    None
                }
            });
        if let Some(chord) = bar_chord {
            tone_events.push((
                start_beat + bar * 4,
                chord,
                if is_dropout_bar { 0.65 } else { 0.55 },
            ));
        }
    }

    if let Some((offset_beats, chord, amplitude)) = section.section_marker {
        tone_events.push((start_beat + offset_beats, chord, amplitude));
    }
}
