//! CoreAudio hardware backend for macOS. Implements [`HardwareBackend`] from `signal-hardware`.
//!
//! On initialization [`CoreAudioBackend`] discovers the current device inventory
//! by invoking `system_profiler SPAudioDataType -json`. In test and CI
//! environments you can inject a fixture by setting the
//! `SIGNAL_COREAUDIO_SPAUDIO_JSON` environment variable to a JSON string in the
//! same format that `system_profiler` produces.

#![warn(missing_docs)]

use serde_json::Value;
use signal_hardware::{
    AudioDeviceDescriptor, BackendHealth, BackendPolicyRecord, BackendPolicyTier, HardwareBackend,
    HardwareBackendIdentity, HardwareClockSource, HardwareClockTopology, HardwareConfigRequest,
    HardwareDiagnosticEvent, HardwareDiagnosticKind, HardwareDiagnosticSeverity,
    HardwareDiagnosticsSnapshot, HardwareLatencyProfile, HardwareLifecycleContract,
    HardwareLifecycleOwnership, HardwareNegotiationError, HardwareNegotiationErrorKind,
    HardwareRestartPolicy, HardwareStreamConfig, HardwareStreamRequest, SampleRate,
};
use std::{collections::HashMap, env, process::Command};

const COREAUDIO_FIXTURE_ENV: &str = "SIGNAL_COREAUDIO_SPAUDIO_JSON";
const COREAUDIO_SYSTEM_PROFILER_DATASET: &str = "SPAudioDataType";

/// CoreAudio hardware backend for macOS.
///
/// Implements [`HardwareBackend`] by querying `system_profiler` for the current
/// device inventory. The backend is immutable after construction; call
/// [`Default::default`] or rebuild to pick up device changes.
///
/// The diagnostic mutation helpers (`simulate_device_loss`, etc.) are provided
/// for testing only — they record fault events and update health state without
/// touching real hardware.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreAudioBackend {
    policy_tier: BackendPolicyTier,
    devices: Vec<AudioDeviceDescriptor>,
    diagnostics: HardwareDiagnosticsSnapshot,
}

impl Default for CoreAudioBackend {
    fn default() -> Self {
        let inventory = discover_inventory();
        Self {
            policy_tier: BackendPolicyTier::Tier0InHost,
            devices: inventory.devices,
            diagnostics: inventory.diagnostics,
        }
    }
}

impl CoreAudioBackend {
    /// Returns the isolation tier this backend is operating under.
    pub fn policy_tier(&self) -> BackendPolicyTier {
        self.policy_tier
    }

    /// Negotiate and return a stream config for the default output device.
    ///
    /// Fails with [`HardwareNegotiationError`] if no default output device is
    /// available or the requested parameters are not supported.
    pub fn default_output_stream(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let device = self.default_output_device().ok_or_else(|| {
            HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::DeviceUnavailable,
                "no CoreAudio default output device is currently available",
            )
        })?;
        let request =
            HardwareStreamRequest::new_output(device.device_id.clone(), sample_rate, buffer_size);
        self.negotiate_stream(&request)
    }

    /// Derive a [`HardwareConfigRequest`] from the default output device at the
    /// given sample rate and buffer size. Panics if no default output device is
    /// available.
    pub fn default_output_request(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> HardwareConfigRequest {
        let stream = self
            .default_output_stream(sample_rate, buffer_size)
            .expect("default CoreAudio output stream should be negotiable");
        HardwareConfigRequest::from_stream(&stream, self.policy_tier)
    }

    /// Reset the diagnostic snapshot to a clean baseline derived from the
    /// current device list.
    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = baseline_diagnostics(&self.devices);
    }

    /// Transition health back to `Healthy` (or `Degraded` if no default output
    /// device is present) after a recovery sequence.
    pub fn mark_recovered(&mut self) {
        self.diagnostics.health = if self.default_output_device().is_some() {
            BackendHealth::Healthy
        } else {
            BackendHealth::Degraded
        };
    }

    /// Inject a simulated device-loss event for testing. Records a
    /// [`HardwareDiagnosticKind::DeviceDisconnected`] event and sets health to
    /// `Degraded`.
    pub fn simulate_device_loss(&mut self, detail: impl Into<String>) {
        let device_id = self.default_output_device().map(|device| device.device_id);
        self.diagnostics.health = BackendHealth::Degraded;
        self.diagnostics.device_loss_count = self.diagnostics.device_loss_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::DeviceDisconnected,
            severity: HardwareDiagnosticSeverity::Critical,
            device_id,
            callback_index: None,
            detail: detail.into(),
        });
    }

    /// Inject a simulated restart-attempt event for testing. Records a
    /// [`HardwareDiagnosticKind::RestartAttempted`] event and sets health to
    /// `Recovering`.
    pub fn simulate_restart_attempt(&mut self, detail: impl Into<String>) {
        let device_id = self.default_output_device().map(|device| device.device_id);
        self.diagnostics.health = BackendHealth::Recovering;
        self.diagnostics.restart_attempt_count =
            self.diagnostics.restart_attempt_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::RestartAttempted,
            severity: HardwareDiagnosticSeverity::Info,
            device_id,
            callback_index: None,
            detail: detail.into(),
        });
    }

    /// Inject a simulated restart-failure event for testing. Records a
    /// [`HardwareDiagnosticKind::RestartFailed`] event and sets health to
    /// `Degraded`.
    pub fn simulate_restart_failure(&mut self, detail: impl Into<String>) {
        let device_id = self.default_output_device().map(|device| device.device_id);
        self.diagnostics.health = BackendHealth::Degraded;
        self.diagnostics.restart_failure_count =
            self.diagnostics.restart_failure_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::RestartFailed,
            severity: HardwareDiagnosticSeverity::Critical,
            device_id,
            callback_index: None,
            detail: detail.into(),
        });
    }
}

impl HardwareBackend for CoreAudioBackend {
    fn backend_identity(&self) -> HardwareBackendIdentity {
        HardwareBackendIdentity::CoreAudio
    }

    fn backend_name(&self) -> &'static str {
        "coreaudio"
    }

    fn policy_record(&self) -> BackendPolicyRecord {
        BackendPolicyRecord {
            backend_identity: self.backend_identity(),
            tier: self.policy_tier,
            in_host_default: true,
        }
    }

    fn health(&self) -> BackendHealth {
        self.diagnostics.health
    }

    fn enumerate_devices(&self) -> Vec<AudioDeviceDescriptor> {
        self.devices.clone()
    }

    fn default_output_device(&self) -> Option<AudioDeviceDescriptor> {
        self.devices
            .iter()
            .find(|device| device.default_output)
            .cloned()
            .or_else(|| {
                self.devices
                    .iter()
                    .find(|device| device.max_output_channels > 0)
                    .cloned()
            })
    }

    fn negotiate_stream(
        &self,
        request: &HardwareStreamRequest,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let device = self
            .devices
            .iter()
            .find(|device| device.device_id == request.device_id)
            .cloned()
            .ok_or_else(|| {
                HardwareNegotiationError::new(
                    HardwareNegotiationErrorKind::DeviceUnavailable,
                    format!("unknown CoreAudio device {}", request.device_id),
                )
            })?;

        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                "sample_rate and buffer_size must be non-zero",
            ));
        }
        if request.output_channels == 0 || request.output_channels > device.max_output_channels {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} output channels exceeds CoreAudio device capacity",
                    request.output_channels
                ),
            ));
        }
        if request.input_channels > device.max_input_channels {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} input channels exceeds CoreAudio device capacity",
                    request.input_channels
                ),
            ));
        }

        Ok(HardwareStreamConfig {
            device,
            direction: request.direction,
            sample_rate: request.sample_rate,
            buffer_size: request.buffer_size,
            input_channels: request.input_channels,
            output_channels: request.output_channels,
            sample_format: request.sample_format,
            interleaved: request.interleaved,
            clock_source: HardwareClockSource::Internal,
            clock_topology: HardwareClockTopology::SingleEndpoint,
            lifecycle: HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::HostMustRestart,
            },
            latency: HardwareLatencyProfile::output_only(request.buffer_size as u32),
            simulated: false,
        })
    }

    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot {
        self.diagnostics.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CoreAudioInventory {
    devices: Vec<AudioDeviceDescriptor>,
    diagnostics: HardwareDiagnosticsSnapshot,
}

fn discover_inventory() -> CoreAudioInventory {
    match read_inventory_json().and_then(|json| parse_inventory(&json)) {
        Ok(devices) => {
            let diagnostics = baseline_diagnostics(&devices);
            CoreAudioInventory {
                devices,
                diagnostics,
            }
        }
        Err(error) => CoreAudioInventory {
            devices: Vec::new(),
            diagnostics: HardwareDiagnosticsSnapshot {
                health: BackendHealth::Degraded,
                xrun_count: 0,
                callback_overrun_count: 0,
                device_loss_count: 0,
                restart_attempt_count: 0,
                restart_failure_count: 0,
                last_event: Some(HardwareDiagnosticEvent {
                    kind: HardwareDiagnosticKind::DeviceDisconnected,
                    severity: HardwareDiagnosticSeverity::Critical,
                    device_id: None,
                    callback_index: None,
                    detail: format!("CoreAudio inventory unavailable: {error}"),
                }),
            },
        },
    }
}

fn baseline_diagnostics(devices: &[AudioDeviceDescriptor]) -> HardwareDiagnosticsSnapshot {
    if devices.iter().any(|device| device.default_output) {
        HardwareDiagnosticsSnapshot::healthy()
    } else {
        HardwareDiagnosticsSnapshot {
            health: BackendHealth::Degraded,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            last_event: Some(HardwareDiagnosticEvent {
                kind: HardwareDiagnosticKind::DeviceDisconnected,
                severity: HardwareDiagnosticSeverity::Warning,
                device_id: None,
                callback_index: None,
                detail: "CoreAudio reported no default output device".into(),
            }),
        }
    }
}

fn read_inventory_json() -> Result<String, String> {
    if let Ok(fixture) = env::var(COREAUDIO_FIXTURE_ENV) {
        return Ok(fixture);
    }

    let output = Command::new("system_profiler")
        .args([COREAUDIO_SYSTEM_PROFILER_DATASET, "-json"])
        .output()
        .map_err(|error| format!("failed to launch system_profiler: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "system_profiler exited with status {}",
            output.status
        ));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("invalid UTF-8 from system_profiler: {error}"))
}

fn parse_inventory(json: &str) -> Result<Vec<AudioDeviceDescriptor>, String> {
    let root: Value =
        serde_json::from_str(json).map_err(|error| format!("invalid CoreAudio JSON: {error}"))?;
    let items = root
        .get(COREAUDIO_SYSTEM_PROFILER_DATASET)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|entry| entry.get("_items").and_then(Value::as_array))
        .ok_or_else(|| "missing SPAudioDataType._items".to_string())?;

    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut devices = Vec::new();
    for item in items {
        let Some(name) = json_string(item, "_name") else {
            continue;
        };
        let input_channels = json_u16(item, "coreaudio_device_input").unwrap_or(0);
        let output_channels = json_u16(item, "coreaudio_device_output").unwrap_or(0);
        if input_channels == 0 && output_channels == 0 {
            continue;
        }
        let base_id = normalize_device_id(name);
        let sequence = slug_counts.entry(base_id.clone()).or_insert(0);
        *sequence += 1;
        let device_id = if *sequence == 1 {
            base_id
        } else {
            format!("{base_id}-{}", sequence)
        };
        devices.push(AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id,
            name: name.to_string(),
            default_input: json_yes(item, "coreaudio_default_audio_input_device"),
            default_output: json_yes(item, "coreaudio_default_audio_output_device")
                || json_yes(item, "coreaudio_default_audio_system_device"),
            max_input_channels: input_channels,
            max_output_channels: output_channels,
            nominal_sample_rate: SampleRate(
                json_u32(item, "coreaudio_device_srate").unwrap_or(48_000),
            ),
            preferred_buffer_sizes: vec![128, 256, 512],
        });
    }

    Ok(devices)
}

fn json_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn json_yes(value: &Value, key: &str) -> bool {
    matches!(json_string(value, key), Some("spaudio_yes"))
}

fn json_u16(value: &Value, key: &str) -> Option<u16> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
}

fn json_u32(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn normalize_device_id(name: &str) -> String {
    let mut slug = String::from("coreaudio:");
    let mut prior_dash = false;
    for ch in name.chars() {
        let lowercase = ch.to_ascii_lowercase();
        if lowercase.is_ascii_alphanumeric() {
            slug.push(lowercase);
            prior_dash = false;
        } else if !prior_dash {
            slug.push('-');
            prior_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug == "coreaudio:" {
        "coreaudio:device".into()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_hardware::{AudioSampleFormat, AudioStreamDirection};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_inventory_fixture<T>(fixture: &str, f: impl FnOnce() -> T) -> T {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let prior = env::var(COREAUDIO_FIXTURE_ENV).ok();
        env::set_var(COREAUDIO_FIXTURE_ENV, fixture);
        let result = f();
        if let Some(prior) = prior {
            env::set_var(COREAUDIO_FIXTURE_ENV, prior);
        } else {
            env::remove_var(COREAUDIO_FIXTURE_ENV);
        }
        result
    }

    const HEALTHY_FIXTURE: &str = r#"{
  "SPAudioDataType": [
    {
      "_items": [
        {
          "_name": "Signal Built-In Microphone",
          "coreaudio_default_audio_input_device": "spaudio_yes",
          "coreaudio_device_input": 2,
          "coreaudio_device_srate": 48000
        },
        {
          "_name": "Signal Built-In Speakers",
          "coreaudio_default_audio_output_device": "spaudio_yes",
          "coreaudio_default_audio_system_device": "spaudio_yes",
          "coreaudio_device_output": 2,
          "coreaudio_device_srate": 48000
        },
        {
          "_name": "Signal Aggregate Interface",
          "coreaudio_device_input": 4,
          "coreaudio_device_output": 4,
          "coreaudio_device_srate": 96000
        }
      ]
    }
  ]
}"#;

    const DEGRADED_FIXTURE: &str = r#"{
  "SPAudioDataType": [
    {
      "_items": [
        {
          "_name": "Signal Input Only Interface",
          "coreaudio_default_audio_input_device": "spaudio_yes",
          "coreaudio_device_input": 2,
          "coreaudio_device_srate": 48000
        }
      ]
    }
  ]
}"#;

    #[test]
    fn coreaudio_backend_enumerates_real_devices_from_inventory_fixture() {
        with_inventory_fixture(HEALTHY_FIXTURE, || {
            let backend = CoreAudioBackend::default();

            let devices = backend.enumerate_devices();
            assert_eq!(devices.len(), 3);
            assert!(devices.iter().any(|device| device.device_id
                == "coreaudio:signal-built-in-speakers"
                && device.default_output
                && device.max_output_channels == 2));
            assert!(devices.iter().any(|device| device.device_id
                == "coreaudio:signal-aggregate-interface"
                && device.max_input_channels == 4
                && device.max_output_channels == 4
                && device.nominal_sample_rate == SampleRate(96_000)));

            let device = backend
                .default_output_device()
                .expect("coreaudio default output device");
            assert_eq!(device.device_id, "coreaudio:signal-built-in-speakers");
            assert_eq!(device.name, "Signal Built-In Speakers");
            assert_eq!(backend.diagnostics().health, BackendHealth::Healthy);

            let stream = backend
                .default_output_stream(48_000, 256)
                .expect("coreaudio default output stream");
            assert_eq!(
                stream.device.device_id,
                "coreaudio:signal-built-in-speakers"
            );
            assert_eq!(stream.direction, AudioStreamDirection::Output);
            assert_eq!(stream.sample_rate, SampleRate(48_000));
            assert_eq!(stream.buffer_size, 256);
            assert_eq!(stream.output_channels, 2);
            assert_eq!(stream.sample_format, AudioSampleFormat::F32);
            assert_eq!(stream.clock_source, HardwareClockSource::Internal);
            assert_eq!(stream.clock_topology, HardwareClockTopology::SingleEndpoint);
            assert_eq!(stream.latency, HardwareLatencyProfile::output_only(256));
            assert_eq!(
                stream.lifecycle,
                HardwareLifecycleContract {
                    ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                    restart_policy: HardwareRestartPolicy::HostMustRestart,
                }
            );
            assert!(!stream.simulated);
        });
    }

    #[test]
    fn coreaudio_backend_reports_degraded_inventory_without_default_output() {
        with_inventory_fixture(DEGRADED_FIXTURE, || {
            let backend = CoreAudioBackend::default();

            assert_eq!(backend.health(), BackendHealth::Degraded);
            assert!(backend.default_output_device().is_none());
            let diagnostics = backend.diagnostics();
            assert_eq!(diagnostics.health, BackendHealth::Degraded);
            assert_eq!(
                diagnostics.last_event.as_ref().map(|event| event.kind),
                Some(HardwareDiagnosticKind::DeviceDisconnected)
            );
        });
    }

    #[test]
    fn coreaudio_backend_tracks_device_loss_and_restart_diagnostics_against_real_device_ids() {
        with_inventory_fixture(HEALTHY_FIXTURE, || {
            let mut backend = CoreAudioBackend::default();

            backend.simulate_device_loss("simulated device disconnect");
            backend.simulate_restart_attempt("simulated restart attempt");
            backend.simulate_restart_failure("simulated restart failure");

            let diagnostics = backend.diagnostics();
            assert_eq!(diagnostics.device_loss_count, 1);
            assert_eq!(diagnostics.restart_attempt_count, 1);
            assert_eq!(diagnostics.restart_failure_count, 1);
            assert_eq!(
                diagnostics
                    .last_event
                    .as_ref()
                    .and_then(|event| event.device_id.as_deref()),
                Some("coreaudio:signal-built-in-speakers")
            );
            assert_eq!(diagnostics.health, BackendHealth::Degraded);
        });
    }
}
