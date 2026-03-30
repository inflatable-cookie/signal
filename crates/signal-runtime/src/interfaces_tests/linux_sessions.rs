use super::*;

#[test]
fn runtime_linux_backend_session_snapshot_classifies_live_ownership_baselines() {
    let alsa = RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        RuntimeHostLifecycleOwnership::HostDrivenCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    ));
    assert_eq!(
        alsa.backend_identity,
        RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa.ownership,
        RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
    );
    assert_eq!(
        alsa.lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::Running
    );
    assert_eq!(
        alsa.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
    );
    assert_eq!(
        alsa.session_role,
        RuntimeLinuxBackendSessionRole::PrimaryAudioIo
    );
    assert_eq!(
        alsa.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::Direct
    );

    let jack = RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    ));
    assert_eq!(
        jack.backend_identity,
        RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack.ownership,
        RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        jack.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
    );
    assert_eq!(
        jack.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );

    let pipewire_recovering =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&linux_host_io_summary(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
            RuntimeHostLifecycleOwnership::BackendManagedCallback,
            RuntimeHostAudioStreamState::Faulted,
            BackendHealth::Recovering,
            1,
            1,
            1,
        ));
    assert_eq!(
        pipewire_recovering.backend_identity,
        RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        pipewire_recovering.lifecycle_state,
        RuntimeLinuxBackendSessionLifecycleState::Recovering
    );
    assert_eq!(
        pipewire_recovering.device_claim_posture,
        RuntimeLinuxBackendDeviceClaimPosture::Lost
    );
    assert_eq!(
        pipewire_recovering.session_role,
        RuntimeLinuxBackendSessionRole::FallbackContinuation
    );
    assert_eq!(
        pipewire_recovering.ownership_fallback,
        RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
    );
}

#[test]
fn runtime_pipewire_alsa_parity_snapshot_derives_runtime_owned_parity_baselines() {
    let alsa_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        RuntimeHostLifecycleOwnership::HostDrivenCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let alsa_linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(&alsa_host_io);
    let alsa = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &alsa_host_io,
        &alsa_linux_session,
    );
    assert_eq!(
        alsa.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        alsa.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
    );
    assert_eq!(
        alsa.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
    );
    assert_eq!(
        alsa.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::Direct
    );

    let pipewire_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let pipewire_linux_session =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_host_io);
    let pipewire = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &pipewire_host_io,
        &pipewire_linux_session,
    );
    assert_eq!(
        pipewire.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        pipewire.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        pipewire.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        pipewire.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::BackendManaged
    );

    let pipewire_recovering_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Faulted,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );
    let pipewire_recovering_linux_session =
        RuntimeLinuxBackendSessionSnapshot::from_host_io(&pipewire_recovering_host_io);
    let pipewire_recovering = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &pipewire_recovering_host_io,
        &pipewire_recovering_linux_session,
    );
    assert_eq!(
        pipewire_recovering.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
    );
    assert_eq!(
        pipewire_recovering.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::Lost
    );
    assert_eq!(
        pipewire_recovering.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::Restarting
    );
    assert_eq!(
        pipewire_recovering.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
    );

    let jack_host_io = linux_host_io_summary(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        RuntimeHostLifecycleOwnership::BackendManagedCallback,
        RuntimeHostAudioStreamState::Running,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack_linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(&jack_host_io);
    let jack = RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
        &jack_host_io,
        &jack_linux_session,
    );
    assert_eq!(
        jack.session_role_parity,
        RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.device_claim_parity,
        RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.stream_policy_parity,
        RuntimePipeWireAlsaStreamPolicyParity::NotPipeWireOrAlsa
    );
    assert_eq!(
        jack.guarded_state,
        RuntimePipeWireAlsaGuardedParityState::NotPipeWireOrAlsa
    );
}
