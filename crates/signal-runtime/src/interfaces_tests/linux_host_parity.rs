use super::*;

#[test]
fn runtime_host_hardware_summary_classifies_linux_backend_baselines() {
    let alsa = RuntimeHostHardwareSummary {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        backend_name: "alsa".into(),
        linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        ),
        linux_backend_portability: RuntimeHostHardwareSummary::classify_linux_backend_portability(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
            false,
            BackendHealth::Healthy,
            0,
            0,
            0,
        ),
        device_id: "alsa:default-output".into(),
        device_name: "ALSA Default Output".into(),
        sample_rate: 48_000,
        buffer_size: 256,
        input_channels: 0,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        simulated: false,
        backend_health: BackendHealth::Healthy,
        xrun_count: 0,
        callback_overrun_count: 0,
        device_loss_count: 0,
        restart_attempt_count: 0,
        restart_failure_count: 0,
    };
    assert_eq!(
        alsa.linux_backend_identity,
        RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa.linux_backend_portability,
        RuntimeLinuxAudioBackendPortabilityBand::Portable
    );

    let jack = RuntimeHostHardwareSummary {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        backend_name: "jack".into(),
        linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        ),
        linux_backend_portability: RuntimeHostHardwareSummary::classify_linux_backend_portability(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
            true,
            BackendHealth::Recovering,
            1,
            1,
            0,
        ),
        device_id: "jack:graph-main".into(),
        device_name: "JACK Graph Main".into(),
        sample_rate: 48_000,
        buffer_size: 128,
        input_channels: 2,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        simulated: true,
        backend_health: BackendHealth::Recovering,
        xrun_count: 2,
        callback_overrun_count: 0,
        device_loss_count: 1,
        restart_attempt_count: 1,
        restart_failure_count: 0,
    };
    assert_eq!(
        jack.linux_backend_identity,
        RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack.linux_backend_portability,
        RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );

    let not_linux = RuntimeHostHardwareSummary::classify_linux_backend_portability(
        HardwareBackendIdentity::CoreAudio,
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    assert_eq!(
        not_linux,
        RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
}

#[test]
fn runtime_host_io_classifies_linux_clocking_duplex_and_endpoint_parity() {
    let alsa_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                alsa_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Portable
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                alsa_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Aligned
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            alsa_identity,
            BackendHealth::Healthy,
            RuntimeHostClockTransitionState::Stable,
            RuntimeHostClockDiscontinuityState::Continuous,
            RuntimeHostDuplexMismatchState::Aligned,
            RuntimeHostEndpointTopology::Duplex,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
    );

    let jack_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                jack_identity,
                BackendHealth::Recovering,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::Aggregate,
                RuntimeHostClockFallbackState::RuntimeResampled,
                RuntimeHostClockTransitionState::EnteredAggregateClock,
                RuntimeHostClockDriftState::AggregateManaged,
                RuntimeHostClockDiscontinuityState::Reconfigured,
                RuntimeHostDuplexMismatchState::CrossClockDiverged,
                RuntimeHostEndpointTopology::Aggregate,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                jack_identity,
                BackendHealth::Recovering,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::Aggregate,
                RuntimeHostClockFallbackState::RuntimeResampled,
                RuntimeHostClockTransitionState::EnteredAggregateClock,
                RuntimeHostClockDriftState::AggregateManaged,
                RuntimeHostClockDiscontinuityState::Reconfigured,
                RuntimeHostDuplexMismatchState::CrossClockDiverged,
                RuntimeHostEndpointTopology::Aggregate,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            jack_identity,
            BackendHealth::Recovering,
            RuntimeHostClockTransitionState::EnteredAggregateClock,
            RuntimeHostClockDiscontinuityState::Reconfigured,
            RuntimeHostDuplexMismatchState::CrossClockDiverged,
            RuntimeHostEndpointTopology::Aggregate,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    let not_linux_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
        HardwareBackendIdentity::CoreAudio,
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_clocking_parity(
            RuntimeHostIoSummary::linux_parity_input(
                not_linux_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_duplex_parity(
            RuntimeHostIoSummary::linux_parity_input(
                not_linux_identity,
                BackendHealth::Healthy,
                RuntimeHostAudioStreamState::Running,
                RuntimeHostClockDomain::SameClock,
                RuntimeHostClockFallbackState::Direct,
                RuntimeHostClockTransitionState::Stable,
                RuntimeHostClockDriftState::Stable,
                RuntimeHostClockDiscontinuityState::Continuous,
                RuntimeHostDuplexMismatchState::Aligned,
                RuntimeHostEndpointTopology::Duplex,
                false,
            )
        ),
        RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
            not_linux_identity,
            BackendHealth::Healthy,
            RuntimeHostClockTransitionState::Stable,
            RuntimeHostClockDiscontinuityState::Continuous,
            RuntimeHostDuplexMismatchState::Aligned,
            RuntimeHostEndpointTopology::Duplex,
            false,
        ),
        RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
}
