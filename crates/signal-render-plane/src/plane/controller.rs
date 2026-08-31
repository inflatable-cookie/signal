//! Control-side render plane API.

use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;

use crate::plan::RenderPlan;
use crate::{
    RenderPlanSpec, RenderPluginEvent, RenderPluginEventKind, LIVE_EVENT_PUSH_CAPACITY,
    METER_SLOT_CAPACITY,
};

use super::command::{RenderCommand, SharedState, TopologyStage};

/// Control-side handle to a render plane: install plans, drive transport,
/// and observe meters/health counters published by the executor.
#[derive(Debug)]
pub struct RenderPlaneController {
    commands: SyncSender<RenderCommand>,
    retired: Receiver<Box<RenderPlan>>,
    shared: Arc<SharedState>,
    /// Stream channel count as reported by the host. Plans compile their
    /// hardware-boundary adaptation against this; before it is known, plans
    /// assume the stream matches their master format.
    stream_channels: Option<u16>,
    /// Identity snapshot of the last successfully installed plan, in its
    /// topological stage order. Used to precompute the state-inheritance
    /// maps for the next install so the executor does pure index copies, and
    /// to resolve fast-path commands (stage gains, live-event pushes).
    last_topology: Option<Vec<TopologyStage>>,
    /// Generation stamp of the most recent successful install; compared
    /// against the shared meter generation when resolving meter slots.
    plan_generation: u64,
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
        let topology: Vec<TopologyStage> = plan
            .stages
            .iter()
            .map(|stage| TopologyStage {
                stage_id: stage.stage_id,
                clip_ids: stage.clips.iter().map(|clip| clip.clip_id).collect(),
                accepts_live_events: stage.accepts_live_events,
            })
            .collect();

        if let Some(previous) = self.last_topology.as_ref() {
            plan.inherit_stage_map = topology
                .iter()
                .map(|stage| {
                    previous
                        .iter()
                        .position(|previous_stage| previous_stage.stage_id == stage.stage_id)
                })
                .collect();
            plan.inherit_clip_maps = topology
                .iter()
                .enumerate()
                .map(|(index, stage)| {
                    let Some(previous_index) = plan.inherit_stage_map[index] else {
                        return Vec::new();
                    };
                    let previous_clips = &previous[previous_index].clip_ids;
                    stage
                        .clip_ids
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

        plan.generation = self.plan_generation + 1;
        self.commands
            .try_send(RenderCommand::InstallPlan(plan))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected plan install: {error}"),
            })?;
        self.plan_generation += 1;
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
        let Some(stage_index) = topology.iter().position(|stage| stage.stage_id == stage_id) else {
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

    /// Live render posture (g13.018): while active, the executor renders
    /// stages even when the transport is stopped, so live-input monitoring
    /// and live-pushed events are audible without the transport rolling.
    /// Compiled timeline content (clips and compiled plugin events) stays
    /// gated on `playing`, and the transport position does not advance while
    /// stopped.
    pub fn set_live_render(&self, active: bool) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::SetLiveRender { active })
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected live render posture: {error}"),
            })
    }

    /// Live-event fast path (g13.018), mirror of [`Self::set_stage_gain`]:
    /// push `events` (absolute stream-clock frames) at the stage's live-event
    /// ring. Resolves `stage_id` against the topology of the most recent
    /// successful install and validates that the stage compiled with
    /// `RenderStageSpec::accepts_live_events`. Batches larger than
    /// [`LIVE_EVENT_PUSH_CAPACITY`] chunk across multiple commands; events
    /// are sorted by frame control-side when needed so ring order stays
    /// chronological. Never blocks: a full command FIFO returns an error
    /// (events past that point are not delivered — push smaller batches or
    /// retry next pump).
    pub fn push_live_events(
        &self,
        stage_id: u64,
        events: &[RenderPluginEvent],
    ) -> Result<(), RenderPlaneError> {
        let Some(topology) = self.last_topology.as_ref() else {
            return Err(RenderPlaneError {
                message: "no plan installed; cannot push live events".to_string(),
            });
        };
        let Some(stage_index) = topology.iter().position(|stage| stage.stage_id == stage_id) else {
            return Err(RenderPlaneError {
                message: format!("stage {stage_id} not present in the installed plan"),
            });
        };
        if !topology[stage_index].accepts_live_events {
            return Err(RenderPlaneError {
                message: format!("stage {stage_id} does not accept live events"),
            });
        }
        if events.is_empty() {
            return Ok(());
        }
        // Chronological ring order without touching the caller's slice:
        // sort a copy only when the batch arrives unsorted (control side —
        // allocation is fine here).
        let sorted;
        let events = if events.windows(2).any(|pair| pair[0].frame > pair[1].frame) {
            let mut copy = events.to_vec();
            copy.sort_by_key(|event| event.frame);
            sorted = copy;
            sorted.as_slice()
        } else {
            events
        };
        const FILL: RenderPluginEvent = RenderPluginEvent {
            frame: 0,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        };
        for chunk in events.chunks(LIVE_EVENT_PUSH_CAPACITY) {
            let mut batch = [FILL; LIVE_EVENT_PUSH_CAPACITY];
            batch[..chunk.len()].copy_from_slice(chunk);
            self.commands
                .try_send(RenderCommand::PushLiveEvents {
                    stage_index,
                    events: batch,
                    len: chunk.len(),
                })
                .map_err(|error| RenderPlaneError {
                    message: format!("command mailbox rejected live events: {error}"),
                })?;
        }
        Ok(())
    }

    /// Live render posture as last applied by the executor.
    pub fn live_render(&self) -> bool {
        self.shared.live_render.load(Ordering::Relaxed)
    }

    /// Cumulative live events dropped instead of delivered (ring overflow,
    /// per-block scratch overflow, or pushes that no longer resolve on the
    /// executor). Monotonic, like [`Self::xrun_count`].
    pub fn live_event_drop_count(&self) -> u64 {
        self.shared.live_event_drop_count.load(Ordering::Relaxed)
    }

    /// Gate rendering on or off (transport play/stop).
    pub fn set_playing(&self, playing: bool) -> Result<(), RenderPlaneError> {
        self.commands
            .try_send(RenderCommand::SetPlaying(playing))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected transport gate: {error}"),
            })
    }

    /// Set (or clear, with `None`) the transport loop region `[start, end)`
    /// on the stream clock. While playing, a block that crosses `end` wraps
    /// to `start` sample-accurately inside the executor (a control-side seek
    /// would jitter by a mailbox round-trip), with a short micro-fade around
    /// the wrap point. Seeking outside the region is allowed; the loop only
    /// triggers when playback crosses `end`. Rejects `start >= end`.
    pub fn set_loop_region(&self, region: Option<(u64, u64)>) -> Result<(), RenderPlaneError> {
        if let Some((start, end)) = region {
            if start >= end {
                return Err(RenderPlaneError {
                    message: format!("loop region start {start} must be before end {end}"),
                });
            }
        }
        self.commands
            .try_send(RenderCommand::SetLoopRegion(region))
            .map_err(|error| RenderPlaneError {
                message: format!("command mailbox rejected loop region: {error}"),
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

    /// Per-stage meters from the most recently rendered block:
    /// `(stage_id, peak, rms)` in the installed plan's topological order.
    ///
    /// Slots are resolved against the topology of the controller's last
    /// successful install (slot `i` = topological stage `i`). When the
    /// executor has not yet switched to that plan (the shared meter table
    /// still carries an older generation), this returns an empty vec rather
    /// than mislabeling slots — the gap lasts at most a block. Stages past
    /// [`METER_SLOT_CAPACITY`] are silently unmetered. Values are read with
    /// relaxed ordering and may tear between peak and RMS of adjacent
    /// blocks; meters are cosmetic.
    pub fn meters(&self) -> Vec<(u64, f32, f32)> {
        let Some(topology) = self.last_topology.as_ref() else {
            return Vec::new();
        };
        if self.shared.meter_generation.load(Ordering::Relaxed) != self.plan_generation {
            return Vec::new();
        }
        topology
            .iter()
            .take(METER_SLOT_CAPACITY)
            .enumerate()
            .map(|(index, stage)| {
                let slot = &self.shared.meter_slots[index];
                (
                    stage.stage_id,
                    f32::from_bits(slot.peak_bits.load(Ordering::Relaxed)),
                    f32::from_bits(slot.rms_bits.load(Ordering::Relaxed)),
                )
            })
            .collect()
    }

    /// Total render callbacks observed by the executor.
    pub fn callback_count(&self) -> u64 {
        self.shared.callback_count.load(Ordering::Relaxed)
    }

    /// Wall-clock duration of the most recent render callback, microseconds.
    pub fn last_callback_duration_micros(&self) -> u64 {
        self.shared
            .last_callback_duration_micros
            .load(Ordering::Relaxed)
    }

    /// Maximum observed render-callback duration, microseconds.
    pub fn max_callback_duration_micros(&self) -> u64 {
        self.shared
            .max_callback_duration_micros
            .load(Ordering::Relaxed)
    }

    /// Inferred missed deadlines: callbacks arriving later than
    /// 1.5 × the block duration after their predecessor.
    pub fn xrun_count(&self) -> u64 {
        self.shared.xrun_count.load(Ordering::Relaxed)
    }
}

/// Create a connected controller/executor pair.
pub fn render_plane() -> (RenderPlaneController, super::RenderPlaneExecutor) {
    use std::sync::mpsc::sync_channel;

    let (command_tx, command_rx) = sync_channel(super::COMMAND_MAILBOX_CAPACITY);
    let (retired_tx, retired_rx) = sync_channel(super::RETIRED_MAILBOX_CAPACITY);
    let shared = Arc::new(SharedState::default());
    (
        RenderPlaneController {
            commands: command_tx,
            retired: retired_rx,
            shared: Arc::clone(&shared),
            stream_channels: None,
            last_topology: None,
            plan_generation: 0,
        },
        super::RenderPlaneExecutor {
            commands: command_rx,
            retired: retired_tx,
            shared,
            plan: None,
            parked_retired: None,
            stream_channels: None,
            playing: false,
            live_render: false,
            timeline_tail: false,
            edge_gain: 0.0,
            pending_seek: None,
            event_discontinuity_from: None,
            loop_region: None,
            position_frames: 0,
            last_callback_instant: None,
        },
    )
}
