use signal_plugin::{PluginFormat, PluginSandboxRequest, SandboxPolicy};
use signal_runtime::{
    GraphProjection, PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxSpec,
};

use super::demo_graph::server_demo_graph_projection;

#[derive(Clone, Debug)]
pub(crate) struct ServerDemoPluginSandboxAssembly {
    pub(crate) request: PluginSandboxRequest,
    pub(crate) plugin_format: PluginFormat,
    pub(crate) bound_node_ids: Vec<&'static str>,
}

impl ServerDemoPluginSandboxAssembly {
    pub(crate) fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
            plugin_type_id: None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ServerDemoRuntimeAssembly {
    pub(crate) graph: GraphProjection,
    pub(crate) plugin_sandboxes: Vec<ServerDemoPluginSandboxAssembly>,
}

impl ServerDemoRuntimeAssembly {
    pub(crate) fn primary_sandbox(&self) -> &ServerDemoPluginSandboxAssembly {
        self.plugin_sandboxes
            .first()
            .expect("server demo assembly should define a primary sandbox")
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

pub(crate) fn server_demo_runtime_assembly() -> ServerDemoRuntimeAssembly {
    ServerDemoRuntimeAssembly {
        graph: server_demo_graph_projection(),
        plugin_sandboxes: vec![ServerDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "server-default-sandbox",
                PluginFormat::Clap,
                SandboxPolicy::Strict,
            ),
            plugin_format: PluginFormat::Clap,
            bound_node_ids: vec!["drive"],
        }],
    }
}
