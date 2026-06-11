//! cpal-backed implementation of Signal's output-stream contract.
//!
//! This is the first crate in the stack that actually produces sound: it
//! opens a real output stream on the default device and pulls interleaved
//! f32 frames from the caller's render callback.
//!
//! # Real-time posture
//!
//! cpal invokes the data callback on the OS audio thread. This crate adds no
//! locks or allocation on that path beyond cpal's own internals: the render
//! callback moves into the stream closure once at open time and is called
//! directly. Callers own the rest of the RT contract (see
//! `signal_hardware::output_stream`).

#![warn(missing_docs)]

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use signal_hardware::{
    OutputRenderFn, OutputStreamBackend, OutputStreamError, OutputStreamHandle, OutputStreamSpec,
    OutputStreamState,
};

const STATE_RUNNING: u8 = 0;
const STATE_STOPPED: u8 = 1;
const STATE_FAULTED: u8 = 2;

/// Output backend over the host's default cpal device.
#[derive(Debug, Default)]
pub struct CpalOutputBackend;

impl CpalOutputBackend {
    /// Construct a backend over the default cpal host.
    pub fn new() -> Self {
        Self
    }
}

struct CpalOutputStream {
    // Held for lifetime; dropping stops the stream.
    _stream: cpal::Stream,
    state: Arc<AtomicU8>,
    sample_rate_hz: u32,
    channels: u16,
}

// cpal::Stream is not Send on every platform; on macOS (coreaudio) the
// handle is safe to move between threads as long as it is not used
// concurrently, which this wrapper guarantees by never touching the stream
// after construction except to drop it.
unsafe impl Send for CpalOutputStream {}

impl OutputStreamHandle for CpalOutputStream {
    fn state(&self) -> OutputStreamState {
        match self.state.load(Ordering::Relaxed) {
            STATE_RUNNING => OutputStreamState::Running,
            STATE_STOPPED => OutputStreamState::Stopped,
            _ => OutputStreamState::Faulted,
        }
    }

    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for CpalOutputStream {
    fn drop(&mut self) {
        self.state.store(STATE_STOPPED, Ordering::Relaxed);
    }
}

impl OutputStreamBackend for CpalOutputBackend {
    fn open_output_stream(
        &self,
        spec: OutputStreamSpec,
        mut render: OutputRenderFn,
    ) -> Result<Box<dyn OutputStreamHandle>, OutputStreamError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| OutputStreamError::new("no default output device"))?;

        let mut config: cpal::StreamConfig = device
            .default_output_config()
            .map_err(|error| {
                OutputStreamError::new(format!("default output config: {error}"))
            })?
            .into();
        config.sample_rate = cpal::SampleRate(spec.sample_rate_hz);
        config.channels = spec.channels;
        if let Some(buffer_frames) = spec.buffer_frames {
            config.buffer_size = cpal::BufferSize::Fixed(buffer_frames);
        }

        let state = Arc::new(AtomicU8::new(STATE_RUNNING));
        let error_state = Arc::clone(&state);

        let stream = device
            .build_output_stream(
                &config,
                move |frames: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    render(frames);
                },
                move |error| {
                    // Stream errors arrive on a backend thread; record the
                    // fault without blocking.
                    let _ = error;
                    error_state.store(STATE_FAULTED, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|error| OutputStreamError::new(format!("build output stream: {error}")))?;

        stream
            .play()
            .map_err(|error| OutputStreamError::new(format!("start output stream: {error}")))?;

        Ok(Box::new(CpalOutputStream {
            _stream: stream,
            state,
            sample_rate_hz: spec.sample_rate_hz,
            channels: spec.channels,
        }))
    }
}
