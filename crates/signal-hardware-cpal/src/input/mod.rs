//! cpal-backed implementation of Signal's input-stream contract, plus real
//! input-device enumeration. Exact mirror of the output side in `lib.rs`:
//! same negotiation semantics, same dedicated owner-thread pattern, same
//! latency/error/device-name capture — direction flipped.

mod backend;
mod enumerate;
mod stream;
mod types;

#[cfg(test)]
mod tests;

pub(crate) const STATE_RUNNING: u8 = 0;
pub(crate) const STATE_STOPPED: u8 = 1;
pub(crate) const STATE_FAULTED: u8 = 2;
pub(crate) const CHANNEL_SELECTION_SCRATCH_SAMPLES: usize = 2048;

pub use backend::CpalInputBackend;
pub use enumerate::{default_input_device_name, enumerate_input_devices};
pub use types::{CpalInputEndpoint, InputChannelDescription, InputDeviceDescription};
