//! CoreAudio backend shell for Signal.

use signal_hardware::{
    BackendHealth, BackendPolicyRecord, BackendPolicyTier, HardwareBackend, HardwareConfigRequest,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreAudioBackend {
    policy_tier: BackendPolicyTier,
}

impl Default for CoreAudioBackend {
    fn default() -> Self {
        Self {
            policy_tier: BackendPolicyTier::Tier0InHost,
        }
    }
}

impl CoreAudioBackend {
    pub fn policy_tier(&self) -> BackendPolicyTier {
        self.policy_tier
    }

    pub fn default_output_request(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> HardwareConfigRequest {
        HardwareConfigRequest::new(sample_rate, buffer_size, self.policy_tier)
    }
}

impl HardwareBackend for CoreAudioBackend {
    fn backend_name(&self) -> &'static str {
        "coreaudio"
    }

    fn policy_record(&self) -> BackendPolicyRecord {
        BackendPolicyRecord {
            tier: self.policy_tier,
            in_host_default: true,
        }
    }

    fn health(&self) -> BackendHealth {
        BackendHealth::Healthy
    }
}
