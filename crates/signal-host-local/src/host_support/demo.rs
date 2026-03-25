use signal_plugin::{
    BlockPayload, ParameterValueEvent, PluginEvent, PluginFormat, PluginSandboxRequest,
    SandboxPolicy, WatchdogTriggerReason,
};
use signal_runtime::{
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxSpec,
    RuntimeWatchdogTrigger, TransportAttachIntent,
};

use super::super::LOCAL_DEMO_PLUGIN_NODE_ID;
use super::demo_graph::{local_demo_graph_contract_projection, local_demo_graph_projection};

#[derive(Clone, Debug)]
pub(crate) struct LocalDemoPluginSandboxAssembly {
    pub(crate) request: PluginSandboxRequest,
    pub(crate) plugin_format: PluginFormat,
    pub(crate) bound_node_ids: Vec<&'static str>,
}

impl LocalDemoPluginSandboxAssembly {
    pub(crate) fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
            plugin_type_id: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LocalDemoRuntimeAssembly {
    pub(crate) graph: signal_runtime::GraphProjection,
    pub(crate) graph_contracts: signal_runtime::GraphContractProjection,
    pub(crate) plugin_sandboxes: Vec<LocalDemoPluginSandboxAssembly>,
}

impl LocalDemoRuntimeAssembly {
    pub(crate) fn primary_sandbox(&self) -> &LocalDemoPluginSandboxAssembly {
        self.plugin_sandboxes
            .first()
            .expect("local demo assembly should define a primary sandbox")
    }

    pub(crate) fn active_plugin_sandbox_count(&self) -> u32 {
        self.plugin_sandboxes.len() as u32
    }

    pub(crate) fn plugin_bindings(&self) -> PluginBackedNodeBindingProjection {
        PluginBackedNodeBindingProjection {
            graph_id: self.graph.graph_id.clone(),
            bindings: self
                .plugin_sandboxes
                .iter()
                .flat_map(|sandbox| {
                    sandbox
                        .bound_node_ids
                        .iter()
                        .map(|node_id| PluginBackedNodeBinding {
                            node_id: (*node_id).into(),
                            sandbox_id: sandbox.request.sandbox_id.clone(),
                        })
                })
                .collect(),
        }
    }
}

pub(crate) fn local_demo_runtime_assembly() -> LocalDemoRuntimeAssembly {
    let graph = local_demo_graph_projection();
    LocalDemoRuntimeAssembly {
        graph_contracts: local_demo_graph_contract_projection(&graph.graph_id),
        graph,
        plugin_sandboxes: vec![LocalDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "local-default-sandbox",
                PluginFormat::Clap,
                SandboxPolicy::Strict,
            ),
            plugin_format: PluginFormat::Clap,
            bound_node_ids: vec![LOCAL_DEMO_PLUGIN_NODE_ID],
        }],
    }
}

pub(crate) fn plugin_automation_value_from_runtime_batch(
    automation_parameter_id: u32,
    parameter_batch: Option<&signal_runtime::ParameterBatch>,
) -> Option<ParameterValueEvent> {
    let parameter_batch = parameter_batch?;
    let value = parameter_batch.events.last()?.normalized_value;
    Some(ParameterValueEvent {
        offset_frames: 0,
        parameter_id: automation_parameter_id,
        normalized_value: value,
    })
}

pub(crate) fn payload_automation_value(
    payload: &BlockPayload,
    automation_parameter_id: u32,
) -> Option<f32> {
    payload.events.events.iter().find_map(|event| match event {
        PluginEvent::ParameterValue(event) if event.parameter_id == automation_parameter_id => {
            Some(event.normalized_value)
        }
        _ => None,
    })
}

pub(crate) fn runtime_watchdog_trigger(reason: WatchdogTriggerReason) -> RuntimeWatchdogTrigger {
    match reason {
        WatchdogTriggerReason::DeadlineMisses => RuntimeWatchdogTrigger::DeadlineMisses,
        WatchdogTriggerReason::HeartbeatMisses => RuntimeWatchdogTrigger::HeartbeatMisses,
    }
}

pub(crate) fn transport_attach_intent(processing_epoch: u64) -> TransportAttachIntent {
    if processing_epoch > 1 {
        TransportAttachIntent::RecoveryOverlap
    } else {
        TransportAttachIntent::SteadyState
    }
}
