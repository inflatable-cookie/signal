use super::*;

pub(crate) fn json_runtime_plugin_recall_snapshot(
    snapshot: &RuntimePluginRecallSnapshot,
) -> String {
    let lifecycle_state = snapshot
        .payload
        .lifecycle_state
        .map(|state| format!("{state:?}"));
    let lifecycle_stage = snapshot
        .payload
        .lifecycle_stage
        .map(|stage| format!("{stage:?}"));
    let transport_stage = snapshot
        .payload
        .transport_stage
        .map(|stage| format!("{stage:?}"));
    let last_restart_intent = snapshot
        .payload
        .last_restart_intent
        .map(|intent| format!("{intent:?}"));
    let last_stop_reason = snapshot
        .payload
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    let last_fault_kind = snapshot
        .payload
        .last_fault_kind
        .map(|kind| format!("{kind:?}"));
    let plugin_format = snapshot
        .payload
        .plugin_format
        .map(|format| format!("{format:?}"));
    format!(
        concat!(
            "{{",
            "\"state\":\"{:?}\",",
            "\"payload\":{{",
            "\"sandbox_id\":{},",
            "\"plugin_type_id\":{},",
            "\"plugin_format\":{},",
            "\"lifecycle_state\":{},",
            "\"lifecycle_stage\":{},",
            "\"transport_stage\":{},",
            "\"readiness_state\":{},",
            "\"recovery_count\":{},",
            "\"restart_count\":{},",
            "\"fault_count\":{},",
            "\"last_restart_intent\":{},",
            "\"last_stop_reason\":{},",
            "\"last_fault_kind\":{},",
            "\"last_fault_detail\":{},",
            "\"degraded_reasons\":{},",
            "\"interchange\":{},",
            "\"ara_context\":{}",
            "}},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.state,
        json_option_string(snapshot.payload.sandbox_id.as_deref()),
        json_option_string(snapshot.payload.plugin_type_id.as_deref()),
        json_option_string(plugin_format.as_deref()),
        json_option_string(lifecycle_state.as_deref()),
        json_option_string(lifecycle_stage.as_deref()),
        json_option_string(transport_stage.as_deref()),
        json_option_string(snapshot.payload.readiness_state.as_deref()),
        snapshot.payload.recovery_count,
        snapshot.payload.restart_count,
        snapshot.payload.fault_count,
        json_option_string(last_restart_intent.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_string(last_fault_kind.as_deref()),
        json_option_string(snapshot.payload.last_fault_detail.as_deref()),
        json_string_vec(&snapshot.payload.degraded_reasons),
        json_runtime_plugin_interchange_snapshot(&snapshot.payload.interchange),
        snapshot
            .payload
            .ara_context
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_ara_context_snapshot),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

pub(crate) fn json_runtime_plugin_preset_descriptor(
    descriptor: &RuntimePluginPresetDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"preset_id\":{},",
            "\"label\":{},",
            "\"origin\":\"{:?}\",",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(descriptor.preset_id.as_deref()),
        json_option_string(descriptor.label.as_deref()),
        descriptor.origin,
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_plugin_interchange_snapshot(snapshot: &RuntimePluginInterchangeSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"portability_class\":\"{:?}\",",
            "\"shared_payload_available\":{},",
            "\"native_supplement_required\":{},",
            "\"preset_descriptor\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.portability_class,
        snapshot.shared_payload_available,
        snapshot.native_supplement_required,
        snapshot
            .preset_descriptor
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_preset_descriptor),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_ara_document_context(context: &RuntimePluginAraDocumentContext) -> String {
    format!(
        concat!(
            "{{",
            "\"document_id\":{},",
            "\"display_label\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(context.document_id.as_str())),
        json_option_string(context.display_label.as_deref()),
        json_option_string(Some(context.summary.as_str())),
    )
}

fn json_runtime_plugin_ara_source_context(context: &RuntimePluginAraSourceContext) -> String {
    format!(
        concat!(
            "{{",
            "\"source_id\":{},",
            "\"display_label\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(context.source_id.as_str())),
        json_option_string(context.display_label.as_deref()),
        json_option_string(Some(context.summary.as_str())),
    )
}

fn json_runtime_plugin_ara_region_context(context: &RuntimePluginAraRegionContext) -> String {
    format!(
        concat!(
            "{{",
            "\"region_id\":{},",
            "\"display_label\":{},",
            "\"timeline_start_samples\":{},",
            "\"duration_samples\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(context.region_id.as_str())),
        json_option_string(context.display_label.as_deref()),
        json_option_i64(context.timeline_start_samples),
        json_option_u32(context.duration_samples),
        json_option_string(Some(context.summary.as_str())),
    )
}

pub(crate) fn json_runtime_plugin_ara_context_snapshot(
    snapshot: &RuntimePluginAraContextSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"portability_class\":\"{:?}\",",
            "\"document_context\":{},",
            "\"source_context\":{},",
            "\"region_context\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.portability_class,
        snapshot
            .document_context
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_ara_document_context),
        snapshot
            .source_context
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_ara_source_context),
        snapshot
            .region_context
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_plugin_ara_region_context),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
