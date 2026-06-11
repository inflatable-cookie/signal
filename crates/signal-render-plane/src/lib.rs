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
//! Plan *content* starts deliberately small (silence and test-tone sources,
//! per-lane gain, master gain): the substrate and its guarantees are the
//! deliverable. Compiling plans from Pulse projections layers on top.

#![warn(missing_docs)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;

use signal_dsp::PolyphaseInterpolationTable;

/// Interpolation table shape for rate-converted media playback. 16 taps ×
/// 512 phases ≈ 32 KB per distinct cutoff; built once per plan compile.
const RESAMPLE_TAPS: usize = 16;
const RESAMPLE_PHASES: usize = 512;

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
        self.sample_rate_hz == other.sample_rate_hz
            && Arc::ptr_eq(&self.frames, &other.frames)
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
/// Full-swing time for lane/master gain changes across plan swaps.
const GAIN_SMOOTHING_SECONDS: f32 = 0.010;
/// Declick fade applied inside each clip window edge (shortened for tiny
/// windows so short clips stay audible).
const CLIP_EDGE_FADE_FRAMES: u64 = 32;

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
    /// Clip plays shared sample data from its window start, with linear
    /// interpolation when the source rate differs from the stream rate.
    Samples(RenderSampleBuffer),
}

/// One clip event in a render lane: a half-open stream-clock window
/// `[start_frames, end_frames)` and the source that plays inside it.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderClipSpec {
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

/// One lane in a render plan spec.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderLaneSpec {
    /// Stable lane identity (control-side bookkeeping only; never read in
    /// the render loop).
    pub lane_id: String,
    /// Linear lane gain.
    pub gain: f32,
    /// Clip events on this lane.
    pub clips: Vec<RenderClipSpec>,
}

/// Control-side description of a render plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlanSpec {
    /// Sample rate the plan renders at.
    pub sample_rate_hz: u32,
    /// Interleaved output channel count.
    pub channels: u16,
    /// Linear master gain applied after the lane sum.
    pub master_gain: f32,
    /// Lanes mixed into the master.
    pub lanes: Vec<RenderLaneSpec>,
}

// ── Compiled plan (render-side data, preallocated at compile time) ─────────

enum CompiledSource {
    Silence,
    Tone { phase: f32, step: f32 },
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
    source: CompiledSource,
}

struct CompiledLane {
    /// Gain the lane is moving toward (spec value).
    gain_target: f32,
    /// Smoothed gain as currently applied; inherited across plan swaps so
    /// gain edits never step.
    gain_current: f32,
    clips: Vec<CompiledClip>,
    /// Matches lane state (smoothed gain, tone phase) across plan swaps.
    lane_id: String,
}


/// A compiled, immutable-topology render plan. Source state (tone phase,
/// smoothed gains) mutates during rendering; structure never does.
pub struct RenderPlan {
    channels: usize,
    sample_rate_hz: u32,
    master_gain_target: f32,
    master_gain_current: f32,
    lanes: Vec<CompiledLane>,
}

impl RenderPlan {
    fn compile(spec: &RenderPlanSpec) -> Box<RenderPlan> {
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
            let table =
                PolyphaseInterpolationTable::new(RESAMPLE_TAPS, RESAMPLE_PHASES, cutoff);
            tables.push((key, table.clone()));
            Some(table)
        };
        Box::new(RenderPlan {
            channels: spec.channels.max(1) as usize,
            sample_rate_hz: stream_rate,
            master_gain_target: spec.master_gain,
            master_gain_current: spec.master_gain,
            lanes: spec
                .lanes
                .iter()
                .map(|lane| CompiledLane {
                    gain_target: lane.gain,
                    gain_current: lane.gain,
                    lane_id: lane.lane_id.clone(),
                    clips: lane
                        .clips
                        .iter()
                        .map(|clip| CompiledClip {
                            start_frames: clip.start_frames,
                            end_frames: clip.end_frames,
                            edge_fade_frames: CLIP_EDGE_FADE_FRAMES.min(
                                clip.end_frames
                                    .saturating_sub(clip.start_frames)
                                    .max(2)
                                    / 2,
                            ),
                            source: match &clip.source {
                                RenderSource::Silence => CompiledSource::Silence,
                                RenderSource::TestTone { frequency_hz } => CompiledSource::Tone {
                                    phase: 0.0,
                                    step: frequency_hz * tau / stream_rate as f32,
                                },
                                RenderSource::Samples(buffer) => {
                                    let step = buffer.sample_rate_hz.max(1) as f64
                                        / stream_rate as f64;
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
                })
                .collect(),
        })
    }

    /// Carry smoothed gains and tone phases over from the plan being
    /// replaced, so a recompile (gain tweak, clip edit) never steps audio.
    /// Runs on the audio thread: comparisons and copies only, no allocation.
    fn inherit_state(&mut self, previous: &RenderPlan) {
        self.master_gain_current = previous.master_gain_current;
        for lane in self.lanes.iter_mut() {
            let Some(previous_lane) = previous
                .lanes
                .iter()
                .find(|candidate| candidate.lane_id == lane.lane_id)
            else {
                continue;
            };
            lane.gain_current = previous_lane.gain_current;
            for (clip, previous_clip) in
                lane.clips.iter_mut().zip(previous_lane.clips.iter())
            {
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

enum RenderCommand {
    InstallPlan(Box<RenderPlan>),
    SetPlaying(bool),
    Seek(u64),
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
    /// Stream channel count as reported by the host; installs of plans with
    /// a different channel count are rejected to protect output framing.
    stream_channels: Option<u16>,
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
    /// Record the channel count of the opened output stream. Subsequent
    /// installs of plans with a different channel count fail instead of
    /// corrupting output framing and the stream clock.
    pub fn set_stream_channels(&mut self, channels: u16) {
        self.stream_channels = Some(channels);
    }

    /// Compile `spec` and install it as the active plan. Eagerly reclaims
    /// any plans the executor has retired.
    pub fn install_plan(&self, spec: &RenderPlanSpec) -> Result<(), RenderPlaneError> {
        if let Some(stream_channels) = self.stream_channels {
            if spec.channels != stream_channels {
                return Err(RenderPlaneError {
                    message: format!(
                        "plan channel count {} does not match stream channel count {stream_channels}",
                        spec.channels,
                    ),
                });
            }
        }
        self.collect_retired();
        let plan = RenderPlan::compile(spec);
        self.commands
            .try_send(RenderCommand::InstallPlan(plan))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected plan install: {error}"),
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
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Render one callback quantum into `frames` (interleaved f32).
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

        let channels = plan.channels;
        let frame_count = frames.len() / channels;
        let block_start_frame = self.position_frames;

        for lane in plan.lanes.iter_mut() {
            // Smoothed gains: move toward targets at a fixed full-swing rate
            // and interpolate across the block, so edits never step audio.
            let gain_step = frame_count as f32
                / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0);
            let master_start = plan.master_gain_current;
            let master_end = master_start
                + (plan.master_gain_target - master_start).clamp(-gain_step, gain_step);
            let lane_start = lane.gain_current;
            let lane_end = lane_start
                + (lane.gain_target - lane_start).clamp(-gain_step, gain_step);
            let gain_begin = lane_start * master_start;
            let gain_finish = lane_end * master_end;
            let gain_slope = (gain_finish - gain_begin) / frame_count.max(1) as f32;
            lane.gain_current = lane_end;

            for clip in lane.clips.iter_mut() {
                // Skip clips entirely outside this block.
                let block_end_frame = block_start_frame + frame_count as u64;
                if clip.end_frames <= block_start_frame || clip.start_frames >= block_end_frame
                {
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
                            let lane_gain = gain_begin + gain_slope * frame_index as f32;
                            let sample = local_phase.sin() * lane_gain;
                            local_phase += *step;
                            if local_phase >= std::f32::consts::TAU {
                                local_phase -= std::f32::consts::TAU;
                            }
                            if frame >= clip_start && frame < clip_end {
                                let sample =
                                    sample * clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                                let base = frame_index * channels;
                                for channel in 0..channels {
                                    frames[base + channel] += sample;
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
                            // Source position via the rate ratio, anchored at
                            // the clip's window start.
                            let mut source_position =
                                (frame - clip_start) as f64 * *step;
                            if *loop_source {
                                source_position %= source_frames as f64;
                            }
                            let source_index = source_position as usize;
                            if source_index >= source_frames {
                                continue;
                            }
                            let fraction = source_position - source_index as f64;
                            let lane_gain = (gain_begin
                                + gain_slope * frame_index as f32)
                                * clip_edge_gain(frame, clip_start, clip_end, clip_fade);
                            let base = frame_index * channels;
                            match table {
                                // Rate conversion: polyphase windowed-sinc
                                // tap dot product (table reads only — no
                                // allocation, no transcendentals).
                                Some(table) => {
                                    let row = table.phase_row(fraction);
                                    let first = table.first_tap_offset();
                                    for channel in 0..channels.min(2) {
                                        let mut acc = 0.0f32;
                                        for (tap, coefficient) in row.iter().enumerate() {
                                            let mut tap_index =
                                                source_index as isize + first + tap as isize;
                                            if *loop_source {
                                                tap_index =
                                                    tap_index.rem_euclid(source_frames as isize);
                                            }
                                            if tap_index >= 0
                                                && (tap_index as usize) < source_frames
                                            {
                                                acc += data
                                                    [tap_index as usize * 2 + channel]
                                                    * coefficient;
                                            }
                                        }
                                        frames[base + channel] += acc * lane_gain;
                                    }
                                }
                                // 1:1 playback: direct read with last-frame
                                // clamp (or wrap when looping).
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
                                        frames[base + channel] +=
                                            (a + (b - a) * fraction) * lane_gain;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        plan.master_gain_current = plan.master_gain_current
            + (plan.master_gain_target - plan.master_gain_current).clamp(
                -(frame_count as f32
                    / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0)),
                frame_count as f32
                    / (GAIN_SMOOTHING_SECONDS * plan.sample_rate_hz as f32).max(1.0),
            );

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
            let edge_step =
                1.0 / (EDGE_RAMP_SECONDS * plan.sample_rate_hz as f32).max(1.0);
            for frame_index in 0..frame_count {
                self.edge_gain +=
                    (edge_target - self.edge_gain).clamp(-edge_step, edge_step);
                let base = frame_index * channels;
                for channel in 0..channels {
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
        },
        RenderPlaneExecutor {
            commands: command_rx,
            retired: retired_tx,
            shared,
            plan: None,
            parked_retired: None,
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

    fn tone_spec(frequency_hz: f32) -> RenderPlanSpec {
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            channels: 2,
            master_gain: 1.0,
            lanes: vec![RenderLaneSpec {
                lane_id: "lane:1".to_string(),
                gain: 0.5,
                clips: vec![RenderClipSpec {
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::TestTone { frequency_hz },
                    loop_source: false,
                }],
            }],
        }
    }

    #[test]
    fn renders_silence_without_plan_and_when_stopped() {
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        controller.seek(96_000).unwrap();

        let mut frames = [0.0f32; 128];
        executor.render_block(&mut frames);
        assert_eq!(controller.position_frames(), 96_000 + 64);
    }

    #[test]
    fn windows_gate_lane_audibility_on_the_stream_clock() {
        let (controller, mut executor) = render_plane();
        let mut spec = tone_spec(440.0);
        spec.lanes[0].clips[0].start_frames = 128;
        spec.lanes[0].clips[0].end_frames = 256;
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
        RenderPlanSpec {
            sample_rate_hz: 48_000,
            channels: 2,
            master_gain: 1.0,
            lanes: vec![RenderLaneSpec {
                lane_id: "lane:s".to_string(),
                gain: 1.0,
                clips: vec![RenderClipSpec {
                    start_frames,
                    end_frames,
                    source: RenderSource::Samples(RenderSampleBuffer {
                        sample_rate_hz: 48_000,
                        frames: data.into(),
                    }),
                    loop_source,
                }],
            }],
        }
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
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
        controller.install_plan(&spec).unwrap();
        controller.set_playing(true).unwrap();
        let mut frames = [0.0f32; 512];
        executor.render_block(&mut frames);
        // Frame 255 is the final source frame; with the clamp it plays.
        assert!(frames[255 * 2].abs() > 0.1);
    }

    #[test]
    fn looping_sample_clips_wrap_to_their_start() {
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
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
        let (controller, mut executor) = render_plane();
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

    #[test]
    fn plan_swap_inherits_smoothed_gain_without_stepping() {
        let (controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        controller.set_playing(true).unwrap();
        warm_up(&mut executor, 2);

        // Same plan with lane gain doubled: swap mid-play.
        let mut louder = tone_spec(440.0);
        louder.lanes[0].gain = 1.0;
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
        let (controller, mut executor) = render_plane();
        let source_rate = 44_100u32;
        let frequency = 1_000.0f64;
        let mut data = Vec::new();
        for n in 0..44_100 {
            let value = (std::f64::consts::TAU * frequency * n as f64
                / source_rate as f64)
                .sin() as f32;
            data.push(value);
            data.push(value);
        }
        let spec = RenderPlanSpec {
            sample_rate_hz: 48_000,
            channels: 2,
            master_gain: 1.0,
            lanes: vec![RenderLaneSpec {
                lane_id: "lane:rc".to_string(),
                gain: 1.0,
                clips: vec![RenderClipSpec {
                    start_frames: 0,
                    end_frames: u64::MAX,
                    source: RenderSource::Samples(RenderSampleBuffer {
                        sample_rate_hz: source_rate,
                        frames: data.into(),
                    }),
                    loop_source: false,
                }],
            }],
        };
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
            let expected = (std::f64::consts::TAU * frequency * position
                / source_rate as f64)
                .sin();
            let actual = frames[frame_index * 2] as f64;
            error += (actual - expected) * (actual - expected);
            power += expected * expected;
        }
        let snr = 10.0 * (power / error.max(1e-30)).log10();
        assert!(snr > 60.0, "rate-converted playback SNR {snr:.1} dB");
    }

    #[test]
    fn install_rejects_stream_channel_mismatch() {
        let (mut controller, _executor) = render_plane();
        controller.set_stream_channels(2);
        let mut spec = tone_spec(440.0);
        spec.channels = 1;
        let error = controller.install_plan(&spec).unwrap_err();
        assert!(error.message.contains("channel count"));
        spec.channels = 2;
        controller.install_plan(&spec).unwrap();
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
        let (controller, mut executor) = render_plane();
        controller.install_plan(&tone_spec(440.0)).unwrap();
        let mut frames = [0.0f32; 64];
        executor.render_block(&mut frames);

        controller.install_plan(&tone_spec(880.0)).unwrap();
        executor.render_block(&mut frames);

        assert_eq!(controller.collect_retired(), 1);
        assert_eq!(controller.retired_parked_blocks(), 0);
    }
}
