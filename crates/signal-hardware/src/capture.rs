//! Alloc-free capture path: input callback → lock-free SPSC ring → non-RT
//! writer thread → Float32 WAV.
//!
//! [`SpscRing`] is the only structure the capture callback touches:
//! `push_slice` is a bounded copy plus two atomic operations — no allocation,
//! no locks, no blocking. When the ring is full the excess samples are
//! dropped and counted as overruns; the callback never waits for the writer.
//!
//! [`CaptureSession`] owns the full pipeline: it opens an input stream whose
//! callback only pushes into the ring, and spawns a writer thread that
//! drains the ring into a WAV file at the stream's *negotiated* rate and
//! channel count. [`CaptureSession::stop`] tears the stream down first, lets
//! the writer drain the ring completely, finalizes the WAV, and reports what
//! was captured.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::input_stream::{
    InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
};

/// Samples the writer thread pops per drain iteration.
const WRITER_CHUNK_SAMPLES: usize = 8_192;

/// Writer thread poll interval while the ring is empty.
const WRITER_IDLE_SLEEP: Duration = Duration::from_millis(2);

/// Fixed-capacity lock-free single-producer single-consumer f32 ring.
///
/// Exactly one thread may call [`SpscRing::push_slice`] (the producer — the
/// input callback) and exactly one thread may call [`SpscRing::pop_slice`]
/// (the consumer — the writer). Both are alloc-free and never block: a full
/// ring drops the excess on push and counts it in
/// [`SpscRing::overrun_samples`].
pub struct SpscRing {
    storage: Box<[std::cell::UnsafeCell<f32>]>,
    mask: usize,
    /// Total samples ever pushed (producer-owned, consumer reads).
    head: AtomicUsize,
    /// Total samples ever popped (consumer-owned, producer reads).
    tail: AtomicUsize,
    overrun_samples: AtomicU64,
}

// SAFETY: the SPSC discipline partitions the storage — the producer only
// writes slots in `[tail + capacity, head)` it has claimed before publishing
// `head` with Release, and the consumer only reads slots in `[tail, head)`
// after observing `head` with Acquire. No slot is ever accessed by both
// threads at once.
unsafe impl Sync for SpscRing {}
// SAFETY: f32 has no thread affinity; moving the ring moves plain data.
unsafe impl Send for SpscRing {}

impl SpscRing {
    /// Build a ring holding at least `min_capacity` samples (rounded up to a
    /// power of two).
    pub fn with_capacity(min_capacity: usize) -> Self {
        let capacity = min_capacity.max(2).next_power_of_two();
        let storage: Box<[std::cell::UnsafeCell<f32>]> = (0..capacity)
            .map(|_| std::cell::UnsafeCell::new(0.0f32))
            .collect();
        Self {
            storage,
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            overrun_samples: AtomicU64::new(0),
        }
    }

    /// Sample capacity of the ring.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Samples currently buffered.
    pub fn len(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Whether the ring currently holds no samples.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total samples dropped because the ring was full at push time.
    pub fn overrun_samples(&self) -> u64 {
        self.overrun_samples.load(Ordering::Relaxed)
    }

    /// Producer side: copy as many of `samples` as fit, drop and count the
    /// rest. Returns the number of samples actually written. Alloc-free,
    /// lock-free, never blocks — safe on the audio thread.
    pub fn push_slice(&self, samples: &[f32]) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let free = self.capacity() - head.wrapping_sub(tail);
        let writable = samples.len().min(free);
        for (offset, &sample) in samples[..writable].iter().enumerate() {
            let slot = &self.storage[head.wrapping_add(offset) & self.mask];
            // SAFETY: slots in [head, head + writable) are free (consumer is
            // at or before `tail`); only the single producer writes them, and
            // they are published to the consumer by the Release store below.
            unsafe { *slot.get() = sample };
        }
        self.head
            .store(head.wrapping_add(writable), Ordering::Release);
        let dropped = samples.len() - writable;
        if dropped > 0 {
            self.overrun_samples
                .fetch_add(dropped as u64, Ordering::Relaxed);
        }
        writable
    }

    /// Consumer side: pop up to `out.len()` samples. Returns the number of
    /// samples actually read.
    pub fn pop_slice(&self, out: &mut [f32]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = head.wrapping_sub(tail);
        let readable = out.len().min(available);
        for (offset, sample) in out[..readable].iter_mut().enumerate() {
            let slot = &self.storage[tail.wrapping_add(offset) & self.mask];
            // SAFETY: slots in [tail, tail + readable) were published by the
            // producer's Release store observed via the Acquire load above;
            // only the single consumer reads them before freeing them with
            // the Release store below.
            *sample = unsafe { *slot.get() };
        }
        self.tail
            .store(tail.wrapping_add(readable), Ordering::Release);
        readable
    }
}

/// What a finished capture produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureReport {
    /// Path of the finalized WAV file.
    pub path: PathBuf,
    /// Interleaved frames written to the file.
    pub frames: u64,
    /// Sample rate the stream actually captured at (negotiated).
    pub sample_rate_hz: u32,
    /// Channel count the stream actually captured with (negotiated).
    pub channels: u16,
    /// Samples dropped because the ring was full (0 on a healthy capture).
    pub overrun_samples: u64,
    /// Input latency reported by the stream's timestamps, when observed.
    pub input_latency_micros: Option<u64>,
}

/// A running capture: input stream → SPSC ring → writer thread → WAV.
pub struct CaptureSession {
    stream: Box<dyn InputStreamHandle>,
    ring: Arc<SpscRing>,
    writer_stop: Arc<AtomicBool>,
    writer: Option<std::thread::JoinHandle<Result<u64, String>>>,
    path: PathBuf,
}

impl CaptureSession {
    /// Open an input stream on `backend` and start capturing to a Float32
    /// WAV at `wav_path`. The WAV is written at the stream's *negotiated*
    /// rate and channel count, which may differ from `spec`.
    ///
    /// The capture callback only pushes into the lock-free ring — no
    /// allocation, no locks, no I/O on the audio thread. The ring holds ~2 s
    /// of audio at the negotiated rate; a writer thread that stalls longer
    /// than that shows up as `overrun_samples` in the report instead of
    /// blocking the callback.
    pub fn start(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
    ) -> Result<Self, InputStreamError> {
        // Open with a placeholder ring sized for the request, then resize on
        // negotiation? No — the callback closure must own its ring before the
        // stream starts. Size generously off the *requested* spec; the
        // negotiated rate rarely exceeds the request by more than 2x and the
        // ring only bounds writer-stall tolerance, not correctness.
        let ring_capacity =
            (spec.sample_rate_hz as usize).max(8_192) * spec.channels.max(1) as usize * 2;
        let ring = Arc::new(SpscRing::with_capacity(ring_capacity));
        let callback_ring = Arc::clone(&ring);
        let stream = backend.open_input_stream(
            spec,
            Box::new(move |frames| {
                // RT path: bounded copy + atomics only (see SpscRing docs).
                callback_ring.push_slice(frames);
            }),
        )?;

        let wav_spec = hound::WavSpec {
            channels: stream.channels(),
            sample_rate: stream.sample_rate_hz(),
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        if let Some(parent) = wav_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    InputStreamError::new(format!("create capture dir: {error}"))
                })?;
            }
        }
        let mut wav_writer = hound::WavWriter::create(wav_path, wav_spec)
            .map_err(|error| InputStreamError::new(format!("create capture wav: {error}")))?;

        let writer_stop = Arc::new(AtomicBool::new(false));
        let writer_ring = Arc::clone(&ring);
        let writer_stop_flag = Arc::clone(&writer_stop);
        let channels = u64::from(stream.channels().max(1));
        let writer = std::thread::Builder::new()
            .name("signal-capture-writer".to_string())
            .spawn(move || -> Result<u64, String> {
                let mut chunk = vec![0.0f32; WRITER_CHUNK_SAMPLES];
                let mut samples_written: u64 = 0;
                loop {
                    let popped = writer_ring.pop_slice(&mut chunk);
                    if popped > 0 {
                        for &sample in &chunk[..popped] {
                            wav_writer
                                .write_sample(sample)
                                .map_err(|error| format!("write capture wav: {error}"))?;
                        }
                        samples_written += popped as u64;
                        continue;
                    }
                    // Empty ring: only exit once stop is flagged AND the
                    // ring stayed empty after the flag was observed (the
                    // producer is gone by then — stop drops the stream
                    // before flagging).
                    if writer_stop_flag.load(Ordering::Acquire) && writer_ring.is_empty() {
                        break;
                    }
                    std::thread::sleep(WRITER_IDLE_SLEEP);
                }
                wav_writer
                    .finalize()
                    .map_err(|error| format!("finalize capture wav: {error}"))?;
                Ok(samples_written / channels)
            })
            .map_err(|error| InputStreamError::new(format!("spawn capture writer: {error}")))?;

        Ok(Self {
            stream,
            ring,
            writer_stop,
            writer: Some(writer),
            path: wav_path.to_path_buf(),
        })
    }

    /// Negotiated sample rate the capture runs at.
    pub fn sample_rate_hz(&self) -> u32 {
        self.stream.sample_rate_hz()
    }

    /// Negotiated channel count the capture runs with.
    pub fn channels(&self) -> u16 {
        self.stream.channels()
    }

    /// Latest input latency reported by the stream, when observed.
    pub fn input_latency_micros(&self) -> Option<u64> {
        self.stream.input_latency_micros()
    }

    /// Stop capturing: drop the input stream (no more producer), let the
    /// writer drain the ring fully, finalize the WAV, and report.
    pub fn stop(mut self) -> Result<CaptureReport, InputStreamError> {
        let sample_rate_hz = self.stream.sample_rate_hz();
        let channels = self.stream.channels();
        let input_latency_micros = self.stream.input_latency_micros();
        // Order matters: stop the producer before flagging the writer so the
        // "stop flagged AND ring empty" exit condition means fully drained.
        self.stream = Box::new(StoppedStream {
            sample_rate_hz,
            channels,
        });
        self.writer_stop.store(true, Ordering::Release);
        let frames = self
            .writer
            .take()
            .expect("writer joined once")
            .join()
            .map_err(|_| InputStreamError::new("capture writer panicked"))?
            .map_err(InputStreamError::new)?;
        Ok(CaptureReport {
            path: self.path.clone(),
            frames,
            sample_rate_hz,
            channels,
            overrun_samples: self.ring.overrun_samples(),
            input_latency_micros,
        })
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        // Abandoned without stop(): release the writer thread so it drains
        // and finalizes rather than leaking. The stream field drops first by
        // declaration order only after this body, so flag stop regardless —
        // the writer keeps draining until the ring is empty.
        self.writer_stop.store(true, Ordering::Release);
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}

/// Placeholder handle installed by [`CaptureSession::stop`] after the real
/// stream is dropped (the box must hold something while the report is
/// assembled).
struct StoppedStream {
    sample_rate_hz: u32,
    channels: u16,
}

impl InputStreamHandle for StoppedStream {
    fn state(&self) -> crate::input_stream::InputStreamState {
        crate::input_stream::InputStreamState::Stopped
    }
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }
    fn channels(&self) -> u16 {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake_input::{FakeInputBackend, FAKE_INPUT_TONE_HZ};

    #[test]
    fn ring_round_trips_in_order() {
        let ring = SpscRing::with_capacity(8);
        assert_eq!(ring.capacity(), 8);
        assert!(ring.is_empty());
        assert_eq!(ring.push_slice(&[1.0, 2.0, 3.0]), 3);
        assert_eq!(ring.len(), 3);
        let mut out = [0.0f32; 2];
        assert_eq!(ring.pop_slice(&mut out), 2);
        assert_eq!(out, [1.0, 2.0]);
        let mut rest = [0.0f32; 4];
        assert_eq!(ring.pop_slice(&mut rest), 1);
        assert_eq!(rest[0], 3.0);
        assert!(ring.is_empty());
        assert_eq!(ring.overrun_samples(), 0);
    }

    #[test]
    fn ring_wraps_around_its_storage() {
        let ring = SpscRing::with_capacity(4);
        let mut out = [0.0f32; 4];
        // Push/pop repeatedly so the indices lap the storage several times.
        for lap in 0..10 {
            let base = lap as f32 * 3.0;
            assert_eq!(ring.push_slice(&[base, base + 1.0, base + 2.0]), 3);
            assert_eq!(ring.pop_slice(&mut out[..3]), 3);
            assert_eq!(&out[..3], &[base, base + 1.0, base + 2.0]);
        }
        assert_eq!(ring.overrun_samples(), 0);
    }

    #[test]
    fn ring_drops_and_counts_overruns_when_full() {
        let ring = SpscRing::with_capacity(4);
        assert_eq!(ring.push_slice(&[1.0, 2.0, 3.0, 4.0]), 4);
        // Full: everything dropped, counted, push never blocks.
        assert_eq!(ring.push_slice(&[5.0, 6.0]), 0);
        assert_eq!(ring.overrun_samples(), 2);
        // Partial fit: one in, one dropped.
        let mut out = [0.0f32; 1];
        assert_eq!(ring.pop_slice(&mut out), 1);
        assert_eq!(ring.push_slice(&[7.0, 8.0]), 1);
        assert_eq!(ring.overrun_samples(), 3);
        let mut drained = [0.0f32; 4];
        assert_eq!(ring.pop_slice(&mut drained), 4);
        assert_eq!(drained, [2.0, 3.0, 4.0, 7.0]);
    }

    #[test]
    fn ring_survives_concurrent_producer_and_consumer() {
        let ring = Arc::new(SpscRing::with_capacity(1024));
        let producer_ring = Arc::clone(&ring);
        const TOTAL: usize = 100_000;
        let producer = std::thread::spawn(move || {
            let mut next = 0usize;
            while next < TOTAL {
                let batch_end = (next + 64).min(TOTAL);
                let batch: Vec<f32> = (next..batch_end).map(|value| value as f32).collect();
                let mut written = 0;
                while written < batch.len() {
                    written += producer_ring.push_slice(&batch[written..]);
                    std::hint::spin_loop();
                }
                next = batch_end;
            }
        });
        let mut received = Vec::with_capacity(TOTAL);
        let mut chunk = [0.0f32; 97];
        while received.len() < TOTAL {
            let popped = ring.pop_slice(&mut chunk);
            received.extend_from_slice(&chunk[..popped]);
            if popped == 0 {
                std::hint::spin_loop();
            }
        }
        producer.join().expect("producer joins");
        // No sample lost, none reordered. (Overruns DO accrue here: the
        // producer deliberately retries against a full ring, and every
        // rejected sample is counted — that is the drop-and-count contract.)
        for (index, &value) in received.iter().enumerate() {
            assert_eq!(value, index as f32, "sample {index} out of order");
        }
    }

    #[test]
    fn capture_session_records_the_fake_tone_to_wav() {
        let dir = std::env::temp_dir().join(format!(
            "signal-capture-e2e-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let wav_path = dir.join("take.wav");
        let backend = FakeInputBackend::new();
        let session = CaptureSession::start(
            &backend,
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 1,
                buffer_frames: Some(256),
            },
            &wav_path,
        )
        .expect("start capture");
        assert_eq!(session.sample_rate_hz(), 48_000);
        assert_eq!(session.channels(), 1);

        std::thread::sleep(Duration::from_millis(1_000));
        let report = session.stop().expect("stop capture");

        assert_eq!(report.path, wav_path);
        assert_eq!(report.sample_rate_hz, 48_000);
        assert_eq!(report.channels, 1);
        assert_eq!(report.overrun_samples, 0, "writer kept up with the ring");
        // ~1 s captured; the fake clock is sleep-based, allow generous slop.
        assert!(
            report.frames > 24_000 && report.frames < 96_000,
            "expected ≈48000 frames, got {}",
            report.frames
        );

        // Read the WAV back and verify the content is the 440 Hz tone.
        let mut reader = hound::WavReader::open(&wav_path).expect("open captured wav");
        let read_spec = reader.spec();
        assert_eq!(read_spec.sample_rate, 48_000);
        assert_eq!(read_spec.channels, 1);
        assert_eq!(read_spec.sample_format, hound::SampleFormat::Float);
        let samples: Vec<f32> = reader
            .samples::<f32>()
            .map(|sample| sample.expect("read sample"))
            .collect();
        assert_eq!(samples.len() as u64, report.frames);

        // RMS of a 0.5-amplitude sine is 0.5/√2 ≈ 0.354.
        let rms = (samples
            .iter()
            .map(|s| f64::from(*s) * f64::from(*s))
            .sum::<f64>()
            / samples.len() as f64)
            .sqrt();
        assert!(
            (rms - 0.3536).abs() < 0.05,
            "expected sine RMS ≈ 0.354, got {rms}"
        );

        // Zero-crossing rate of a 440 Hz sine at 48 kHz: 880 crossings/s →
        // ≈ 0.01833 crossings per sample.
        let crossings = samples
            .windows(2)
            .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
            .count();
        let crossings_per_sample = crossings as f64 / samples.len() as f64;
        let expected = 2.0 * f64::from(FAKE_INPUT_TONE_HZ) / 48_000.0;
        assert!(
            (crossings_per_sample - expected).abs() < expected * 0.1,
            "zero-crossing rate {crossings_per_sample} vs expected {expected}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
