#[path = "rich_graphs/complex_io.rs"]
mod complex_io;
#[path = "rich_graphs/multi_bus.rs"]
mod multi_bus;
#[path = "rich_graphs/spatial.rs"]
mod spatial;

pub(crate) use complex_io::apply_public_complex_io_graph;
pub(crate) use multi_bus::apply_public_multi_bus_graph;
pub(crate) use spatial::apply_public_spatial_graph;
