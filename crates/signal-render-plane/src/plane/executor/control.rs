use std::sync::atomic::Ordering;
use std::sync::mpsc::{TryRecvError, TrySendError};

use crate::plan::RenderPlan;
use crate::LIVE_EVENT_PUSH_CAPACITY;

use super::super::command::RenderCommand;
use super::RenderPlaneExecutor;

impl RenderPlaneExecutor {
    /// Offline-bounce hook: snap the transport edge envelope to `gain`
    /// without ramping. Realtime transport must NEVER use this — the 5 ms
    /// edge ramp is what keeps play/stop/seek click-free on speakers — but
    /// an offline export has no speaker and must start at full level, so
    /// the offline driver (`offline::render_plan_to_pcm`) snaps the
    /// envelope open after draining transport commands and before the first
    /// rendered block. Crate-private on purpose: hosts cannot reach it.
    pub(crate) fn set_edge_gain_immediate(&mut self, gain: f32) {
        self.edge_gain = gain.clamp(0.0, 1.0);
    }

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

    pub(in crate::plane::executor) fn apply_seek(&mut self, position_frames: u64) {
        self.event_discontinuity_from = Some(self.position_frames);
        self.position_frames = position_frames;
        self.shared
            .position_frames
            .store(position_frames, Ordering::Relaxed);
    }

    pub(crate) fn drain_commands(&mut self) {
        loop {
            match self.commands.try_recv() {
                Ok(RenderCommand::InstallPlan(mut next_plan)) => {
                    if let Some(mut previous) = self.plan.take() {
                        next_plan.inherit_state(&mut previous);
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
                Ok(RenderCommand::SetLoopRegion(region)) => {
                    self.loop_region = region;
                }
                Ok(RenderCommand::SetLiveRender { active }) => {
                    self.live_render = active;
                    self.shared.live_render.store(active, Ordering::Relaxed);
                }
                Ok(RenderCommand::PushLiveEvents {
                    stage_index,
                    events,
                    len,
                }) => {
                    // Append into the stage's preallocated ring; anything
                    // that cannot land (no plan, stage gone or no longer
                    // accepting after a reinstall raced the push, ring full)
                    // drops and counts — never blocks, never allocates.
                    let len = len.min(LIVE_EVENT_PUSH_CAPACITY);
                    let mut accepted = 0usize;
                    if let Some(plan) = self.plan.as_mut() {
                        if let Some(stage) = plan.stages.get_mut(stage_index) {
                            if stage.accepts_live_events {
                                for event in events.iter().take(len) {
                                    if stage.live_events.len() < stage.live_events.capacity() {
                                        stage.live_events.push(*event);
                                        accepted += 1;
                                    }
                                }
                            }
                        }
                    }
                    if accepted < len {
                        self.shared
                            .live_event_drop_count
                            .fetch_add((len - accepted) as u64, Ordering::Relaxed);
                    }
                }
                Ok(RenderCommand::SetStreamChannels(channels)) => {
                    self.stream_channels = Some(channels.max(1));
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }
}
