// Tests for signal-analysis-loudness
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        LoudnessAnalysisResult, LoudnessMeter, LoudnessMeterConfig, RUNTIME_MOMENTARY_TAIL_POINTS,
        RUNTIME_SHORT_TERM_TAIL_POINTS,
    };
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = Vec::with_capacity(frames);
        for index in 0..frames {
            let t = index as f32 / sample_rate as f32;
            samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn sine_sequence(sample_rate: u32, sections: &[(f32, f32, f32)]) -> AudioBuffer {
        let mut samples = Vec::new();
        for (frequency, amplitude, seconds) in sections {
            samples
                .extend_from_slice(sine(sample_rate, *frequency, *amplitude, *seconds).samples());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn loudness_metrics(result: &LoudnessAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("integrated_lufs", result.integrated_lufs),
            AnalysisMetricValue::new("true_peak_dbtp", result.true_peak_dbtp),
            AnalysisMetricValue::new("loudness_range_lu", result.loudness_range_lu),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("momentary_range_lu", result.dynamics.momentary_range_lu),
        ]
    }

    fn loudness_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:quiet-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Quiet tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.1, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-20.5),
                    Some(-19.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:loud-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Loud tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.5, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-6.5),
                    Some(-5.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:level-step",
                    AnalysisCorpusFamily::Loudness,
                    "Two-section level-step range reference",
                ),
                sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "loudness_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "momentary_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    mod acceptance_and_diagnostics;
    mod known_answer;
    mod meter_behavior;
}
