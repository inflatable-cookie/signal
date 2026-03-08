//! Common hardware and device abstractions for Signal.

use signal_primitives::SampleRate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendPolicyTier {
    Tier0InHost,
    Tier1Brokered,
    Tier2StrongContainment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDeviceDescriptor {
    pub device_id: String,
    pub name: String,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareConfigRequest {
    pub sample_rate: SampleRate,
    pub buffer_size: usize,
    pub backend_policy: BackendPolicyTier,
}

impl HardwareConfigRequest {
    pub fn new(sample_rate: u32, buffer_size: usize, backend_policy: BackendPolicyTier) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            buffer_size,
            backend_policy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyRecord {
    pub tier: BackendPolicyTier,
    pub in_host_default: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded,
    Recovering,
}

pub trait HardwareBackend {
    fn backend_name(&self) -> &'static str;
    fn policy_record(&self) -> BackendPolicyRecord;
    fn health(&self) -> BackendHealth;
}
