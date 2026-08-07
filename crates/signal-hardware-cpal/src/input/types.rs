/// A machine-local cpal input endpoint.
///
/// `device_id` is the opaque selector returned by [`super::enumerate_input_devices`].
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
