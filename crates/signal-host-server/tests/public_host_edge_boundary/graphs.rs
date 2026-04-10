#[path = "graphs/baseline.rs"]
mod baseline;
#[path = "graphs/routing.rs"]
mod routing;

pub(crate) use baseline::{
    apply_public_capture_graph, apply_public_plugin_continuity_graph, apply_public_render_graph,
};
pub(crate) use routing::{
    apply_public_complex_io_graph, apply_public_multi_bus_graph, apply_public_multichannel_graph,
    apply_public_sidechain_graph, apply_public_spatial_graph,
};
