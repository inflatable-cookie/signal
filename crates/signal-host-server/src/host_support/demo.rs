use signal_plugin::{PluginFormat, PluginSandboxRequest, SandboxPolicy};
use signal_runtime::{
    GraphProjection, PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxSpec,
};

use super::demo_graph::server_demo_graph_projection;

#[derive(Clone, Debug)]
pub(crate) struct ServerDemoPluginSandboxAssembly {
    pub(crate) request: PluginSandboxRequest,
    pub(crate) plugin_format: PluginFormat,
    pub(crate) plugin_type_id: Option<String>,
    pub(crate) bound_node_ids: Vec<&'static str>,
}

impl ServerDemoPluginSandboxAssembly {
    pub(crate) fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
            plugin_type_id: self.plugin_type_id.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ServerDemoRuntimeAssembly {
    pub(crate) graph: GraphProjection,
    pub(crate) scan_roots: Vec<String>,
    pub(crate) scan_formats: Vec<PluginFormat>,
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
    let broker_override = broker_demo_plugin_override();
    ServerDemoRuntimeAssembly {
        graph: server_demo_graph_projection(),
        scan_roots: broker_override
            .as_ref()
            .map(|override_spec| vec![override_spec.scan_root.clone()])
            .unwrap_or_else(|| vec!["/srv/plugins/clap".into()]),
        scan_formats: vec![broker_override
            .as_ref()
            .map(|override_spec| override_spec.plugin_format)
            .unwrap_or(PluginFormat::Clap)],
        plugin_sandboxes: vec![ServerDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "server-default-sandbox",
                broker_override
                    .as_ref()
                    .map(|override_spec| override_spec.plugin_format)
                    .unwrap_or(PluginFormat::Clap),
                SandboxPolicy::Strict,
            ),
            plugin_format: broker_override
                .as_ref()
                .map(|override_spec| override_spec.plugin_format)
                .unwrap_or(PluginFormat::Clap),
            plugin_type_id: broker_override
                .as_ref()
                .map(|override_spec| override_spec.plugin_type_id.clone()),
            bound_node_ids: vec!["drive"],
        }],
    }
}

#[derive(Clone, Debug)]
struct BrokerDemoPluginOverride {
    plugin_format: PluginFormat,
    plugin_type_id: String,
    scan_root: String,
}

fn broker_demo_plugin_override() -> Option<BrokerDemoPluginOverride> {
    let plugin_format = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_FORMAT")
        .ok()
        .and_then(|value| parse_demo_plugin_format(&value))?;
    let plugin_type_id = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID").ok()?;
    let scan_root = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_ROOT").ok()?;
    Some(BrokerDemoPluginOverride {
        plugin_format,
        plugin_type_id,
        scan_root,
    })
}

fn parse_demo_plugin_format(value: &str) -> Option<PluginFormat> {
    match value {
        "clap" => Some(PluginFormat::Clap),
        "vst3" => Some(PluginFormat::Vst3),
        "au" => Some(PluginFormat::Au),
        "lv2" => Some(PluginFormat::Lv2),
        _ => None,
    }
}
