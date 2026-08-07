use super::*;

#[test]
fn realtime_preview_contract_reports_latency_and_callback_blocker() {
    let contract =
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 2, 128))
            .expect("default preview contract should plan");

    assert_eq!(contract.config.window_size, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(contract.config.analysis_hop, REALTIME_PREVIEW_ANALYSIS_HOP);
    assert_eq!(contract.input_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(contract.output_latency_frames, REALTIME_PREVIEW_WINDOW_SIZE);
    assert_eq!(
        contract.ratio_change_alignment_tolerance_frames,
        REALTIME_PREVIEW_ANALYSIS_HOP + 128
    );
    assert_eq!(
        contract.integration_mode,
        RealtimePreviewIntegrationMode::AnticipativePreRender
    );
    assert_eq!(
        contract.callback_timeline_mode,
        RealtimePreviewCallbackTimelineMode::QuantumLocked
    );
    assert!(!contract.audio_thread_processing_supported);
    assert_eq!(
        contract.unsupported_mode,
        Some(RealtimePreviewUnsupportedMode::SourceBufferingContract)
    );
}

#[test]
fn realtime_preview_contract_rejects_invalid_streams() {
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(0), 2, 128,)),
        Err(RealtimePreviewPlanError::InvalidSampleRate)
    );
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 6, 128,)),
        Err(RealtimePreviewPlanError::UnsupportedChannelCount(6))
    );
    assert_eq!(
        plan_realtime_preview_stream(RealtimePreviewStreamConfig::new(SampleRate(48_000), 2, 0,)),
        Err(RealtimePreviewPlanError::InvalidBlockSize)
    );
}
