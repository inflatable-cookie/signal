//! Control-side plan specification vocabulary.

use crate::{
    RenderLiveInputHandle, RenderNoteBuffer, RenderPluginEventBuffer, RenderPluginProcessor,
    RenderSampleBuffer, RenderStreamHandle,
};

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
    /// Clip streams media from disk through a chunk mailbox (see
    /// `render_stream`); same anchoring and interpolation as `Samples`,
    /// but source data arrives in feeder-posted chunks and missing frames
    /// render silence (counted on the handle). `loop_source` is ignored.
    Stream(RenderStreamHandle),
    /// Clip monitors a live input (see `render_live_input`): each block
    /// drains whatever the input callback pushed into the handle's ring,
    /// ignoring the timeline position entirely — live input renders "now".
    /// Window a live clip `[0, u64::MAX)` so it is always audible while the
    /// transport plays. Underrun (input behind) renders silence and counts
    /// on the handle; `loop_source` is ignored.
    LiveInput(RenderLiveInputHandle),
    /// Clip synthesizes its note events through the built-in instrument: a
    /// STATELESS additive sine voice per note (phase, attack/sustain/release
    /// envelope all pure functions of the stream position), so seeks and
    /// plan swaps are inherently sample-exact — there is no voice state to
    /// inherit. Notes are clip-relative and must be sorted by `start_frame`
    /// (compile rejects unsorted buffers). At most
    /// `NOTE_POLYPHONY_LIMIT` notes render simultaneously per block
    /// (earliest-started win). `loop_source` is ignored.
    Notes(RenderNoteBuffer),
    /// Rate-warped playback of a media source (g12.027 repitch/varispeed):
    /// the inner source's samples are consumed at `rate` times normal speed,
    /// so pitch shifts with the rate — like vinyl. `rate` multiplies the
    /// source-frames-per-stream-frame step the inner source would compile to
    /// (its own sample-rate conversion included), reusing the polyphase
    /// windowed-sinc interpolation path; non-finite or non-positive rates
    /// compile as 1.0. Valid over [`RenderSource::Samples`] and
    /// [`RenderSource::Stream`] only — anything else (including nesting)
    /// rejects at compile with
    /// [`RenderPlanCompileError::WarpedSourceUnsupported`].
    Warped {
        /// The media source being rate-warped.
        source: Box<RenderSource>,
        /// Playback-rate multiplier: 2.0 plays double speed (octave up),
        /// 0.5 half speed (octave down).
        rate: f64,
    },
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
    /// Equal-power fade-in length in stream frames, measured from
    /// `start_frames`. Zero (the default) keeps the fixed edge declick on
    /// that side — today's behavior byte-for-byte. Non-zero REPLACES the
    /// declick on the start side with a `sin(π/2·t)` quarter-wave, so two
    /// overlapping clips (one fading out, one fading in over the same span)
    /// form a true equal-power crossfade. Clamped to the window length at
    /// compile.
    pub fade_in_frames: u32,
    /// Equal-power fade-out length in stream frames, ending at `end_frames`.
    /// Zero keeps the edge declick on the end side; non-zero replaces it
    /// with the complementary quarter-wave. Clamped to the window length at
    /// compile.
    pub fade_out_frames: u32,
}

/// What a stage does with its inputs (and whether it generates content).
#[derive(Debug, Clone, PartialEq)]
pub enum RenderStageKind {
    /// Source stage: renders clip content at its format. May also sum inputs
    /// (clips render first, then edges add in).
    Source {
        /// Clip events rendered by this source stage.
        clips: Vec<RenderClipSpec>,
    },
    /// Sums its inputs through their edge matrices.
    Sum,
    /// Delays its summed inputs by an exact number of stream frames. The
    /// delay line is allocated when the plan compiles; rendering only swaps
    /// samples through the fixed-size ring. Consumers decide where to place
    /// compensation stages and how many frames they require.
    Delay {
        /// Number of stream frames to delay. Zero is an identity stage.
        frames: u32,
    },
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

/// One compiled parameter automation envelope for a processor stage:
/// sorted `(absolute stream-clock frame, normalized value)` breakpoints for
/// one plugin parameter (g13.029 offline param bake).
///
/// OFFLINE ONLY in v1: the offline driver samples each envelope at every
/// block boundary (linear interpolation between points, end values held
/// outside the point range) and applies the value through the processor's
/// `PluginBlockProcessor::set_parameter_normalized` seam before the block
/// renders — the offline mirror of the host's live block-boundary parameter
/// forwarding. Block-boundary resolution is the honest fidelity bound: a
/// sweep steps once per offline block (default 1024 frames), exactly like
/// the live playback-poll cadence it mirrors. The realtime executor ignores
/// this field entirely (live parameter playback stays host-driven).
#[derive(Debug, Clone, PartialEq)]
pub struct RenderParamEnvelope {
    /// Plugin-format-native parameter id (u32 fits CLAP param ids and VST3
    /// ParamIDs — the same id space the host's live set-parameter path uses).
    pub parameter_id: u32,
    /// Breakpoints `(absolute stream-clock frame, normalized 0..=1 value)`,
    /// sorted by frame (compile rejects unsorted envelopes).
    pub points: Vec<(u64, f32)>,
}

impl RenderParamEnvelope {
    /// Envelope value at `frame`: linear interpolation between the
    /// surrounding breakpoints, first/last value held outside the point
    /// range (the same boundary-hold rule gain automation uses). `None`
    /// when the envelope has no points.
    pub fn value_at(&self, frame: u64) -> Option<f32> {
        let first = self.points.first()?;
        if frame <= first.0 {
            return Some(first.1);
        }
        let last = self.points.last()?;
        if frame >= last.0 {
            return Some(last.1);
        }
        // First breakpoint strictly past `frame`; the guards above ensure
        // both neighbours exist.
        let next = self.points.partition_point(|(point, _)| *point <= frame);
        let (frame_a, value_a) = self.points[next - 1];
        let (frame_b, value_b) = self.points[next];
        if frame_b == frame_a {
            return Some(value_b);
        }
        let t = (frame - frame_a) as f64 / (frame_b - frame_a) as f64;
        Some(value_a + (value_b - value_a) * t as f32)
    }
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
    /// Optional plugin processor applied to this stage's summed scratch
    /// before consumers read it (g11.012). Valid on [`RenderStageKind::Sum`]
    /// stages only — compile rejects it elsewhere. Bypass (backend returns
    /// `false`) leaves the scratch untouched, so absent/bypassed processors
    /// keep the golden render hash unchanged.
    pub processor: Option<RenderPluginProcessor>,
    /// Optional compiled plugin event stream (notes + CC on the absolute
    /// stream clock, sorted by frame) delivered to `processor` per block
    /// with intra-block sample offsets. Valid only alongside a processor —
    /// compile rejects events without one. `None` keeps the pre-event
    /// behavior bit-identical.
    pub events: Option<RenderPluginEventBuffer>,
    /// When true, the executor allocates a per-stage live-event ring at
    /// install and accepts `RenderPlaneController::push_live_events` for
    /// this stage (g13.018): host-pushed events (hardware MIDI live-thru)
    /// deliver to `processor` every block regardless of transport. Same
    /// placement rule as compiled events: valid only on a Sum stage with a
    /// processor — compile rejects it elsewhere. `false` keeps behavior
    /// bit-identical.
    pub accepts_live_events: bool,
    /// Offline-only parameter automation envelopes for `processor`
    /// (g13.029): sampled at block boundaries by the OFFLINE driver and
    /// applied through the processor set-parameter seam; the realtime
    /// executor ignores them (live parameter playback stays host-driven).
    /// Valid only alongside a processor — compile rejects envelopes without
    /// one. Empty (the default) keeps every render byte-identical.
    pub parameter_envelopes: Vec<RenderParamEnvelope>,
}

/// Optional soft limiter guarding the hardware boundary. Mechanism only —
/// thresholds and whether to limit at all are consumer policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderLimiterSpec {
    /// Linear level where limiting centers (clamped below 0 dBFS).
    pub threshold: f32,
    /// Linear soft-knee width centered on the threshold.
    pub knee_width: f32,
    /// One-pole gain-recovery time constant in seconds.
    pub release_seconds: f32,
}

/// Control-side description of a render plan: a format-typed stage graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlanSpec {
    /// Sample rate the plan renders at.
    pub sample_rate_hz: u32,
    /// Linear master gain applied at the master stage (kept for consumer
    /// compatibility; multiplies the master stage's own gain).
    pub master_gain: f32,
    /// Optional soft limiter applied to the stream buffer after the
    /// boundary write and before the transport edge envelope. `None` = no
    /// limiting (bit-transparent master path).
    pub master_limiter: Option<RenderLimiterSpec>,
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
            || self.master_limiter != other.master_limiter
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
                || old.processor != new.processor
                || old.events != new.events
                || old.accepts_live_events != new.accepts_live_events
                || old.parameter_envelopes != new.parameter_envelopes
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
    /// A clip's note buffer is not sorted by `start_frame`.
    UnsortedNotes {
        /// Stage owning the clip.
        stage_id: u64,
        /// Clip whose notes are unsorted.
        clip_id: u64,
    },
    /// A plugin processor was attached to a non-Sum stage.
    ProcessorOnNonSumStage(u64),
    /// A plugin event stream was attached to a stage without a processor.
    EventsWithoutProcessor(u64),
    /// A plugin event buffer is not sorted by `frame`.
    UnsortedEvents(u64),
    /// `accepts_live_events` was set on a stage without a plugin processor
    /// (live events require a processor, which itself requires a Sum stage).
    LiveEventsWithoutProcessor(u64),
    /// A parameter envelope was attached to a stage without a processor.
    ParameterEnvelopeWithoutProcessor(u64),
    /// A parameter envelope's breakpoints are not sorted by frame.
    UnsortedParameterEnvelope(u64),
    /// A `Warped` source wraps something other than `Samples` or `Stream`.
    WarpedSourceUnsupported {
        /// Stage owning the clip.
        stage_id: u64,
        /// Clip carrying the unsupported warp.
        clip_id: u64,
    },
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
            RenderPlanCompileError::ProcessorOnNonSumStage(stage_id) => write!(
                formatter,
                "stage {stage_id} attaches a plugin processor but is not a Sum stage",
            ),
            RenderPlanCompileError::EventsWithoutProcessor(stage_id) => write!(
                formatter,
                "stage {stage_id} attaches plugin events without a plugin processor",
            ),
            RenderPlanCompileError::UnsortedEvents(stage_id) => write!(
                formatter,
                "stage {stage_id} plugin events are not sorted by frame",
            ),
            RenderPlanCompileError::LiveEventsWithoutProcessor(stage_id) => write!(
                formatter,
                "stage {stage_id} accepts live events without a plugin processor",
            ),
            RenderPlanCompileError::ParameterEnvelopeWithoutProcessor(stage_id) => write!(
                formatter,
                "stage {stage_id} attaches parameter envelopes without a plugin processor",
            ),
            RenderPlanCompileError::UnsortedParameterEnvelope(stage_id) => write!(
                formatter,
                "stage {stage_id} parameter envelope breakpoints are not sorted by frame",
            ),
            RenderPlanCompileError::WarpedSourceUnsupported { stage_id, clip_id } => write!(
                formatter,
                "stage {stage_id} clip {clip_id} warps a source that is not Samples or Stream",
            ),
            RenderPlanCompileError::UnsortedNotes { stage_id, clip_id } => write!(
                formatter,
                "stage {stage_id} clip {clip_id} notes are not sorted by start frame",
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
