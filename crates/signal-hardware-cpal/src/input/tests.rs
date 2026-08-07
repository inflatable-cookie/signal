use std::sync::atomic::AtomicU64 as TestAtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use cpal::traits::HostTrait;
use signal_hardware::{InputStreamBackend, InputStreamSpec, InputStreamState};

use super::enumerate::validate_channel_indices;
use super::stream::copy_selected_channels;
use super::{enumerate_input_devices, CpalInputBackend, CpalInputEndpoint};

/// Smoke test against real hardware; skips quietly when no input device
/// is available (CI, or a mac with mic permission denied — cpal then
/// fails to open, which we also treat as a skip rather than a failure
/// since it is an environment property, not a code defect).
#[test]
fn opens_negotiates_and_reports_honestly() {
    if cpal::default_host().default_input_device().is_none() {
        eprintln!("no input device; skipping");
        return;
    }
    let backend = CpalInputBackend::new();
    let captured = Arc::new(TestAtomicU64::new(0));
    let callback_captured = Arc::clone(&captured);
    let stream = match backend.open_input_stream(
        InputStreamSpec {
            sample_rate_hz: 48_000,
            channels: 1,
            buffer_frames: Some(256),
        },
        Box::new(move |frames| {
            callback_captured.fetch_add(frames.len() as u64, Ordering::Relaxed);
        }),
    ) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("cannot open input stream ({error}); skipping");
            return;
        }
    };
    assert_eq!(stream.state(), InputStreamState::Running);
    // Negotiated values are real device properties, never echoes.
    assert!(stream.sample_rate_hz() > 0);
    assert!(stream.channels() > 0);
    assert!(stream.device_name().is_some(), "device name recorded");
    // Input latency comes from real callback timestamps; give the
    // stream a few hundred ms of callbacks to observe one.
    let mut measured = None;
    for _ in 0..50 {
        measured = stream.input_latency_micros();
        if measured.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if let Some(latency) = measured {
        assert!(latency > 0);
        eprintln!(
            "measured input latency: {latency} us on {:?}",
            stream.device_name()
        );
    } else {
        eprintln!("no usable input timestamps observed; latency stays None");
    }
    assert!(
        captured.load(Ordering::Relaxed) > 0,
        "capture callback delivered samples"
    );
    drop(stream);
}

#[test]
fn enumerates_input_devices_when_present() {
    let devices = enumerate_input_devices().expect("enumerate");
    if devices.is_empty() {
        eprintln!("no input devices; skipping");
        return;
    }
    assert!(devices.iter().any(|device| device.is_default));
    for device in &devices {
        assert_eq!(device.device_id, device.name);
        assert!(device.default_sample_rate_hz > 0);
        assert!(device.max_channels > 0);
        assert_eq!(device.channels.len(), device.max_channels as usize);
        assert_eq!(device.channels[0].index, 0);
    }
}

#[test]
fn explicit_missing_input_device_is_a_typed_error() {
    let backend = CpalInputBackend::with_endpoint(CpalInputEndpoint::new(
        "signal-test-no-such-input-device-7f3a",
        vec![0],
    ));
    let result = backend.open_input_stream(
        InputStreamSpec {
            sample_rate_hz: 48_000,
            channels: 1,
            buffer_frames: None,
        },
        Box::new(|_| {}),
    );
    let error = match result {
        Ok(_) => panic!("missing device must not open"),
        Err(error) => error,
    };
    assert!(error.message.contains("input device not found"));
}

#[test]
fn channel_selection_rejects_duplicates_and_out_of_range_indices() {
    assert!(validate_channel_indices(&[0, 1], 2).is_ok());
    assert!(validate_channel_indices(&[0, 0], 2)
        .unwrap_err()
        .message
        .contains("more than once"));
    assert!(validate_channel_indices(&[2], 2)
        .unwrap_err()
        .message
        .contains("unavailable"));
}

#[test]
fn selected_channels_are_extracted_in_requested_order() {
    let frames = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let mut selected = [0.0; 4];
    let written = copy_selected_channels(&frames, 3, &[2, 0], &mut selected);
    assert_eq!(written, 4);
    assert_eq!(selected, [3.0, 1.0, 6.0, 4.0]);
}
