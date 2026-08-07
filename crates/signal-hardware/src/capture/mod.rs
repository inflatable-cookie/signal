//! Alloc-free capture path: input callback → lock-free SPSC ring → non-RT
//! writer thread → Float32 WAV.
//!
//! [`SpscRing`] (re-exported from `signal-primitives`, where it lives as a
//! shared mechanism primitive) is the only structure the capture callback
//! touches: `push_slice` is a bounded copy plus two atomic operations — no
//! allocation, no locks, no blocking. When the ring is full the excess
//! samples are dropped and counted as overruns; the callback never waits for
//! the writer.
//!
//! Monitoring: [`CaptureSession::start_with_monitor`] tees the same input
//! callback into a [`MonitorSink`] (interleaved stereo) alongside the WAV
//! ring, and [`MonitorSession`] runs the monitor path alone — the
//! arm-without-record mode where input flows to the render plane's live
//! monitor without capturing a take.
//!
//! [`CaptureSession`] owns the full pipeline: it opens an input stream whose
//! callback only pushes into the ring, and spawns a writer thread that
//! drains the ring into a WAV file at the stream's *negotiated* rate and
//! channel count. [`CaptureSession::stop`] tears the stream down first, lets
//! the writer drain the ring completely, finalizes the WAV, and reports what
//! was captured.

mod monitor;
mod session;
mod stopped;

pub use monitor::{CaptureActivationGate, MonitorSession, MonitorSink};
pub use session::{CaptureReport, CaptureSession};
pub use signal_primitives::SpscRing;

#[cfg(test)]
mod tests;
