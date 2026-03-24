#[path = "support/public_contract_boundary_host_io_linux.rs"]
mod public_contract_boundary_host_io_linux_support;

use public_contract_boundary_host_io_linux_support::sample_public_linux_backend_host_io;
use signal_hardware::{BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind};
use signal_runtime::{
    RuntimeConfig, RuntimeEventRecorder, RuntimeObservationReport, RuntimeSupervisorReport,
    SignalRuntime,
};

#[test]
fn public_runtime_linux_backend_clock_topology_boundary_reports_runtime_owned_linux_parity_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );

    let alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:graph-main",
        "JACK Graph Main",
        true,
        BackendHealth::Recovering,
        1,
        1,
        0,
    );
    let pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        false,
        BackendHealth::Degraded,
        0,
        1,
        1,
    );

    let alsa_observation = baseline.clone().with_host_external_io(&alsa);
    let jack_observation = baseline.clone().with_host_external_io(&jack);
    let pipewire_observation = baseline.with_host_external_io(&pipewire);

    assert_eq!(
        alsa_observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Portable
    );
    assert_eq!(
        alsa_observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Aligned
    );
    assert_eq!(
        alsa_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
    );

    assert_eq!(
        jack_observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        jack_observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        jack_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_clocking_parity\":\"Guarded\""));
    assert!(observation_json.contains("\"linux_duplex_parity\":\"Guarded\""));
    assert!(observation_json.contains("\"linux_endpoint_topology_parity\":\"Guarded\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor.observation.clone().with_host_external_io(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"linux_clocking_parity\":\"Portable\""));
    assert!(supervisor_json.contains("\"linux_duplex_parity\":\"Aligned\""));
    assert!(supervisor_json.contains("\"linux_endpoint_topology_parity\":\"Portable\""));
}
