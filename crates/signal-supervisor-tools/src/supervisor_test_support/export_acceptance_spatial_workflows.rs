use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection, GraphNodeContractProjection,
    GraphNodeTopologyProjection, HandshakeRequest, PluginSandboxLifecycleStage, RuntimeConfig,
    RuntimeConfigRequest, RuntimeEventRecorder, RuntimeLifecycleApi, RuntimeProjectionApi,
    RuntimeSupervisorReport, SignalRuntime,
};

use super::{
    sample_control_preview_workflow_external_midi_snapshot, sample_g07_acceptance_host_io,
};

fn build_spatial_acceptance_runtime(
    client_version: &str,
    graph_id: &str,
    stereo_sandbox_id: &str,
    surround_sandbox_id: &str,
) -> SignalRuntime {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 128));
    runtime
        .handshake(HandshakeRequest {
            client_version: client_version.into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("spatial acceptance export handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 128))
        .expect("spatial acceptance export configure should succeed");
    runtime
        .apply_graph_projection(signal_runtime::GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                signal_runtime::GraphNodeProjection {
                    node_id: "spatial-stereo".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                },
                signal_runtime::GraphNodeProjection {
                    node_id: "spatial-surround".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 20,
                    stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                },
            ],
        })
        .expect("spatial acceptance graph should apply");
    runtime
        .apply_graph_contract_projection(signal_runtime::GraphContractProjection {
            graph_id: graph_id.into(),
            contract_count: 2,
            nodes: vec![
                GraphNodeContractProjection {
                    node_id: "spatial-stereo".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:in".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:stereo".into(),
                            channels: ChannelLayout::Stereo,
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:stereo".into()),
                        bus_group_id: Some("bus:spatial:stereo".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
                GraphNodeContractProjection {
                    node_id: "spatial-surround".into(),
                    buffer_contract: GraphNodeBufferContractProjection {
                        input: GraphNodeBusEndpointProjection {
                            bus_id: "main:surround-in".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        output: GraphNodeBusEndpointProjection {
                            bus_id: "bus:spatial:surround".into(),
                            channels: ChannelLayout::Count(ChannelCount(6)),
                        },
                        ..GraphNodeBufferContractProjection::default()
                    },
                    topology: GraphNodeTopologyProjection {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:surround".into()),
                        bus_group_id: Some("bus:spatial:surround".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                },
            ],
        })
        .expect("spatial acceptance graph contract should apply");
    runtime
        .apply_plugin_backed_node_bindings(signal_runtime::PluginBackedNodeBindingProjection {
            graph_id: graph_id.into(),
            bindings: vec![
                signal_runtime::PluginBackedNodeBinding {
                    node_id: "spatial-stereo".into(),
                    sandbox_id: stereo_sandbox_id.into(),
                },
                signal_runtime::PluginBackedNodeBinding {
                    node_id: "spatial-surround".into(),
                    sandbox_id: surround_sandbox_id.into(),
                },
            ],
        })
        .expect("spatial acceptance bindings should apply");
    runtime.record_plugin_sandbox_lifecycle(
        stereo_sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_lifecycle(
        surround_sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime
}

pub(crate) fn verify_export_json_carries_cross_family_immersive_acceptance_evidence() {
    let runtime = build_spatial_acceptance_runtime(
        "immersive-acceptance-export",
        "graph:supervisor:immersive-acceptance",
        "sandbox:spatial-stereo",
        "sandbox:spatial-surround",
    );
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"execution_topology_summary\":{"));
    assert!(export.contains("\"plugin_chain_snapshot\":{"));
    assert!(export.contains("\"immersive_spatial_node_count\":"));
    assert!(export.contains("\"fallback_monitoring_scene_spatial_node_count\":"));
    assert!(export.contains("\"renderer_capability_spatial_node_count\":"));
    assert!(export.contains("\"immersive_export_spatial_node_count\":"));
    assert!(export.contains("\"immersive_room_policy\":{"));
    assert!(export.contains("\"deployment_monitoring\":{"));
    assert!(export.contains("\"renderer_export\":{"));
}

pub(crate) fn verify_export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence()
{
    let runtime = build_spatial_acceptance_runtime(
        "integrated-live-workflow-acceptance-export",
        "graph:supervisor:integrated-live-workflow",
        "sandbox:integrated-live-stereo",
        "sandbox:integrated-live-surround",
    );
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_g07_acceptance_host_io())
        .with_external_midi_snapshot(sample_control_preview_workflow_external_midi_snapshot());
    let report = RuntimeSupervisorReport {
        observation,
        ..report
    };

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"linux_backend_session_snapshot\":{"));
    assert!(export.contains("\"jack_coordination_snapshot\":{"));
    assert!(export.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(export.contains("\"external_midi_snapshot\":{"));
    assert!(export.contains("\"live_ownership\":{"));
    assert!(export.contains("\"control_surface_snapshot\":{"));
    assert!(export.contains("\"advanced_hardware_snapshot\":{"));
    assert!(export.contains("\"preview_transform_snapshot\":{"));
    assert!(export.contains("\"preview_workflow\":{"));
    assert!(export.contains("\"execution_topology_summary\":{"));
    assert!(export.contains("\"plugin_chain_snapshot\":{"));
    assert!(export.contains("\"immersive_room_policy\":{"));
    assert!(export.contains("\"deployment_monitoring\":{"));
    assert!(export.contains("\"renderer_export\":{"));
}
