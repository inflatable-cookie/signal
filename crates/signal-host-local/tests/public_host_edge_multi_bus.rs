#[path = "support/public_host_edge_multi_bus_graph.rs"]
mod public_host_edge_multi_bus_graph_support;

use public_host_edge_multi_bus_graph_support::apply_public_multi_bus_graph;
use signal_graph::synthetic_stereo_block;
use signal_host_local::LocalRuntimeHost;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    RuntimeAuxiliaryPathKind, RuntimeBusRole, RuntimeConfig, RuntimeConfigRequest,
    RuntimeLifecycleApi, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_multi_bus_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-multi-bus".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:host-local:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("local host-edge multi-bus block should succeed");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.bus_connection_count, 5);
    assert_eq!(topology.auxiliary_path_count, 3);
    assert!(topology.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == RuntimeBusRole::AuxSend
            && connection.target_bus_role == RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(RuntimeAuxiliaryPathKind::SendReturn)
    }));
    assert!(topology.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:master"
            && path.path_kind == RuntimeAuxiliaryPathKind::Submix
            && path.bus_role == RuntimeBusRole::Submix
    }));
    assert_eq!(report.observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(report.observation.metering_snapshot.auxiliary_path_count, 3);

}
