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

const COMMAND_MAILBOX_CAPACITY: usize = 64;
// Sized so that even a full command mailbox of plan installs can retire
// without saturating; install_plan also reclaims eagerly. The executor's
// single parked slot is belt-and-braces on top of this invariant.
const RETIRED_MAILBOX_CAPACITY: usize = COMMAND_MAILBOX_CAPACITY + 2;

/// Audio source for one render lane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderSource {
    /// Lane renders silence (the default for lanes with no media yet).
    Silence,
    /// Lane renders a sine tone — the audible stand-in until clip audio
    /// streaming lands, and the soak-test source.
    TestTone {
        /// Tone frequency in hertz.
        frequency_hz: f32,
    },
}

/// One lane in a render plan spec.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderLaneSpec {
    /// Stable lane identity (control-side bookkeeping only; never read in
    /// the render loop).
    pub lane_id: String,
    /// Linear lane gain.
    pub gain: f32,
    /// Lane source.
    pub source: RenderSource,
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
}

struct CompiledLane {
    gain: f32,
    source: CompiledSource,
    // Retained for control-side debugging when plans are retired; the render
    // loop never touches it.
    #[allow(dead_code)]
    lane_id: String,
}

/// A compiled, immutable-topology render plan. Source state (tone phase)
/// mutates during rendering; structure never does.
pub struct RenderPlan {
    channels: usize,
    master_gain: f32,
    lanes: Vec<CompiledLane>,
}

impl RenderPlan {
    fn compile(spec: &RenderPlanSpec) -> Box<RenderPlan> {
        let tau = std::f32::consts::TAU;
        Box::new(RenderPlan {
            channels: spec.channels.max(1) as usize,
            master_gain: spec.master_gain,
            lanes: spec
                .lanes
                .iter()
                .map(|lane| CompiledLane {
                    gain: lane.gain,
                    lane_id: lane.lane_id.clone(),
                    source: match lane.source {
                        RenderSource::Silence => CompiledSource::Silence,
                        RenderSource::TestTone { frequency_hz } => CompiledSource::Tone {
                            phase: 0.0,
                            step: frequency_hz * tau / spec.sample_rate_hz.max(1) as f32,
                        },
                    },
                })
                .collect(),
        })
    }
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
    /// Compile `spec` and install it as the active plan. Eagerly reclaims
    /// any plans the executor has retired.
    pub fn install_plan(&self, spec: &RenderPlanSpec) -> Result<(), RenderPlaneError> {
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
    playing: bool,
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

    fn drain_commands(&mut self) {
        loop {
            match self.commands.try_recv() {
                Ok(RenderCommand::InstallPlan(next_plan)) => {
                    if let Some(previous) = self.plan.replace(next_plan) {
                        self.retire(previous);
                    }
                }
                Ok(RenderCommand::SetPlaying(playing)) => {
                    self.playing = playing;
                    self.shared.playing.store(playing, Ordering::Relaxed);
                }
                Ok(RenderCommand::Seek(position_frames)) => {
                    self.position_frames = position_frames;
                    self.shared
                        .position_frames
                        .store(position_frames, Ordering::Relaxed);
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
        if !self.playing {
            return;
        }

        let channels = plan.channels;
        let frame_count = frames.len() / channels;

        for lane in plan.lanes.iter_mut() {
            match &mut lane.source {
                CompiledSource::Silence => {}
                CompiledSource::Tone { phase, step } => {
                    let gain = lane.gain * plan.master_gain;
                    let mut local_phase = *phase;
                    for frame_index in 0..frame_count {
                        let sample = local_phase.sin() * gain;
                        local_phase += *step;
                        if local_phase >= std::f32::consts::TAU {
                            local_phase -= std::f32::consts::TAU;
                        }
                        let base = frame_index * channels;
                        for channel in 0..channels {
                            frames[base + channel] += sample;
                        }
                    }
                    *phase = local_phase;
                }
            }
        }

        self.position_frames += frame_count as u64;
        self.shared
            .position_frames
            .store(self.position_frames, Ordering::Relaxed);
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
        },
        RenderPlaneExecutor {
            commands: command_rx,
            retired: retired_tx,
            shared,
            plan: None,
            parked_retired: None,
            playing: false,
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
                source: RenderSource::TestTone {
                    frequency_hz,
                },
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
