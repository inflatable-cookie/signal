use signal_analysis::AnalysisStage;
use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
use signal_analysis_embed::{SemanticEmbedder, SemanticEmbedderConfig};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

fn main() {
    let preset = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tone".to_string());
    let audio = match preset.as_str() {
        "noise" => noise_audio(2.0, 48_000, 0.5),
        "pulse" => adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
        _ => sine_audio(440.0, 2.0, 48_000, 1.0),
    };

    let mut character = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let character_result = character.analyze(&audio);

    let mut semantic = SemanticEmbedder::new(SemanticEmbedderConfig::default())
        .expect("built-in semantic model should load");
    let semantic_result = semantic.analyze(&audio);
    let top_tag = &semantic_result.semantic_tags[0];

    println!("preset={preset}");
    println!("character_confidence={:.4}", character_result.confidence.0);
    println!(
        "character_spectral=centroid_hz:{:.2},flatness:{:.5},spread_hz:{:.2}",
        character_result.spectral_shape.centroid_hz,
        character_result.spectral_shape.flatness,
        character_result.spectral_shape.spread_hz,
    );
    println!(
        "character_temporal=onset_density:{:.4},transient_density:{:.4},sustain_ratio:{:.4},peak_transient_strength:{:.4}",
        character_result.temporal.onset_density,
        character_result.temporal.transient_density,
        character_result.temporal.sustain_ratio,
        character_result.temporal_shape.peak_transient_strength,
    );
    println!(
        "character_dynamics=rms_energy:{:.4},peak_amplitude:{:.4},dynamic_range:{:.4}",
        character_result.dynamics.rms_energy,
        character_result.dynamics.peak_amplitude,
        character_result.dynamics.dynamic_range,
    );
    println!("semantic_top_tag={:?}", top_tag.label);
    println!("semantic_driver={}", top_tag.evidence.primary_driver);
    println!("semantic_confidence={:.4}", top_tag.confidence.0);
    println!(
        "semantic_margin={:.4}",
        semantic_result.diagnostics.top_tag_margin
    );
}

fn sine_audio(
    frequency_hz: f32,
    seconds: f32,
    sample_rate: u32,
    amplitude: f32,
) -> AudioBuffer {
    let frames = (seconds * sample_rate as f32).round() as usize;
    let mut samples = Vec::with_capacity(frames);
    for index in 0..frames {
        let phase = core::f32::consts::TAU * frequency_hz * index as f32 / sample_rate as f32;
        samples.push(amplitude * phase.sin());
    }
    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn noise_audio(seconds: f32, sample_rate: u32, amplitude: f32) -> AudioBuffer {
    let frames = (seconds * sample_rate as f32).round() as usize;
    let mut state = 0x1234_5678u32;
    let mut samples = Vec::with_capacity(frames);
    for _ in 0..frames {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let value = (((state >> 8) & 0x00ff_ffff) as f32 / 0x00ff_ffff as f32) * 2.0 - 1.0;
        samples.push(value * amplitude);
    }
    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

fn adsr_pulse_audio(
    pulse_count: usize,
    bpm: u32,
    attack_ms: usize,
    release_ms: usize,
    sustain_frames: usize,
    sample_rate: u32,
    amplitude: f32,
) -> AudioBuffer {
    let frames_per_beat = ((60.0 / bpm as f32) * sample_rate as f32).round() as usize;
    let total_frames = pulse_count * frames_per_beat;
    let attack_frames = (attack_ms as f32 * sample_rate as f32 / 1_000.0).round() as usize;
    let release_frames = (release_ms as f32 * sample_rate as f32 / 1_000.0).round() as usize;
    let mut samples = vec![0.0f32; total_frames];

    for pulse_index in 0..pulse_count {
        let pulse_start = pulse_index * frames_per_beat;
        for frame_offset in 0..attack_frames {
            let frame = pulse_start + frame_offset;
            if frame >= samples.len() {
                break;
            }
            let level = frame_offset as f32 / attack_frames.max(1) as f32;
            samples[frame] = amplitude * level;
        }
        for sustain_offset in 0..sustain_frames {
            let frame = pulse_start + attack_frames + sustain_offset;
            if frame >= samples.len() {
                break;
            }
            samples[frame] = amplitude;
        }
        for frame_offset in 0..release_frames {
            let frame = pulse_start + attack_frames + sustain_frames + frame_offset;
            if frame >= samples.len() {
                break;
            }
            let level = 1.0 - frame_offset as f32 / release_frames.max(1) as f32;
            samples[frame] = amplitude * level.max(0.0);
        }
    }

    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}
