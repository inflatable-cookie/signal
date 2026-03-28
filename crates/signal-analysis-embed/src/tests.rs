// Tests for signal-analysis-embed
#[allow(clippy::module_inception)]
mod tests {
    use crate::*;
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{ChannelLayout, SampleRate};

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

    fn top_label(result: &SemanticAnalysisResult) -> SemanticTagLabel {
        result.semantic_tags.first().map(|tag| tag.label).unwrap()
    }

    fn semantic_score(result: &SemanticAnalysisResult, label: SemanticTagLabel) -> f32 {
        result
            .semantic_tags
            .iter()
            .find(|tag| tag.label == label)
            .map(|tag| tag.score)
            .unwrap_or(0.0)
    }

    fn semantic_metrics(result: &SemanticAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new(
                "tonal_focus_score",
                semantic_score(result, SemanticTagLabel::TonalFocus),
            ),
            AnalysisMetricValue::new(
                "textural_noise_score",
                semantic_score(result, SemanticTagLabel::TexturalNoise),
            ),
            AnalysisMetricValue::new(
                "pulse_driven_score",
                semantic_score(result, SemanticTagLabel::PulseDriven),
            ),
            AnalysisMetricValue::new(
                "dynamic_punch_score",
                semantic_score(result, SemanticTagLabel::DynamicPunch),
            ),
            AnalysisMetricValue::new(
                "semantic_confidence",
                result.diagnostics.semantic_confidence.0,
            ),
            AnalysisMetricValue::new(
                "descriptor_confidence",
                result.diagnostics.descriptor_confidence.0,
            ),
        ]
    }

    fn semantic_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:tone:sine440",
                    AnalysisCorpusFamily::Semantic,
                    "Tonal semantic reference",
                ),
                sine_audio(440.0, 2.0, 48_000, 1.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "tonal_focus_score",
                    Some(0.60),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:noise:deterministic",
                    AnalysisCorpusFamily::Semantic,
                    "Noise semantic reference",
                ),
                noise_audio(2.0, 48_000, 0.5),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "textural_noise_score",
                    Some(0.50),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:pulse:adsr",
                    AnalysisCorpusFamily::Semantic,
                    "Pulse semantic reference",
                ),
                adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "pulse_driven_score",
                    Some(0.40),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "dynamic_punch_score",
                    Some(0.40),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
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

    mod model_contract;
    mod semantic_behavior;
}
