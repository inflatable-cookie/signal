mod rich_graphs;
mod simple_routing;

pub(crate) use rich_graphs::{
    apply_public_complex_io_graph, apply_public_multi_bus_graph, apply_public_spatial_graph,
};
pub(crate) use simple_routing::{apply_public_multichannel_graph, apply_public_sidechain_graph};
