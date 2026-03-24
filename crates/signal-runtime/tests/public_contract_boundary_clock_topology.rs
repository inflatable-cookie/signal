#[path = "support/public_contract_boundary_host_io_clock.rs"]
mod public_contract_boundary_host_io_clock_support;

use public_contract_boundary_host_io_clock_support::sample_public_clock_topology_host_io;
use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeHostClockDiscontinuityState,
    RuntimeHostClockDomain, RuntimeHostClockDriftState, RuntimeHostClockFallbackState,
    RuntimeHostClockTransitionState, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostObservationReport, RuntimeHostSupervisorReport, RuntimeLifecycleApi,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-runtime-clock-topology".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime clock topology handshake should succeed");
    runtime
        .configure(signal_runtime::RuntimeConfigRequest::new(44_100, 256))
        .expect("public runtime clock topology configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let cross_clock_duplex = sample_public_clock_topology_host_io(
        RuntimeHostClockDomain::CrossClock,
        RuntimeHostClockFallbackState::RuntimeResampled,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback,
        RuntimeHostClockDriftState::CrossClockManaged,
        RuntimeHostClockDiscontinuityState::Reconfigured,
        RuntimeHostDuplexMismatchState::CrossClockDiverged,
        RuntimeHostEndpointTopology::Duplex,
        false,
    );
    let host_observation = RuntimeHostObservationReport::new(
        observation
            .clone()
            .with_host_device_supervision(&cross_clock_duplex),
        cross_clock_duplex.clone(),
    );

    assert_eq!(
        host_observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::CrossClockManaged
    );
    assert_eq!(
        host_observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert_eq!(
        host_observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::CrossClockDiverged
    );
    assert_eq!(
        host_observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert!(!host_observation.host_io.clocking.partial_availability);

    let partial_duplex = sample_public_clock_topology_host_io(
        RuntimeHostClockDomain::SameClock,
        RuntimeHostClockFallbackState::Direct,
        RuntimeHostClockTransitionState::Stable,
        RuntimeHostClockDriftState::Stable,
        RuntimeHostClockDiscontinuityState::Continuous,
        RuntimeHostDuplexMismatchState::PartialAvailability,
        RuntimeHostEndpointTopology::Duplex,
        true,
    );
    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_host_device_supervision(&partial_duplex);
    let host_supervisor = RuntimeHostSupervisorReport::new(supervisor, partial_duplex);

    assert_eq!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::PartialAvailability
    );
    assert_eq!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert!(
        host_supervisor
            .observation
            .host_io
            .clocking
            .partial_availability
    );

    let rendered = host_supervisor.render_json();
    assert!(rendered.contains("\"drift_state\":\"Stable\""));
    assert!(rendered.contains("\"discontinuity_state\":\"Continuous\""));
    assert!(rendered.contains("\"duplex_mismatch_state\":\"PartialAvailability\""));
    assert!(rendered.contains("\"endpoint_topology\":\"Duplex\""));
    assert!(rendered.contains("\"partial_availability\":true"));
}
