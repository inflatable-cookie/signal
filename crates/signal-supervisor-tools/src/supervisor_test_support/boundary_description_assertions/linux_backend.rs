pub(crate) fn assert_linux_audio_backend_boundary_text(rendered: &str) {
    for expected in [
        "linux_audio_backend_boundary: signal.runtime.linux-audio-backend-boundary",
        "acceptance_task: effigy acceptance:linux-audio-backend-boundary",
        "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
        "cargo test -p signal-runtime public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth",
        "cargo run -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_audio_backend_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.linux-audio-backend-boundary\"",
        "\"contract_path\":\"docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-audio-backend-boundary\"",
        "\"id\":\"runtime-linux-audio-observation-report\"",
        "\"id\":\"server-host-linux-audio-supervisor-report\"",
        "\"id\":\"server-host-linux-audio-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_live_ownership_boundary_text(rendered: &str) {
    for expected in [
        "linux_live_ownership_boundary: signal.runtime.linux-live-ownership-boundary",
        "acceptance_task: effigy acceptance:linux-live-ownership-boundary",
        "surface: RuntimeObservationReport::linux_backend_session_snapshot and RuntimeSupervisorReport::observation.linux_backend_session_snapshot",
        "cargo test -p signal-runtime public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth",
        "cargo run -p signal-supervisor-tools -- --describe-linux-live-ownership-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_live_ownership_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.linux-live-ownership-boundary\"",
        "\"contract_path\":\"docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-live-ownership-boundary\"",
        "\"id\":\"runtime-linux-live-session-report\"",
        "\"id\":\"local-host-linux-live-session-report\"",
        "\"id\":\"server-host-linux-live-session-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_jack_coordination_boundary_text(rendered: &str) {
    for expected in [
        "jack_coordination_boundary: signal.runtime.jack-coordination-boundary",
        "acceptance_task: effigy acceptance:jack-coordination-boundary",
        "surface: RuntimeObservationReport::jack_coordination_snapshot and RuntimeSupervisorReport::observation.jack_coordination_snapshot",
        "cargo test -p signal-runtime public_runtime_jack_coordination_boundary_reports_runtime_owned_transport_graph_truth",
        "cargo run -p signal-supervisor-tools -- --describe-jack-coordination-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_jack_coordination_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.jack-coordination-boundary\"",
        "\"contract_path\":\"docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:jack-coordination-boundary\"",
        "\"id\":\"runtime-jack-coordination-report\"",
        "\"id\":\"runtime-transport-session-report\"",
        "\"id\":\"shared-host-jack-supervisor-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_pipewire_alsa_parity_boundary_text(rendered: &str) {
    for expected in [
        "pipewire_alsa_parity_boundary: signal.runtime.pipewire-alsa-parity-boundary",
        "acceptance_task: effigy acceptance:pipewire-alsa-parity-boundary",
        "surface: RuntimeObservationReport::pipewire_alsa_parity_snapshot and RuntimeSupervisorReport::observation.pipewire_alsa_parity_snapshot",
        "cargo test -p signal-runtime public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth",
        "cargo run -p signal-supervisor-tools -- --describe-pipewire-alsa-parity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_pipewire_alsa_parity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.pipewire-alsa-parity-boundary\"",
        "\"contract_path\":\"docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:pipewire-alsa-parity-boundary\"",
        "\"id\":\"runtime-pipewire-alsa-parity-report\"",
        "\"id\":\"shared-host-pipewire-alsa-supervisor-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_backend_clock_topology_boundary_text(rendered: &str) {
    for expected in [
        "linux_backend_clock_topology_boundary: signal.runtime.linux-backend-clock-topology-boundary",
        "acceptance_task: effigy acceptance:linux-backend-clock-topology-boundary",
        "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
        "cargo test -p signal-runtime public_runtime_linux_backend_clock_topology_boundary_reports_runtime_owned_linux_parity_truth",
        "cargo run -p signal-supervisor-tools -- --describe-linux-backend-clock-topology-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_backend_clock_topology_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.linux-backend-clock-topology-boundary\"",
        "\"contract_path\":\"docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-backend-clock-topology-boundary\"",
        "\"id\":\"runtime-linux-backend-clock-topology-report\"",
        "\"id\":\"local-host-linux-backend-clock-topology-report\"",
        "\"id\":\"server-host-linux-backend-clock-topology-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
