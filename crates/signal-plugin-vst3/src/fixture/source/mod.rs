mod factory;
mod prelude;
mod view;
mod vtables;

use factory::factory_fragment;
use prelude::prelude_fragment;
use view::view_fragment;
use vtables::vtables_fragment;

/// Full Rust source of the fixture cdylib.
pub fn vst3_fixture_source(plugin_name: &str) -> String {
    vst3_fixture_source_with_default_bus_channels(plugin_name, 2)
}

pub(crate) fn vst3_fixture_source_with_default_bus_channels(
    plugin_name: &str,
    default_bus_channels: u16,
) -> String {
    format!(
        "{}{}{}{}",
        prelude_fragment(plugin_name),
        vtables_fragment(default_bus_channels),
        view_fragment(),
        factory_fragment(),
    )
}
