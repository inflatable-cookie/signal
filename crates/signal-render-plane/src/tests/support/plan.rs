use super::super::*;

pub(in crate::tests) const MASTER_ID: u64 = 1_000;
pub(in crate::tests) const LANE_ID: u64 = 1;

pub(in crate::tests) fn lane_node(
    stage_id: u64,
    gain: f32,
    clips: Vec<RenderClipSpec>,
) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain,
        gain_automation: None,
        kind: RenderStageKind::Source { clips },
        inputs: Vec::new(),
    }
}

pub(in crate::tests) fn master_node(inputs: Vec<RenderEdgeSpec>) -> RenderStageSpec {
    RenderStageSpec {
        parameter_envelopes: Vec::new(),
        accepts_live_events: false,
        processor: None,
        events: None,
        stage_id: MASTER_ID,
        format: ChannelFormat::stereo(),
        gain: 1.0,
        gain_automation: None,
        kind: RenderStageKind::Output,
        inputs,
    }
}

pub(in crate::tests) fn identity_edge(source_stage_id: u64) -> RenderEdgeSpec {
    RenderEdgeSpec {
        source_stage_id,
        gain: 1.0,
        matrix: None,
    }
}

/// The old flat shape: one stereo lane summed into a stereo master.
pub(in crate::tests) fn lane_master_spec(
    lane_gain: f32,
    clips: Vec<RenderClipSpec>,
) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(LANE_ID, lane_gain, clips),
            master_node(vec![identity_edge(LANE_ID)]),
        ],
    }
}

pub(in crate::tests) fn tone_clip(frequency_hz: f32) -> RenderClipSpec {
    RenderClipSpec {
        clip_id: 1003,
        start_frames: 0,
        end_frames: u64::MAX,
        source: RenderSource::TestTone { frequency_hz },
        loop_source: false,
        fade_in_frames: 0,
        fade_out_frames: 0,
    }
}

pub(in crate::tests) fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
    lane_master_spec(0.5, vec![tone_clip(frequency_hz)])
}
pub(in crate::tests) fn samples_spec(
    values: &[f32],
    start_frames: u64,
    end_frames: u64,
    loop_source: bool,
) -> RenderPlanSpec {
    // Stereo frames with identical channels at the stream rate.
    let mut data = Vec::new();
    for value in values {
        data.push(*value);
        data.push(*value);
    }
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1004,
            start_frames,
            end_frames,
            source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
            loop_source,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Constant-content plan with a Sum insert stage carrying `processor`.
pub(in crate::tests) fn processor_spec(processor: Option<RenderPluginProcessor>) -> RenderPlanSpec {
    let values = vec![0.5f32; 480];
    let mut data = Vec::new();
    for value in &values {
        data.push(*value);
        data.push(*value);
    }
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(
                LANE_ID,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 2001,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                    loop_source: true,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor,
                events: None,
                stage_id: 77,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(77)]),
        ],
    }
}

pub(in crate::tests) fn impulse_delay_spec(delay_frames: u32) -> RenderPlanSpec {
    let mut data = vec![0.0f32; 512 * 2];
    data[100 * 2] = 1.0;
    data[100 * 2 + 1] = 1.0;
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(
                LANE_ID,
                1.0,
                vec![RenderClipSpec {
                    clip_id: 2002,
                    start_frames: 0,
                    end_frames: 512,
                    source: RenderSource::Samples(RenderSampleBuffer::stereo(48_000, data.into())),
                    loop_source: false,
                    fade_in_frames: 0,
                    fade_out_frames: 0,
                }],
            ),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                processor: None,
                events: None,
                stage_id: 78,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Delay {
                    frames: delay_frames,
                },
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(78)]),
        ],
    }
}

/// `processor_spec` with an event stream on the insert stage.
pub(in crate::tests) fn events_spec(
    handle: RenderPluginProcessor,
    events: RenderPluginEventBuffer,
) -> RenderPlanSpec {
    let mut spec = processor_spec(Some(handle));
    spec.stages[1].events = Some(events);
    spec
}

/// Spec with one stream clip windowed `[start, end)` at lane gain 1.
pub(in crate::tests) fn stream_spec(
    handle: &RenderStreamHandle,
    start_frames: u64,
    end_frames: u64,
) -> RenderPlanSpec {
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1006,
            start_frames,
            end_frames,
            source: RenderSource::Stream(handle.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

/// Spec with one live-input clip windowed `[0, u64::MAX)` at `lane_gain`.
pub(in crate::tests) fn live_input_spec(
    handle: &RenderLiveInputHandle,
    lane_gain: f32,
) -> RenderPlanSpec {
    lane_master_spec(
        lane_gain,
        vec![RenderClipSpec {
            clip_id: 1007,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::LiveInput(handle.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}

pub(in crate::tests) const LIVE_INSERT_ID: u64 = 77;

/// Silent lane into a Sum instrument stage that accepts live events.
pub(in crate::tests) fn live_instrument_spec(handle: RenderPluginProcessor) -> RenderPlanSpec {
    RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane_node(LANE_ID, 1.0, Vec::new()),
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: true,
                processor: Some(handle),
                events: None,
                stage_id: LIVE_INSERT_ID,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![identity_edge(LANE_ID)],
            },
            master_node(vec![identity_edge(LIVE_INSERT_ID)]),
        ],
    }
}

/// Spec with one notes clip windowed `[start, end)` at lane gain 1.
pub(in crate::tests) fn notes_spec(
    buffer: &RenderNoteBuffer,
    start_frames: u64,
    end_frames: u64,
) -> RenderPlanSpec {
    lane_master_spec(
        1.0,
        vec![RenderClipSpec {
            clip_id: 1008,
            start_frames,
            end_frames,
            source: RenderSource::Notes(buffer.clone()),
            loop_source: false,
            fade_in_frames: 0,
            fade_out_frames: 0,
        }],
    )
}
