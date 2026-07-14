//! cpal-backed implementation of Signal's input-stream contract, plus real
//! input-device enumeration. Exact mirror of the output side in `lib.rs`:
//! same negotiation semantics, same dedicated owner-thread pattern, same
//! latency/error/device-name capture — direction flipped.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use signal_hardware::{
    InputCaptureFn, InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
    InputStreamState,
};

const STATE_RUNNING: u8 = 0;
const STATE_STOPPED: u8 = 1;
const STATE_FAULTED: u8 = 2;
const CHANNEL_SELECTION_SCRATCH_SAMPLES: usize = 2048;

/// A machine-local cpal input endpoint.
///
/// `device_id` is the opaque selector returned by [`enumerate_input_devices`].
/// Channel indices are zero-based physical input channels. An empty list uses
/// the stream's negotiated channel layout unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpalInputEndpoint {
    /// Opaque machine-local device selector.
    pub device_id: String,
    /// Zero-based physical input channels to expose to the capture callback.
    pub channel_indices: Vec<u16>,
}

impl CpalInputEndpoint {
    /// Build an endpoint from an enumerated device id and physical channels.
    pub fn new(device_id: impl Into<String>, channel_indices: Vec<u16>) -> Self {
        Self {
            device_id: device_id.into(),
            channel_indices,
        }
    }
}

/// Input backend over the host's default cpal device or one explicit endpoint.
#[derive(Debug, Default)]
pub struct CpalInputBackend {
    endpoint: Option<CpalInputEndpoint>,
}

impl CpalInputBackend {
    /// Construct a backend over the default cpal host.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a backend that opens one enumerated device and exposes the
    /// requested physical channels. Missing devices and invalid channel
    /// layouts fail visibly; this constructor never opens or starts a stream.
    pub fn with_endpoint(endpoint: CpalInputEndpoint) -> Self {
        Self {
            endpoint: Some(endpoint),
        }
    }
}

/// A real input device as enumerated by cpal.
#[derive(Debug, Clone, PartialEq)]
pub struct InputDeviceDescription {
    /// Opaque machine-local selector accepted by [`CpalInputEndpoint`].
    pub device_id: String,
    /// Device name as reported by the OS.
    pub name: String,
    /// Whether this is the host's current default input device.
    pub is_default: bool,
    /// Sample rate of the device's default input config.
    pub default_sample_rate_hz: u32,
    /// Channel count of the device's default input config.
    pub default_channels: u16,
    /// Distinct sample rates supported across the device's input configs
    /// (supported ranges sampled at common audio rates), ascending.
    pub supported_sample_rates_hz: Vec<u32>,
    /// Maximum input channel count across supported configs.
    pub max_channels: u16,
    /// Stable zero-based channel choices for the largest reported layout.
    pub channels: Vec<InputChannelDescription>,
}

/// One selectable physical input channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputChannelDescription {
    /// Zero-based physical channel index used by route bindings.
    pub index: u16,
    /// Human-readable fallback label. Backends may replace this with a native
    /// label when the host API exposes one.
    pub label: String,
}

const COMMON_RATES_HZ: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];

/// Name of the host's current default input device, when one exists and
/// reports a name. Hosts compare this against an open stream's
/// [`InputStreamHandle::device_name`] to detect that the OS default moved.
pub fn default_input_device_name() -> Option<String> {
    cpal::default_host()
        .default_input_device()
        .and_then(|device| device.name().ok())
}

/// Enumerate real input devices with their supported configurations.
pub fn enumerate_input_devices() -> Result<Vec<InputDeviceDescription>, InputStreamError> {
    let host = cpal::default_host();
    let default_name = default_input_device_name();
    let devices = host
        .input_devices()
        .map_err(|error| InputStreamError::new(format!("enumerate input devices: {error}")))?;

    let mut descriptions = Vec::new();
    for device in devices {
        let Ok(name) = device.name() else { continue };
        let Ok(default_config) = device.default_input_config() else {
            continue;
        };
        let mut rates = Vec::new();
        let mut max_channels = default_config.channels();
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                max_channels = max_channels.max(config.channels());
                for rate in COMMON_RATES_HZ {
                    if rate >= config.min_sample_rate().0
                        && rate <= config.max_sample_rate().0
                        && !rates.contains(&rate)
                    {
                        rates.push(rate);
                    }
                }
            }
        }
        rates.sort_unstable();
        let channels = (0..max_channels)
            .map(|index| InputChannelDescription {
                index,
                label: format!("Input {}", index + 1),
            })
            .collect();
        descriptions.push(InputDeviceDescription {
            device_id: name.clone(),
            is_default: default_name.as_deref() == Some(name.as_str()),
            default_sample_rate_hz: default_config.sample_rate().0,
            default_channels: default_config.channels(),
            supported_sample_rates_hz: rates,
            max_channels,
            channels,
            name,
        });
    }
    Ok(descriptions)
}

/// Pick the best supported input config for `spec` on `device`.
///
/// Preference order: exact channels with the exact rate, then exact channels
/// with the nearest supported rate, then the device default config. The
/// result is what the stream actually runs at.
fn negotiate_input_config(
    device: &cpal::Device,
    spec: &InputStreamSpec,
    minimum_channels: Option<u16>,
) -> Result<cpal::StreamConfig, InputStreamError> {
    let default_config: cpal::StreamConfig = device
        .default_input_config()
        .map_err(|error| InputStreamError::new(format!("default input config: {error}")))?
        .into();

    let mut best: Option<((u16, u32), cpal::StreamConfig)> = None;
    if let Ok(configs) = device.supported_input_configs() {
        for config in configs {
            let channel_distance = match minimum_channels {
                Some(minimum) if config.channels() >= minimum => config.channels() - minimum,
                Some(_) => continue,
                None if config.channels() == spec.channels => 0,
                None => continue,
            };
            let rate = spec
                .sample_rate_hz
                .clamp(config.min_sample_rate().0, config.max_sample_rate().0);
            let distance = rate.abs_diff(spec.sample_rate_hz);
            let candidate = cpal::StreamConfig {
                channels: config.channels(),
                sample_rate: cpal::SampleRate(rate),
                buffer_size: cpal::BufferSize::Default,
            };
            let score = (channel_distance, distance);
            if best.as_ref().map(|(best, _)| score < *best).unwrap_or(true) {
                best = Some((score, candidate));
            }
        }
    }

    let mut config = best.map(|(_, config)| config).unwrap_or(default_config);
    if let Some(buffer_frames) = spec.buffer_frames {
        config.buffer_size = cpal::BufferSize::Fixed(buffer_frames);
    }
    Ok(config)
}

fn validate_channel_indices(
    channel_indices: &[u16],
    physical_channels: u16,
) -> Result<(), InputStreamError> {
    if channel_indices.is_empty() {
        return Ok(());
    }
    if channel_indices.len() > CHANNEL_SELECTION_SCRATCH_SAMPLES {
        return Err(InputStreamError::new("too many selected input channels"));
    }
    for (position, index) in channel_indices.iter().enumerate() {
        if *index >= physical_channels {
            return Err(InputStreamError::new(format!(
                "input channel index {index} is unavailable on a {physical_channels}-channel device"
            )));
        }
        if channel_indices[..position].contains(index) {
            return Err(InputStreamError::new(format!(
                "input channel index {index} is selected more than once"
            )));
        }
    }
    Ok(())
}

fn copy_selected_channels(
    frames: &[f32],
    physical_channels: usize,
    channel_indices: &[u16],
    output: &mut [f32],
) -> usize {
    let mut written = 0;
    for frame in frames
        .chunks_exact(physical_channels)
        .take(output.len() / channel_indices.len())
    {
        for index in channel_indices {
            output[written] = frame[*index as usize];
            written += 1;
        }
    }
    written
}

/// Handle over a stream owned by its dedicated thread. Dropping signals the
/// owner thread, which drops the stream on the thread that built it.
struct CpalInputStream {
    state: Arc<AtomicU8>,
    /// Display detail of the most recent cpal stream error. cpal delivers
    /// stream errors on a backend worker thread — NOT the audio callback —
    /// so formatting the string (allocation) and taking this mutex there is
    /// acceptable; the audio path never touches it.
    last_error: Arc<Mutex<Option<String>>>,
    /// Most recent ADC→callback latency in µs, stored by the data callback
    /// from cpal's `InputCallbackInfo` timestamps. 0 = not yet observed.
    latency_micros: Arc<AtomicU64>,
    sample_rate_hz: u32,
    channels: u16,
    device_name: Option<String>,
    stop: Option<mpsc::Sender<()>>,
    owner: Option<std::thread::JoinHandle<()>>,
}

impl InputStreamHandle for CpalInputStream {
    fn state(&self) -> InputStreamState {
        match self.state.load(Ordering::Relaxed) {
            STATE_RUNNING => InputStreamState::Running,
            STATE_STOPPED => InputStreamState::Stopped,
            _ => InputStreamState::Faulted,
        }
    }

    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .ok()
            .and_then(|detail| detail.clone())
    }

    fn input_latency_micros(&self) -> Option<u64> {
        match self.latency_micros.load(Ordering::Relaxed) {
            0 => None,
            micros => Some(micros),
        }
    }

    fn device_name(&self) -> Option<String> {
        self.device_name.clone()
    }
}

impl Drop for CpalInputStream {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
        if self.state.load(Ordering::Relaxed) == STATE_RUNNING {
            self.state.store(STATE_STOPPED, Ordering::Relaxed);
        }
    }
}

impl InputStreamBackend for CpalInputBackend {
    fn open_input_stream(
        &self,
        spec: InputStreamSpec,
        mut capture: InputCaptureFn,
    ) -> Result<Box<dyn InputStreamHandle>, InputStreamError> {
        let endpoint = self.endpoint.clone();
        let state = Arc::new(AtomicU8::new(STATE_RUNNING));
        let thread_state = Arc::clone(&state);
        let last_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let thread_last_error = Arc::clone(&last_error);
        let latency_micros = Arc::new(AtomicU64::new(0));
        let thread_latency = Arc::clone(&latency_micros);
        type Negotiated = (u32, u16, Option<String>);
        let (ready_tx, ready_rx) = mpsc::channel::<Result<Negotiated, InputStreamError>>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // The stream lives and dies on this thread; cpal::Stream is not Send
        // on every platform, so it never crosses a thread boundary.
        let owner = std::thread::Builder::new()
            .name("signal-input-stream".to_string())
            .spawn(move || {
                let open = (|| {
                    let host = cpal::default_host();
                    let device = match endpoint.as_ref() {
                        Some(endpoint) => host
                            .input_devices()
                            .map_err(|error| {
                                InputStreamError::new(format!("enumerate input devices: {error}"))
                            })?
                            .find(|device| {
                                device.name().ok().as_deref() == Some(endpoint.device_id.as_str())
                            })
                            .ok_or_else(|| {
                                InputStreamError::new(format!(
                                    "input device not found: {}",
                                    endpoint.device_id
                                ))
                            })?,
                        None => host
                            .default_input_device()
                            .ok_or_else(|| InputStreamError::new("no default input device"))?,
                    };
                    let device_name = device.name().ok();
                    let selected_channels = endpoint
                        .as_ref()
                        .map(|endpoint| endpoint.channel_indices.as_slice())
                        .unwrap_or_default();
                    let minimum_channels = selected_channels
                        .iter()
                        .copied()
                        .max()
                        .map(|index| index + 1);
                    let config = negotiate_input_config(&device, &spec, minimum_channels)?;
                    validate_channel_indices(selected_channels, config.channels)?;
                    let reported_channels = if selected_channels.is_empty() {
                        config.channels
                    } else {
                        selected_channels.len() as u16
                    };
                    let physical_channels = config.channels as usize;
                    let selected_channels = selected_channels.to_vec();
                    let error_state = Arc::clone(&thread_state);
                    let error_detail = Arc::clone(&thread_last_error);
                    let callback_latency = Arc::clone(&thread_latency);
                    let stream = device
                        .build_input_stream(
                            &config,
                            move |frames: &[f32], info: &cpal::InputCallbackInfo| {
                                // Input latency = when the callback ran minus
                                // when the first frame in this buffer was
                                // captured at the ADC. RT-safe: StreamInstant
                                // subtraction plus one atomic store, no
                                // allocation. Early callbacks may not have a
                                // usable pair (`None`): keep the last
                                // observation (0 = never observed).
                                let timestamp = info.timestamp();
                                if let Some(latency) =
                                    timestamp.callback.duration_since(&timestamp.capture)
                                {
                                    callback_latency
                                        .store(latency.as_micros() as u64, Ordering::Relaxed);
                                }
                                if selected_channels.is_empty() {
                                    capture(frames);
                                } else {
                                    // Fixed stack scratch keeps arbitrary
                                    // channel extraction allocation-free on
                                    // the audio callback.
                                    let selected_count = selected_channels.len();
                                    let mut scratch = [0.0_f32; CHANNEL_SELECTION_SCRATCH_SAMPLES];
                                    let frames_per_chunk = scratch.len() / selected_count;
                                    let physical_samples_per_chunk =
                                        frames_per_chunk * physical_channels;
                                    for physical_chunk in frames.chunks(physical_samples_per_chunk)
                                    {
                                        let written = copy_selected_channels(
                                            physical_chunk,
                                            physical_channels,
                                            &selected_channels,
                                            &mut scratch,
                                        );
                                        if written > 0 {
                                            capture(&scratch[..written]);
                                        }
                                    }
                                }
                            },
                            move |error| {
                                // Stream errors arrive on a cpal backend
                                // worker thread, NOT the audio callback, so
                                // allocating the Display string and taking
                                // the mutex here is safe; capture the detail
                                // instead of discarding it.
                                if let Ok(mut detail) = error_detail.lock() {
                                    *detail = Some(error.to_string());
                                }
                                error_state.store(STATE_FAULTED, Ordering::Relaxed);
                            },
                            None,
                        )
                        .map_err(|error| {
                            InputStreamError::new(format!("build input stream: {error}"))
                        })?;
                    stream.play().map_err(|error| {
                        InputStreamError::new(format!("start input stream: {error}"))
                    })?;
                    Ok((
                        stream,
                        (config.sample_rate.0, reported_channels, device_name),
                    ))
                })();

                match open {
                    Ok((stream, negotiated)) => {
                        if ready_tx.send(Ok(negotiated)).is_err() {
                            return;
                        }
                        // Park until the handle drops (sender disconnects).
                        let _ = stop_rx.recv();
                        drop(stream);
                        if thread_state.load(Ordering::Relaxed) == STATE_RUNNING {
                            thread_state.store(STATE_STOPPED, Ordering::Relaxed);
                        }
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            })
            .map_err(|error| InputStreamError::new(format!("spawn stream thread: {error}")))?;

        let negotiated = ready_rx
            .recv()
            .map_err(|_| InputStreamError::new("stream thread exited before reporting"))?;
        match negotiated {
            Ok((sample_rate_hz, channels, device_name)) => Ok(Box::new(CpalInputStream {
                state,
                last_error,
                latency_micros,
                sample_rate_hz,
                channels,
                device_name,
                stop: Some(stop_tx),
                owner: Some(owner),
            })),
            Err(error) => {
                let _ = owner.join();
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::traits::HostTrait;
    use std::sync::atomic::AtomicU64 as TestAtomicU64;

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
}
