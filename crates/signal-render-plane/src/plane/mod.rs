//! Control/render split: controller, executor, and command mailbox.

mod command;
mod controller;
mod executor;

pub use controller::{render_plane, RenderPlaneController, RenderPlaneError};
pub use executor::RenderPlaneExecutor;

/// Capacity of the control→render command mailbox.
pub(crate) const COMMAND_MAILBOX_CAPACITY: usize = 64;
/// Sized so that even a full command mailbox of plan installs can retire
/// without saturating; install_plan also reclaims eagerly. The executor's
/// single parked slot is belt-and-braces on top of this invariant.
pub(crate) const RETIRED_MAILBOX_CAPACITY: usize = COMMAND_MAILBOX_CAPACITY + 2;

/// Transport edge ramp length: play, stop, and seek gate through this
/// envelope instead of stepping, so transport actions never click.
pub(crate) const EDGE_RAMP_SECONDS: f32 = 0.005;
/// Full-swing time for stage gain changes across plan swaps.
pub(crate) const GAIN_SMOOTHING_SECONDS: f32 = 0.010;
/// Micro-fade applied to the output buffer around a loop-region wrap point.
pub(crate) const LOOP_WRAP_FADE_FRAMES: usize = 64;
