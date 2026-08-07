//! Capture activation gate and monitor-only sessions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::input_stream::{
    InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec, InputStreamState,
};

/// Shared start gate for a cohort of already-open capture streams.
///
/// Input callbacks continue feeding their monitor sinks while gated, but WAV
/// rings receive nothing until [`Self::activate`] publishes the common start.
/// The audio-thread check is one atomic load.
#[derive(Clone, Debug, Default)]
pub struct CaptureActivationGate {
    pub(crate) active: Arc<AtomicBool>,
}

impl CaptureActivationGate {
    /// Build a closed gate.
    pub fn new() -> Self {
        Self::default()
    }

    /// Release every capture callback sharing this gate.
    pub fn activate(&self) {
        self.active.store(true, Ordering::Release);
    }
}

/// Monitor sink: consumes interleaved STEREO f32 frames on the audio thread
/// (the capture callback converts the stream's negotiated channel count to
/// stereo before calling it — see [`monitor_frames_to_stereo`]). The sink
/// MUST be alloc-free, lock-free, and non-blocking; feed a lock-free ring
/// (e.g. the render plane's live-input feeder), never a mutex.
pub type MonitorSink = Box<dyn FnMut(&[f32]) + Send + 'static>;

/// Fixed stack scratch for the monitor tee's channel conversion, in frames.
/// Callbacks longer than this are converted chunk-wise (still alloc-free).
const MONITOR_CHUNK_FRAMES: usize = 512;

/// The monitor tee's stereo conversion state: the negotiated channel count
/// is only known AFTER the stream opens (the callback closure is built
/// before), so it publishes through an atomic — 0 means "not yet known" and
/// the callback drops those first frames rather than misinterleaving them.
pub(crate) struct MonitorTee {
    pub(crate) channels: Arc<std::sync::atomic::AtomicUsize>,
    pub(crate) sink: MonitorSink,
}

impl MonitorTee {
    pub(crate) fn new(sink: MonitorSink) -> Self {
        Self {
            channels: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            sink,
        }
    }

    /// Convert one callback quantum of interleaved negotiated-channel frames
    /// to stereo and hand it to the sink chunk-wise: mono duplicates to both
    /// channels, stereo copies through, wider layouts take the first two
    /// channels. Alloc-free (fixed stack scratch); safe on the audio thread.
    pub(crate) fn feed(&mut self, frames: &[f32]) {
        let channels = self.channels.load(Ordering::Acquire);
        if channels == 0 {
            return; // Negotiation not yet published: drop, never guess.
        }
        if channels == 2 {
            (self.sink)(frames);
            return;
        }
        let mut scratch = [0.0f32; MONITOR_CHUNK_FRAMES * 2];
        let mut frame_index = 0usize;
        let frame_count = frames.len() / channels;
        while frame_index < frame_count {
            let take = (frame_count - frame_index).min(MONITOR_CHUNK_FRAMES);
            for offset in 0..take {
                let base = (frame_index + offset) * channels;
                let left = frames[base];
                let right = if channels == 1 {
                    left
                } else {
                    frames[base + 1]
                };
                scratch[offset * 2] = left;
                scratch[offset * 2 + 1] = right;
            }
            (self.sink)(&scratch[..take * 2]);
            frame_index += take;
        }
    }
}

/// A monitor-only input session: input stream → stereo conversion → monitor
/// sink, with NO wav capture. Serves arm-without-record: the host opens the
/// input on arm so monitoring (and measured input latency) exist before the
/// first take rolls. Dropping the session stops the stream.
pub struct MonitorSession {
    stream: Box<dyn InputStreamHandle>,
}

impl MonitorSession {
    /// Open an input stream on `backend` whose callback converts each
    /// quantum to interleaved stereo and pushes it into `sink`. The stream
    /// negotiates rate/channels as usual; read them back from the session.
    /// (A handful of pre-negotiation callbacks may be dropped — the closure
    /// cannot know the negotiated channel count before open returns.)
    pub fn start(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        sink: MonitorSink,
    ) -> Result<Self, InputStreamError> {
        let mut tee = MonitorTee::new(sink);
        let negotiated_channels = Arc::clone(&tee.channels);
        let stream = backend.open_input_stream(
            spec,
            Box::new(move |frames| {
                tee.feed(frames);
            }),
        )?;
        negotiated_channels.store(stream.channels().max(1) as usize, Ordering::Release);
        Ok(Self { stream })
    }

    /// Negotiated sample rate the monitor input runs at.
    pub fn sample_rate_hz(&self) -> u32 {
        self.stream.sample_rate_hz()
    }

    /// Negotiated channel count of the underlying input stream (the sink
    /// always receives stereo).
    pub fn channels(&self) -> u16 {
        self.stream.channels()
    }

    /// Latest input latency reported by the stream, when observed.
    pub fn input_latency_micros(&self) -> Option<u64> {
        self.stream.input_latency_micros()
    }

    /// Current backend lifecycle, including asynchronous device loss.
    pub fn state(&self) -> InputStreamState {
        self.stream.state()
    }

    /// Backend detail associated with a faulted stream, when available.
    pub fn last_error(&self) -> Option<String> {
        self.stream.last_error()
    }
}
