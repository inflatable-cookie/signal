//! Audio-thread render plane executor.

mod control;
mod health;
mod render;

use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;
use std::time::Instant;

use crate::plan::RenderPlan;

use super::command::{RenderCommand, SharedState};

/// Audio-thread executor: drains the command mailbox between blocks and
/// runs [`RenderPlaneExecutor::render_block`] with a hard no-alloc contract.
pub struct RenderPlaneExecutor {
    pub(crate) commands: Receiver<RenderCommand>,
    pub(crate) retired: SyncSender<Box<RenderPlan>>,
    pub(crate) shared: Arc<SharedState>,
    pub(crate) plan: Option<Box<RenderPlan>>,
    pub(crate) parked_retired: Option<Box<RenderPlan>>,
    /// Stream channel count as told by the controller; the active plan's
    /// expectation (its master format when no stream was known at install)
    /// applies until the first [`RenderCommand::SetStreamChannels`].
    pub(crate) stream_channels: Option<u16>,
    /// Transport gate target. Audio follows through `edge_gain`.
    pub(crate) playing: bool,
    /// Live render posture (g13.018): while set, stages render even when the
    /// transport is stopped (live monitoring and live events stay audible);
    /// compiled timeline content stays `playing`-gated and the position does
    /// not advance while stopped.
    pub(crate) live_render: bool,
    /// True while timeline content is still winding down after a stop: set
    /// while playing and held through the stop edge ramp-out, cleared once
    /// the edge envelope closes (or immediately under the live render
    /// posture, whose stop cuts timeline content at the block boundary).
    /// Distinguishes "ramping out after playback" from "edge held open by
    /// the live render posture" — only the former renders clips and
    /// advances the position while `playing` is false.
    pub(crate) timeline_tail: bool,
    /// Transport edge envelope: ramps toward 1 when playing, 0 when stopped
    /// or before an in-flight seek, so transport actions never step audio.
    pub(crate) edge_gain: f32,
    /// Seek requested while audible: applied once `edge_gain` reaches zero.
    pub(crate) pending_seek: Option<u64>,
    /// Timeline position abandoned by the most recently applied seek. The
    /// next event-bearing block uses it to release old notes and chase the
    /// destination state, then clears it.
    pub(crate) event_discontinuity_from: Option<u64>,
    /// Transport loop region `[start, end)`; playback wraps to `start` when
    /// a rendered block crosses `end` (see `render_block_inner`).
    pub(crate) loop_region: Option<(u64, u64)>,
    pub(crate) position_frames: u64,
    /// Start instant of the previous callback, for xrun inference.
    pub(crate) last_callback_instant: Option<Instant>,
}
