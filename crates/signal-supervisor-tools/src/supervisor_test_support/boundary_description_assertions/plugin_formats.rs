pub(crate) fn assert_vst3_boundary_text(rendered: &str) {
    for expected in [
        "vst3_boundary: signal.runtime.vst3-boundary",
        "acceptance_task: effigy acceptance:vst3-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
        "cargo test -p signal-runtime public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
        "cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_vst3_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.vst3-boundary\"",
        "\"contract_path\":\"docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:vst3-boundary\"",
        "\"id\":\"runtime-vst3-discovery-report\"",
        "\"id\":\"runtime-vst3-lifecycle-snapshot\"",
        "\"id\":\"shared-host-vst3-supervisor-report\"",
        "\"id\":\"server-host-vst3-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_au_boundary_text(rendered: &str) {
    for expected in [
        "au_boundary: signal.runtime.au-boundary",
        "acceptance_task: effigy acceptance:au-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
        "cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
        "cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_au_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.au-boundary\"",
        "\"contract_path\":\"docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:au-boundary\"",
        "\"id\":\"runtime-au-discovery-report\"",
        "\"id\":\"runtime-au-lifecycle-snapshot\"",
        "\"id\":\"shared-host-au-supervisor-report\"",
        "\"id\":\"server-host-au-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_macos_au_coreaudio_boundary_text(rendered: &str) {
    for expected in [
        "macos_au_coreaudio_boundary: signal.runtime.macos-au-coreaudio-boundary",
        "acceptance_task: effigy acceptance:macos-au-coreaudio-boundary",
        "surface: enumerate_devices() -> Vec<AudioDeviceDescriptor>",
        "surface: RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeHostObservationReport::host_io",
        "surface: host_supervisor_report() -> RuntimeHostSupervisorReport",
        "cargo test -p signal-hardware-coreaudio",
        "cargo test -p signal-host-local --test public_host_edge_au -- --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-macos-au-coreaudio-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_macos_au_coreaudio_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.macos-au-coreaudio-boundary\"",
        "\"contract_path\":\"docs/contracts/073-native-backend-device-truth-and-coreaudio-implementation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:macos-au-coreaudio-boundary\"",
        "\"id\":\"coreaudio-device-enumeration\"",
        "\"id\":\"runtime-au-lifecycle-and-fault-report\"",
        "\"id\":\"local-host-au-coreaudio-supervisor-report\"",
        "\"id\":\"local-host-supervision-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_lv2_boundary_text(rendered: &str) {
    for expected in [
        "lv2_boundary: signal.runtime.lv2-boundary",
        "acceptance_task: effigy acceptance:lv2-boundary",
        "surface: RuntimeObservationReport::lv2_extension_snapshot and RuntimeSupervisorReport::observation.lv2_extension_snapshot",
        "surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
        "crate: signal-host-local",
        "crate: signal-host-server",
        "cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
        "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_lv2_extension_truth",
        "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_lv2_extension_truth",
        "cargo run -p signal-supervisor-tools -- --describe-lv2-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_lv2_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.lv2-boundary\"",
        "\"contract_path\":\"docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:lv2-boundary\"",
        "\"id\":\"runtime-lv2-extension-report\"",
        "\"id\":\"runtime-lv2-lifecycle-snapshot\"",
        "\"id\":\"local-host-lv2-supervisor-report\"",
        "\"id\":\"server-host-lv2-supervisor-report\"",
        "\"id\":\"local-host-lv2-proof\"",
        "\"id\":\"server-host-lv2-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_lv2_execution_boundary_text(rendered: &str) {
    for expected in [
        "linux_lv2_execution_boundary: signal.runtime.linux-lv2-execution-boundary",
        "acceptance_task: effigy acceptance:linux-lv2-execution-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot",
        "surface: supervisor_report() -> RuntimeSupervisorReport",
        "cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_lv2_sandbox_through_broker_process -- --exact --nocapture --test-threads=1",
        "cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-linux-lv2-execution-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_lv2_execution_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.linux-lv2-execution-boundary\"",
        "\"contract_path\":\"docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-lv2-execution-boundary\"",
        "\"id\":\"runtime-lv2-discovery-report\"",
        "\"id\":\"runtime-lv2-broker-lifecycle-report\"",
        "\"id\":\"server-host-lv2-broker-supervisor-report\"",
        "\"id\":\"server-host-lv2-broker-recovery-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_cross_adapter_parity_boundary_text(rendered: &str) {
    for expected in [
        "cross_adapter_parity_boundary: signal.runtime.cross-adapter-parity-boundary",
        "acceptance_task: effigy acceptance:cross-adapter-parity-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
        "cargo test -p signal-runtime public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth",
        "cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_cross_adapter_parity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.cross-adapter-parity-boundary\"",
        "\"contract_path\":\"docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:cross-adapter-parity-boundary\"",
        "\"id\":\"runtime-cross-adapter-discovery-report\"",
        "\"id\":\"runtime-cross-adapter-lifecycle-snapshot\"",
        "\"id\":\"shared-host-cross-adapter-supervisor-report\"",
        "\"id\":\"server-host-cross-adapter-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_plugin_parity_boundary_text(rendered: &str) {
    for expected in [
        "linux_plugin_parity_boundary: signal.runtime.linux-plugin-parity-boundary",
        "acceptance_task: effigy acceptance:linux-plugin-parity-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
        "cargo test -p signal-runtime public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth",
        "cargo run -p signal-supervisor-tools -- --describe-linux-plugin-parity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_plugin_parity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.linux-plugin-parity-boundary\"",
        "\"contract_path\":\"docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-plugin-parity-boundary\"",
        "\"id\":\"runtime-linux-parity-discovery-report\"",
        "\"id\":\"runtime-linux-parity-lifecycle-snapshot\"",
        "\"id\":\"server-host-linux-parity-supervisor-report\"",
        "\"id\":\"server-host-linux-parity-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
