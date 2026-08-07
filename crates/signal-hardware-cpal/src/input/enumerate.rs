use cpal::traits::{DeviceTrait, HostTrait};
use signal_hardware::{InputStreamError, InputStreamSpec};

use super::types::{InputChannelDescription, InputDeviceDescription};
use super::CHANNEL_SELECTION_SCRATCH_SAMPLES;

const COMMON_RATES_HZ: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];

/// Name of the host's current default input device, when one exists and
/// reports a name. Hosts compare this against an open stream's
/// [`signal_hardware::InputStreamHandle::device_name`] to detect that the OS default moved.
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
pub(crate) fn negotiate_input_config(
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

pub(crate) fn validate_channel_indices(
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
