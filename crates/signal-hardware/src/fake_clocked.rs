//! Device-less clocked output backend for CI soak runs.
//!
//! [`FakeClockedBackend`] implements [`OutputStreamBackend`] without any
//! audio hardware: a dedicated thread ticks at the spec's block cadence and
//! pulls the render callback into a discard buffer. The requested spec is
//! honoured exactly (no negotiation — there is no device to negotiate with),
//! so soak tests exercise the real callback path, timing inference, and
//! shared-state publication on machines with no output device.
//!
//! Timing fidelity: the ticker sleeps until the next block deadline after
//! each callback returns. A callback that overruns its deadline makes the
//! next tick fire immediately — late, exactly like a starved real stream —
//! which is what lets xrun-inference tests starve the clock deliberately.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::output_stream::{
    OutputRenderFn, OutputStreamBackend, OutputStreamError, OutputStreamHandle, OutputStreamSpec,
    OutputStreamState,
};

/// Block size used when the spec leaves `buffer_frames` unset.
const DEFAULT_BLOCK_FRAMES: u32 = 256;

/// Output backend that runs the render callback on a clocked thread instead
/// of a device. See the module docs.
#[derive(Debug, Default)]
pub struct FakeClockedBackend;

impl FakeClockedBackend {
    /// Construct a fake clocked backend.
    pub fn new() -> Self {
        Self
    }
}

struct FakeClockedStream {
    sample_rate_hz: u32,
    channels: u16,
    stopped: Arc<AtomicBool>,
    ticker: Option<std::thread::JoinHandle<()>>,
}

impl OutputStreamHandle for FakeClockedStream {
    fn state(&self) -> OutputStreamState {
        if self.stopped.load(Ordering::Relaxed) {
            OutputStreamState::Stopped
        } else {
            OutputStreamState::Running
        }
    }

    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for FakeClockedStream {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(ticker) = self.ticker.take() {
            let _ = ticker.join();
        }
    }
}

impl OutputStreamBackend for FakeClockedBackend {
    fn open_output_stream(
        &self,
        spec: OutputStreamSpec,
        mut render: OutputRenderFn,
    ) -> Result<Box<dyn OutputStreamHandle>, OutputStreamError> {
        if spec.sample_rate_hz == 0 || spec.channels == 0 {
            return Err(OutputStreamError::new(
                "fake clocked stream needs a non-zero rate and channel count",
            ));
        }
        let block_frames = spec.buffer_frames.unwrap_or(DEFAULT_BLOCK_FRAMES).max(1) as usize;
        let channels = spec.channels as usize;
        let block_duration =
            Duration::from_secs_f64(block_frames as f64 / spec.sample_rate_hz as f64);
        let stopped = Arc::new(AtomicBool::new(false));
        let ticker_stopped = Arc::clone(&stopped);
        let ticker = std::thread::Builder::new()
            .name("signal-fake-clocked-output".to_string())
            .spawn(move || {
                // Discard buffer allocated once, outside the callback.
                let mut frames = vec![0.0f32; block_frames * channels];
                let mut next_deadline = Instant::now() + block_duration;
                while !ticker_stopped.load(Ordering::Relaxed) {
                    render(&mut frames);
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
            .map_err(|error| {
                OutputStreamError::new(format!("spawn fake clocked ticker: {error}"))
            })?;
        Ok(Box::new(FakeClockedStream {
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
    fn ticks_the_callback_at_block_cadence_and_honours_the_spec() {
        let backend = FakeClockedBackend::new();
        let calls = Arc::new(AtomicU64::new(0));
        let callback_calls = Arc::clone(&calls);
        let stream = backend
            .open_output_stream(
                OutputStreamSpec {
                    sample_rate_hz: 48_000,
                    channels: 2,
                    buffer_frames: Some(128),
                },
                Box::new(move |frames| {
                    assert_eq!(frames.len(), 128 * 2);
                    callback_calls.fetch_add(1, Ordering::Relaxed);
                }),
            )
            .expect("open fake stream");
        assert_eq!(stream.state(), OutputStreamState::Running);
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
        drop(stream);
    }

    #[test]
    fn rejects_degenerate_specs() {
        let backend = FakeClockedBackend::new();
        assert!(backend
            .open_output_stream(
                OutputStreamSpec {
                    sample_rate_hz: 0,
                    channels: 2,
                    buffer_frames: None,
                },
                Box::new(|_| {}),
            )
            .is_err());
    }
}
