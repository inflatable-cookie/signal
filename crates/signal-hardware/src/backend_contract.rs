use crate::{BackendHealth, HardwareDiagnosticsSnapshot, SampleRate};

/// Security and isolation tier for a hardware backend.
///
/// Tier0 runs in-process with the host, Tier1 uses a brokered channel, and
/// Tier2 applies strong containment (e.g. a sandbox process).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendPolicyTier {
    /// Backend runs directly inside the host process. Lowest latency.
    Tier0InHost,
    /// Backend communicates through a broker. Moderate isolation.
    Tier1Brokered,
    /// Backend runs under strong containment (e.g. sandboxed process).
    Tier2StrongContainment,
}

/// Linux audio subsystem variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxAudioBackendKind {
    /// ALSA (Advanced Linux Sound Architecture).
    Alsa,
    /// JACK Audio Connection Kit.
    Jack,
    /// PipeWire audio/video routing daemon.
    PipeWire,
}

/// Identifies which hardware backend implementation is in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareBackendIdentity {
    /// Apple CoreAudio (macOS).
    CoreAudio,
    /// Linux audio backend with the specific subsystem variant.
    Linux(LinuxAudioBackendKind),
    /// Platform not supported by any built-in backend.
    Unsupported,
}

/// Direction of audio data flow for a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioStreamDirection {
    /// Audio flows from the device into the host (microphone / line-in).
    Input,
    /// Audio flows from the host to the device (speaker / line-out).
    Output,
    /// Simultaneous input and output on the same device.
    Duplex,
}

/// Sample representation used in an audio stream buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
    /// 32-bit IEEE 754 float, normalized to [-1.0, 1.0].
    F32,
    /// 16-bit signed integer.
    I16,
    /// 32-bit signed integer.
    I32,
}

/// Source driving the audio clock for a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareClockSource {
    /// Device's own internal oscillator.
    Internal,
    /// External word-clock signal (e.g. AES3 or BNC).
    ExternalWordClock,
    /// Clock recovered from a digital audio input.
    DigitalInput,
    /// Software-generated virtual clock (used by simulated backends).
    Virtual,
}

/// How many clock domains are involved in the stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareClockTopology {
    /// Single device, single clock domain.
    SingleEndpoint,
    /// Multiple devices merged into an aggregate device.
    Aggregate,
}

/// Who drives the audio callback loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareLifecycleOwnership {
    /// The host invokes the callback on demand.
    HostDrivenCallback,
    /// The backend schedules and drives its own callback thread.
    BackendManagedCallback,
}

/// Policy governing how the backend recovers after a stream failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareRestartPolicy {
    /// The host is responsible for tearing down and restarting the stream.
    HostMustRestart,
    /// The backend may attempt to restart the stream autonomously.
    BackendMayRestart,
}

/// Describes a single audio device available through a backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDeviceDescriptor {
    /// Which backend reported this device.
    pub backend_identity: HardwareBackendIdentity,
    /// Short stable name of the backend (e.g. `"coreaudio"`).
    pub backend_name: &'static str,
    /// Opaque stable identifier for the device, unique within the backend.
    pub device_id: String,
    /// Human-readable display name.
    pub name: String,
    /// `true` if this is the system default input device.
    pub default_input: bool,
    /// `true` if this is the system default output device.
    pub default_output: bool,
    /// Maximum number of input channels the device can provide.
    pub max_input_channels: u16,
    /// Maximum number of output channels the device can accept.
    pub max_output_channels: u16,
    /// The sample rate the device reports as its preferred rate.
    pub nominal_sample_rate: SampleRate,
    /// Buffer sizes (in frames) the device is known to handle well.
    pub preferred_buffer_sizes: Vec<usize>,
}

/// Caller-supplied parameters for opening an audio stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareStreamRequest {
    /// Target device, identified by [`AudioDeviceDescriptor::device_id`].
    pub device_id: String,
    /// Whether the stream is input, output, or duplex.
    pub direction: AudioStreamDirection,
    /// Desired sample rate.
    pub sample_rate: SampleRate,
    /// Desired buffer size in frames.
    pub buffer_size: usize,
    /// Number of input channels to open (0 for output-only streams).
    pub input_channels: u16,
    /// Number of output channels to open (0 for input-only streams).
    pub output_channels: u16,
    /// Sample format for the buffer data.
    pub sample_format: AudioSampleFormat,
    /// If `true`, channels are interleaved in the buffer.
    pub interleaved: bool,
}

impl HardwareStreamRequest {
    /// Construct a stereo F32 interleaved output-only request for the given device.
    pub fn new_output(device_id: impl Into<String>, sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            device_id: device_id.into(),
            direction: AudioStreamDirection::Output,
            sample_rate: SampleRate(sample_rate),
            buffer_size,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            interleaved: true,
        }
    }

    /// Override the output channel count.
    pub fn with_output_channels(mut self, output_channels: u16) -> Self {
        self.output_channels = output_channels;
        self
    }

    /// Override the input channel count.
    pub fn with_input_channels(mut self, input_channels: u16) -> Self {
        self.input_channels = input_channels;
        self
    }

    /// Override the sample format.
    pub fn with_sample_format(mut self, sample_format: AudioSampleFormat) -> Self {
        self.sample_format = sample_format;
        self
    }

    /// Override whether the buffer layout is interleaved.
    pub fn with_interleaved(mut self, interleaved: bool) -> Self {
        self.interleaved = interleaved;
        self
    }
}

/// Callback ownership and restart policy agreed for a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareLifecycleContract {
    /// Who drives the audio callback loop.
    pub ownership: HardwareLifecycleOwnership,
    /// How the stream is restarted after a fault.
    pub restart_policy: HardwareRestartPolicy,
}

impl Default for HardwareLifecycleContract {
    fn default() -> Self {
        Self {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        }
    }
}

/// Reported latency figures for an open stream, measured in samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareLatencyProfile {
    /// Input latency, if this is an input or duplex stream.
    pub input_latency_samples: Option<u32>,
    /// Output latency from the host callback to the DAC.
    pub output_latency_samples: u32,
    /// Combined input-to-output latency, if known.
    pub round_trip_latency_samples: Option<u32>,
}

impl HardwareLatencyProfile {
    /// Construct a profile with only output latency set.
    pub fn output_only(output_latency_samples: u32) -> Self {
        Self {
            input_latency_samples: None,
            output_latency_samples,
            round_trip_latency_samples: None,
        }
    }
}

/// Negotiated stream configuration returned by [`HardwareBackend::negotiate_stream`].
///
/// Represents everything the host needs to know to drive the audio callback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareStreamConfig {
    /// The device this stream was opened against.
    pub device: AudioDeviceDescriptor,
    /// Actual stream direction.
    pub direction: AudioStreamDirection,
    /// Negotiated sample rate.
    pub sample_rate: SampleRate,
    /// Negotiated buffer size in frames.
    pub buffer_size: usize,
    /// Number of active input channels.
    pub input_channels: u16,
    /// Number of active output channels.
    pub output_channels: u16,
    /// Sample format in use.
    pub sample_format: AudioSampleFormat,
    /// Whether channels are interleaved in the callback buffer.
    pub interleaved: bool,
    /// Clock source driving this stream.
    pub clock_source: HardwareClockSource,
    /// Clock topology for this stream.
    pub clock_topology: HardwareClockTopology,
    /// Lifecycle contract agreed between host and backend.
    pub lifecycle: HardwareLifecycleContract,
    /// Latency reported by the backend.
    pub latency: HardwareLatencyProfile,
    /// `true` for streams produced by a simulated backend.
    pub simulated: bool,
}

/// High-level parameters used to request a hardware configuration, typically
/// derived from a [`HardwareStreamConfig`] that has already been negotiated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareConfigRequest {
    /// Requested sample rate.
    pub sample_rate: SampleRate,
    /// Requested buffer size in frames.
    pub buffer_size: usize,
    /// Policy tier to apply.
    pub backend_policy: BackendPolicyTier,
    /// Number of input channels.
    pub input_channels: u16,
    /// Number of output channels.
    pub output_channels: u16,
    /// Sample format.
    pub sample_format: AudioSampleFormat,
    /// Whether channels are interleaved.
    pub interleaved: bool,
}

impl HardwareConfigRequest {
    /// Construct a default stereo F32 interleaved output-only config request.
    pub fn new(sample_rate: u32, buffer_size: usize, backend_policy: BackendPolicyTier) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            buffer_size,
            backend_policy,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            interleaved: true,
        }
    }

    /// Derive a config request from an already-negotiated stream config.
    pub fn from_stream(stream: &HardwareStreamConfig, backend_policy: BackendPolicyTier) -> Self {
        Self {
            sample_rate: stream.sample_rate,
            buffer_size: stream.buffer_size,
            backend_policy,
            input_channels: stream.input_channels,
            output_channels: stream.output_channels,
            sample_format: stream.sample_format,
            interleaved: stream.interleaved,
        }
    }
}

/// Describes the policy record associated with a specific backend identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyRecord {
    /// Which backend this record applies to.
    pub backend_identity: HardwareBackendIdentity,
    /// Isolation tier for this backend.
    pub tier: BackendPolicyTier,
    /// Whether this backend is used by default when running in-host.
    pub in_host_default: bool,
}

/// Reason category for a stream negotiation failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareNegotiationErrorKind {
    /// The requested device was not found or is unavailable.
    DeviceUnavailable,
    /// The requested configuration (channels, rate, format) is not supported.
    UnsupportedConfiguration,
}

/// Error returned when a stream cannot be negotiated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareNegotiationError {
    /// Category of the failure.
    pub kind: HardwareNegotiationErrorKind,
    /// Human-readable explanation.
    pub message: String,
}

impl HardwareNegotiationError {
    /// Construct an error with the given kind and message.
    pub fn new(kind: HardwareNegotiationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Core abstraction for an audio hardware backend.
///
/// Implementors expose device enumeration, stream negotiation, and diagnostic
/// health reporting. The host drives the audio callback; the backend provides
/// the configuration contract.
pub trait HardwareBackend {
    /// Returns the identity enum for this backend.
    fn backend_identity(&self) -> HardwareBackendIdentity;

    /// Returns the short stable name for this backend (e.g. `"coreaudio"`).
    fn backend_name(&self) -> &'static str;

    /// Returns the policy record describing the isolation tier for this backend.
    fn policy_record(&self) -> BackendPolicyRecord;

    /// Returns the current health state of the backend.
    fn health(&self) -> BackendHealth;

    /// Returns all audio devices currently available through this backend.
    fn enumerate_devices(&self) -> Vec<AudioDeviceDescriptor>;

    /// Returns the default output device, if one is available.
    fn default_output_device(&self) -> Option<AudioDeviceDescriptor>;

    /// Negotiate a concrete stream configuration from the given request.
    ///
    /// Returns the agreed [`HardwareStreamConfig`] on success, or a
    /// [`HardwareNegotiationError`] if the device is unavailable or the
    /// requested parameters are not supported.
    fn negotiate_stream(
        &self,
        request: &HardwareStreamRequest,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError>;

    /// Returns a snapshot of current diagnostic counters and the last event.
    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot;
}
