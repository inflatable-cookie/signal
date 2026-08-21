mod build;
mod entry;
mod extensions;
mod host;
mod paths;
mod probe;
mod util;

pub(crate) use extensions::{
    audio_buses_from_extension, parameter_descriptors_from_extension, PluginAudioBusDescriptorList,
};
pub(crate) use paths::clap_bundle_binary_for_platform;

pub(crate) fn discover_clap_plugins_for_roots(
    platform: crate::ClapHostPlatform,
    roots: &[String],
    probe_capabilities: bool,
) -> Vec<crate::ClapDiscoveredPluginType> {
    roots
        .iter()
        .flat_map(|root| paths::scan_clap_root(root, platform, probe_capabilities))
        .collect()
}
