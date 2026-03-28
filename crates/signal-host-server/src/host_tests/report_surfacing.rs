use super::super::host_test_support::temp_media_fixture_path;
use super::super::ServerRuntimeHost;
use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
    GraphProjection, HandshakeRequest, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, RuntimeConfig, RuntimeConfigRequest,
    RuntimeExternalIoDeviceChangeState,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeMediaPreviewState, RuntimeObservationApi, RuntimeProjectionApi, SignalRuntime,
};
use std::fs;

#[path = "report_surfacing/baselines.rs"]
mod baselines;
#[path = "report_surfacing/topology_media.rs"]
mod topology_media;
