//! WAV capture session pipeline.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::input_stream::{
    InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
};

use super::monitor::{CaptureActivationGate, MonitorSink, MonitorTee};
use super::stopped::StoppedStream;
use signal_primitives::SpscRing;

const WRITER_CHUNK_SAMPLES: usize = 8_192;
const WRITER_IDLE_SLEEP: Duration = Duration::from_millis(2);

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
        Self::start_with_skip(backend, spec, wav_path, 0)
    }

    /// [`CaptureSession::start`] with a count-in style pre-roll discard: the
    /// writer thread drops the first `skip_initial_frames` frames arriving
    /// from the ring before writing anything, so the WAV's first frame is
    /// the first post-pre-roll capture frame. `skip_initial_frames` is
    /// interpreted at the *requested* `spec.sample_rate_hz` and rescaled to
    /// the negotiated stream rate when they differ. The capture report
    /// counts written frames only (skipped frames never reach the file).
    pub fn start_with_skip(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        skip_initial_frames: u64,
    ) -> Result<Self, InputStreamError> {
        Self::start_internal(backend, spec, wav_path, skip_initial_frames, None)
    }

    /// [`CaptureSession::start_with_skip`] with a monitor tee: the input
    /// callback pushes each quantum into BOTH the capture ring (→ WAV) and
    /// `monitor` (converted to interleaved stereo, see [`MonitorSink`]) —
    /// two bounded pushes, still alloc-free. This is how a take records
    /// sample-aligned while the musician hears themselves through the mix.
    pub fn start_with_monitor(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        skip_initial_frames: u64,
        monitor: MonitorSink,
    ) -> Result<Self, InputStreamError> {
        Self::start_internal(backend, spec, wav_path, skip_initial_frames, Some(monitor))
    }

    /// Open a stream behind a shared cohort activation gate. Monitoring keeps
    /// flowing before activation; only WAV capture waits for the gate.
    pub fn start_gated_with_monitor(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        gate: CaptureActivationGate,
        monitor: Option<MonitorSink>,
    ) -> Result<Self, InputStreamError> {
        Self::start_internal_gated(backend, spec, wav_path, 0, monitor, Some(gate))
    }

    /// [`CaptureSession::start_gated_with_monitor`] with a count-in pre-roll
    /// discard. Composition order is gate THEN skip: while the gate is
    /// closed the callback pushes nothing into the WAV ring, so
    /// `skip_initial_frames` counts against the first frames captured AFTER
    /// activation — a gate released anywhere in the skip window can never
    /// leak pre-activation frames into the file. The skip is interpreted at
    /// the requested `spec.sample_rate_hz` and rescaled to the negotiated
    /// stream rate (the [`CaptureSession::start_with_skip`] contract).
    pub fn start_gated_with_monitor_and_skip(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        skip_initial_frames: u64,
        gate: CaptureActivationGate,
        monitor: Option<MonitorSink>,
    ) -> Result<Self, InputStreamError> {
        Self::start_internal_gated(
            backend,
            spec,
            wav_path,
            skip_initial_frames,
            monitor,
            Some(gate),
        )
    }

    fn start_internal(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        skip_initial_frames: u64,
        monitor: Option<MonitorSink>,
    ) -> Result<Self, InputStreamError> {
        Self::start_internal_gated(backend, spec, wav_path, skip_initial_frames, monitor, None)
    }

    fn start_internal_gated(
        backend: &dyn InputStreamBackend,
        spec: InputStreamSpec,
        wav_path: &Path,
        skip_initial_frames: u64,
        monitor: Option<MonitorSink>,
        gate: Option<CaptureActivationGate>,
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
        let capture_gate = gate.map(|gate| gate.active);
        let mut tee = monitor.map(MonitorTee::new);
        let negotiated_channels = tee.as_ref().map(|tee| Arc::clone(&tee.channels));
        let stream = backend.open_input_stream(
            spec,
            Box::new(move |frames| {
                // RT path: bounded copies + atomics only (see SpscRing docs);
                // the monitor tee is a second bounded push, never a lock.
                if capture_gate
                    .as_ref()
                    .is_none_or(|active| active.load(Ordering::Acquire))
                {
                    callback_ring.push_slice(frames);
                }
                if let Some(tee) = tee.as_mut() {
                    tee.feed(frames);
                }
            }),
        )?;
        if let Some(channels) = negotiated_channels {
            channels.store(stream.channels().max(1) as usize, Ordering::Release);
        }

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
        // Pre-roll discard, rescaled from the requested to the negotiated
        // rate (frames at the rate the audio actually arrives at), then
        // expanded to interleaved samples for the writer's counter.
        let skip_frames_negotiated = if stream.sample_rate_hz() == spec.sample_rate_hz {
            skip_initial_frames
        } else {
            (u128::from(skip_initial_frames) * u128::from(stream.sample_rate_hz())
                / u128::from(spec.sample_rate_hz.max(1))) as u64
        };
        let mut skip_samples = skip_frames_negotiated.saturating_mul(channels);
        let writer = std::thread::Builder::new()
            .name("signal-capture-writer".to_string())
            .spawn(move || -> Result<u64, String> {
                let mut chunk = vec![0.0f32; WRITER_CHUNK_SAMPLES];
                let mut samples_written: u64 = 0;
                loop {
                    let popped = writer_ring.pop_slice(&mut chunk);
                    if popped > 0 {
                        // Count-in pre-roll: discard before writing.
                        let discard = (popped as u64).min(skip_samples) as usize;
                        skip_samples -= discard as u64;
                        for &sample in &chunk[discard..popped] {
                            wav_writer
                                .write_sample(sample)
                                .map_err(|error| format!("write capture wav: {error}"))?;
                        }
                        samples_written += (popped - discard) as u64;
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
