use std::path::Path;

use clap_sys::{
    entry::clap_plugin_entry,
    factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID},
};

use crate::{hosting::LoadedClapEntry, ClapDiscoveredPluginType};

use super::build;

pub(super) fn discover_from_clap_library(
    library_path: &Path,
    probe_capabilities: bool,
) -> Option<Vec<ClapDiscoveredPluginType>> {
    // Entry loading is shared with hosting (`hosting::LoadedClapEntry`):
    // discovery and the sandbox child dlopen and initialize CLAP entries
    // through the same path. The entry deinitializes on drop.
    let entry = LoadedClapEntry::load(library_path).ok()?;
    Some(unsafe { discover_from_entry(entry.entry(), library_path, probe_capabilities) })
}

unsafe fn discover_from_entry(
    entry: clap_plugin_entry,
    library_path: &Path,
    probe_capabilities: bool,
) -> Vec<ClapDiscoveredPluginType> {
    let Some(get_factory) = entry.get_factory else {
        return Vec::new();
    };
    let factory_ptr = get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr());
    if factory_ptr.is_null() {
        return Vec::new();
    }
    let factory = factory_ptr.cast::<clap_plugin_factory>();
    let Some(get_plugin_count) = (*factory).get_plugin_count else {
        return Vec::new();
    };
    let Some(get_plugin_descriptor) = (*factory).get_plugin_descriptor else {
        return Vec::new();
    };

    let mut discovered = Vec::new();
    for index in 0..get_plugin_count(factory) {
        let descriptor_ptr = get_plugin_descriptor(factory, index);
        if descriptor_ptr.is_null() {
            continue;
        }
        if let Some(plugin) = build::build_discovered_plugin(
            factory,
            descriptor_ptr,
            library_path,
            probe_capabilities,
        ) {
            discovered.push(plugin);
        }
    }
    discovered
}
