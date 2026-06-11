#[path = "support/public_contract_boundary_host_io_clock.rs"]
mod public_contract_boundary_host_io_clock_support;

use public_contract_boundary_host_io_clock_support::{
    sample_public_clock_topology_host_io, PublicClockTopologyHostIoConfig,
};
use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeLifecycleApi,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-runtime-external-io".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime external io handshake should succeed");
    runtime
        .configure(signal_runtime::RuntimeConfigRequest::new(44_100, 256))
        .expect("public runtime external io configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Unavailable
    );
    assert_eq!(
        baseline.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Unavailable
    );

    let cross_clock_duplex =
        sample_public_clock_topology_host_io(PublicClockTopologyHostIoConfig {
            clock_domain: RuntimeHostClockDomain::CrossClock,
            fallback_state: RuntimeHostClockFallbackState::RuntimeResampled,
            transition_state: RuntimeHostClockTransitionState::EnteredCrossClockFallback,
            drift_state: RuntimeHostClockDriftState::CrossClockManaged,
            discontinuity_state: RuntimeHostClockDiscontinuityState::Reconfigured,
            duplex_mismatch_state: RuntimeHostDuplexMismatchState::CrossClockDiverged,
            endpoint_topology: RuntimeHostEndpointTopology::Duplex,
            partial_availability: false,
        });
    let observation = baseline.with_host_external_io(&cross_clock_duplex);
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_host_external_io(&cross_clock_duplex);

    assert_eq!(
        observation.external_io_snapshot.primary_role,
        RuntimeExternalIoPrimaryRole::ProgramDuplex
    );
    assert_eq!(
        observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        observation.external_io_snapshot.monitoring_tap_point,
        RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Guarded
    );
    assert_eq!(
        supervisor.observation.external_io_snapshot.monitoring_state,
        RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        supervisor.observation.external_io_snapshot.loopback_state,
        RuntimeExternalIoLoopbackState::Guarded
    );

}
