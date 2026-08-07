use std::sync::Arc;

use signal_dsp::{default_adapter_matrix, LimiterState, PolyphaseInterpolationTable};

use crate::notes::{NOTE_ATTACK_SECONDS, NOTE_RELEASE_SECONDS};
use crate::plugin_events::{LIVE_EVENT_RING_CAPACITY, PLUGIN_EVENTS_PER_BLOCK_CAPACITY};
use crate::{
    RenderPlanCompileError, RenderPlanSpec, RenderSource, RenderStageKind, MAX_BLOCK_FRAMES,
};

use super::types::{
    CompiledClip, CompiledInput, CompiledNode, CompiledSource, CLIP_EDGE_FADE_FRAMES,
};

/// Interpolation table shape for rate-converted media playback. 16 taps ×
/// 512 phases ≈ 32 KB per distinct cutoff; built once per plan compile.
const RESAMPLE_TAPS: usize = 16;
const RESAMPLE_PHASES: usize = 512;

/// A compiled, immutable-topology render plan. Source state (tone phases,
/// smoothed gains) mutates during rendering; structure never does. Nodes are
/// stored in topological order (inputs strictly before consumers).
pub struct RenderPlan {
    pub(crate) sample_rate_hz: u32,
    pub(crate) stages: Vec<CompiledNode>,
    /// Position of the master stage in `stages`.
    pub(crate) master_index: usize,
    /// Stream channel count the boundary was compiled for (master channels
    /// when the stream is unknown at install time).
    pub(crate) stream_channels: usize,
    /// Hardware-boundary downmix (row-major `output_channels ×
    /// stream_channels`); empty unless the stream is narrower than the
    /// master format. Compiled at install time — the controller knows the
    /// stream's channel count then. Never applies inside the creative graph.
    pub(crate) boundary_matrix: Vec<f32>,
    /// Per-stage inheritance: `inherit_stage_map[i]` is the index of the
    /// matching stage in the PREVIOUS plan (by stage_id), precomputed by the
    /// controller at install time so the executor never compares identities
    /// on the audio thread.
    pub(crate) inherit_stage_map: Vec<Option<usize>>,
    /// Per-stage, per-clip inheritance into the previous plan's clip list
    /// (by clip_id within the matched stage).
    pub(crate) inherit_clip_maps: Vec<Vec<Option<usize>>>,
    /// Monotonic install stamp assigned by the controller. The executor
    /// publishes it with the meter table so readers can map meter slots to
    /// the stage ids of the matching topology.
    pub(crate) generation: u64,
    /// Optional master soft limiter guarding the hardware boundary; its
    /// smoothed gain inherits across plan swaps.
    pub(crate) limiter: Option<LimiterState>,
}

impl RenderPlan {
    pub(crate) fn compile(
        spec: &RenderPlanSpec,
        stream_channels: Option<u16>,
    ) -> Result<Box<RenderPlan>, RenderPlanCompileError> {
        // Identity and shape validation.
        for stage in &spec.stages {
            if stage.format.channels == 0 {
                return Err(RenderPlanCompileError::InvalidChannelCount(stage.stage_id));
            }
            if spec
                .stages
                .iter()
                .filter(|candidate| candidate.stage_id == stage.stage_id)
                .count()
                > 1
            {
                return Err(RenderPlanCompileError::DuplicateNodeId(stage.stage_id));
            }
        }
        let master_count = spec
            .stages
            .iter()
            .filter(|stage| matches!(stage.kind, RenderStageKind::Output))
            .count();
        if master_count != 1 {
            return Err(RenderPlanCompileError::MasterCount(master_count));
        }
        for stage in &spec.stages {
            if stage.processor.is_some() && !matches!(stage.kind, RenderStageKind::Sum) {
                return Err(RenderPlanCompileError::ProcessorOnNonSumStage(
                    stage.stage_id,
                ));
            }
            if stage.accepts_live_events && stage.processor.is_none() {
                // Same placement rule as compiled events: a processor is
                // required, and the processor check above already pins
                // processors to Sum stages.
                return Err(RenderPlanCompileError::LiveEventsWithoutProcessor(
                    stage.stage_id,
                ));
            }
            if let Some(events) = &stage.events {
                if stage.processor.is_none() {
                    return Err(RenderPlanCompileError::EventsWithoutProcessor(
                        stage.stage_id,
                    ));
                }
                if events
                    .events
                    .windows(2)
                    .any(|pair| pair[0].frame > pair[1].frame)
                {
                    return Err(RenderPlanCompileError::UnsortedEvents(stage.stage_id));
                }
            }
            if !stage.parameter_envelopes.is_empty() && stage.processor.is_none() {
                return Err(RenderPlanCompileError::ParameterEnvelopeWithoutProcessor(
                    stage.stage_id,
                ));
            }
            for envelope in &stage.parameter_envelopes {
                if envelope.points.windows(2).any(|pair| pair[0].0 > pair[1].0) {
                    return Err(RenderPlanCompileError::UnsortedParameterEnvelope(
                        stage.stage_id,
                    ));
                }
            }
        }
        let position_of = |stage_id: u64| -> Option<usize> {
            spec.stages
                .iter()
                .position(|candidate| candidate.stage_id == stage_id)
        };
        for stage in &spec.stages {
            for input in &stage.inputs {
                let Some(source_index) = position_of(input.source_stage_id) else {
                    return Err(RenderPlanCompileError::UnknownInputNode {
                        stage_id: stage.stage_id,
                        source_stage_id: input.source_stage_id,
                    });
                };
                let expected = spec.stages[source_index].format.channels as usize
                    * stage.format.channels as usize;
                if let Some(matrix) = &input.matrix {
                    if matrix.len() != expected {
                        return Err(RenderPlanCompileError::MatrixDimensions {
                            stage_id: stage.stage_id,
                            source_stage_id: input.source_stage_id,
                            expected,
                            actual: matrix.len(),
                        });
                    }
                }
            }
        }

        // Kahn topological sort (spec order preserved among ready stages).
        let node_count = spec.stages.len();
        let mut indegree: Vec<usize> = spec.stages.iter().map(|stage| stage.inputs.len()).collect();
        let mut consumers: Vec<Vec<usize>> = vec![Vec::new(); node_count];
        for (index, stage) in spec.stages.iter().enumerate() {
            for input in &stage.inputs {
                let source_index = position_of(input.source_stage_id).expect("validated above");
                consumers[source_index].push(index);
            }
        }
        let mut order: Vec<usize> = Vec::with_capacity(node_count);
        let mut ready: Vec<usize> = (0..node_count)
            .filter(|index| indegree[*index] == 0)
            .collect();
        let mut next_ready = 0usize;
        while next_ready < ready.len() {
            let index = ready[next_ready];
            next_ready += 1;
            order.push(index);
            for consumer in &consumers[index] {
                indegree[*consumer] -= 1;
                if indegree[*consumer] == 0 {
                    ready.push(*consumer);
                }
            }
        }
        if order.len() != node_count {
            return Err(RenderPlanCompileError::Cycle);
        }
        // Spec index → topological position, for edge re-indexing.
        let mut topo_position = vec![0usize; node_count];
        for (position, spec_index) in order.iter().enumerate() {
            topo_position[*spec_index] = position;
        }

        let tau = std::f32::consts::TAU;
        let stream_rate = spec.sample_rate_hz.max(1);
        // One table per distinct cutoff in the plan (typically zero or one).
        let mut tables: Vec<(u64, PolyphaseInterpolationTable)> = Vec::new();
        let mut table_for_step = |step: f64| -> Option<PolyphaseInterpolationTable> {
            if step == 1.0 {
                return None;
            }
            let cutoff = (1.0 / step).min(1.0);
            let key = cutoff.to_bits();
            if let Some((_, table)) = tables.iter().find(|(bits, _)| *bits == key) {
                return Some(table.clone());
            }
            let table = PolyphaseInterpolationTable::new(RESAMPLE_TAPS, RESAMPLE_PHASES, cutoff);
            tables.push((key, table.clone()));
            Some(table)
        };

        let mut stages: Vec<CompiledNode> = Vec::with_capacity(node_count);
        let mut master_index = 0usize;
        for (position, spec_index) in order.iter().enumerate() {
            let stage = &spec.stages[*spec_index];
            let dest_channels = stage.format.channels as usize;
            if matches!(stage.kind, RenderStageKind::Output) {
                master_index = position;
            }
            let clips = match &stage.kind {
                RenderStageKind::Source { clips } => clips
                    .iter()
                    .map(|clip| {
                        // Unwrap a rate warp (g12.027): the warp multiplies
                        // the source-frames-per-stream-frame step its inner
                        // media source compiles to. Warping anything but
                        // media (or nesting warps) is a spec bug — reject.
                        let (spec_source, warp_rate) = match &clip.source {
                            RenderSource::Warped { source, rate } => {
                                if !matches!(
                                    source.as_ref(),
                                    RenderSource::Samples(_) | RenderSource::Stream(_)
                                ) {
                                    return Err(RenderPlanCompileError::WarpedSourceUnsupported {
                                        stage_id: stage.stage_id,
                                        clip_id: clip.clip_id,
                                    });
                                }
                                let rate = if rate.is_finite() && *rate > 0.0 {
                                    *rate
                                } else {
                                    1.0
                                };
                                (source.as_ref(), rate)
                            }
                            source => (source, 1.0),
                        };
                        let source = match spec_source {
                            RenderSource::Warped { .. } => {
                                unreachable!("nested warps rejected above")
                            }
                            RenderSource::Silence => CompiledSource::Silence,
                            RenderSource::TestTone { frequency_hz } => CompiledSource::Tone {
                                phase: 0.0,
                                step: frequency_hz * tau / stream_rate as f32,
                            },
                            RenderSource::Samples(buffer) => {
                                let step = buffer.sample_rate_hz.max(1) as f64 / stream_rate as f64
                                    * warp_rate;
                                // Adapt the source's channels to the stage format
                                // only when they differ; matched counts keep the
                                // direct read (golden-hash stable).
                                let source_channels = buffer.channels.max(1) as usize;
                                let channel_adapter = if source_channels == dest_channels {
                                    None
                                } else {
                                    Some(signal_dsp::default_adapter_matrix(
                                        buffer.channels.max(1),
                                        dest_channels as u16,
                                    ))
                                };
                                CompiledSource::Samples {
                                    table: table_for_step(step),
                                    step,
                                    buffer: buffer.clone(),
                                    loop_source: clip.loop_source,
                                    channel_adapter,
                                }
                            }
                            RenderSource::Stream(handle) => {
                                let step = handle.source_sample_rate_hz().max(1) as f64
                                    / stream_rate as f64
                                    * warp_rate;
                                CompiledSource::Stream {
                                    table: table_for_step(step),
                                    step,
                                    handle: handle.clone(),
                                    held: std::array::from_fn(|_| None),
                                }
                            }
                            RenderSource::LiveInput(handle) => CompiledSource::LiveInput {
                                handle: handle.clone(),
                            },
                            RenderSource::Notes(buffer) => {
                                if buffer
                                    .notes
                                    .windows(2)
                                    .any(|pair| pair[0].start_frame > pair[1].start_frame)
                                {
                                    return Err(RenderPlanCompileError::UnsortedNotes {
                                        stage_id: stage.stage_id,
                                        clip_id: clip.clip_id,
                                    });
                                }
                                // Per-note phase steps precomputed here, on
                                // the control side — the render loop does no
                                // transcendentals beyond sin().
                                let steps: Arc<[f64]> = buffer
                                    .notes
                                    .iter()
                                    .map(|note| {
                                        note.frequency_hz() * std::f64::consts::TAU
                                            / stream_rate as f64
                                    })
                                    .collect();
                                CompiledSource::Notes {
                                    steps,
                                    attack_frames: ((NOTE_ATTACK_SECONDS * stream_rate as f64)
                                        as u64)
                                        .max(1),
                                    release_frames: ((NOTE_RELEASE_SECONDS * stream_rate as f64)
                                        as u64)
                                        .max(1),
                                    max_duration_frames: buffer
                                        .notes
                                        .iter()
                                        .map(|note| note.duration_frames)
                                        .max()
                                        .unwrap_or(0),
                                    buffer: buffer.clone(),
                                }
                            }
                        };
                        let window_frames = clip.end_frames.saturating_sub(clip.start_frames);
                        Ok(CompiledClip {
                            clip_id: clip.clip_id,
                            start_frames: clip.start_frames,
                            end_frames: clip.end_frames,
                            edge_fade_frames: CLIP_EDGE_FADE_FRAMES.min(window_frames.max(2) / 2),
                            fade_in_frames: (clip.fade_in_frames as u64).min(window_frames),
                            fade_out_frames: (clip.fade_out_frames as u64).min(window_frames),
                            source,
                        })
                    })
                    .collect::<Result<Vec<_>, RenderPlanCompileError>>()?,
                RenderStageKind::Sum | RenderStageKind::Delay { .. } | RenderStageKind::Output => {
                    Vec::new()
                }
            };
            let inputs = stage
                .inputs
                .iter()
                .map(|input| {
                    let source_spec_index =
                        position_of(input.source_stage_id).expect("validated above");
                    let source_channels = spec.stages[source_spec_index].format.channels as usize;
                    // Explicit matrix or default adapter, with the static
                    // edge gain folded into the coefficients (v1: per-edge
                    // gain is compile-time data, not a smoothed parameter).
                    let mut matrix = match &input.matrix {
                        Some(matrix) => matrix.clone(),
                        None => {
                            default_adapter_matrix(source_channels as u16, dest_channels as u16)
                        }
                    };
                    for coefficient in matrix.iter_mut() {
                        *coefficient *= input.gain;
                    }
                    CompiledInput {
                        source_index: topo_position[source_spec_index],
                        source_channels,
                        matrix,
                    }
                })
                .collect();
            // The plan's master_gain multiplies the master stage's own gain
            // so the legacy single-knob master keeps working.
            let master_factor = if matches!(stage.kind, RenderStageKind::Output) {
                spec.master_gain
            } else {
                1.0
            };
            let gain = stage.gain * master_factor;
            let delay_ring = match stage.kind {
                RenderStageKind::Delay { frames } => {
                    vec![0.0; frames as usize * dest_channels]
                }
                _ => Vec::new(),
            };
            // Automation envelope: sorted, master factor folded in so the
            // render path samples one curve.
            let gain_envelope: Vec<(u64, f32)> = match &stage.gain_automation {
                Some(points) => {
                    if points.windows(2).any(|pair| pair[0].0 > pair[1].0) {
                        return Err(RenderPlanCompileError::UnsortedAutomation(stage.stage_id));
                    }
                    points
                        .iter()
                        .map(|(frame, value)| (*frame, *value * master_factor))
                        .collect()
                }
                None => Vec::new(),
            };
            stages.push(CompiledNode {
                stage_id: stage.stage_id,
                channels: dest_channels,
                gain_target: gain,
                gain_current: gain,
                block_gain_begin: gain,
                block_gain_slope: 0.0,
                gain_envelope,
                clips,
                inputs,
                processor: stage.processor.clone(),
                events: stage
                    .events
                    .as_ref()
                    .map(|buffer| Arc::clone(&buffer.events))
                    .unwrap_or_else(|| Arc::from([])),
                event_scratch: if stage.events.is_some() || stage.accepts_live_events {
                    Vec::with_capacity(PLUGIN_EVENTS_PER_BLOCK_CAPACITY)
                } else {
                    Vec::new()
                },
                accepts_live_events: stage.accepts_live_events,
                live_events: if stage.accepts_live_events {
                    Vec::with_capacity(LIVE_EVENT_RING_CAPACITY)
                } else {
                    Vec::new()
                },
                delay_ring,
                delay_cursor: 0,
                scratch: vec![0.0f32; MAX_BLOCK_FRAMES * dest_channels],
            });
        }

        // Hardware boundary: compiled here, at install time, because the
        // controller knows the stream's channel count now. A narrower stream
        // gets a standard downmix matrix; a wider stream gets the master's
        // channels copied and the extras silence-filled (no upmix policy is
        // invented at the hardware stage); equal formats copy through.
        let output_channels = stages[master_index].channels;
        let stream_channels = stream_channels
            .map(|channels| channels.max(1) as usize)
            .unwrap_or(output_channels);
        let boundary_matrix = if stream_channels < output_channels {
            default_adapter_matrix(output_channels as u16, stream_channels as u16)
        } else {
            Vec::new()
        };

        Ok(Box::new(RenderPlan {
            sample_rate_hz: stream_rate,
            stages,
            master_index,
            stream_channels,
            boundary_matrix,
            inherit_stage_map: Vec::new(),
            inherit_clip_maps: Vec::new(),
            generation: 0,
            limiter: spec.master_limiter.as_ref().map(|limiter| {
                LimiterState::new(
                    signal_primitives::SampleRate(stream_rate),
                    limiter.threshold,
                    limiter.knee_width,
                    signal_primitives::Seconds(limiter.release_seconds),
                )
            }),
        }))
    }
}
