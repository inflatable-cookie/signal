#[path = "support/public_contract_boundary_graph_bus.rs"]
mod public_contract_boundary_graph_bus_support;

use public_contract_boundary_graph_bus_support::apply_public_multi_bus_graph;
use signal_graph::synthetic_stereo_block;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, RuntimeAuxiliaryPathKind, RuntimeAuxiliaryPathSummary, RuntimeBusRole,
    RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder, RuntimeLifecycleApi,
    RuntimeObservationReport, SignalRuntime,
};

fn has_aux_path(
    paths: &[RuntimeAuxiliaryPathSummary],
    auxiliary_path_id: &str,
    path_kind: RuntimeAuxiliaryPathKind,
    bus_role: RuntimeBusRole,
) -> bool {
    paths.iter().any(|path| {
        path.auxiliary_path_id == auxiliary_path_id
            && path.path_kind == path_kind
            && path.bus_role == bus_role
    })
}

#[test]
fn public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-multi-bus-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime multi-bus handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime multi-bus configure should succeed");
    apply_public_multi_bus_graph(&mut runtime, "graph:public:multi-bus");
    runtime
        .process_engine_block(
            3,
            5,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2),
        )
        .expect("public runtime multi-bus block should succeed");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.bus_connection_count, 5);
    assert_eq!(topology.auxiliary_path_count, 3);
    assert!(topology.bus_connections.iter().any(|connection| {
        connection.connection_id == "send-fx:bus:fx:plate->return-fx:bus:fx:plate"
            && connection.source_bus_role == RuntimeBusRole::AuxSend
            && connection.target_bus_role == RuntimeBusRole::AuxReturn
            && connection.auxiliary_path_kind == Some(RuntimeAuxiliaryPathKind::SendReturn)
            && connection.auxiliary_path_id.as_deref() == Some("send_return:fx:plate")
    }));
    assert!(has_aux_path(
        &topology.auxiliary_paths,
        "bus_group:mix:master",
        RuntimeAuxiliaryPathKind::Submix,
        RuntimeBusRole::Submix,
    ));
    assert_eq!(observation.metering_snapshot.bus_connection_count, 5);
    assert_eq!(observation.metering_snapshot.auxiliary_path_count, 3);
    assert!(observation
        .metering_snapshot
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id == "return-fx:bus:mix:master->output-main:bus:mix:master"
        }));

}
