//! Local hardware backend over real cpal output-device enumeration.
//!
//! Replaces the retired `signal-hardware-coreaudio` crate (a `system_profiler`
//! JSON parser). Device inventory comes from
//! [`signal_hardware_output_cpal::enumerate_output_devices`] — the same source
//! of truth the real output streams open against. Device identity stays in the
//! `coreaudio:*` namespace because the cpal host on macOS is CoreAudio.
//!
//! The `simulate_*` diagnostic mutation helpers record fault events and update
//! health state without touching real hardware; they power the device-loss
//! recovery paths exercised by the kept host tests.

use signal_hardware::{
    AudioDeviceDescriptor, BackendHealth, BackendPolicyTier, HardwareBackendIdentity,
    HardwareClockSource, HardwareClockTopology, HardwareDiagnosticEvent, HardwareDiagnosticKind,
    HardwareDiagnosticSeverity, HardwareDiagnosticsSnapshot, HardwareLatencyProfile,
    HardwareLifecycleContract, HardwareLifecycleOwnership, HardwareNegotiationError,
    HardwareNegotiationErrorKind, HardwareRestartPolicy, HardwareStreamConfig,
    HardwareStreamRequest, SampleRate,
};
use signal_hardware_output_cpal::enumerate_output_devices;

/// Local hardware backend that owns the enumerated device inventory and a
/// mutable diagnostics snapshot for the host's supervision surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalHardwareBackend {
    policy_tier: BackendPolicyTier,
    devices: Vec<AudioDeviceDescriptor>,
    diagnostics: HardwareDiagnosticsSnapshot,
}

impl Default for LocalHardwareBackend {
    fn default() -> Self {
        let inventory = discover_inventory();
        Self {
            policy_tier: BackendPolicyTier::Tier0InHost,
            devices: inventory.devices,
            diagnostics: inventory.diagnostics,
        }
    }
}

impl LocalHardwareBackend {
    pub(crate) fn backend_identity(&self) -> HardwareBackendIdentity {
        HardwareBackendIdentity::CoreAudio
    }

    pub(crate) fn backend_name(&self) -> &'static str {
        "coreaudio"
    }

    pub(crate) fn policy_tier(&self) -> BackendPolicyTier {
        self.policy_tier
    }

    pub(crate) fn diagnostics(&self) -> HardwareDiagnosticsSnapshot {
        self.diagnostics.clone()
    }

    pub(crate) fn default_output_device(&self) -> Option<AudioDeviceDescriptor> {
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

    /// Negotiate and return a stream config for the default output device.
    pub(crate) fn default_output_stream(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let device = self.default_output_device().ok_or_else(|| {
            HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::DeviceUnavailable,
                "no default output device is currently available",
            )
        })?;
        let request =
            HardwareStreamRequest::new_output(device.device_id.clone(), sample_rate, buffer_size);
        self.negotiate_stream(&request)
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
                    format!("unknown output device {}", request.device_id),
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
                    "requested {} output channels exceeds device capacity",
                    request.output_channels
                ),
            ));
        }
        if request.input_channels > device.max_input_channels {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} input channels exceeds device capacity",
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

    /// Transition health back to `Healthy` (or `Degraded` if no default output
    /// device is present) after a recovery sequence.
    pub(crate) fn mark_recovered(&mut self) {
        self.diagnostics.health = if self.default_output_device().is_some() {
            BackendHealth::Healthy
        } else {
            BackendHealth::Degraded
        };
    }

    /// Inject a simulated device-loss event. Records a
    /// `DeviceDisconnected` event and sets health to `Degraded`.
    pub(crate) fn simulate_device_loss(&mut self, detail: impl Into<String>) {
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

    /// Inject a simulated restart-attempt event. Records a `RestartAttempted`
    /// event and sets health to `Recovering`.
    pub(crate) fn simulate_restart_attempt(&mut self, detail: impl Into<String>) {
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

    /// Inject a simulated restart-failure event. Records a `RestartFailed`
    /// event and sets health to `Degraded`.
    pub(crate) fn simulate_restart_failure(&mut self, detail: impl Into<String>) {
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

struct LocalHardwareInventory {
    devices: Vec<AudioDeviceDescriptor>,
    diagnostics: HardwareDiagnosticsSnapshot,
}

fn discover_inventory() -> LocalHardwareInventory {
    match enumerate_output_devices() {
        Ok(descriptions) => {
            let mut slug_counts = std::collections::HashMap::<String, usize>::new();
            let devices: Vec<AudioDeviceDescriptor> = descriptions
                .into_iter()
                .map(|description| {
                    let base_id = normalize_device_id(&description.name);
                    let sequence = slug_counts.entry(base_id.clone()).or_insert(0);
                    *sequence += 1;
                    let device_id = if *sequence == 1 {
                        base_id
                    } else {
                        format!("{base_id}-{}", sequence)
                    };
                    AudioDeviceDescriptor {
                        backend_identity: HardwareBackendIdentity::CoreAudio,
                        backend_name: "coreaudio",
                        device_id,
                        name: description.name,
                        default_input: false,
                        default_output: description.is_default,
                        max_input_channels: 0,
                        max_output_channels: description.max_channels,
                        nominal_sample_rate: SampleRate(description.default_sample_rate_hz),
                        preferred_buffer_sizes: vec![128, 256, 512],
                    }
                })
                .collect();
            let diagnostics = baseline_diagnostics(&devices);
            LocalHardwareInventory {
                devices,
                diagnostics,
            }
        }
        Err(error) => LocalHardwareInventory {
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
                    detail: format!("output device inventory unavailable: {error}"),
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
                detail: "no default output device reported".into(),
            }),
        }
    }
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
    use super::normalize_device_id;

    #[test]
    fn normalizes_device_names_into_stable_ids() {
        assert_eq!(
            normalize_device_id("MacBook Pro Speakers"),
            "coreaudio:macbook-pro-speakers"
        );
        assert_eq!(normalize_device_id("  --  "), "coreaudio:device");
        assert_eq!(normalize_device_id("USB DAC (2)"), "coreaudio:usb-dac-2");
    }
}
