//! Metadata derivation helpers for VST3 introspection.

use signal_plugin::{PluginFeature, PluginIoLayout};
use std::{ffi::c_char, io};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use super::types::*;

pub(crate) fn match_vst3_controller(
    component: &Vst3FactoryClass,
    classes: &[Vst3FactoryClass],
    component_count: usize,
) -> Option<Vst3FactoryClass> {
    let controllers = classes
        .iter()
        .filter(|class| class.role == Vst3FactoryClassRole::Controller)
        .cloned()
        .collect::<Vec<_>>();
    controllers
        .iter()
        .find(|controller| controller.name == component.name)
        .cloned()
        .or_else(|| {
            if component_count == 1 && controllers.len() == 1 {
                controllers.into_iter().next()
            } else {
                None
            }
        })
}

pub(crate) fn derive_io_layout(bundle: &Vst3BundleInfo, is_instrument: bool) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: bundle
            .signal_audio_inputs
            .unwrap_or(if is_instrument { 0 } else { 2 }),
        audio_outputs: bundle.signal_audio_outputs.unwrap_or(2),
        midi_inputs: bundle
            .signal_midi_inputs
            .unwrap_or(if is_instrument { 1 } else { 0 }),
        midi_outputs: bundle.signal_midi_outputs.unwrap_or(0),
    }
}

pub(crate) fn default_features(is_instrument: bool) -> Vec<PluginFeature> {
    if is_instrument {
        vec![PluginFeature::Instrument, PluginFeature::Analyzer]
    } else {
        vec![PluginFeature::AudioEffect, PluginFeature::Utility]
    }
}

pub(crate) fn class_is_instrument(class: &Vst3FactoryClass) -> bool {
    class.subcategories.iter().any(|subcategory| {
        subcategory.eq_ignore_ascii_case("instrument") || subcategory.eq_ignore_ascii_case("synth")
    }) || class.category.eq_ignore_ascii_case("Instrument")
}

pub(crate) fn derive_plugin_type_id(bundle: &Vst3BundleInfo, class: &Vst3FactoryClass) -> String {
    let base = bundle
        .bundle_identifier
        .clone()
        .or_else(|| bundle.bundle_name.clone())
        .unwrap_or_else(|| "vst3-plugin".to_string());
    let bundle_key = sanitize_plugin_id_segment(&base);
    let class_key = class.class_id.to_ascii_lowercase();
    format!("plugin:vst3:{bundle_key}:{class_key}")
}

pub(crate) fn fallback_vendor(bundle: &Vst3BundleInfo, factory_vendor: Option<&str>) -> String {
    factory_vendor
        .map(str::to_string)
        .or_else(|| {
            bundle
                .bundle_identifier
                .as_deref()
                .and_then(|bundle_id| bundle_id.split('.').next_back().map(str::to_string))
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

pub(crate) fn role_from_category(category: &str) -> Vst3FactoryClassRole {
    if category.eq_ignore_ascii_case("Component Controller Class")
        || category.eq_ignore_ascii_case("Controller")
    {
        Vst3FactoryClassRole::Controller
    } else if category.eq_ignore_ascii_case("Audio Module Class")
        || category.eq_ignore_ascii_case("Audio Mix Processor")
        || category.eq_ignore_ascii_case("Instrument")
        || category.eq_ignore_ascii_case("Fx")
    {
        Vst3FactoryClassRole::Component
    } else {
        Vst3FactoryClassRole::Other
    }
}

pub(crate) fn sanitize_plugin_id_segment(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            sanitized.push(lower);
        } else if !sanitized.ends_with('-') {
            sanitized.push('-');
        }
    }
    sanitized.trim_matches('-').to_string()
}

pub(crate) fn plist_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    dict.get(key)
        .and_then(plist::Value::as_string)
        .map(str::to_string)
}

pub(crate) fn plist_u16(dict: &plist::Dictionary, key: &str) -> Option<u16> {
    dict.get(key)
        .and_then(plist::Value::as_signed_integer)
        .and_then(|value| u16::try_from(value).ok())
}

pub(crate) fn plist_string_array(dict: &plist::Dictionary, key: &str) -> Option<Vec<String>> {
    dict.get(key)
        .and_then(plist::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(plist::Value::as_string)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
}

pub(crate) fn parse_feature_list(raw: &str) -> io::Result<Vec<PluginFeature>> {
    raw.split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| match feature {
            "Instrument" => Ok(PluginFeature::Instrument),
            "Analyzer" => Ok(PluginFeature::Analyzer),
            "AudioEffect" => Ok(PluginFeature::AudioEffect),
            "Utility" => Ok(PluginFeature::Utility),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported VST3 feature `{other}`"),
            )),
        })
        .collect::<io::Result<Vec<_>>>()
}

pub(crate) fn bytes_to_upper_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

pub(crate) fn c_char_array_to_string<const N: usize>(value: &[c_char; N]) -> String {
    let bytes = value
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .map(|byte| byte as u8)
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&bytes).trim().to_string()
}

pub(crate) fn plist_to_io_error(error: plist::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn libloading_to_io(error: libloading::Error) -> io::Error {
    io::Error::other(error.to_string())
}

impl Vst3FactoryClass {
    pub(crate) fn vendor(&self) -> Option<String> {
        self.vendor.clone()
    }

    pub(crate) fn version(&self) -> Option<String> {
        self.version.clone()
    }
}
