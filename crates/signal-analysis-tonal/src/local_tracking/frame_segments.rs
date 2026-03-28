use signal_dsp_spectral::{bin_frequency, Spectrogram};

use crate::{
    profile_scoring::score_chroma, KeyDetectorConfig, TonalSegmentSummary,
};

const SEMITONE_WIDTH_RATIO: f32 = 0.057_762_265;

pub(super) fn build_tonal_segments(
    spectrogram: &Spectrogram,
    config: KeyDetectorConfig,
    reference_hz: f32,
) -> Vec<TonalSegmentSummary> {
    let frame_chromas = spectrogram_frame_chromas(spectrogram, reference_hz);
    let frame_count = frame_chromas.len();
    if frame_count == 0 || spectrogram.sample_rate.0 == 0 {
        return Vec::new();
    }

    let frame_hop_seconds = spectrogram.config.hop_size.0 as f32 / spectrogram.sample_rate.0 as f32;
    let window_frames =
        ((config.section_window_seconds as f32 / frame_hop_seconds).round() as usize).max(1);
    let hop_frames =
        ((config.section_hop_seconds as f32 / frame_hop_seconds).round() as usize).max(1);
    let mut segments = Vec::new();
    let mut start = 0usize;

    loop {
        let end = (start + window_frames).min(frame_count);
        let chroma = aggregate_chroma(&frame_chromas[start..end]);
        let segment_result = score_chroma(chroma, config.profile);
        let start_seconds = frame_index_to_seconds(start, spectrogram);
        let end_seconds = frame_end_to_seconds(end.saturating_sub(1), spectrogram);

        segments.push(TonalSegmentSummary {
            index: segments.len(),
            start_seconds,
            end_seconds,
            key: segment_result.key,
            confidence: segment_result.confidence,
            chroma,
            scoring: segment_result.scoring,
            ambiguity: None,
        });

        if end >= frame_count {
            break;
        }
        start = start.saturating_add(hop_frames);
    }

    segments
}

fn spectrogram_frame_chromas(spectrogram: &Spectrogram, reference_hz: f32) -> Vec<[f32; 12]> {
    let window_size = spectrogram.config.window_size.0;
    if spectrogram.frames.is_empty() || window_size == 0 || spectrogram.sample_rate.0 == 0 {
        return Vec::new();
    }

    let bin_spacing = spectrogram.sample_rate.0 as f32 / window_size as f32;
    let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;
    let mut chromas = Vec::with_capacity(spectrogram.frames.len());

    for frame in &spectrogram.frames {
        let mut chroma = [0.0; 12];
        for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
            let frequency = bin_frequency(bin_index, spectrogram.sample_rate, window_size);
            if frequency < min_frequency || frequency > 5_000.0 {
                continue;
            }
            let midi = 69.0 + 12.0 * (frequency / reference_hz.max(1.0)).log2();
            let pitch_class = (midi.round() as i32).rem_euclid(12) as usize;
            chroma[pitch_class] += *magnitude / frequency;
        }
        normalize_array(&mut chroma);
        chromas.push(chroma);
    }

    chromas
}

fn aggregate_chroma(frame_chromas: &[[f32; 12]]) -> [f32; 12] {
    let mut chroma = [0.0; 12];
    for frame in frame_chromas {
        for (slot, value) in chroma.iter_mut().zip(frame.iter().copied()) {
            *slot += value;
        }
    }
    normalize_array(&mut chroma);
    chroma
}

fn normalize_array(values: &mut [f32; 12]) {
    let max_value = values.iter().copied().fold(0.0f32, f32::max);
    if max_value > 0.0 {
        for value in values.iter_mut() {
            *value /= max_value;
        }
    }
}

fn frame_index_to_seconds(frame_index: usize, spectrogram: &Spectrogram) -> f32 {
    frame_index as f32 * spectrogram.config.hop_size.0 as f32 / spectrogram.sample_rate.0 as f32
}

fn frame_end_to_seconds(frame_index: usize, spectrogram: &Spectrogram) -> f32 {
    let end_samples = frame_index
        .saturating_mul(spectrogram.config.hop_size.0)
        .saturating_add(spectrogram.config.window_size.0);
    end_samples as f32 / spectrogram.sample_rate.0 as f32
}
