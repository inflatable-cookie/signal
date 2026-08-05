//! Device-less clocked input backend for CI capture tests.
//!
//! [`FakeInputBackend`] implements [`InputStreamBackend`] without any audio
//! hardware: a dedicated thread ticks at the spec's block cadence and
//! synthesizes a known sine tone into the capture callback. The requested
//! spec is honoured exactly (no negotiation — there is no device to
//! negotiate with), so capture tests exercise the real callback path on
//! machines with no input device.
//!
//! Mirror of [`crate::fake_clocked::FakeClockedBackend`] for the input
//! direction; the same deadline-anchored ticking keeps cadence realistic
//! without burst-spinning after a slow callback.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::input_stream::{
    InputCaptureFn, InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
    InputStreamState,
};

/// Block size used when the spec leaves `buffer_frames` unset.
const DEFAULT_BLOCK_FRAMES: u32 = 256;

/// Frequency of the synthesized test tone, hertz.
pub const FAKE_INPUT_TONE_HZ: f32 = 440.0;

/// Peak amplitude of the synthesized test tone.
pub const FAKE_INPUT_TONE_AMPLITUDE: f32 = 0.5;

/// Input backend that synthesizes a known 440 Hz sine into the capture
/// callback on a clocked thread instead of a device. See the module docs.
#[derive(Debug, Default)]
pub struct FakeInputBackend;

impl FakeInputBackend {
    /// Construct a fake input backend.
    pub fn new() -> Self {
        Self
    }
}

struct FakeInputStream {
    sample_rate_hz: u32,
    channels: u16,
    stopped: Arc<AtomicBool>,
    ticker: Option<std::thread::JoinHandle<()>>,
}

impl InputStreamHandle for FakeInputStream {
    fn state(&self) -> InputStreamState {
        if self.stopped.load(Ordering::Relaxed) {
            InputStreamState::Stopped
        } else {
            InputStreamState::Running
        }
    }

    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for FakeInputStream {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(ticker) = self.ticker.take() {
            let _ = ticker.join();
        }
    }
}

impl InputStreamBackend for FakeInputBackend {
    fn open_input_stream(
        &self,
        spec: InputStreamSpec,
        mut capture: InputCaptureFn,
    ) -> Result<Box<dyn InputStreamHandle>, InputStreamError> {
        if spec.sample_rate_hz == 0 || spec.channels == 0 {
            return Err(InputStreamError::new(
                "fake input stream needs a non-zero rate and channel count",
            ));
        }
        let block_frames = spec.buffer_frames.unwrap_or(DEFAULT_BLOCK_FRAMES).max(1) as usize;
        let channels = spec.channels as usize;
        let sample_rate_hz = spec.sample_rate_hz;
        let block_duration = Duration::from_secs_f64(block_frames as f64 / sample_rate_hz as f64);
        let stopped = Arc::new(AtomicBool::new(false));
        let ticker_stopped = Arc::clone(&stopped);
        let ticker = std::thread::Builder::new()
            .name("signal-fake-input".to_string())
            .spawn(move || {
                // Synthesis buffer allocated once, outside the callback.
                let mut frames = vec![0.0f32; block_frames * channels];
                let phase_step = std::f32::consts::TAU * FAKE_INPUT_TONE_HZ / sample_rate_hz as f32;
                let mut phase = 0.0f32;
                let mut next_deadline = Instant::now() + block_duration;
                while !ticker_stopped.load(Ordering::Relaxed) {
                    for frame in frames.chunks_exact_mut(channels) {
                        let sample = FAKE_INPUT_TONE_AMPLITUDE * phase.sin();
                        frame.fill(sample);
                        phase += phase_step;
                        if phase >= std::f32::consts::TAU {
                            phase -= std::f32::consts::TAU;
                        }
                    }
                    capture(&frames);
                    let now = Instant::now();
                    if next_deadline > now {
                        std::thread::sleep(next_deadline - now);
                        next_deadline += block_duration;
                    } else {
                        // Deadline already missed (slow callback): tick
                        // immediately and re-anchor so we do not burst-spin
                        // to catch up.
                        next_deadline = Instant::now() + block_duration;
                    }
                }
            })
            .map_err(|error| InputStreamError::new(format!("spawn fake input ticker: {error}")))?;
        Ok(Box::new(FakeInputStream {
            sample_rate_hz: spec.sample_rate_hz,
            channels: spec.channels,
            stopped,
            ticker: Some(ticker),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn ticks_the_capture_callback_with_a_tone_at_block_cadence() {
        let backend = FakeInputBackend::new();
        let calls = Arc::new(AtomicU64::new(0));
        let callback_calls = Arc::clone(&calls);
        let peaked = Arc::new(AtomicBool::new(false));
        let callback_peaked = Arc::clone(&peaked);
        let stream = backend
            .open_input_stream(
                InputStreamSpec {
                    sample_rate_hz: 48_000,
                    channels: 2,
                    buffer_frames: Some(128),
                },
                Box::new(move |frames| {
                    assert_eq!(frames.len(), 128 * 2);
                    callback_calls.fetch_add(1, Ordering::Relaxed);
                    if frames.iter().any(|sample| sample.abs() > 0.4) {
                        callback_peaked.store(true, Ordering::Relaxed);
                    }
                }),
            )
            .expect("open fake input stream");
        assert_eq!(stream.state(), InputStreamState::Running);
        assert_eq!(stream.sample_rate_hz(), 48_000);
        assert_eq!(stream.channels(), 2);
        assert_eq!(stream.last_error(), None);

        // 128 frames at 48 kHz ≈ 2.67 ms per block. Wait for ten blocks rather
        // than sleeping a fixed span and asserting a count: a fixed sleep turns
        // this into a claim about how fast the host is, which fails on a loaded
        // machine or a shared CI runner and proves nothing when it passes. The
        // upper bound is derived from the time actually waited, so it still
        // catches a callback ticking faster than block cadence.
        const BLOCK_PERIOD: Duration = Duration::from_micros(2_667);
        const WANTED_BLOCKS: u64 = 10;
        let started = std::time::Instant::now();
        while calls.load(Ordering::Relaxed) < WANTED_BLOCKS
            && started.elapsed() < Duration::from_secs(5)
        {
            std::thread::sleep(Duration::from_millis(2));
        }
        let waited = started.elapsed();
        let observed = calls.load(Ordering::Relaxed);
        assert!(
            observed >= WANTED_BLOCKS,
            "expected {WANTED_BLOCKS} clocked callbacks within 5s, saw {observed}",
        );
        // Two blocks of slack over the elapsed-time budget covers scheduler jitter.
        let ceiling = (waited.as_secs_f64() / BLOCK_PERIOD.as_secs_f64()).ceil() as u64 + 2;
        assert!(
            observed <= ceiling,
            "callback ticked too fast: {observed} blocks in {waited:?} (ceiling {ceiling})",
        );
        assert!(peaked.load(Ordering::Relaxed), "tone never neared its peak");
        drop(stream);
    }

    #[test]
    fn rejects_degenerate_specs() {
        let backend = FakeInputBackend::new();
        assert!(backend
            .open_input_stream(
                InputStreamSpec {
                    sample_rate_hz: 0,
                    channels: 2,
                    buffer_frames: None,
                },
                Box::new(|_| {}),
            )
            .is_err());
    }
}
