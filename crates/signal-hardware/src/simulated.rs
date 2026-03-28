//! Simulated hardware backend for testing and development.
//!
//! This module provides a [`SimulatedHardwareBackend`] that implements the
//! [`HardwareBackend`] trait for testing stream negotiation, lifecycle
//! contracts, and diagnostic reporting without requiring actual hardware.

use crate::{
    AudioDeviceDescriptor, BackendHealth, BackendPolicyRecord, BackendPolicyTier, HardwareBackend,
    HardwareBackendIdentity, HardwareClockSource, HardwareClockTopology,
    HardwareDiagnosticsSnapshot, HardwareLatencyProfile, HardwareLifecycleContract,
    HardwareNegotiationError, HardwareNegotiationErrorKind, HardwareStreamConfig,
    HardwareStreamRequest,
};

mod configuration;

/// A simulated hardware backend for testing and development.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulatedHardwareBackend {
    backend_identity: HardwareBackendIdentity,
    backend_name: &'static str,
    policy_tier: BackendPolicyTier,
    devices: Vec<AudioDeviceDescriptor>,
    lifecycle: HardwareLifecycleContract,
    clock_source: HardwareClockSource,
    clock_topology: HardwareClockTopology,
    diagnostics: HardwareDiagnosticsSnapshot,
}

impl SimulatedHardwareBackend {
    /// Create a new simulated backend with the given identity, name, tier, and devices.
    pub fn new(
        backend_identity: HardwareBackendIdentity,
        backend_name: &'static str,
        policy_tier: BackendPolicyTier,
        devices: Vec<AudioDeviceDescriptor>,
    ) -> Self {
        Self {
            backend_identity,
            backend_name,
            policy_tier,
            devices,
            lifecycle: HardwareLifecycleContract::default(),
            clock_source: HardwareClockSource::Virtual,
            clock_topology: HardwareClockTopology::SingleEndpoint,
            diagnostics: HardwareDiagnosticsSnapshot::healthy(),
        }
    }
}

impl HardwareBackend for SimulatedHardwareBackend {
    fn backend_identity(&self) -> HardwareBackendIdentity {
        self.backend_identity
    }

    fn backend_name(&self) -> &'static str {
        self.backend_name
    }

    fn policy_record(&self) -> BackendPolicyRecord {
        BackendPolicyRecord {
            backend_identity: self.backend_identity,
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
                    format!("unknown device {}", request.device_id),
                )
            })?;

        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                "sample_rate and buffer_size must be non-zero",
            ));
        }
        if request.output_channels > device.max_output_channels
            || request.input_channels > device.max_input_channels
        {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} input and {} output channels exceeds device capacity",
                    request.input_channels, request.output_channels
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
            clock_source: self.clock_source,
            clock_topology: self.clock_topology,
            lifecycle: self.lifecycle,
            latency: HardwareLatencyProfile::output_only(request.buffer_size as u32),
            simulated: true,
        })
    }

    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot {
        self.diagnostics.clone()
    }
}

#[cfg(test)]
mod tests;
