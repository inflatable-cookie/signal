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

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::input_stream::{
    InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec, InputStreamState,
};

/// Lock-free SPSC sample ring; re-homed to `signal-primitives` as a shared
/// mechanism primitive (the render plane's live-input monitor path uses it
/// too). Re-exported here so capture code keeps its public path.
pub use signal_primitives::SpscRing;

/// Samples the writer thread pops per drain iteration.
const WRITER_CHUNK_SAMPLES: usize = 8_192;

/// Writer thread poll interval while the ring is empty.
const WRITER_IDLE_SLEEP: Duration = Duration::from_millis(2);

/// Shared start gate for a cohort of already-open capture streams.
///
/// Input callbacks continue feeding their monitor sinks while gated, but WAV
/// rings receive nothing until [`Self::activate`] publishes the common start.
/// The audio-thread check is one atomic load.
#[derive(Clone, Debug, Default)]
pub struct CaptureActivationGate {
    active: Arc<AtomicBool>,
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
struct MonitorTee {
    channels: Arc<std::sync::atomic::AtomicUsize>,
    sink: MonitorSink,
}

impl MonitorTee {
    fn new(sink: MonitorSink) -> Self {
        Self {
            channels: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            sink,
        }
    }

    /// Convert one callback quantum of interleaved negotiated-channel frames
    /// to stereo and hand it to the sink chunk-wise: mono duplicates to both
    /// channels, stereo copies through, wider layouts take the first two
    /// channels. Alloc-free (fixed stack scratch); safe on the audio thread.
    fn feed(&mut self, frames: &[f32]) {
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

    // SpscRing's unit tests moved with it to `signal_primitives::spsc`.

    #[test]
    fn monitor_session_duplicates_mono_input_to_stereo() {
        // Monitor-only mode: the fake backend synthesizes a mono tone; the
        // sink must receive interleaved STEREO with identical channels.
        let backend = FakeInputBackend::new();
        let received: Arc<std::sync::Mutex<Vec<f32>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink_received = Arc::clone(&received);
        let session = MonitorSession::start(
            &backend,
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 1,
                buffer_frames: Some(128),
            },
            Box::new(move |frames| {
                // Test sink only: a mutex is fine here (no real audio thread).
                sink_received.lock().unwrap().extend_from_slice(frames);
            }),
        )
        .expect("start monitor session");
        assert_eq!(session.sample_rate_hz(), 48_000);
        assert_eq!(session.channels(), 1);

        std::thread::sleep(Duration::from_millis(100));
        drop(session);

        let samples = received.lock().unwrap();
        assert!(
            samples.len() >= 256,
            "monitor sink saw {} samples",
            samples.len()
        );
        assert!(samples.len().is_multiple_of(2), "stereo interleave");
        for frame in samples.chunks_exact(2) {
            assert_eq!(frame[0], frame[1], "mono input duplicates to both channels");
        }
        // The tone actually flowed (not silence).
        assert!(samples.iter().any(|sample| sample.abs() > 0.4));
    }

    #[test]
    fn capture_with_monitor_tees_input_to_both_wav_and_sink() {
        let dir = std::env::temp_dir().join(format!(
            "signal-capture-tee-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let wav_path = dir.join("take.wav");
        let backend = FakeInputBackend::new();
        let received: Arc<std::sync::Mutex<Vec<f32>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink_received = Arc::clone(&received);
        let session = CaptureSession::start_with_monitor(
            &backend,
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 1,
                buffer_frames: Some(128),
            },
            &wav_path,
            0,
            Box::new(move |frames| {
                sink_received.lock().unwrap().extend_from_slice(frames);
            }),
        )
        .expect("start capture with monitor");

        std::thread::sleep(Duration::from_millis(300));
        let report = session.stop().expect("stop capture");

        // The WAV captured normally...
        assert!(report.frames > 4_800, "captured {} frames", report.frames);
        assert_eq!(report.overrun_samples, 0);
        assert!(wav_path.exists());
        // ...and the monitor sink heard the same audio, as stereo.
        let samples = received.lock().unwrap();
        assert!(
            samples.len() as u64 >= report.frames,
            "monitor sink saw {} samples for {} captured frames",
            samples.len(),
            report.frames,
        );
        for frame in samples.chunks_exact(2) {
            assert_eq!(frame[0], frame[1]);
        }
        assert!(samples.iter().any(|sample| sample.abs() > 0.4));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn gated_capture_waits_for_common_activation_but_keeps_monitoring() {
        let dir = std::env::temp_dir().join(format!(
            "signal-capture-gated-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let wav_path = dir.join("take.wav");
        let backend = FakeInputBackend::new();
        let gate = CaptureActivationGate::new();
        let received: Arc<std::sync::Mutex<Vec<f32>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink_received = Arc::clone(&received);
        let session = CaptureSession::start_gated_with_monitor(
            &backend,
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 1,
                buffer_frames: Some(128),
            },
            &wav_path,
            gate.clone(),
            Some(Box::new(move |frames| {
                sink_received.lock().unwrap().extend_from_slice(frames);
            })),
        )
        .expect("start gated capture");

        std::thread::sleep(Duration::from_millis(100));
        assert!(!received.lock().unwrap().is_empty(), "monitor stays live");
        gate.activate();
        std::thread::sleep(Duration::from_millis(150));
        let report = session.stop().expect("stop gated capture");
        assert!(report.frames > 2_400, "captured {} frames", report.frames);
        assert!(report.frames < 12_000, "pre-gate frames were excluded");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn capture_session_skips_initial_frames_before_writing() {
        // Count-in pre-roll: the writer discards the first N frames from the
        // ring, so the WAV begins at the fake tone's frame N — provable
        // because the fake backend synthesizes sample(n) deterministically.
        let dir = std::env::temp_dir().join(format!(
            "signal-capture-skip-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let wav_path = dir.join("take.wav");
        let backend = FakeInputBackend::new();
        const SKIP_FRAMES: u64 = 480;
        let session = CaptureSession::start_with_skip(
            &backend,
            InputStreamSpec {
                sample_rate_hz: 48_000,
                channels: 1,
                buffer_frames: Some(256),
            },
            &wav_path,
            SKIP_FRAMES,
        )
        .expect("start capture with skip");
        std::thread::sleep(Duration::from_millis(500));
        let report = session.stop().expect("stop capture");

        // Report counts written frames only; ~0.5 s captured minus the skip.
        assert!(
            report.frames > 12_000 && report.frames < 48_000,
            "expected ≈23_500 written frames, got {}",
            report.frames
        );
        assert_eq!(report.overrun_samples, 0);

        // Reproduce the fake backend's synthesis up to the skip point: the
        // first written sample must be the tone at frame SKIP_FRAMES.
        let mut reader = hound::WavReader::open(&wav_path).expect("open captured wav");
        let first: f32 = reader
            .samples::<f32>()
            .next()
            .expect("captured at least one sample")
            .expect("read sample");
        let phase_step = std::f32::consts::TAU * FAKE_INPUT_TONE_HZ / 48_000.0;
        let mut phase = 0.0f32;
        for _ in 0..SKIP_FRAMES {
            phase += phase_step;
            if phase >= std::f32::consts::TAU {
                phase -= std::f32::consts::TAU;
            }
        }
        let expected = 0.5 * phase.sin();
        assert!(
            (first - expected).abs() < 1e-4,
            "first written sample {first} vs tone at frame {SKIP_FRAMES} = {expected}",
        );

        std::fs::remove_dir_all(&dir).ok();
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
