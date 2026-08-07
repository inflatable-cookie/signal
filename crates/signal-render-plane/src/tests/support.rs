//! Shared test fixtures and helpers.

use super::*;

pub(super) const MASTER_ID: u64 = 1_000;
pub(super) const LANE_ID: u64 = 1;

pub(super) fn lane_node(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
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

pub(super) fn master_node(inputs: Vec<RenderEdgeSpec>) -> RenderStageSpec {
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

pub(super) fn identity_edge(source_stage_id: u64) -> RenderEdgeSpec {
    RenderEdgeSpec {
        source_stage_id,
        gain: 1.0,
        matrix: None,
    }
}

/// The old flat shape: one stereo lane summed into a stereo master.
pub(super) fn lane_master_spec(lane_gain: f32, clips: Vec<RenderClipSpec>) -> RenderPlanSpec {
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

pub(super) fn tone_clip(frequency_hz: f32) -> RenderClipSpec {
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

pub(super) fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
    lane_master_spec(0.5, vec![tone_clip(frequency_hz)])
}
pub(super) fn samples_spec(
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

/// Run blocks until the transport edge ramp has fully opened.
pub(super) fn warm_up(executor: &mut RenderPlaneExecutor, blocks: usize) {
    let mut frames = [0.0f32; 512];
    for _ in 0..blocks {
        executor.render_block(&mut frames);
    }
}
/// DC-1.0 stereo samples clip filling its window exactly, with fades.
pub(super) fn dc_clip(
    clip_id: u64,
    start_frames: u64,
    end_frames: u64,
    fade_in_frames: u32,
    fade_out_frames: u32,
) -> RenderClipSpec {
    let frames = (end_frames - start_frames) as usize;
    RenderClipSpec {
        clip_id,
        start_frames,
        end_frames,
        source: RenderSource::Samples(RenderSampleBuffer::stereo(
            48_000,
            vec![1.0f32; frames * 2].into(),
        )),
        loop_source: false,
        fade_in_frames,
        fade_out_frames,
    }
}
pub(super) fn max_left_step(frames: &[f32]) -> f32 {
    frames
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .fold(0.0f32, f32::max)
}
pub(super) struct FakeGainProcessor {
    pub(super) gain: f32,
    pub(super) calls: AtomicU64,
}

impl PluginBlockProcessor for FakeGainProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.calls.fetch_add(1, Ordering::Relaxed);
        for sample in &mut scratch[..frame_count * channels] {
            *sample *= self.gain;
        }
        true
    }
}

/// Fake backend that always misses: returns `false` and must leave the
/// scratch untouched (the bypass contract under test).
pub(super) struct AlwaysMissProcessor {
    pub(super) misses: AtomicU64,
}

impl PluginBlockProcessor for AlwaysMissProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.misses.fetch_add(1, Ordering::Relaxed);
        false
    }
}

/// Minimal alloc-free instrument backend: note-on starts a constant
/// signal at the event velocity; note-off returns to silence.
pub(super) struct EventInstrumentProcessor {
    pub(super) amplitude_bits: AtomicU32,
}

impl EventInstrumentProcessor {
    fn render(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        let mut amplitude = f32::from_bits(self.amplitude_bits.load(Ordering::Relaxed));
        let mut event_index = 0;
        for frame in 0..frame_count {
            while event_index < events.len() && events[event_index].offset_frames as usize == frame
            {
                amplitude = match events[event_index].kind {
                    RenderPluginEventKind::NoteOn { velocity, .. } => velocity,
                    RenderPluginEventKind::NoteOff { .. } => 0.0,
                    RenderPluginEventKind::ControlChange { .. }
                    | RenderPluginEventKind::PitchBend { .. }
                    | RenderPluginEventKind::ChannelPressure { .. }
                    | RenderPluginEventKind::NoteExpression { .. }
                    | RenderPluginEventKind::VoiceStart { .. }
                    | RenderPluginEventKind::VoiceStop { .. }
                    | RenderPluginEventKind::VoiceParam { .. } => amplitude,
                };
                event_index += 1;
            }
            for channel in 0..channels {
                scratch[frame * channels + channel] = amplitude;
            }
        }
        self.amplitude_bits
            .store(amplitude.to_bits(), Ordering::Relaxed);
        true
    }
}

impl PluginBlockProcessor for EventInstrumentProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.render(scratch, frame_count, channels, &[])
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.render(scratch, frame_count, channels, events)
    }
}

/// Constant-content plan with a Sum insert stage carrying `processor`.
pub(super) fn processor_spec(processor: Option<RenderPluginProcessor>) -> RenderPlanSpec {
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

pub(super) fn impulse_delay_spec(delay_frames: u32) -> RenderPlanSpec {
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
pub(super) struct RecordingEventProcessor {
    calls: std::sync::Mutex<Vec<Vec<RenderBlockPluginEvent>>>,
}

impl RecordingEventProcessor {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub(super) fn calls(&self) -> Vec<Vec<RenderBlockPluginEvent>> {
        self.calls.lock().unwrap().clone()
    }
}

impl PluginBlockProcessor for RecordingEventProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.calls
            .lock()
            .unwrap()
            .push(vec![RenderBlockPluginEvent {
                offset_frames: u32::MAX,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 0 },
            }]);
        true
    }

    fn process_with_events(
        &self,
        _scratch: &mut [f32],
        _frames: usize,
        _channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.calls.lock().unwrap().push(events.to_vec());
        true
    }
}

pub(super) fn event_buffer(events: Vec<RenderPluginEvent>) -> RenderPluginEventBuffer {
    RenderPluginEventBuffer {
        events: events.into(),
    }
}

/// `processor_spec` with an event stream on the insert stage.
pub(super) fn events_spec(
    handle: RenderPluginProcessor,
    events: RenderPluginEventBuffer,
) -> RenderPlanSpec {
    let mut spec = processor_spec(Some(handle));
    spec.stages[1].events = Some(events);
    spec
}
/// Spec with one stream clip windowed `[start, end)` at lane gain 1.
pub(super) fn stream_spec(
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

/// Feed `[from, to)` of a ramp (value = frame / total) in fixed chunks.
pub(super) fn feed_ramp(feeder: &StreamFeeder, total: u64, from: u64, to: u64, chunk_frames: u64) {
    let mut start = from - from % chunk_frames;
    while start < to.min(total) {
        let count = chunk_frames.min(total - start);
        let mut data = Vec::with_capacity(count as usize * 2);
        for frame in start..start + count {
            let value = frame as f32 / total as f32;
            data.push(value);
            data.push(value);
        }
        if feeder
            .try_send_chunk(StreamChunk {
                start_frame: start,
                frames: data.into(),
            })
            .is_err()
        {
            return; // Mailbox full: enough read-ahead for the test.
        }
        start += count;
    }
}
/// Spec with one live-input clip windowed `[0, u64::MAX)` at `lane_gain`.
pub(super) fn live_input_spec(handle: &RenderLiveInputHandle, lane_gain: f32) -> RenderPlanSpec {
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

/// Push `count` stereo frames of a ramp starting at `value_base`
/// (value = (value_base + i) / 10_000) and return the next base.
pub(super) fn push_ramp(feeder: &LiveInputFeeder, value_base: u64, count: usize) -> u64 {
    let mut data = Vec::with_capacity(count * 2);
    for index in 0..count {
        let value = (value_base + index as u64) as f32 / 10_000.0;
        data.push(value);
        data.push(value);
    }
    assert_eq!(feeder.push_slice(&data), count, "test ring overflowed");
    value_base + count as u64
}
pub(super) const LIVE_INSERT_ID: u64 = 77;

/// Silent lane into a Sum instrument stage that accepts live events.
pub(super) fn live_instrument_spec(handle: RenderPluginProcessor) -> RenderPlanSpec {
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

pub(super) fn live_note_on(frame: u64, key: u8, velocity: f32) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn { key, velocity },
    }
}

pub(super) fn live_note_off(frame: u64, key: u8) -> RenderPluginEvent {
    RenderPluginEvent {
        frame,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key },
    }
}
pub(super) fn note(
    start_frame: u64,
    duration_frames: u64,
    degree: i32,
    velocity: f32,
) -> RenderNote {
    RenderNote {
        start_frame,
        duration_frames,
        degree,
        pitch_intent: None,
        velocity,
    }
}
pub(super) fn note_buffer(notes: Vec<RenderNote>) -> RenderNoteBuffer {
    RenderNoteBuffer {
        notes: notes.into(),
    }
}

/// Spec with one notes clip windowed `[start, end)` at lane gain 1.
pub(super) fn notes_spec(
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

/// Offline-render `frame_count` frames of `spec` from `start_frame` and
/// return the LEFT channel (channels are identical for note sources).
pub(super) fn render_notes_left(
    spec: &RenderPlanSpec,
    start_frame: u64,
    frame_count: u64,
) -> Vec<f32> {
    let output = crate::render_plan_to_pcm(
        spec,
        &crate::OfflineRenderOptions {
            start_frame,
            frame_count,
            ..crate::OfflineRenderOptions::default()
        },
    )
    .expect("offline note render");
    output
        .master
        .chunks_exact(2)
        .map(|frame| frame[0])
        .collect()
}
pub(super) fn soak_tests_enabled() -> bool {
    if std::env::var("SIGNAL_SOAK_TESTS").as_deref() == Ok("1") {
        return true;
    }
    eprintln!("SKIPPED: wall-clock soak test; set SIGNAL_SOAK_TESTS=1 (or run `effigy test:soak`)");
    false
}
/// FNV-1a 64 over the bit pattern of rendered samples.
pub(super) fn fnv1a_hash_pcm(frames: &[f32]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for sample in frames {
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}
/// Recorded output hash for `golden_render_hash_is_stable` (captured on
/// the test's first run; see the regeneration note in the test body).
pub(super) const GOLDEN_RENDER_HASH: u64 = 0x494b_7128_ef17_1a6a;
