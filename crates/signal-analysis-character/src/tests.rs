#[cfg(test)]
mod tests {
    use crate::*;
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
        Confidence,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine_audio(
        frequency_hz: f32,
        duration_seconds: f32,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        for (index, sample) in data.iter_mut().enumerate() {
            let time = index as f32 / sample_rate_hz as f32;
            *sample = amplitude * (core::f32::consts::TAU * frequency_hz * time).sin();
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn noise_audio(duration_seconds: f32, sample_rate_hz: u32, amplitude: f32) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        let mut state = 0x1234_5678u32;
        for sample in &mut data {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let unit = ((state >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sample = amplitude * unit;
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn adsr_pulse_audio(
        attack_ms: u32,
        sustain_ms: u32,
        decay_ms: u32,
        interval_ms: u32,
        event_count: usize,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let interval_samples = (interval_ms as usize * sample_rate_hz as usize) / 1_000;
        let attack_samples = (attack_ms as usize * sample_rate_hz as usize) / 1_000;
        let sustain_samples = (sustain_ms as usize * sample_rate_hz as usize) / 1_000;
        let decay_samples = (decay_ms as usize * sample_rate_hz as usize) / 1_000;
        let total_samples = interval_samples * event_count.max(1);
        let mut data = vec![0.0f32; total_samples.max(1)];

        for event_index in 0..event_count {
            let start = event_index * interval_samples;

            for offset in 0..attack_samples {
                let index = start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = (offset + 1) as f32 / attack_samples.max(1) as f32;
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }

            let sustain_start = start + attack_samples;
            for offset in 0..sustain_samples {
                let index = sustain_start + offset;
                if index >= data.len() {
                    break;
                }
                data[index] = amplitude;
            }

            let decay_start = sustain_start + sustain_samples;
            for offset in 0..decay_samples {
                let index = decay_start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = 1.0 - ((offset + 1) as f32 / decay_samples.max(1) as f32);
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn character_metrics(result: &CharacterAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("spectral_flatness", result.spectral_shape.flatness),
            AnalysisMetricValue::new("spectral_spread_hz", result.spectral_shape.spread_hz),
            AnalysisMetricValue::new("rms_energy", result.dynamics.rms_energy),
            AnalysisMetricValue::new("sustain_ratio", result.temporal.sustain_ratio),
            AnalysisMetricValue::new(
                "peak_transient_strength",
                result.temporal_shape.peak_transient_strength,
            ),
            AnalysisMetricValue::new("descriptor_confidence", result.confidence.0),
        ]
    }

    fn character_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:tone:sine440",
                    AnalysisCorpusFamily::Tonal,
                    "Sustained tonal descriptor reference",
                ),
                sine_audio(440.0, 2.0, 48_000, 1.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_flatness",
                    Some(0.0),
                    Some(0.05),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.65),
                    Some(0.75),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:noise:deterministic",
                    AnalysisCorpusFamily::Noise,
                    "Broadband descriptor reference",
                ),
                noise_audio(2.0, 48_000, 0.5),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "spectral_spread_hz",
                    Some(2_000.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "rms_energy",
                    Some(0.45),
                    Some(0.55),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "sustain_ratio",
                    Some(0.95),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.15),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "character:pulse:adsr",
                    AnalysisCorpusFamily::Pulse,
                    "Transient-heavy descriptor reference",
                ),
                adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "peak_transient_strength",
                    Some(0.80),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.25),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    mod acceptance;
    mod spectral;
    mod temporal;
}
