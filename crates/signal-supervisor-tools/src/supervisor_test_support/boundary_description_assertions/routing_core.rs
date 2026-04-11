pub(crate) fn assert_multichannel_boundary_text(rendered: &str) {
    for expected in [
        "multichannel_boundary: signal.runtime.multichannel-boundary",
        "acceptance_task: effigy acceptance:multichannel-boundary",
        "surface: RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::external_io_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,external_io_snapshot}",
        "surface: RuntimeObservationApi::get_plugin_discovery_snapshot()",
        "cargo test -p signal-runtime --test public_contract_boundary_multichannel public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-multichannel-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_multichannel_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.multichannel-boundary\"",
        "\"contract_path\":\"docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:multichannel-boundary\"",
        "\"id\":\"runtime-multichannel-topology-report\"",
        "\"id\":\"runtime-multichannel-plugin-discovery-snapshot\"",
        "\"id\":\"shared-host-multichannel-report\"",
        "\"id\":\"runtime-multichannel-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_multi_bus_boundary_text(rendered: &str) {
    for expected in [
        "multi_bus_boundary: signal.runtime.multi-bus-boundary",
        "acceptance_task: effigy acceptance:multi-bus-boundary",
        "surface: RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::metering_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,metering_snapshot}",
        "surface: RuntimeOfflineRenderContractPreview::chain_contract",
        "cargo test -p signal-runtime --test public_contract_boundary_multi_bus public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-multi-bus-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_multi_bus_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.multi-bus-boundary\"",
        "\"contract_path\":\"docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:multi-bus-boundary\"",
        "\"id\":\"runtime-multi-bus-topology-report\"",
        "\"id\":\"runtime-multi-bus-render-contract-preview\"",
        "\"id\":\"shared-host-multi-bus-report\"",
        "\"id\":\"runtime-multi-bus-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_sidechain_boundary_text(rendered: &str) {
    for expected in [
        "sidechain_boundary: signal.runtime.sidechain-boundary",
        "acceptance_task: effigy acceptance:sidechain-boundary",
        "surface: RuntimeObservationReport::execution_topology_summary, RuntimeSupervisorReport::observation.{execution_topology_summary,plugin_chain_snapshot}, and RuntimeOfflineRenderContractPreview::chain_contract",
        "surface: GraphNodeBufferContractProjection::secondary_input",
        "cargo test -p signal-runtime --test public_contract_boundary_sidechain public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-sidechain-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_sidechain_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.sidechain-boundary\"",
        "\"contract_path\":\"docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:sidechain-boundary\"",
        "\"id\":\"runtime-sidechain-topology-report\"",
        "\"id\":\"runtime-sidechain-contract-projection\"",
        "\"id\":\"shared-host-sidechain-report\"",
        "\"id\":\"runtime-sidechain-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_complex_io_boundary_text(rendered: &str) {
    for expected in [
        "complex_io_boundary: signal.runtime.complex-io-boundary",
        "acceptance_task: effigy acceptance:complex-io-boundary",
        "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
        "surface: RuntimeObservationReport::plugin_pin_matrix_snapshot and RuntimeSupervisorReport::observation.plugin_pin_matrix_snapshot",
        "surface: RuntimeOfflineRenderContractPreview::chain_contract",
        "cargo test -p signal-runtime public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth",
        "cargo run -p signal-supervisor-tools -- --describe-complex-io-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_complex_io_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.complex-io-boundary\"",
        "\"contract_path\":\"docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:complex-io-boundary\"",
        "\"id\":\"runtime-complex-io-discovery-report\"",
        "\"id\":\"runtime-plugin-pin-matrix-report\"",
        "\"id\":\"runtime-complex-io-plugin-chain-snapshot\"",
        "\"id\":\"runtime-complex-io-render-contract-preview\"",
        "\"id\":\"shared-host-complex-io-report\"",
        "\"id\":\"runtime-complex-io-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
