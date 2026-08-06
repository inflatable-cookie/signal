#![cfg(target_os = "macos")]

use signal_plugin_au::{AuHostedInstance, AU_REGISTRY_COMPONENT_PATH};
use std::path::Path;

#[test]
fn stock_apple_generator_and_panner_can_be_loaded_and_activated() {
    for load_key in ["augn:sspl:appl", "aupn:ambi:appl"] {
        let mut instance = AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), load_key)
            .unwrap_or_else(|error| panic!("{load_key} failed to load: {error}"));
        instance
            .activate(48_000.0, 64, 512)
            .unwrap_or_else(|error| panic!("{load_key} failed to activate: {error}"));
        instance
            .deactivate()
            .unwrap_or_else(|error| panic!("{load_key} failed to deactivate: {error}"));
    }
}
