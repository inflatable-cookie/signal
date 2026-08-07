use super::support::*;
use super::*;

/// Fixture backend for the offline param bake: parameter 7 is a linear
/// output gain, settable through the set-parameter seam. Any other id is
/// rejected (`false`), like a real backend refusing an unknown param.
struct ParamGainProcessor {
    gain_bits: std::sync::atomic::AtomicU32,
}

impl ParamGainProcessor {
    const GAIN_PARAM_ID: u32 = 7;

    fn with_gain(gain: f32) -> Self {
        ParamGainProcessor {
            gain_bits: std::sync::atomic::AtomicU32::new(gain.to_bits()),
        }
    }
}

impl crate::PluginBlockProcessor for ParamGainProcessor {
    fn process(&self, scratch: &mut [f32], _frame_count: usize, _channels: usize) -> bool {
        let gain = f32::from_bits(self.gain_bits.load(std::sync::atomic::Ordering::Relaxed));
        for sample in scratch.iter_mut() {
            *sample *= gain;
        }
        true
    }

    fn set_parameter_normalized(&self, parameter_id: u32, normalized: f32) -> bool {
        if parameter_id != Self::GAIN_PARAM_ID {
            return false;
        }
        self.gain_bits
            .store(normalized.to_bits(), std::sync::atomic::Ordering::Relaxed);
        true
    }
}

/// Fixed-gain backend WITHOUT parameter transport (trait default
/// rejects the write): envelopes aimed at it must leave audio untouched.
struct FixedGainProcessor {
    gain: f32,
}

impl crate::PluginBlockProcessor for FixedGainProcessor {
    fn process(&self, scratch: &mut [f32], _frame_count: usize, _channels: usize) -> bool {
        for sample in scratch.iter_mut() {
            *sample *= self.gain;
        }
        true
    }
}

fn processor_sum_stage(
    stage_id: u64,
    input_stage_id: u64,
    processor: crate::RenderPluginProcessor,
    parameter_envelopes: Vec<RenderParamEnvelope>,
) -> RenderStageSpec {
    RenderStageSpec {
        accepts_live_events: false,
        processor: Some(processor),
        events: None,
        stage_id,
        format: ChannelFormat::stereo(),
        gain: 1.0,
        gain_automation: None,
        kind: RenderStageKind::Sum,
        inputs: vec![RenderEdgeSpec {
            source_stage_id: input_stage_id,
            gain: 1.0,
            matrix: None,
        }],
        parameter_envelopes,
    }
}

#[test]
fn offline_param_envelope_applies_at_block_boundaries() {
    // DC 0.5 through a param-gain processor swept 0 -> 1 over 1024
    // frames, rendered at 256-frame blocks: the output steps once per
    // block, holding the envelope value sampled at each block START —
    // the recorded block-boundary fidelity bound.
    let processor = crate::RenderPluginProcessor::new(Arc::new(ParamGainProcessor::with_gain(1.0)));
    let envelope = RenderParamEnvelope {
        parameter_id: ParamGainProcessor::GAIN_PARAM_ID,
        points: vec![(0, 0.0), (1_024, 1.0)],
    };
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 0.5)]),
            processor_sum_stage(5, 1, processor, vec![envelope]),
            master(vec![5]),
        ],
    };
    let output = render_plan_to_pcm(
        &spec,
        &OfflineRenderOptions {
            start_frame: 0,
            frame_count: 2_048,
            block_frames: 256,
            ..OfflineRenderOptions::default()
        },
    )
    .unwrap();

    // Sample the left channel mid-block (clear of the 32-frame clip
    // edge declick in block zero).
    let left = |frame: usize| output.master[frame * 2];
    let expectations = [
        (128, 0.0),   // block 0: envelope(0) = 0.0
        (384, 0.125), // block 1: envelope(256) = 0.25 -> 0.5 * 0.25
        (640, 0.25),  // block 2: envelope(512) = 0.5
        (896, 0.375), // block 3: envelope(768) = 0.75
        (1_152, 0.5), // block 4: envelope(1024) = 1.0
        (1_920, 0.5), // past the last point: end value held
    ];
    for (frame, expected) in expectations {
        assert!(
            (left(frame) - expected).abs() < 1e-4,
            "frame {frame}: read {} expected {expected}",
            left(frame),
        );
    }
    // The steps land AT block boundaries: constant within a block.
    assert!((left(300) - left(500)).abs() < 1e-6);
    assert!((left(260) - left(510)).abs() < 1e-6);
    // Non-static overall: the sweep is audible in the bounce.
    assert!((left(1_152) - left(384)).abs() > 0.3);
}

#[test]
fn param_envelope_on_transportless_backend_leaves_render_byte_identical() {
    // A backend without parameter transport rejects the set-parameter
    // write (trait default): the envelope must change NOTHING.
    let build = |parameter_envelopes: Vec<RenderParamEnvelope>| {
        let processor =
            crate::RenderPluginProcessor::new(Arc::new(FixedGainProcessor { gain: 0.7 }));
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            master_limiter: None,
            stages: vec![
                lane(1, 1.0, vec![constant_clip(11, 0.5)]),
                processor_sum_stage(5, 1, processor, parameter_envelopes),
                master(vec![5]),
            ],
        }
    };
    let options = OfflineRenderOptions {
        start_frame: 0,
        frame_count: 1_024,
        block_frames: 128,
        ..OfflineRenderOptions::default()
    };
    let with_envelope = render_plan_to_pcm(
        &build(vec![RenderParamEnvelope {
            parameter_id: 3,
            points: vec![(0, 0.0), (512, 1.0)],
        }]),
        &options,
    )
    .unwrap();
    let without_envelope = render_plan_to_pcm(&build(Vec::new()), &options).unwrap();
    assert_eq!(with_envelope.master.len(), without_envelope.master.len());
    assert!(with_envelope
        .master
        .iter()
        .zip(without_envelope.master.iter())
        .all(|(a, b)| a.to_bits() == b.to_bits()));
}

#[test]
fn param_envelopes_reject_processorless_and_unsorted_stages() {
    let processorless = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            {
                let mut stage = lane(1, 1.0, vec![constant_clip(11, 0.5)]);
                stage.parameter_envelopes = vec![RenderParamEnvelope {
                    parameter_id: 7,
                    points: vec![(0, 0.5)],
                }];
                stage
            },
            master(vec![1]),
        ],
    };
    let options = OfflineRenderOptions {
        frame_count: 64,
        ..OfflineRenderOptions::default()
    };
    assert!(render_plan_to_pcm(&processorless, &options).is_err());

    let processor = crate::RenderPluginProcessor::new(Arc::new(ParamGainProcessor::with_gain(1.0)));
    let unsorted = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            lane(1, 1.0, vec![constant_clip(11, 0.5)]),
            processor_sum_stage(
                5,
                1,
                processor,
                vec![RenderParamEnvelope {
                    parameter_id: 7,
                    points: vec![(512, 1.0), (0, 0.0)],
                }],
            ),
            master(vec![5]),
        ],
    };
    assert!(render_plan_to_pcm(&unsorted, &options).is_err());
}

#[test]
fn envelope_value_sampling_interpolates_and_holds_ends() {
    let envelope = RenderParamEnvelope {
        parameter_id: 1,
        points: vec![(100, 0.2), (300, 0.8)],
    };
    assert_eq!(envelope.value_at(0), Some(0.2)); // held before first
    assert_eq!(envelope.value_at(100), Some(0.2));
    assert!((envelope.value_at(200).unwrap() - 0.5).abs() < 1e-6);
    assert_eq!(envelope.value_at(300), Some(0.8));
    assert_eq!(envelope.value_at(9_999), Some(0.8)); // held past last
    let empty = RenderParamEnvelope {
        parameter_id: 1,
        points: Vec::new(),
    };
    assert_eq!(empty.value_at(0), None);
}
