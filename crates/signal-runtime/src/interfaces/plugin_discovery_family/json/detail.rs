use super::*;

pub(super) fn json_runtime_plugin_discovered_type_record_vec(
    records: &[RuntimePluginDiscoveredTypeRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(json_runtime_plugin_discovered_type_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_lv2_extension_capability_summary(
    summary: &RuntimeLv2ExtensionCapabilitySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"worker_capability\":{},",
            "\"urid_capability\":{},",
            "\"patch_capability\":{},",
            "\"negotiated_extension_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", summary.worker_capability)),
        json_string(&format!("{:?}", summary.urid_capability)),
        json_string(&format!("{:?}", summary.patch_capability)),
        summary.negotiated_extension_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_plugin_discovered_type_record(
    record: &RuntimePluginDiscoveredTypeRecord,
) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_id\":{},",
            "\"plugin_id\":{},",
            "\"vendor\":{},",
            "\"name\":{},",
            "\"format\":{},",
            "\"version\":{},",
            "\"features\":{},",
            "\"default_io_layout\":{},",
            "\"default_multichannel_io\":{},",
            "\"complex_io_summary\":{},",
            "\"audio_bus_count\":{},",
            "\"parameter_count\":{},",
            "\"state_contract\":{},",
            "\"processing_contract\":{},",
            "\"lifecycle_contract\":{},",
            "\"lv2_extension_capabilities\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(record.plugin_type_id.as_str())),
        json_option_string(Some(record.plugin_id.as_str())),
        json_option_string(Some(record.vendor.as_str())),
        json_option_string(Some(record.name.as_str())),
        json_option_string(Some(&format!("{:?}", record.format))),
        json_option_string(record.version.as_deref()),
        json_plugin_feature_vec(&record.features),
        json_plugin_io_layout(record.default_io_layout),
        json_runtime_multichannel_io_summary(&record.default_multichannel_io),
        super::discovery::json_runtime_plugin_complex_io_summary(&record.complex_io_summary),
        record.audio_bus_count,
        record.parameter_count,
        json_plugin_state_contract(record.state_contract),
        json_plugin_processing_contract(record.processing_contract),
        json_plugin_lifecycle_contract(record.lifecycle_contract),
        record.lv2_extension_capabilities.as_ref().map_or_else(
            || "null".into(),
            json_runtime_lv2_extension_capability_summary
        ),
        json_option_string(Some(record.summary.as_str())),
    )
}

fn json_runtime_lv2_extension_record(record: &RuntimeLv2ExtensionRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_id\":{},",
            "\"plugin_id\":{},",
            "\"worker_posture\":{},",
            "\"urid_negotiation_posture\":{},",
            "\"patch_exchange_posture\":{},",
            "\"extension_negotiation_state\":{},",
            "\"strongest_lifecycle_state\":{},",
            "\"sandbox_count\":{},",
            "\"active_sandbox_count\":{},",
            "\"faulted_sandbox_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(record.plugin_type_id.as_str())),
        json_option_string(Some(record.plugin_id.as_str())),
        json_string(&format!("{:?}", record.worker_posture)),
        json_string(&format!("{:?}", record.urid_negotiation_posture)),
        json_string(&format!("{:?}", record.patch_exchange_posture)),
        json_string(&format!("{:?}", record.extension_negotiation_state)),
        json_option_string(
            record
                .strongest_lifecycle_state
                .map(|state| format!("{state:?}"))
                .as_deref(),
        ),
        record.sandbox_count,
        record.active_sandbox_count,
        record.faulted_sandbox_count,
        json_option_string(Some(record.summary.as_str())),
    )
}

pub(super) fn json_runtime_lv2_extension_snapshot(
    snapshot: &RuntimeLv2ExtensionSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"plugin_type_count\":{},",
            "\"sandbox_count\":{},",
            "\"worker_required_type_count\":{},",
            "\"worker_guarded_type_count\":{},",
            "\"urid_negotiated_type_count\":{},",
            "\"patch_supported_type_count\":{},",
            "\"negotiated_type_count\":{},",
            "\"guarded_type_count\":{},",
            "\"unavailable_type_count\":{},",
            "\"records\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.plugin_type_count,
        snapshot.sandbox_count,
        snapshot.worker_required_type_count,
        snapshot.worker_guarded_type_count,
        snapshot.urid_negotiated_type_count,
        snapshot.patch_supported_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
        format!(
            "[{}]",
            snapshot
                .records
                .iter()
                .map(json_runtime_lv2_extension_record)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
