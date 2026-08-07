use super::*;
use std::sync::Arc;
use std::time::Duration;

use crate::fake_input::{FakeInputBackend, FAKE_INPUT_TONE_HZ};
use crate::input_stream::{
    InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec, InputStreamState,
};

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
    // Liveness floor: 480 frames is 10 ms, well under any plausible host
    // slowdown, and the tee content assertions below carry the real claim.
    assert!(report.frames > 480, "captured {} frames", report.frames);
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
    // Liveness floor; see the note on the skip test above.
    assert!(report.frames > 480, "captured {} frames", report.frames);
    assert!(report.frames < 12_000, "pre-gate frames were excluded");

    std::fs::remove_dir_all(&dir).ok();
}

/// Test-only backend whose callback the TEST drives synchronously, so
/// gate/skip interleavings are frame-deterministic (no clocked thread).
struct ManualInputBackend {
    capture: Arc<std::sync::Mutex<Option<crate::input_stream::InputCaptureFn>>>,
}

struct ManualInputStream {
    sample_rate_hz: u32,
    channels: u16,
}

impl InputStreamHandle for ManualInputStream {
    fn state(&self) -> InputStreamState {
        InputStreamState::Running
    }
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }
    fn channels(&self) -> u16 {
        self.channels
    }
}

impl InputStreamBackend for ManualInputBackend {
    fn open_input_stream(
        &self,
        spec: InputStreamSpec,
        capture: crate::input_stream::InputCaptureFn,
    ) -> Result<Box<dyn InputStreamHandle>, InputStreamError> {
        *self.capture.lock().unwrap() = Some(capture);
        Ok(Box::new(ManualInputStream {
            sample_rate_hz: spec.sample_rate_hz,
            channels: spec.channels,
        }))
    }
}

#[test]
fn gate_released_inside_the_skip_window_leaks_no_pre_roll_frames() {
    // Composition ordering proof: samples encode their absolute stream
    // frame index, the gate opens mid-stream, and the skip must count
    // against POST-gate frames only — the first written sample is the
    // frame at (gate-open index + skip), never a pre-gate frame and
    // never (gate-open index) alone.
    let dir = std::env::temp_dir().join(format!(
        "signal-capture-gate-skip-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let wav_path = dir.join("take.wav");
    let capture: Arc<std::sync::Mutex<Option<crate::input_stream::InputCaptureFn>>> =
        Arc::new(std::sync::Mutex::new(None));
    let backend = ManualInputBackend {
        capture: Arc::clone(&capture),
    };
    let gate = CaptureActivationGate::new();
    const SKIP_FRAMES: u64 = 480;
    let session = CaptureSession::start_gated_with_monitor_and_skip(
        &backend,
        InputStreamSpec {
            sample_rate_hz: 48_000,
            channels: 1,
            buffer_frames: Some(256),
        },
        &wav_path,
        SKIP_FRAMES,
        gate.clone(),
        None,
    )
    .expect("start gated capture with skip");

    let mut callback = capture.lock().unwrap().take().expect("callback installed");
    let feed = |callback: &mut crate::input_stream::InputCaptureFn, range: std::ops::Range<u64>| {
        let frames: Vec<f32> = range.map(|index| index as f32).collect();
        callback(&frames);
    };
    // Pre-roll: 960 frames while the gate is closed — the WAV ring
    // receives nothing (the skip counter must NOT consume these).
    feed(&mut callback, 0..960);
    // Gate opens INSIDE what would be the skip window, then more frames
    // arrive than the skip discards.
    gate.activate();
    feed(&mut callback, 960..2_400);
    drop(callback);

    let report = session.stop().expect("stop capture");
    // Post-gate frames: 1_440; minus the 480-frame skip = 960 written.
    assert_eq!(report.frames, 960, "skip consumed post-gate frames only");
    let mut reader = hound::WavReader::open(&wav_path).expect("open captured wav");
    let samples: Vec<f32> = reader
        .samples::<f32>()
        .map(|sample| sample.expect("read sample"))
        .collect();
    assert_eq!(samples.len(), 960);
    // First written frame = gate-open index (960) + skip (480) = 1_440:
    // the skip applied AFTER the gate, so no pre-roll frame leaked and
    // the count-in discard was not satisfied by gated-out frames.
    assert_eq!(samples[0], 1_440.0);
    assert_eq!(*samples.last().unwrap(), 2_399.0);

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
    // Liveness floor, not a throughput floor. The previous bound required
    // roughly half of real-time over the sleep, which is a claim about how
    // fast the host is: CI measured 41% and failed. What this test proves
    // lives in the content assertions below; the floor only needs to show
    // that writing happened, and the ceiling still catches over-capture.
    assert!(
        report.frames > SKIP_FRAMES && report.frames < 48_000,
        "expected writing past the skip and under 1s of audio, got {}",
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
    // Liveness floor, not a throughput floor. The previous bound required
    // roughly half of real-time over the sleep, which is a claim about how
    // fast the host is: CI measured 41% and failed. What this test proves
    // lives in the content assertions below; the floor only needs to show
    // that writing happened, and the ceiling still catches over-capture.
    // 4_800 frames is 100 ms of audio, enough for the RMS and
    // zero-crossing statistics below to be meaningful.
    assert!(
        report.frames > 4_800 && report.frames < 96_000,
        "expected ≥100ms and <2s of captured audio, got {}",
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
