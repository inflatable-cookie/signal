fn rhythm_acceptance_cases() -> Vec<AnalysisCorpusCase> {
    let sample_rate = 48_000;
    vec![
        AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "rhythm:steady-click120",
                AnalysisCorpusFamily::Pulse,
                "Stable click-track tempo reference",
            ),
            click_track(sample_rate, 120.0, 8.0),
        )
        .with_acceptance_thresholds(vec![
            signal_analysis::AcceptanceThreshold::range(
                "bpm",
                Some(119.9),
                Some(120.1),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "confidence",
                Some(0.2),
                Some(1.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "tempo_ambiguity",
                Some(0.0),
                Some(0.4),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "has_meter",
                Some(0.0),
                Some(0.0),
                AcceptanceSeverity::Fail,
            ),
        ]),
        AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "rhythm:structured-harmony120",
                AnalysisCorpusFamily::Pulse,
                "Structured meter reference with stable whole-track bar grid",
            ),
            build_structured_harmony_preset(sample_rate, 120.0, HarmonicRhythmVariant::Active),
        )
        .with_acceptance_thresholds(vec![
            signal_analysis::AcceptanceThreshold::range(
                "bpm",
                Some(118.0),
                Some(122.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "has_meter",
                Some(1.0),
                Some(1.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "beats_per_bar",
                Some(4.0),
                Some(4.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "meter_confidence",
                Some(0.2),
                Some(1.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "structure_bar_count",
                Some(4.0),
                None,
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "recovered_bar_count",
                Some(0.0),
                Some(0.0),
                AcceptanceSeverity::Fail,
            ),
        ]),
        AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "rhythm:ambiguous-subdivision90",
                AnalysisCorpusFamily::Pulse,
                "Subdivision-heavy ambiguity reference",
            ),
            grid_click_track(sample_rate, 90.0, 2, 8.0, &[1.0, 0.3], None),
        )
        .with_acceptance_thresholds(vec![
            signal_analysis::AcceptanceThreshold::range(
                "bpm",
                Some(88.0),
                Some(92.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "confidence",
                Some(0.1),
                Some(1.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "tempo_ambiguity",
                Some(0.2),
                Some(1.0),
                AcceptanceSeverity::Fail,
            ),
            signal_analysis::AcceptanceThreshold::range(
                "has_meter",
                Some(0.0),
                Some(0.0),
                AcceptanceSeverity::Fail,
            ),
        ]),
    ]
}

fn trailing_window_audio(audio: &AudioBuffer, seconds: f32) -> AudioBuffer {
    let sample_rate = audio.sample_rate();
    let channel_count = audio.channel_count().0.max(1);
    let requested_frames = sample_rate.seconds_to_frames(Seconds(seconds)).0.max(1);
    let frames = requested_frames.min(audio.frames().0);
    let start_frame = audio.frames().0.saturating_sub(frames);
    let start_sample = start_frame.saturating_mul(channel_count);
    AudioBuffer::from_interleaved(
        sample_rate,
        audio.channels(),
        audio.samples()[start_sample..].to_vec(),
    )
}

fn analyze_trailing_window(
    audio: &AudioBuffer,
    config: super::BeatTrackerConfig,
    seconds: f32,
) -> super::BeatAnalysisResult {
    let window = trailing_window_audio(audio, seconds);
    let mut tracker = super::BeatTracker::new(config);
    tracker.analyze(&window)
}

