#![allow(dead_code)]

#[cfg(test)]
pub(crate) fn au_scaffold_component_metadata_contents(plugin_type_id: &str) -> Option<String> {
    let (component_type, component_subtype, manufacturer_code) = match plugin_type_id {
        "plugin:au:instrument" => ("aumu", "sigi", "sigl"),
        "plugin:au:multiout-instrument" => ("aumu", "sigm", "sigl"),
        "plugin:au:utility" => ("aufx", "sigu", "sigl"),
        "plugin:au:bus-fx" => ("aufx", "sigb", "sigl"),
        _ => return None,
    };
    let metadata = match plugin_type_id {
        "plugin:au:instrument" => (
            "Signal",
            "Signal Instrument AU Plugin",
            "0.1.0",
            0,
            2,
            1,
            0,
            "Instrument,Analyzer",
        ),
        "plugin:au:multiout-instrument" => (
            "Signal",
            "Signal Multi Output Instrument AU Plugin",
            "0.1.0",
            0,
            6,
            1,
            0,
            "Instrument,Analyzer",
        ),
        "plugin:au:utility" => (
            "Signal",
            "Signal Utility AU Plugin",
            "0.1.0",
            2,
            2,
            0,
            0,
            "AudioEffect,Utility",
        ),
        "plugin:au:bus-fx" => (
            "Signal",
            "Signal Bus FX AU Plugin",
            "0.1.0",
            4,
            4,
            0,
            0,
            "AudioEffect,Utility",
        ),
        _ => unreachable!(),
    };
    Some(format!(
        "plugin_type_id={}\ncomponent_type={}\ncomponent_subtype={}\nmanufacturer_code={}\nvendor={}\nname={}\nversion={}\naudio_inputs={}\naudio_outputs={}\nmidi_inputs={}\nmidi_outputs={}\nfeatures={}\n",
        plugin_type_id,
        component_type,
        component_subtype,
        manufacturer_code,
        metadata.0,
        metadata.1,
        metadata.2,
        metadata.3,
        metadata.4,
        metadata.5,
        metadata.6,
        metadata.7,
    ))
}
