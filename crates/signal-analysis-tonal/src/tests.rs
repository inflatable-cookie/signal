#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        cents_offset_from_standard, reference_hz_from_cents, HarmonicChangeKind, KeyDetector,
        KeyDetectorConfig, KeyMode, KeyProfile, TonalAmbiguityKind, Tonic, TuningReferenceMode,
        TuningReferenceSource,
    };
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn tonal_mix(sample_rate: u32, freqs: &[f32], seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = vec![0.0f32; frames];
        let scale = if freqs.is_empty() {
            0.0
        } else {
            1.0 / freqs.len() as f32
        };

        for (index, sample) in samples.iter_mut().enumerate() {
            let t = index as f32 / sample_rate as f32;
            let mut value = 0.0;
            for freq in freqs {
                value += (core::f32::consts::TAU * *freq * t).sin();
            }
            *sample = value * scale;
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn detuned_tonal_mix(
        sample_rate: u32,
        freqs: &[f32],
        seconds: f32,
        reference_hz: f32,
    ) -> AudioBuffer {
        let ratio = reference_hz / 440.0;
        let detuned: Vec<f32> = freqs.iter().map(|frequency| frequency * ratio).collect();
        tonal_mix(sample_rate, &detuned, seconds)
    }

    fn tonal_sequence_mix(sample_rate: u32, sections: &[(&[f32], f32)]) -> AudioBuffer {
        let mut samples = Vec::new();
        for (freqs, seconds) in sections {
            samples.extend_from_slice(tonal_mix(sample_rate, freqs, *seconds).samples());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn tonic_metric(key: Option<crate::Key>) -> f32 {
        match key.map(|key| key.tonic) {
            Some(Tonic::C) => 0.0,
            Some(Tonic::Cs) => 1.0,
            Some(Tonic::D) => 2.0,
            Some(Tonic::Ds) => 3.0,
            Some(Tonic::E) => 4.0,
            Some(Tonic::F) => 5.0,
            Some(Tonic::Fs) => 6.0,
            Some(Tonic::G) => 7.0,
            Some(Tonic::Gs) => 8.0,
            Some(Tonic::A) => 9.0,
            Some(Tonic::As) => 10.0,
            Some(Tonic::B) => 11.0,
            None => -1.0,
        }
    }

    fn mode_metric(key: Option<crate::Key>) -> f32 {
        match key.map(|key| key.mode) {
            Some(KeyMode::Major) => 0.0,
            Some(KeyMode::Minor) => 1.0,
            None => -1.0,
        }
    }

    fn count_ambiguities(result: &crate::TonalAnalysisResult, kind: TonalAmbiguityKind) -> usize {
        result
            .local_tracking
            .ambiguities
            .iter()
            .filter(|ambiguity| ambiguity.kind == kind)
            .count()
    }

    fn tonal_metrics(result: &crate::TonalAnalysisResult) -> Vec<AnalysisMetricValue> {
        let first_segment = result
            .local_tracking
            .segments
            .first()
            .and_then(|segment| segment.key);
        let last_segment = result
            .local_tracking
            .segments
            .last()
            .and_then(|segment| segment.key);

        vec![
            AnalysisMetricValue::new("key_tonic", tonic_metric(result.key)),
            AnalysisMetricValue::new("key_mode", mode_metric(result.key)),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("tuning_reference_hz", result.tuning.reference_hz),
            AnalysisMetricValue::new("tuning_cents_offset", result.tuning.cents_offset),
            AnalysisMetricValue::new(
                "local_segment_count",
                result.local_tracking.segments.len() as f32,
            ),
            AnalysisMetricValue::new(
                "local_change_count",
                result.local_tracking.changes.len() as f32,
            ),
            AnalysisMetricValue::new(
                "local_ambiguity_count",
                result.local_tracking.ambiguities.len() as f32,
            ),
            AnalysisMetricValue::new(
                "modulation_ambiguity_count",
                count_ambiguities(result, TonalAmbiguityKind::Modulation) as f32,
            ),
            AnalysisMetricValue::new("first_segment_tonic", tonic_metric(first_segment)),
            AnalysisMetricValue::new("last_segment_tonic", tonic_metric(last_segment)),
        ]
    }

    fn tonal_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:c-major-triad",
                    AnalysisCorpusFamily::Tonal,
                    "Stable C-major global and local key reference",
                ),
                tonal_mix(48_000, &[261.63, 329.63, 392.0], 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "key_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "key_mode",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.01),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_reference_hz",
                    Some(438.0),
                    Some(442.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "local_ambiguity_count",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:detuned-c-major-432",
                    AnalysisCorpusFamily::RatePolicy,
                    "Detuned tuning-reference reference",
                ),
                detuned_tonal_mix(48_000, &[261.63, 329.63, 392.0], 5.0, 432.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "key_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "key_mode",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_reference_hz",
                    Some(429.5),
                    Some(434.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "tuning_cents_offset",
                    Some(-40.0),
                    Some(-20.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "tonal:modulation-c-to-g",
                    AnalysisCorpusFamily::Tonal,
                    "Section-local modulation and ambiguity reference",
                ),
                tonal_sequence_mix(
                    48_000,
                    &[
                        (&[261.63, 329.63, 392.0], 6.0),
                        (&[196.0, 246.94, 293.66], 6.0),
                    ],
                ),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "local_segment_count",
                    Some(2.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "local_change_count",
                    Some(1.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "modulation_ambiguity_count",
                    Some(1.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "first_segment_tonic",
                    Some(0.0),
                    Some(0.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "last_segment_tonic",
                    Some(7.0),
                    Some(7.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    mod global_detection;
    mod local_tracking;
}
