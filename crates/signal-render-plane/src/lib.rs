//! Real-time render plane: the control/render split for Signal playback.
//!
//! The runtime's `process_engine_block` is a proof/observation API: every
//! block it allocates (graph-id strings, summary recomputation, snapshot
//! clones, by-value buffers), which disqualifies it from the audio thread.
//! This crate is the execution substrate that *is* allowed there:
//!
//! - the **control side** compiles a [`RenderPlanSpec`] into an immutable,
//!   fully preallocated plan and hands it across a bounded lock-free mailbox;
//! - the **render side** ([`RenderPlaneExecutor`]) swaps plans atomically
//!   between blocks and executes [`RenderPlaneExecutor::render_block`] with a
//!   hard no-alloc / no-lock / no-I/O contract;
//! - retired plans travel **back** to the control side for deallocation —
//!   nothing is ever freed on the audio thread.
//!
//! Both mailboxes are `std::sync::mpsc::sync_channel`s: bounded array-backed
//! channels whose send/receive operations neither allocate nor free.
//!
//! Plans are **graphs**, not lane lists: a spec is a set of format-typed
//! stages ([`RenderStageSpec`] — lanes, busses, exactly one master) connected
//! by edges that each carry a gain and an N×M channel mix matrix. Compile
//! topologically sorts the graph into a flat execution schedule, preallocates
//! a per-stage scratch buffer ([`MAX_BLOCK_FRAMES`] × stage channels — the
//! buffer pool *is* the plan), and resolves every edge's matrix (explicit
//! coefficients, or a default adapter from `signal_dsp::default_adapter_matrix`
//! when formats differ). Per chorus a14 the graph is channel-format-typed:
//! nothing in it assumes stereo. The only forced collapse is the hardware
//! boundary, where the master stage's format is adapted to the negotiated
//! stream format (downmix matrix when the device is narrower, silence-filled
//! extra channels when wider).

#![warn(missing_docs)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;

use signal_dsp::{default_adapter_matrix, PolyphaseInterpolationTable};

/// Interpolation table shape for rate-converted media playback. 16 taps ×
/// 512 phases ≈ 32 KB per distinct cutoff; built once per plan compile.
const RESAMPLE_TAPS: usize = 16;
const RESAMPLE_PHASES: usize = 512;

/// Largest callback quantum the plan's scratch buffers are sized for. Every
/// stage owns `MAX_BLOCK_FRAMES × channels` samples of scratch, preallocated
/// at compile; `render_block` clamps (and debug-asserts) the frame count.
pub const MAX_BLOCK_FRAMES: usize = 4096;

/// Shared immutable sample data: interleaved stereo f32 at a source rate.
///
/// Equality is pointer-based so plan specs containing large buffers compare
/// cheaply and a cached buffer keeps reinstalls idempotent.
#[derive(Clone, Debug)]
pub struct RenderSampleBuffer {
    /// Source sample rate of the buffer.
    pub sample_rate_hz: u32,
    /// Interleaved stereo frames (length is even; frame count = len / 2).
    pub frames: Arc<[f32]>,
}

impl PartialEq for RenderSampleBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.sample_rate_hz == other.sample_rate_hz && Arc::ptr_eq(&self.frames, &other.frames)
    }
}

impl RenderSampleBuffer {
    /// Number of stereo frames in the buffer.
    pub fn frame_count(&self) -> usize {
        self.frames.len() / 2
    }
}

const COMMAND_MAILBOX_CAPACITY: usize = 64;
// Sized so that even a full command mailbox of plan installs can retire
// without saturating; install_plan also reclaims eagerly. The executor's
// single parked slot is belt-and-braces on top of this invariant.
const RETIRED_MAILBOX_CAPACITY: usize = COMMAND_MAILBOX_CAPACITY + 2;

/// Transport edge ramp length: play, stop, and seek gate through this
/// envelope instead of stepping, so transport actions never click.
const EDGE_RAMP_SECONDS: f32 = 0.005;
/// Full-swing time for stage gain changes across plan swaps.
const GAIN_SMOOTHING_SECONDS: f32 = 0.010;
/// Declick fade applied inside each clip window edge (shortened for tiny
/// windows so short clips stay audible).
const CLIP_EDGE_FADE_FRAMES: u64 = 32;

// ── Spec vocabulary (control-side description of a plan) ───────────────────

/// Channel layout semantics carried alongside a channel count.
///
/// Extensible past stereo from day one; `Generic` means count-only semantics
/// (no named speaker positions). Named multichannel layouts (5.1, 7.1.4, …)
/// are added here as the engine grows layout-aware coefficient generators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    /// Single channel.
    Mono,
    /// Two channels, left/right semantics.
    Stereo,
    /// Count-only semantics: channels have no named positions.
    Generic,
}

/// Channel format of a stage's output: count plus layout semantics. Scratch
/// sizing, matrix validation, and boundary adaptation all key off this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelFormat {
    /// Number of interleaved channels.
    pub channels: u16,
    /// Layout semantics of those channels.
    pub layout: ChannelLayout,
}

impl ChannelFormat {
    /// Single mono channel.
    pub fn mono() -> Self {
        ChannelFormat {
            channels: 1,
            layout: ChannelLayout::Mono,
        }
    }

    /// Two channels, left/right.
    pub fn stereo() -> Self {
        ChannelFormat {
            channels: 2,
            layout: ChannelLayout::Stereo,
        }
    }
}

/// Audio source for one render clip.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderSource {
    /// Clip renders silence.
    Silence,
    /// Clip renders a sine tone — the audible stand-in for media-less clips
    /// and the soak-test source.
    TestTone {
        /// Tone frequency in hertz.
        frequency_hz: f32,
    },
    /// Clip plays shared sample data from its window start, with windowed-sinc
    /// interpolation when the source rate differs from the stream rate.
    Samples(RenderSampleBuffer),
}

/// One clip event in a lane stage: a half-open stream-clock window
/// `[start_frames, end_frames)` and the source that plays inside it.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderClipSpec {
    /// Stable clip identity (consumer-supplied). Used control-side to build
    /// the state-inheritance map across plan swaps, so a clip keeps its
    /// playback state (tone phase, later streaming cursors) when neighbours
    /// are inserted or removed.
    pub clip_id: u64,
    /// First audible frame.
    pub start_frames: u64,
    /// First frame past the audible range.
    pub end_frames: u64,
    /// Source playing inside the window, anchored at `start_frames`.
    pub source: RenderSource,
    /// When true, a `Samples` source repeats from its start once exhausted
    /// instead of going silent; ignored for other sources.
    pub loop_source: bool,
}

/// What a stage does with its inputs (and whether it generates content).
#[derive(Debug, Clone, PartialEq)]
pub enum RenderStageKind {
    /// Source stage: renders clip content at its format. May also sum inputs
    /// like a bus (clips render first, then edges add in).
    Source {
        /// Clip events rendered by this source stage.
        clips: Vec<RenderClipSpec>,
    },
    /// Sums its inputs through their edge matrices.
    Sum,
    /// The output boundary: exactly one per plan. Its scratch is what the
    /// hardware boundary adapts onto the stream.
    Output,
}

/// One input edge into a stage: where the signal comes from, how loud, and
/// how its channels map onto this stage's channels.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderEdgeSpec {
    /// `stage_id` of the upstream stage feeding this edge.
    pub source_stage_id: u64,
    /// Linear edge gain (static in v1: compiled into the matrix).
    pub gain: f32,
    /// Row-major `source_channels × dest_channels` mix matrix; `None` picks
    /// the default adapter (identity when channel counts match; standard
    /// up/downmix coefficients otherwise — see
    /// `signal_dsp::default_adapter_matrix` for the exact rules).
    pub matrix: Option<Vec<f32>>,
}

/// One stage in a render plan graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderStageSpec {
    /// Stable consumer-supplied identity; matches smoothed-gain and tone
    /// state across plan swaps. Never read in the render loop.
    pub stage_id: u64,
    /// The stage's output format.
    pub format: ChannelFormat,
    /// Linear stage output gain, smoothed (10 ms full swing) and applied
    /// where the stage's output is consumed. When `gain_automation` is
    /// present it replaces this value during playback.
    pub gain: f32,
    /// Optional compiled automation: sorted `(frame, linear gain)`
    /// breakpoints on the stream clock, linearly interpolated and sampled
    /// per block into sample-accurate ramps. Stateless: the value is a pure
    /// function of the stream position, so plan swaps stay continuous (the
    /// block ramp always starts from the inherited smoothed gain).
    pub gain_automation: Option<Vec<(u64, f32)>>,
    /// What the stage does.
    pub kind: RenderStageKind,
    /// Edges feeding this stage.
    pub inputs: Vec<RenderEdgeSpec>,
}

/// Control-side description of a render plan: a format-typed stage graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlanSpec {
    /// Sample rate the plan renders at.
    pub sample_rate_hz: u32,
    /// Linear master gain applied at the master stage (kept for consumer
    /// compatibility; multiplies the master stage's own gain).
    pub master_gain: f32,
    /// The stage graph; any order, exactly one [`RenderStageKind::Output`].
    pub stages: Vec<RenderStageSpec>,
}

impl RenderPlanSpec {
    /// When `other` differs from `self` ONLY in stage gains (same stages,
    /// formats, automation, clips, edges), return the changed
    /// `(stage_id, new_gain)` pairs — hosts use this to take the parameter
    /// fast path (`set_stage_gain`) instead of recompiling the plan.
    /// Returns `None` when anything structural differs (install required).
    pub fn differs_only_in_gains(&self, other: &RenderPlanSpec) -> Option<Vec<(u64, f32)>> {
        if self.sample_rate_hz != other.sample_rate_hz
            || self.master_gain != other.master_gain
            || self.stages.len() != other.stages.len()
        {
            return None;
        }
        let mut changes = Vec::new();
        for (old, new) in self.stages.iter().zip(other.stages.iter()) {
            if old.stage_id != new.stage_id
                || old.format != new.format
                || old.gain_automation != new.gain_automation
                || old.kind != new.kind
                || old.inputs != new.inputs
            {
                return None;
            }
            if old.gain != new.gain {
                changes.push((new.stage_id, new.gain));
            }
        }
        Some(changes)
    }

    /// Channel count of the master stage (the format the plan mixes at), or
    /// 2 when the spec has no master (such a spec fails compile anyway).
    pub fn output_channels(&self) -> u16 {
        self.stages
            .iter()
            .find(|stage| matches!(stage.kind, RenderStageKind::Output))
            .map(|stage| stage.format.channels)
            .unwrap_or(2)
    }
}

/// Typed error rejecting a plan spec at compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderPlanCompileError {
    /// Two stages share a `stage_id`.
    DuplicateNodeId(u64),
    /// An edge references a `source_stage_id` that does not exist.
    UnknownInputNode {
        /// Stage owning the edge.
        stage_id: u64,
        /// Missing upstream id.
        source_stage_id: u64,
    },
    /// The graph must contain exactly one output stage.
    MasterCount(usize),
    /// The graph contains a cycle and cannot be scheduled.
    Cycle,
    /// A stage declared a zero-channel format.
    InvalidChannelCount(u64),
    /// A gain-automation envelope's breakpoints are not sorted by frame.
    UnsortedAutomation(u64),
    /// An explicit edge matrix has the wrong number of coefficients.
    MatrixDimensions {
        /// Stage owning the edge.
        stage_id: u64,
        /// Upstream stage feeding the edge.
        source_stage_id: u64,
        /// `source_channels × dest_channels`.
        expected: usize,
        /// Length supplied.
        actual: usize,
    },
}

impl std::fmt::Display for RenderPlanCompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderPlanCompileError::DuplicateNodeId(stage_id) => {
                write!(formatter, "duplicate stage id {stage_id}")
            }
            RenderPlanCompileError::UnknownInputNode {
                stage_id,
                source_stage_id,
            } => write!(
                formatter,
                "stage {stage_id} references unknown input stage {source_stage_id}",
            ),
            RenderPlanCompileError::MasterCount(count) => write!(
                formatter,
                "plan must contain exactly one output stage (found {count})",
            ),
            RenderPlanCompileError::Cycle => {
                write!(formatter, "plan graph contains a cycle")
            }
            RenderPlanCompileError::InvalidChannelCount(stage_id) => {
                write!(formatter, "stage {stage_id} declares zero channels")
            }
            RenderPlanCompileError::UnsortedAutomation(stage_id) => write!(
                formatter,
                "stage {stage_id} gain automation breakpoints are not sorted by frame",
            ),
            RenderPlanCompileError::MatrixDimensions {
                stage_id,
                source_stage_id,
                expected,
                actual,
            } => write!(
                formatter,
                "edge {source_stage_id} -> {stage_id} matrix has {actual} coefficients, expected {expected}",
            ),
        }
    }
}

impl std::error::Error for RenderPlanCompileError {}

// ── Compiled plan (render-side data, preallocated at compile time) ─────────

enum CompiledSource {
    Silence,
    Tone {
        phase: f32,
        step: f32,
    },
    Samples {
        buffer: RenderSampleBuffer,
        /// Source frames advanced per stream frame (rate ratio).
        step: f64,
        /// Repeat from the source start once exhausted.
        loop_source: bool,
        /// Polyphase windowed-sinc table for rate-converted playback; `None`
        /// at 1:1, where samples are read directly.
        table: Option<PolyphaseInterpolationTable>,
    },
}

struct CompiledClip {
    start_frames: u64,
    end_frames: u64,
    /// Declick fade length at each window edge, shortened for tiny windows.
    edge_fade_frames: u64,
    /// Stable identity, read control-side when building inheritance maps.
    clip_id: u64,
    source: CompiledSource,
}

/// One compiled input edge. `source_index` is a position in the plan's
/// topologically-ordered stage list and is always strictly less than the
/// consuming stage's position, so the executor can split-borrow.
struct CompiledInput {
    source_index: usize,
    source_channels: usize,
    /// Row-major `source_channels × dest_channels`; edge gain folded in.
    matrix: Vec<f32>,
}

struct CompiledNode {
    /// Matches stage state (smoothed gain, tone phase) across plan swaps.
    stage_id: u64,
    channels: usize,
    /// Gain the stage is moving toward (spec value).
    gain_target: f32,
    /// Smoothed gain as currently applied; inherited across plan swaps so
    /// gain edits never step.
    gain_current: f32,
    /// Per-block smoothed-gain interpolation, written when the stage renders
    /// and read wherever its output is consumed (edges, boundary).
    block_gain_begin: f32,
    block_gain_slope: f32,
    /// Sorted automation breakpoints `(frame, gain)`; empty = no automation
    /// (static `gain_target` smoothing applies).
    gain_envelope: Vec<(u64, f32)>,
    /// Clip content (lane stages; empty for bus/master).
    clips: Vec<CompiledClip>,
    inputs: Vec<CompiledInput>,
    /// Interleaved scratch at the stage's format: `MAX_BLOCK_FRAMES × channels`.
    scratch: Vec<f32>,
}

/// A compiled, immutable-topology render plan. Source state (tone phases,
/// smoothed gains) mutates during rendering; structure never does. Nodes are
/// stored in topological order (inputs strictly before consumers).
pub struct RenderPlan {
    sample_rate_hz: u32,
    stages: Vec<CompiledNode>,
    /// Position of the master stage in `stages`.
    master_index: usize,
    /// Stream channel count the boundary was compiled for (master channels
    /// when the stream is unknown at install time).
    stream_channels: usize,
    /// Hardware-boundary downmix (row-major `output_channels ×
    /// stream_channels`); empty unless the stream is narrower than the
    /// master format. Compiled at install time — the controller knows the
    /// stream's channel count then. Never applies inside the creative graph.
    boundary_matrix: Vec<f32>,
    /// Per-stage inheritance: `inherit_stage_map[i]` is the index of the
    /// matching stage in the PREVIOUS plan (by stage_id), precomputed by the
    /// controller at install time so the executor never compares identities
    /// on the audio thread.
    inherit_stage_map: Vec<Option<usize>>,
    /// Per-stage, per-clip inheritance into the previous plan's clip list
    /// (by clip_id within the matched stage).
    inherit_clip_maps: Vec<Vec<Option<usize>>>,
}

impl RenderPlan {
    fn compile(
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
                    .map(|clip| CompiledClip {
                        clip_id: clip.clip_id,
                        start_frames: clip.start_frames,
                        end_frames: clip.end_frames,
                        edge_fade_frames: CLIP_EDGE_FADE_FRAMES
                            .min(clip.end_frames.saturating_sub(clip.start_frames).max(2) / 2),
                        source: match &clip.source {
                            RenderSource::Silence => CompiledSource::Silence,
                            RenderSource::TestTone { frequency_hz } => CompiledSource::Tone {
                                phase: 0.0,
                                step: frequency_hz * tau / stream_rate as f32,
                            },
                            RenderSource::Samples(buffer) => {
                                let step = buffer.sample_rate_hz.max(1) as f64 / stream_rate as f64;
                                CompiledSource::Samples {
                                    table: table_for_step(step),
                                    step,
                                    buffer: buffer.clone(),
                                    loop_source: clip.loop_source,
                                }
                            }
                        },
                    })
                    .collect(),
                RenderStageKind::Sum | RenderStageKind::Output => Vec::new(),
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
        }))
    }

    /// Carry smoothed gains and tone phases over from the plan being
    /// replaced, so a recompile (gain tweak, clip edit) never steps audio.
    /// Matching is precomputed by the controller into `inherit_stage_map` /
    /// `inherit_clip_maps` at install time (by stage_id and clip_id), so this
    /// is O(stages + clips) index copies — no identity comparisons run on
    /// the audio thread, and inserting a clip mid-lane no longer cross-wires
    /// its neighbours' state.
    fn inherit_state(&mut self, previous: &RenderPlan) {
        if self.inherit_stage_map.len() != self.stages.len() {
            // No map (first install or controller skipped): nothing carries.
            return;
        }
        for (index, stage) in self.stages.iter_mut().enumerate() {
            let Some(previous_index) = self.inherit_stage_map[index] else {
                continue;
            };
            let Some(previous_node) = previous.stages.get(previous_index) else {
                continue;
            };
            stage.gain_current = previous_node.gain_current;
            let clip_map = self
                .inherit_clip_maps
                .get(index)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for (clip_index, clip) in stage.clips.iter_mut().enumerate() {
                let Some(previous_clip_index) = clip_map.get(clip_index).copied().flatten() else {
                    continue;
                };
                let Some(previous_clip) = previous_node.clips.get(previous_clip_index) else {
                    continue;
                };
                if let (
                    CompiledSource::Tone { phase, step },
                    CompiledSource::Tone {
                        phase: previous_phase,
                        step: previous_step,
                    },
                ) = (&mut clip.source, &previous_clip.source)
                {
                    if *step == *previous_step {
                        *phase = *previous_phase;
                    }
                }
            }
        }
    }
}

/// Sample a sorted `(frame, value)` automation envelope at `frame`: linear
/// interpolation between breakpoints, clamped to the first/last values
/// outside the span. Binary search + arithmetic only — audio-thread safe.
#[inline]
fn sample_envelope(points: &[(u64, f32)], frame: u64) -> f32 {
    match points.binary_search_by(|(point_frame, _)| point_frame.cmp(&frame)) {
        Ok(index) => points[index].1,
        Err(0) => points[0].1,
        Err(index) if index == points.len() => points[points.len() - 1].1,
        Err(index) => {
            let (start_frame, start_value) = points[index - 1];
            let (end_frame, end_value) = points[index];
            let span = (end_frame - start_frame).max(1) as f64;
            let progress = (frame - start_frame) as f64 / span;
            start_value + (end_value - start_value) * progress as f32
        }
    }
}

/// Declick gain for a frame inside a clip window: linear fade over
/// `fade_frames` at each edge, unity in between.
#[inline]
fn clip_edge_gain(frame: u64, start_frames: u64, end_frames: u64, fade_frames: u64) -> f32 {
    if fade_frames == 0 {
        return 1.0;
    }
    let fade = fade_frames as f32;
    let from_start = (frame - start_frames + 1) as f32;
    let to_end = (end_frames - frame) as f32;
    (from_start / fade).min(to_end / fade).min(1.0)
}

/// Render a lane stage's clips into its scratch at unity gain (stage and edge
/// gains apply downstream, where the scratch is consumed). Sources write
/// `channels.min(2)` of the stage's format: clip sources are mono/stereo
/// today; source-format handling generalizes when sources grow formats of
/// their own.
fn render_clips_into_scratch(
    clips: &mut [CompiledClip],
    scratch: &mut [f32],
    channels: usize,
    block_start_frame: u64,
    frame_count: usize,
) {
    let block_end_frame = block_start_frame + frame_count as u64;
    for clip in clips.iter_mut() {
        // Skip clips entirely outside this block.
        if clip.end_frames <= block_start_frame || clip.start_frames >= block_end_frame {
            continue;
        }
        let clip_start = clip.start_frames;
        let clip_end = clip.end_frames;
        let clip_fade = clip.edge_fade_frames;
        match &mut clip.source {
            CompiledSource::Silence => {}
            CompiledSource::Tone { phase, step } => {
                let mut local_phase = *phase;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    let sample = local_phase.sin();
                    local_phase += *step;
                    if local_phase >= std::f32::consts::TAU {
                        local_phase -= std::f32::consts::TAU;
                    }
                    if frame >= clip_start && frame < clip_end {
                        let sample =
                            sample * clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                        let base = frame_index * channels;
                        for channel in 0..channels.min(2) {
                            scratch[base + channel] += sample;
                        }
                    }
                }
                *phase = local_phase;
            }
            CompiledSource::Samples {
                buffer,
                step,
                loop_source,
                table,
            } => {
                let source_frames = buffer.frame_count();
                if source_frames == 0 {
                    continue;
                }
                let data = &buffer.frames;
                for frame_index in 0..frame_count {
                    let frame = block_start_frame + frame_index as u64;
                    if frame < clip_start || frame >= clip_end {
                        continue;
                    }
                    // Source position via the rate ratio, anchored at the
                    // clip's window start.
                    let mut source_position = (frame - clip_start) as f64 * *step;
                    if *loop_source {
                        source_position %= source_frames as f64;
                    }
                    let source_index = source_position as usize;
                    if source_index >= source_frames {
                        continue;
                    }
                    let fraction = source_position - source_index as f64;
                    let window_gain = clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                    let base = frame_index * channels;
                    match table {
                        // Rate conversion: polyphase windowed-sinc tap dot
                        // product (table reads only — no allocation, no
                        // transcendentals).
                        Some(table) => {
                            let row = table.phase_row(fraction);
                            let first = table.first_tap_offset();
                            for channel in 0..channels.min(2) {
                                let mut acc = 0.0f32;
                                for (tap, coefficient) in row.iter().enumerate() {
                                    let mut tap_index =
                                        source_index as isize + first + tap as isize;
                                    if *loop_source {
                                        tap_index = tap_index.rem_euclid(source_frames as isize);
                                    }
                                    if tap_index >= 0 && (tap_index as usize) < source_frames {
                                        acc += data[tap_index as usize * 2 + channel] * coefficient;
                                    }
                                }
                                scratch[base + channel] += acc * window_gain;
                            }
                        }
                        // 1:1 playback: direct read with last-frame clamp
                        // (or wrap when looping).
                        None => {
                            let next_index = if source_index + 1 < source_frames {
                                source_index + 1
                            } else if *loop_source {
                                0
                            } else {
                                source_index
                            };
                            let fraction = fraction as f32;
                            for channel in 0..channels.min(2) {
                                let a = data[source_index * 2 + channel];
                                let b = data[next_index * 2 + channel];
                                scratch[base + channel] += (a + (b - a) * fraction) * window_gain;
                            }
                        }
                    }
                }
            }
        }
    }
}

enum RenderCommand {
    InstallPlan(Box<RenderPlan>),
    SetPlaying(bool),
    Seek(u64),
    SetStreamChannels(u16),
    /// Parameter fast path: retarget one stage's smoothed gain without a
    /// plan recompile. `stage_index` addresses the ACTIVE plan's topological
    /// stage list; the controller resolves stage ids against the topology of
    /// the most recent install, and the FIFO mailbox guarantees the command
    /// lands after that plan.
    SetStageGain {
        stage_index: usize,
        target: f32,
    },
}

/// Counters shared between the two sides without locks.
#[derive(Debug, Default)]
struct SharedState {
    /// Stream-clock position in frames, written by the executor.
    position_frames: AtomicU64,
    /// Transport gate as last applied by the executor.
    playing: AtomicBool,
    /// Blocks rendered while a retired plan could not be returned because the
    /// retired mailbox was full (plan held in the parking slot instead —
    /// never dropped on the audio thread).
    retired_parked_blocks: AtomicU64,
}

/// Control-side handle: compiles and installs plans, drives transport, and
/// deallocates retired plans.
pub struct RenderPlaneController {
    commands: SyncSender<RenderCommand>,
    retired: Receiver<Box<RenderPlan>>,
    shared: Arc<SharedState>,
    /// Stream channel count as reported by the host. Plans compile their
    /// hardware-boundary adaptation against this; before it is known, plans
    /// assume the stream matches their master format.
    stream_channels: Option<u16>,
    /// Identity snapshot of the last successfully installed plan, in its
    /// topological stage order: (stage_id, clip ids). Used to precompute the
    /// state-inheritance maps for the next install so the executor does pure
    /// index copies.
    last_topology: Option<Vec<(u64, Vec<u64>)>>,
}

/// Error installing a plan or sending a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlaneError {
    /// Human-readable description.
    pub message: String,
}

impl std::fmt::Display for RenderPlaneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "render plane error: {}", self.message)
    }
}

impl std::error::Error for RenderPlaneError {}

impl RenderPlaneController {
    /// Record the channel count of the opened output stream and inform the
    /// executor (output framing keys off the *stream*, not the plan's master
    /// format). Subsequent installs compile their hardware-boundary
    /// adaptation against this count.
    pub fn set_stream_channels(&mut self, channels: u16) -> Result<(), RenderPlaneError> {
        self.stream_channels = Some(channels);
        self.commands
            .try_send(RenderCommand::SetStreamChannels(channels))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected stream channel update: {error}"),
            })
    }

    /// Compile `spec` and install it as the active plan. Eagerly reclaims
    /// any plans the executor has retired. State-inheritance maps (stage and
    /// clip identity vs the previously installed plan) are precomputed here,
    /// control-side, so the executor's hand-off is pure index copies.
    pub fn install_plan(&mut self, spec: &RenderPlanSpec) -> Result<(), RenderPlaneError> {
        self.collect_retired();
        let mut plan =
            RenderPlan::compile(spec, self.stream_channels).map_err(|error| RenderPlaneError {
                message: format!("plan rejected at compile: {error}"),
            })?;

        // Topology of the freshly compiled plan (topo order).
        let topology: Vec<(u64, Vec<u64>)> = plan
            .stages
            .iter()
            .map(|stage| {
                (
                    stage.stage_id,
                    stage.clips.iter().map(|clip| clip.clip_id).collect(),
                )
            })
            .collect();

        if let Some(previous) = self.last_topology.as_ref() {
            plan.inherit_stage_map = topology
                .iter()
                .map(|(stage_id, _)| {
                    previous
                        .iter()
                        .position(|(previous_id, _)| previous_id == stage_id)
                })
                .collect();
            plan.inherit_clip_maps = topology
                .iter()
                .enumerate()
                .map(|(index, (_, clip_ids))| {
                    let Some(previous_index) = plan.inherit_stage_map[index] else {
                        return Vec::new();
                    };
                    let previous_clips = &previous[previous_index].1;
                    clip_ids
                        .iter()
                        .map(|clip_id| {
                            previous_clips
                                .iter()
                                .position(|previous_id| previous_id == clip_id)
                        })
                        .collect()
                })
                .collect();
        }

        self.commands
            .try_send(RenderCommand::InstallPlan(plan))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected plan install: {error}"),
            })?;
        self.last_topology = Some(topology);
        Ok(())
    }

    /// Parameter fast path: retarget one stage's smoothed gain without a
    /// plan recompile. Resolves `stage_id` against the topology of the most
    /// recent successful install; the FIFO mailbox guarantees the command
    /// reaches the executor after that plan. Returns an error when the
    /// stage is unknown (callers should fall back to a plan install).
    pub fn set_stage_gain(&self, stage_id: u64, target: f32) -> Result<(), RenderPlaneError> {
        let Some(topology) = self.last_topology.as_ref() else {
            return Err(RenderPlaneError {
                message: "no plan installed; cannot set stage gain".to_string(),
            });
        };
        let Some(stage_index) = topology.iter().position(|(id, _)| *id == stage_id) else {
            return Err(RenderPlaneError {
                message: format!("stage {stage_id} not present in the installed plan"),
            });
        };
        self.commands
            .try_send(RenderCommand::SetStageGain {
                stage_index,
                target,
            })
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected stage gain: {error}"),
            })
    }

    /// Gate rendering on or off (transport play/stop).
    pub fn set_playing(&self, playing: bool) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::SetPlaying(playing))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected transport gate: {error}"),
            })
    }

    /// Move the stream clock to `position_frames`.
    pub fn seek(&self, position_frames: u64) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::Seek(position_frames))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected seek: {error}"),
            })
    }

    /// Current stream-clock position in frames as last written by the
    /// executor.
    pub fn position_frames(&self) -> u64 {
        self.shared.position_frames.load(Ordering::Relaxed)
    }

    /// Whether the executor is currently rendering (transport gate as
    /// applied).
    pub fn playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    /// Drain and deallocate plans the executor has retired. Call this
    /// periodically from the control side; returns how many were freed.
    pub fn collect_retired(&self) -> usize {
        let mut freed = 0;
        while self.retired.try_recv().is_ok() {
            freed += 1;
        }
        freed
    }

    /// Diagnostic: blocks during which a retired plan sat parked because the
    /// retired mailbox was full.
    pub fn retired_parked_blocks(&self) -> u64 {
        self.shared.retired_parked_blocks.load(Ordering::Relaxed)
    }
}

/// Render-side executor. Owns the active plan between blocks and renders it
/// inside the audio callback.
///
/// # Real-time contract
///
/// [`RenderPlaneExecutor::render_block`] performs no allocation, takes no
/// locks, and does no I/O. Plan hand-off uses bounded array-backed channels;
/// retired plans are returned to the control side (or parked locally when the
/// return mailbox is full) and are never dropped on the audio thread.
pub struct RenderPlaneExecutor {
    commands: Receiver<RenderCommand>,
    retired: SyncSender<Box<RenderPlan>>,
    shared: Arc<SharedState>,
    plan: Option<Box<RenderPlan>>,
    parked_retired: Option<Box<RenderPlan>>,
    /// Stream channel count as told by the controller; the active plan's
    /// expectation (its master format when no stream was known at install)
    /// applies until the first [`RenderCommand::SetStreamChannels`].
    stream_channels: Option<u16>,
    /// Transport gate target. Audio follows through `edge_gain`.
    playing: bool,
    /// Transport edge envelope: ramps toward 1 when playing, 0 when stopped
    /// or before an in-flight seek, so transport actions never step audio.
    edge_gain: f32,
    /// Seek requested while audible: applied once `edge_gain` reaches zero.
    pending_seek: Option<u64>,
    position_frames: u64,
}

impl RenderPlaneExecutor {
    fn retire(&mut self, plan: Box<RenderPlan>) {
        // Re-offer any parked plan first to preserve retirement order.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }
        if let Err(TrySendError::Full(plan)) | Err(TrySendError::Disconnected(plan)) =
            self.retired.try_send(plan)
        {
            self.shared
                .retired_parked_blocks
                .fetch_add(1, Ordering::Relaxed);
            debug_assert!(
                self.parked_retired.is_none(),
                "retired mailbox capacity invariant violated",
            );
            // Never deallocate on the audio thread: hold the plan and
            // re-offer it next block. The mailbox capacity invariant makes
            // double saturation unreachable in correct use.
            if self.parked_retired.is_none() {
                self.parked_retired = Some(plan);
            }
        }
    }

    fn apply_seek(&mut self, position_frames: u64) {
        self.position_frames = position_frames;
        self.shared
            .position_frames
            .store(position_frames, Ordering::Relaxed);
    }

    fn drain_commands(&mut self) {
        loop {
            match self.commands.try_recv() {
                Ok(RenderCommand::InstallPlan(mut next_plan)) => {
                    if let Some(previous) = self.plan.take() {
                        next_plan.inherit_state(&previous);
                        self.retire(previous);
                    }
                    self.plan = Some(next_plan);
                }
                Ok(RenderCommand::SetPlaying(playing)) => {
                    self.playing = playing;
                    self.shared.playing.store(playing, Ordering::Relaxed);
                }
                Ok(RenderCommand::Seek(position_frames)) => {
                    if self.edge_gain <= 0.0 {
                        // Inaudible: jump immediately.
                        self.pending_seek = None;
                        self.apply_seek(position_frames);
                    } else {
                        // Audible: ramp out first, jump at the zero crossing,
                        // ramp back in (handled in render_block).
                        self.pending_seek = Some(position_frames);
                    }
                }
                Ok(RenderCommand::SetStageGain {
                    stage_index,
                    target,
                }) => {
                    if let Some(plan) = self.plan.as_mut() {
                        if let Some(stage) = plan.stages.get_mut(stage_index) {
                            stage.gain_target = target;
                        }
                    }
                }
                Ok(RenderCommand::SetStreamChannels(channels)) => {
                    self.stream_channels = Some(channels.max(1));
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Render one callback quantum into `frames` (interleaved f32 at the
    /// stream's channel count).
    ///
    /// Safe on the audio thread: no allocation, no locks, no I/O.
    pub fn render_block(&mut self, frames: &mut [f32]) {
        self.drain_commands();

        // Re-offer a parked retired plan without rendering work attached.
        if let Some(parked) = self.parked_retired.take() {
            if let Err(TrySendError::Full(parked)) | Err(TrySendError::Disconnected(parked)) =
                self.retired.try_send(parked)
            {
                self.parked_retired = Some(parked);
            }
        }

        frames.fill(0.0);

        let Some(plan) = self.plan.as_mut() else {
            return;
        };
        // Audible while the gate is open, while ramping out after a stop, or
        // while ramping around an in-flight seek.
        if !self.playing && self.edge_gain <= 0.0 {
            return;
        }

        let stream_channels = self
            .stream_channels
            .map(|channels| channels as usize)
            .unwrap_or(plan.stream_channels)
            .max(1);
        let frame_count = frames.len() / stream_channels;
        debug_assert!(
            frame_count <= MAX_BLOCK_FRAMES,
            "render_block called with {frame_count} frames; scratch holds {MAX_BLOCK_FRAMES}",
        );
        let frame_count = frame_count.min(MAX_BLOCK_FRAMES);
        let block_start_frame = self.position_frames;

        // Smoothed gains: move toward targets at a fixed full-swing rate and
        // interpolate across the block, so edits never step audio.
        let gain_step =
            frame_count as f32 / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0);

        // Walk the schedule: stages are in topological order, so every edge's
        // source sits strictly earlier and split-borrowing is safe.
        for node_index in 0..plan.stages.len() {
            let (earlier, rest) = plan.stages.split_at_mut(node_index);
            let stage = &mut rest[0];

            let gain_begin = stage.gain_current;
            let gain_end = if stage.gain_envelope.is_empty() {
                gain_begin + (stage.gain_target - gain_begin).clamp(-gain_step, gain_step)
            } else {
                // Automation: the target is the envelope's value at the
                // block end. The ramp starts from the inherited smoothed
                // gain, so plan swaps and seeks stay continuous while normal
                // playback tracks the curve sample-accurately block-wise.
                sample_envelope(&stage.gain_envelope, block_start_frame + frame_count as u64)
            };
            stage.block_gain_begin = gain_begin;
            stage.block_gain_slope = (gain_end - gain_begin) / frame_count.max(1) as f32;
            stage.gain_current = gain_end;

            let CompiledNode {
                channels,
                clips,
                inputs,
                scratch,
                ..
            } = stage;
            let channels = *channels;
            let scratch = &mut scratch[..frame_count * channels];
            scratch.fill(0.0);

            // Lane content first (at unity), then summed inputs.
            render_clips_into_scratch(clips, scratch, channels, block_start_frame, frame_count);

            for input in inputs.iter() {
                let source = &earlier[input.source_index];
                let source_channels = input.source_channels;
                let matrix = &input.matrix;
                // Per-frame: dest[c_out] += src[c_in] * m[c_in][c_out] *
                // source stage's smoothed gain (edge gain is folded into the
                // matrix at compile). Plain loops, alloc-free.
                for frame_index in 0..frame_count {
                    let source_gain =
                        source.block_gain_begin + source.block_gain_slope * frame_index as f32;
                    let source_base = frame_index * source_channels;
                    let dest_base = frame_index * channels;
                    for source_channel in 0..source_channels {
                        let sample = source.scratch[source_base + source_channel] * source_gain;
                        let row = &matrix[source_channel * channels..][..channels];
                        for (dest_channel, coefficient) in row.iter().enumerate() {
                            scratch[dest_base + dest_channel] += sample * coefficient;
                        }
                    }
                }
            }
        }

        // Hardware boundary: adapt the master stage's scratch onto the stream
        // with the master's smoothed gain. Equal formats copy; a narrower
        // stream applies the install-time downmix matrix; a wider stream
        // carries the master's channels and leaves the extras silent (the
        // creative mix never upmixes at the hardware stage).
        let master = &plan.stages[plan.master_index];
        let output_channels = master.channels;
        let boundary_matrix = &plan.boundary_matrix;
        let boundary_matrix_valid = boundary_matrix.len() == output_channels * stream_channels;
        for frame_index in 0..frame_count {
            let master_gain =
                master.block_gain_begin + master.block_gain_slope * frame_index as f32;
            let source_base = frame_index * output_channels;
            let dest_base = frame_index * stream_channels;
            if stream_channels < output_channels && boundary_matrix_valid {
                for dest_channel in 0..stream_channels {
                    let mut acc = 0.0f32;
                    for source_channel in 0..output_channels {
                        acc += master.scratch[source_base + source_channel]
                            * boundary_matrix[source_channel * stream_channels + dest_channel];
                    }
                    frames[dest_base + dest_channel] = acc * master_gain;
                }
            } else {
                // Equal formats, a wider stream (extras stay zero-filled),
                // or a stale boundary (stream changed without a reinstall):
                // copy the overlapping channels.
                for channel in 0..output_channels.min(stream_channels) {
                    frames[dest_base + channel] =
                        master.scratch[source_base + channel] * master_gain;
                }
            }
        }

        // Transport edge envelope over the mixed block: ramps toward the
        // gate target (zero while a seek is in flight) and never steps.
        let edge_target = if self.pending_seek.is_some() {
            0.0
        } else if self.playing {
            1.0
        } else {
            0.0
        };
        if self.edge_gain != edge_target || edge_target < 1.0 {
            let edge_step = 1.0 / (EDGE_RAMP_SECONDS * plan.sample_rate_hz as f32).max(1.0);
            for frame_index in 0..frame_count {
                self.edge_gain += (edge_target - self.edge_gain).clamp(-edge_step, edge_step);
                let base = frame_index * stream_channels;
                for channel in 0..stream_channels {
                    frames[base + channel] *= self.edge_gain;
                }
            }
        }

        self.position_frames += frame_count as u64;
        self.shared
            .position_frames
            .store(self.position_frames, Ordering::Relaxed);

        // Seek lands at the envelope's zero crossing; the next block ramps
        // back in from the new position.
        if self.edge_gain <= 0.0 {
            if let Some(position) = self.pending_seek.take() {
                self.apply_seek(position);
            }
        }
    }
}

/// Create a connected controller/executor pair.
pub fn render_plane() -> (RenderPlaneController, RenderPlaneExecutor) {
    let (command_tx, command_rx) = sync_channel(COMMAND_MAILBOX_CAPACITY);
    let (retired_tx, retired_rx) = sync_channel(RETIRED_MAILBOX_CAPACITY);
    let shared = Arc::new(SharedState::default());
    (
        RenderPlaneController {
            commands: command_tx,
            retired: retired_rx,
            shared: Arc::clone(&shared),
            stream_channels: None,
            last_topology: None,
        },
        RenderPlaneExecutor {
            commands: command_rx,
            retired: retired_tx,
            shared,
            plan: None,
            parked_retired: None,
            stream_channels: None,
            playing: false,
            edge_gain: 0.0,
            pending_seek: None,
            position_frames: 0,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_dsp::equal_power_pan_matrix;

    const MASTER_ID: u64 = 1_000;
    const LANE_ID: u64 = 1;

    fn lane_node(stage_id: u64, gain: f32, clips: Vec<RenderClipSpec>) -> RenderStageSpec {
        RenderStageSpec {
            stage_id,
            format: ChannelFormat::stereo(),
            gain,
            gain_automation: None,
            kind: RenderStageKind::Source { clips },
            inputs: Vec::new(),
        }
    }

    fn master_node(inputs: Vec<RenderEdgeSpec>) -> RenderStageSpec {
        RenderStageSpec {
            stage_id: MASTER_ID,
            format: ChannelFormat::stereo(),
            gain: 1.0,
            gain_automation: None,
            kind: RenderStageKind::Output,
            inputs,
        }
    }

    fn identity_edge(source_stage_id: u64) -> RenderEdgeSpec {
        RenderEdgeSpec {
            source_stage_id,
            gain: 1.0,
            matrix: None,
        }
    }

    /// The old flat shape: one stereo lane summed into a stereo master.
    fn lane_master_spec(lane_gain: f32, clips: Vec<RenderClipSpec>) -> RenderPlanSpec {
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                lane_node(LANE_ID, lane_gain, clips),
                master_node(vec![identity_edge(LANE_ID)]),
            ],
        }
    }

    fn tone_clip(frequency_hz: f32) -> RenderClipSpec {
        RenderClipSpec {
            clip_id: 1003,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone { frequency_hz },
            loop_source: false,
        }
    }

    fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
        lane_master_spec(0.5, vec![tone_clip(frequency_hz)])
    }

    #[test]
    fn renders_silence_without_plan_and_when_stopped() {
        let (mut controller, mut executor) = render_plane();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        controller.install_plan(&tone_spec(440.0)).unwrap();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(controller.position_frames(), 0);
    }

    #[test]
    fn renders_tone_and_advances_clock_when_playing() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
        assert_eq!(controller.position_frames(), 256);
        assert!(controller.playing());

        // Both channels carry the same mono sum.
        assert_eq!(frames[10], frames[11]);
    }

    #[test]
    fn seek_moves_the_stream_clock() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        controller.seek(96_000).unwrap();

        let mut frames = [0.0f32; 128];
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 96_000 + 64);
    }

    #[test]
    fn windows_gate_lane_audibility_on_the_stream_clock() {
        let (mut controller, mut executor) = render_plane();
        let mut clip = tone_clip(440.0);
        clip.start_frames = 128;
        clip.end_frames = 256;
        let spec = lane_master_spec(0.5, vec![clip]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Block 0 covers frames 0..128: outside the window, silent.
        let mut frames = [0.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        // Block 1 covers frames 128..256: inside the window, audible.
        let mut frames = [0.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    }

    fn samples_spec(
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
                source: RenderSource::Samples(RenderSampleBuffer {
                    sample_rate_hz: 48_000,
                    frames: data.into(),
                }),
                loop_source,
            }],
        )
    }

    /// Run blocks until the transport edge ramp has fully opened.
    fn warm_up(executor: &mut RenderPlaneExecutor, blocks: usize) {
        let mut frames = [0.0f32; 512];
        for _ in 0..blocks {
            executor.render_block(&mut frames);
        }
    }

    #[test]
    fn sample_clips_play_buffer_content_at_their_window() {
        let (mut controller, mut executor) = render_plane();
        // 1024 source frames: value = index / 1024.
        let values: Vec<f32> = (0..1024).map(|index| index as f32 / 1024.0).collect();
        // Window starts at frame 512, well past the edge ramp warm-up.
        let spec = samples_spec(&values, 512, 512 + 1024, false);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Two 256-frame blocks open the edge ramp and reach frame 512.
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 512+128 plays source frame 128, past the clip edge fade.
        let index = 128usize;
        let expected = 128.0 / 1024.0;
        assert!((frames[index * 2] - expected).abs() < 1e-5);
        // Same-rate playback: equality on both channels.
        assert_eq!(frames[index * 2], frames[index * 2 + 1]);
    }

    #[test]
    fn sample_clips_play_their_final_frame() {
        let (mut controller, mut executor) = render_plane();
        // 256 source frames of a constant; window longer than the source.
        let values = vec![0.5f32; 256];
        let spec = samples_spec(&values, 0, u64::MAX, false);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 1);

        // Frames 0..256 played in the warm-up block. The final source frame
        // (255) must have rendered; beyond the source, silence.
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));

        // Replay from the start and inspect the last in-range frame.
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 255 is the final source frame; with the clamp it plays.
        assert!(frames[255 * 2].abs() > 0.1);
    }

    #[test]
    fn looping_sample_clips_wrap_to_their_start() {
        let (mut controller, mut executor) = render_plane();
        // 100 source frames: value = (index + 1) / 100, looped.
        let values: Vec<f32> = (0..100).map(|index| (index + 1) as f32 / 100.0).collect();
        let spec = samples_spec(&values, 0, u64::MAX, true);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2); // 512 frames: ramp open, loop wrapped 5x.

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Block covers frames 512..768; frame 512 plays source 512 % 100 = 12.
        let expected = 13.0 / 100.0;
        assert!((frames[0] - expected).abs() < 1e-5);
        // Frame 600 wraps to source 0.
        let wrapped = (600 - 512) * 2;
        assert!((frames[wrapped] - 1.0 / 100.0).abs() < 1e-5);
    }

    #[test]
    fn transport_stop_ramps_out_instead_of_stepping() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        controller.set_playing(false).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        // Ramp-out block: starts audible, ends silent, no step bigger than
        // the tone's own slope plus the ramp slope.
        assert!(frames[0].abs() > 0.0 || frames[2].abs() > 0.0);
        let tail = &frames[1000..];
        assert!(tail.iter().all(|sample| *sample == 0.0));
        let max_step = frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(max_step < 0.05, "stop produced a step of {max_step}");

        // Fully stopped afterwards: silence and a held clock.
        let position = controller.position_frames();
        let mut frames = [1.0f32; 256];
        executor.render_block(&mut frames);
        assert!(frames.iter().all(|sample| *sample == 0.0));
        assert_eq!(controller.position_frames(), position);
    }

    #[test]
    fn seek_while_playing_ramps_out_then_jumps() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);
        let before = controller.position_frames();

        controller.seek(96_000).unwrap();
        let mut frames = [0.0f32; 512];
        // Ramp-out block at the old position; seek lands at its end.
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 96_000);
        let _ = before;
        // Next block plays from the new position, ramping back in.
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
        assert_eq!(controller.position_frames(), 96_000 + 256);
    }

    fn max_left_step(frames: &[f32]) -> f32 {
        frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max)
    }

    #[test]
    fn set_stage_gain_retargets_without_recompile() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Fast path: no install, just a retarget; the smoothing ramp keeps
        // the transition step-free.
        controller.set_stage_gain(LANE_ID, 1.0).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        assert!(step < 0.08, "fast-path gain stepped audio by {step}");

        // Unknown stage: typed error, callers fall back to install.
        assert!(controller.set_stage_gain(999, 0.5).is_err());
    }

    #[test]
    fn gain_automation_follows_the_envelope_sample_accurately() {
        let (mut controller, mut executor) = render_plane();
        // Constant-amplitude source (DC-ish loopable samples) under a gain
        // ramp envelope 0.0 -> 1.0 over 9600 frames, then hold.
        let values = vec![0.5f32; 480];
        let mut spec = samples_spec(&values, 0, u64::MAX, true);
        spec.stages[0].gain_automation = Some(vec![(0, 0.0), (9_600, 1.0), (19_200, 0.25)]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        // Render 19_200 frames in 256-frame blocks; spot-check the envelope.
        let mut output = Vec::new();
        let mut frames = [0.0f32; 512];
        for _ in 0..75 {
            executor.render_block(&mut frames);
            output.extend(frames.chunks_exact(2).map(|frame| frame[0]));
        }
        // At frame 9_600 the gain is 1.0: sample value 0.5 * 1.0.
        let mid = output[9_600];
        assert!((mid - 0.5).abs() < 0.02, "envelope peak read {mid}");
        // At frame 14_400 (halfway down to 0.25): gain ≈ 0.625.
        let down = output[14_400];
        assert!(
            (down - 0.5 * 0.625).abs() < 0.02,
            "envelope descent read {down}"
        );
        // Monotonic rise across the first segment (block-ramped).
        assert!(output[2_000] < output[4_000] && output[4_000] < output[8_000]);
    }

    #[test]
    fn envelope_swap_mid_play_stays_continuous() {
        let (mut controller, mut executor) = render_plane();
        let values = vec![0.5f32; 480];
        let mut spec = samples_spec(&values, 0, u64::MAX, true);
        spec.stages[0].gain_automation = Some(vec![(0, 1.0)]);
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Swap to a very different envelope mid-play: the block ramp anchors
        // at the inherited smoothed gain, so no step.
        let mut louder = samples_spec(&values, 0, u64::MAX, true);
        louder.stages[0].gain_automation = Some(vec![(0, 0.1)]);
        controller.install_plan(&louder).unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        assert!(step < 0.05, "envelope swap stepped audio by {step}");
    }

    #[test]
    fn gain_only_spec_diffs_take_the_fast_path() {
        let base = tone_spec(440.0);
        let mut louder = base.clone();
        louder.stages[0].gain = 0.9;
        assert_eq!(
            base.differs_only_in_gains(&louder),
            Some(vec![(LANE_ID, 0.9)])
        );
        // Structural change: no fast path.
        let mut reshaped = base.clone();
        reshaped.stages[0].gain_automation = Some(vec![(0, 1.0)]);
        assert_eq!(base.differs_only_in_gains(&reshaped), None);
        assert_eq!(base.differs_only_in_gains(&base), Some(vec![]));
    }

    #[test]
    fn mid_lane_clip_insert_preserves_neighbour_state() {
        // A tone clip keeps its phase when a new clip is inserted BEFORE it
        // in the lane's clip list — the clip-id inheritance map prevents the
        // zip-index cross-wiring the old code had.
        let (mut controller, mut executor) = render_plane();
        let survivor = RenderClipSpec {
            clip_id: 7,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone {
                frequency_hz: 440.0,
            },
            loop_source: false,
        };
        controller
            .install_plan(&lane_master_spec(0.5, vec![survivor.clone()]))
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Insert a silent clip at index 0; survivor moves to index 1.
        let inserted = RenderClipSpec {
            clip_id: 8,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::Silence,
            loop_source: false,
        };
        controller
            .install_plan(&lane_master_spec(0.5, vec![inserted, survivor]))
            .unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        // Phase carried: the 440 Hz tone continues without a step.
        let step = max_left_step(&frames);
        assert!(step < 0.05, "clip insert stepped audio by {step}");
        assert!(frames.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn stage_reorder_preserves_state_through_the_identity_map() {
        // Two tone lanes swap positions in the stage list across a plan
        // swap; both keep phase and smoothed gain (no audible step).
        let (mut controller, mut executor) = render_plane();
        let lane_a = lane_node(10, 0.4, vec![tone_clip(330.0)]);
        let lane_b = lane_node(11, 0.4, vec![tone_clip(550.0)]);
        let master = master_node(vec![identity_edge(10), identity_edge(11)]);
        controller
            .install_plan(&RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                stages: vec![lane_a.clone(), lane_b.clone(), master.clone()],
            })
            .unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        controller
            .install_plan(&RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                stages: vec![lane_b, lane_a, master],
            })
            .unwrap();
        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let step = max_left_step(&frames);
        // Two tones at 0.4 gain: their combined slope stays well under this
        // bound only if both phases carried.
        assert!(step < 0.07, "stage reorder stepped audio by {step}");
    }

    #[test]
    fn plan_churn_keeps_a_surviving_tone_continuous() {
        // Property-style: a seeded LCG drives 24 plan installs mid-play —
        // adding/removing extra lanes, inserting silent clips around the
        // survivor, jittering other lanes' gains. The surviving tone lane
        // must never step.
        let (mut controller, mut executor) = render_plane();
        let survivor_clip = RenderClipSpec {
            clip_id: 1,
            start_frames: 0,
            end_frames: u64::MAX,
            source: RenderSource::TestTone {
                frequency_hz: 440.0,
            },
            loop_source: false,
        };
        let mut seed: u64 = 0x5EED_CAFE;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        let build = |extra_lanes: u32, clips_before: u32, extra_gain: f32| -> RenderPlanSpec {
            let mut clips = Vec::new();
            for index in 0..clips_before {
                clips.push(RenderClipSpec {
                    clip_id: 100 + index as u64,
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Silence,
                    loop_source: false,
                });
            }
            clips.push(survivor_clip.clone());
            let mut stages = vec![lane_node(LANE_ID, 0.5, clips)];
            let mut edges = vec![identity_edge(LANE_ID)];
            for index in 0..extra_lanes {
                let stage_id = 50 + index as u64;
                // Extra lanes are silent so the survivor's continuity is the
                // only signal under test.
                stages.push(lane_node(stage_id, extra_gain, vec![]));
                edges.push(identity_edge(stage_id));
            }
            stages.push(master_node(edges));
            RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                stages,
            }
        };
        controller.install_plan(&build(0, 0, 0.3)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut worst_step = 0.0f32;
        let mut previous_tail = None::<f32>;
        for _ in 0..24 {
            let extra = next() % 4;
            let before = next() % 3;
            let gain = (next() % 100) as f32 / 100.0;
            controller
                .install_plan(&build(extra, before, gain))
                .unwrap();
            let mut frames = [0.0f32; 512];
            executor.render_block(&mut frames);
            if let Some(tail) = previous_tail {
                worst_step = worst_step.max((frames[0] - tail).abs());
            }
            worst_step = worst_step.max(max_left_step(&frames));
            previous_tail = Some(frames[frames.len() - 2]);
        }
        assert!(
            worst_step < 0.05,
            "plan churn stepped the surviving tone by {worst_step}",
        );
    }

    #[test]
    fn plan_swap_inherits_smoothed_gain_without_stepping() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Same plan with lane gain doubled: swap mid-play. Stage ids are
        // stable, so the smoothed gain carries over and ramps.
        let louder = lane_master_spec(1.0, vec![tone_clip(440.0)]);
        controller.install_plan(&louder).unwrap();

        let mut frames = [0.0f32; 1024];
        executor.render_block(&mut frames);
        let max_step = frames
            .chunks_exact(2)
            .map(|frame| frame[0])
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .fold(0.0f32, f32::max);
        // 440 Hz at 48k moves at most ~0.058/sample at unity; the gain ramp
        // must not add a visible step on top.
        assert!(max_step < 0.08, "gain swap produced a step of {max_step}");
    }

    #[test]
    fn rate_converted_clips_play_through_the_sinc_path() {
        // 1 kHz sine at 44.1k played on a 48k stream: after the edge ramp
        // and clip fade, output must track the analytic sine to ~60 dB
        // (linear interpolation fails this at ~35-40 dB).
        let (mut controller, mut executor) = render_plane();
        let source_rate = 44_100u32;
        let frequency = 1_000.0f64;
        let mut data = Vec::new();
        for n in 0..44_100 {
            let value =
                (std::f64::consts::TAU * frequency * n as f64 / source_rate as f64).sin() as f32;
            data.push(value);
            data.push(value);
        }
        let spec = lane_master_spec(
            1.0,
            vec![RenderClipSpec {
                clip_id: 1005,
                start_frames: 0,
                end_frames: u64::MAX,
                source: RenderSource::Samples(RenderSampleBuffer {
                    sample_rate_hz: source_rate,
                    frames: data.into(),
                }),
                loop_source: false,
            }],
        );
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 4); // 1024 frames: ramp open, fades passed.

        let mut frames = vec![0.0f32; 2048];
        executor.render_block(&mut frames);
        let step = source_rate as f64 / 48_000.0;
        let mut error = 0.0f64;
        let mut power = 0.0f64;
        for frame_index in 0..1024usize {
            let stream_frame = 1024 + frame_index as u64;
            let position = stream_frame as f64 * step;
            let expected =
                (std::f64::consts::TAU * frequency * position / source_rate as f64).sin();
            let actual = frames[frame_index * 2] as f64;
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "rate-converted playback SNR {snr:.1} dB");
    }

    #[test]
    fn sample_buffers_compare_by_pointer_for_cheap_spec_equality() {
        let data: Arc<[f32]> = vec![0.0f32; 8].into();
        let a = RenderSampleBuffer {
            sample_rate_hz: 48_000,
            frames: Arc::clone(&data),
        };
        let b = RenderSampleBuffer {
            sample_rate_hz: 48_000,
            frames: data,
        };
        let c = RenderSampleBuffer {
            sample_rate_hz: 48_000,
            frames: vec![0.0f32; 8].into(),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn retired_plans_return_to_the_control_side() {
        let (mut controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        let mut frames = [0.0f32; 64];
        executor.render_block(&mut frames);

        controller.install_plan(&tone_spec(880.0)).unwrap();
        executor.render_block(&mut frames);

        assert_eq!(controller.collect_retired(), 1);
        assert_eq!(controller.retired_parked_blocks(), 0);
    }

    // ── Graph-shaped plans ──────────────────────────────────────────────────

    #[test]
    fn compile_rejects_cycles() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                RenderStageSpec {
                    stage_id: 1,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(2)],
                },
                RenderStageSpec {
                    stage_id: 2,
                    format: ChannelFormat::stereo(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(1)],
                },
                master_node(vec![identity_edge(1)]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("cycle"), "{}", error.message);
    }

    #[test]
    fn compile_rejects_duplicate_node_ids() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                lane_node(7, 1.0, vec![]),
                lane_node(7, 1.0, vec![]),
                master_node(vec![identity_edge(7)]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("duplicate"), "{}", error.message);
    }

    #[test]
    fn compile_rejects_wrong_master_count() {
        let (mut controller, _executor) = render_plane();
        let no_master = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![lane_node(1, 1.0, vec![])],
        };
        let error = controller.install_plan(&no_master).unwrap_err();
        assert!(
            error.message.contains("exactly one output stage"),
            "{}",
            error.message
        );

        let mut two_masters = master_node(vec![]);
        two_masters.stage_id = MASTER_ID + 1;
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![master_node(vec![]), two_masters],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(
            error.message.contains("exactly one output stage"),
            "{}",
            error.message
        );
    }

    #[test]
    fn compile_rejects_unknown_inputs_and_bad_matrices() {
        let (mut controller, _executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![master_node(vec![identity_edge(99)])],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("unknown input"), "{}", error.message);

        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                lane_node(1, 1.0, vec![]),
                master_node(vec![RenderEdgeSpec {
                    source_stage_id: 1,
                    gain: 1.0,
                    matrix: Some(vec![1.0, 0.0, 0.0]), // 2x2 edge needs 4.
                }]),
            ],
        };
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(
            error.message.contains("matrix") && error.message.contains("expected 4"),
            "{}",
            error.message
        );
    }

    #[test]
    fn bus_chain_renders_in_topological_order() {
        // Nodes listed deliberately out of order: master first, then the bus
        // chain, then the lane. The schedule must still run lane → bus A →
        // bus B → master, with each stage's gain applied at consumption.
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                master_node(vec![identity_edge(20)]),
                RenderStageSpec {
                    stage_id: 20,
                    format: ChannelFormat::stereo(),
                    gain: 0.5,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(10)],
                },
                RenderStageSpec {
                    stage_id: 10,
                    format: ChannelFormat::stereo(),
                    gain: 0.5,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![identity_edge(LANE_ID)],
                },
                lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Reference: the same tone through a single lane at the chain's
        // composite gain (0.5 × 0.5 = 0.25).
        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut chained = [0.0f32; 512];
        let mut reference = [0.0f32; 512];
        executor.render_block(&mut chained);
        reference_executor.render_block(&mut reference);
        for (a, b) in chained.iter().zip(reference.iter()) {
            assert!((a - b).abs() < 1e-6, "chain diverged: {a} vs {b}");
        }
    }

    #[test]
    fn pan_matrix_places_a_lane_in_the_stereo_field() {
        // The pan primitive per chorus a14: an explicit 2×2 equal-power
        // matrix on the lane → master edge.
        let render_with_pan = |pan: f32| -> [f32; 512] {
            let (mut controller, mut executor) = render_plane();
            let spec = RenderPlanSpec {
                sample_rate_hz: 48_000,
                master_gain: 1.0,
                stages: vec![
                    lane_node(LANE_ID, 1.0, vec![tone_clip(440.0)]),
                    master_node(vec![RenderEdgeSpec {
                        source_stage_id: LANE_ID,
                        gain: 1.0,
                        matrix: Some(equal_power_pan_matrix(pan).to_vec()),
                    }]),
                ],
            };
            controller.install_plan(&spec).unwrap();
            controller.set_playing(true).unwrap();
            warm_up(&mut executor, 2);
            let mut frames = [0.0f32; 512];
            executor.render_block(&mut frames);
            frames
        };

        let hard_left = render_with_pan(-1.0);
        assert!(hard_left.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
        assert!(hard_left.chunks_exact(2).all(|frame| frame[1] == 0.0));

        let hard_right = render_with_pan(1.0);
        assert!(hard_right
            .chunks_exact(2)
            .all(|frame| frame[0].abs() < 1e-6));
        assert!(hard_right.chunks_exact(2).any(|frame| frame[1].abs() > 0.1));

        let center = render_with_pan(0.0);
        let minus_3db = std::f32::consts::FRAC_1_SQRT_2;
        for frame in center.chunks_exact(2) {
            assert!((frame[0] - frame[1]).abs() < 1e-6);
        }
        // Center sits -3 dB against the hard-left reference.
        let left_peak = hard_left
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        let center_peak = center
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        assert!((center_peak - left_peak * minus_3db).abs() < 1e-3);
    }

    #[test]
    fn mono_lane_upmixes_to_stereo_through_the_default_adapter() {
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                RenderStageSpec {
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                master_node(vec![identity_edge(LANE_ID)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        let peak = frames
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0f32, f32::max);
        assert!(peak > 0.1, "mono lane should be audible after upmix");
        for frame in frames.chunks_exact(2) {
            // Equal distribution at -3 dB: both channels identical.
            assert_eq!(frame[0], frame[1]);
        }
        // -3 dB against a stereo lane at unity.
        assert!((peak - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.05);
    }

    #[test]
    fn send_topology_sums_both_paths() {
        // Lane feeds bus A and bus B (a send); both feed the master. The
        // output must be exactly double the single-path render.
        let (mut controller, mut executor) = render_plane();
        let bus = |stage_id: u64| RenderStageSpec {
            stage_id,
            format: ChannelFormat::stereo(),
            gain: 1.0,
            gain_automation: None,
            kind: RenderStageKind::Sum,
            inputs: vec![identity_edge(LANE_ID)],
        };
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                lane_node(LANE_ID, 0.25, vec![tone_clip(440.0)]),
                bus(10),
                bus(11),
                master_node(vec![identity_edge(10), identity_edge(11)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(0.25, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut sent = [0.0f32; 512];
        let mut single = [0.0f32; 512];
        executor.render_block(&mut sent);
        reference_executor.render_block(&mut single);
        for (doubled, reference) in sent.iter().zip(single.iter()) {
            assert!(
                (doubled - reference * 2.0).abs() < 1e-6,
                "send sum diverged: {doubled} vs 2×{reference}",
            );
        }
    }

    #[test]
    fn wider_master_downmixes_at_the_hardware_boundary() {
        // 4-channel master on a 2-channel stream: the boundary matrix
        // (compiled at install, when the stream is known) folds channels
        // 0/2 onto left and 1/3 onto right at equal weight.
        let (mut controller, mut executor) = render_plane();
        controller.set_stream_channels(2).unwrap();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                RenderStageSpec {
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    stage_id: MASTER_ID,
                    format: ChannelFormat {
                        channels: 4,
                        layout: ChannelLayout::Generic,
                    },
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Output,
                    // Distinct synthetic spread: [1.0, 0.5, 0.25, 0.75].
                    inputs: vec![RenderEdgeSpec {
                        source_stage_id: LANE_ID,
                        gain: 1.0,
                        matrix: Some(vec![1.0, 0.5, 0.25, 0.75]),
                    }],
                },
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Mono reference at unity for the same tone.
        let (mut reference_controller, mut reference_executor) = render_plane();
        reference_controller
            .install_plan(&lane_master_spec(1.0, vec![tone_clip(440.0)]))
            .unwrap();
        reference_controller.set_playing(true).unwrap();
        warm_up(&mut reference_executor, 2);

        let mut downmixed = [0.0f32; 512];
        let mut reference = [0.0f32; 512];
        executor.render_block(&mut downmixed);
        reference_executor.render_block(&mut reference);
        // Boundary fold (4→2): L = (c0 + c2)/2 = (1.0 + 0.25)/2 = 0.625×tone,
        // R = (c1 + c3)/2 = (0.5 + 0.75)/2 = 0.625×tone.
        for (frame, reference_frame) in downmixed.chunks_exact(2).zip(reference.chunks_exact(2)) {
            let tone = reference_frame[0];
            assert!((frame[0] - tone * 0.625).abs() < 1e-5);
            assert!((frame[1] - tone * 0.625).abs() < 1e-5);
        }
        // Clock advances by stream frames (2-channel framing): 512+256.
        assert_eq!(controller.position_frames(), 768);
    }

    #[test]
    fn narrower_master_leaves_extra_stream_channels_silent() {
        // Mono master on a stereo stream: the hardware stage never invents
        // an upmix — channel 0 carries the master, channel 1 stays silent.
        let (mut controller, mut executor) = render_plane();
        controller.set_stream_channels(2).unwrap();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 1.0,
            stages: vec![
                RenderStageSpec {
                    stage_id: LANE_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(440.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    stage_id: MASTER_ID,
                    format: ChannelFormat::mono(),
                    gain: 1.0,
                    gain_automation: None,
                    kind: RenderStageKind::Output,
                    inputs: vec![identity_edge(LANE_ID)],
                },
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        assert!(frames.chunks_exact(2).any(|frame| frame[0].abs() > 0.1));
        assert!(frames.chunks_exact(2).all(|frame| frame[1] == 0.0));
    }

    /// FNV-1a 64 over the bit pattern of rendered samples.
    fn fnv1a_hash_pcm(frames: &[f32]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for sample in frames {
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    }

    #[test]
    fn golden_render_hash_is_stable() {
        // Reference plan: two tone lanes panned hard left/right through a
        // bus, summed with a centered mono lane at the master. Renders 8 ×
        // 256-frame blocks from transport start (edge ramp included) and
        // hashes the PCM. Gates every render-plane change: any behavioral
        // drift in declick, smoothing, scheduling, or matrix application
        // moves the hash.
        //
        // Regenerating after an INTENTIONAL change: run with the assert
        // relaxed (or print `hash`), paste the new value, and justify the
        // change in the commit. Never regenerate to silence a failure you
        // cannot explain.
        let (mut controller, mut executor) = render_plane();
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            master_gain: 0.8,
            stages: vec![
                lane_node(1, 0.5, vec![tone_clip(440.0)]),
                lane_node(2, 0.4, vec![tone_clip(660.0)]),
                RenderStageSpec {
                    stage_id: 3,
                    format: ChannelFormat::mono(),
                    gain: 0.3,
                    gain_automation: None,
                    kind: RenderStageKind::Source {
                        clips: vec![tone_clip(220.0)],
                    },
                    inputs: Vec::new(),
                },
                RenderStageSpec {
                    stage_id: 10,
                    format: ChannelFormat::stereo(),
                    gain: 0.9,
                    gain_automation: None,
                    kind: RenderStageKind::Sum,
                    inputs: vec![
                        RenderEdgeSpec {
                            source_stage_id: 1,
                            gain: 1.0,
                            matrix: Some(equal_power_pan_matrix(-1.0).to_vec()),
                        },
                        RenderEdgeSpec {
                            source_stage_id: 2,
                            gain: 0.8,
                            matrix: Some(equal_power_pan_matrix(1.0).to_vec()),
                        },
                    ],
                },
                master_node(vec![identity_edge(10), identity_edge(3)]),
            ],
        };
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();

        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        let mut frames = [0.0f32; 512];
        for _ in 0..8 {
            executor.render_block(&mut frames);
            // Chain the per-block hashes by re-seeding from the running value.
            hash ^= fnv1a_hash_pcm(&frames);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Recorded on first run (see regeneration note above).
        assert_eq!(
            hash, GOLDEN_RENDER_HASH,
            "golden render drifted: {hash:#018x}"
        );
    }

    /// Recorded output hash for `golden_render_hash_is_stable` (captured on
    /// the test's first run; see the regeneration note in the test body).
    const GOLDEN_RENDER_HASH: u64 = 0x494b_7128_ef17_1a6a;
}
