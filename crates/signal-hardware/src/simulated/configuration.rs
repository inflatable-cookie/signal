use signal_primitives::SampleRate;

use crate::{
    AudioDeviceDescriptor, BackendPolicyTier, HardwareBackendIdentity, HardwareClockSource,
    HardwareClockTopology, HardwareDiagnosticsSnapshot, HardwareLifecycleContract,
    HardwareLifecycleOwnership, HardwareRestartPolicy, LinuxAudioBackendKind,
};

use super::SimulatedHardwareBackend;

impl SimulatedHardwareBackend {
    /// Create a default stereo output backend with standard configuration.
    pub fn default_stereo_output(
        backend_identity: HardwareBackendIdentity,
        backend_name: &'static str,
        device_id: &str,
        name: &str,
    ) -> Self {
        Self::new(
            backend_identity,
            backend_name,
            BackendPolicyTier::Tier0InHost,
            vec![AudioDeviceDescriptor {
                backend_identity,
                backend_name,
                device_id: device_id.into(),
                name: name.into(),
                default_input: false,
                default_output: true,
                max_input_channels: 0,
                max_output_channels: 2,
                nominal_sample_rate: SampleRate(48_000),
                preferred_buffer_sizes: vec![128, 256, 512],
            }],
        )
    }

    /// Create a simulated ALSA default output backend.
    pub fn linux_alsa_default_output() -> Self {
        Self::default_stereo_output(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
            "alsa",
            "alsa:default-output",
            "ALSA Default Output",
        )
        .with_clock_source(HardwareClockSource::Internal)
        .with_clock_topology(HardwareClockTopology::SingleEndpoint)
        .with_lifecycle(HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        })
    }

    /// Create a simulated JACK duplex backend.
    pub fn linux_jack_duplex() -> Self {
        Self::new(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            "jack",
            BackendPolicyTier::Tier1Brokered,
            vec![AudioDeviceDescriptor {
                backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
                backend_name: "jack",
                device_id: "jack:graph-main".into(),
                name: "JACK Graph Main".into(),
                default_input: true,
                default_output: true,
                max_input_channels: 2,
                max_output_channels: 2,
                nominal_sample_rate: SampleRate(48_000),
                preferred_buffer_sizes: vec![128, 256],
            }],
        )
        .with_clock_source(HardwareClockSource::ExternalWordClock)
        .with_clock_topology(HardwareClockTopology::Aggregate)
        .with_lifecycle(HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::BackendManagedCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        })
    }

    /// Create a simulated PipeWire duplex backend.
    pub fn linux_pipewire_duplex() -> Self {
        Self::new(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
            "pipewire",
            BackendPolicyTier::Tier1Brokered,
            vec![AudioDeviceDescriptor {
                backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
                backend_name: "pipewire",
                device_id: "pipewire:default-graph".into(),
                name: "PipeWire Default Graph".into(),
                default_input: true,
                default_output: true,
                max_input_channels: 2,
                max_output_channels: 2,
                nominal_sample_rate: SampleRate(48_000),
                preferred_buffer_sizes: vec![128, 256, 512],
            }],
        )
        .with_clock_source(HardwareClockSource::Virtual)
        .with_clock_topology(HardwareClockTopology::Aggregate)
        .with_lifecycle(HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::BackendManagedCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        })
    }

    /// Set the lifecycle contract for this backend.
    pub fn with_lifecycle(mut self, lifecycle: HardwareLifecycleContract) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Set the clock source for this backend.
    pub fn with_clock_source(mut self, clock_source: HardwareClockSource) -> Self {
        self.clock_source = clock_source;
        self
    }

    /// Set the clock topology for this backend.
    pub fn with_clock_topology(mut self, clock_topology: HardwareClockTopology) -> Self {
        self.clock_topology = clock_topology;
        self
    }

    /// Set the diagnostics snapshot for this backend.
    pub fn with_diagnostics(mut self, diagnostics: HardwareDiagnosticsSnapshot) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}
