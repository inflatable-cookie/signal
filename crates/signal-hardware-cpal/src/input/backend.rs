use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use signal_hardware::{
    InputCaptureFn, InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
};

use super::enumerate::{negotiate_input_config, validate_channel_indices};
use super::stream::{copy_selected_channels, CpalInputStream};
use super::types::CpalInputEndpoint;
use super::{CHANNEL_SELECTION_SCRATCH_SAMPLES, STATE_FAULTED, STATE_RUNNING, STATE_STOPPED};

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
