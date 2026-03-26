use super::*;

pub(crate) fn json_runtime_canonical_channel_layout(
    layout: Option<RuntimeCanonicalChannelLayout>,
) -> String {
    json_option_string(layout.map(|value| format!("{value:?}")).as_deref())
}

pub(crate) fn json_runtime_channel_role_vec(roles: &[RuntimeChannelRole]) -> String {
    format!(
        "[{}]",
        roles
            .iter()
            .map(|role| match role {
                RuntimeChannelRole::Discrete(index) => {
                    format!(
                        "{{\"kind\":{},\"index\":{}}}",
                        json_option_string(Some("Discrete")),
                        index
                    )
                }
                _ => format!(
                    "{{\"kind\":{}}}",
                    json_option_string(Some(&format!("{role:?}")))
                ),
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_multichannel_layout_summary(
    summary: &RuntimeMultichannelLayoutSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"channel_count\":{},",
            "\"canonical_layout\":{},",
            "\"channel_roles\":{},",
            "\"uses_custom_fallback\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.channel_count,
        json_runtime_canonical_channel_layout(summary.canonical_layout),
        json_runtime_channel_role_vec(&summary.channel_roles),
        summary.uses_custom_fallback,
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(crate) fn json_runtime_bus_intent(intent: RuntimeBusIntent) -> String {
    json_option_string(Some(&format!("{intent:?}")))
}

pub(crate) fn json_runtime_secondary_input_source_kind(
    kind: RuntimeSecondaryInputSourceKind,
) -> String {
    json_option_string(Some(&format!("{kind:?}")))
}

pub(crate) fn json_runtime_secondary_input_target_kind(
    kind: RuntimeSecondaryInputTargetKind,
) -> String {
    json_option_string(Some(&format!("{kind:?}")))
}

pub(crate) fn json_runtime_secondary_input_attachment_policy(
    policy: RuntimeSecondaryInputAttachmentPolicy,
) -> String {
    json_option_string(Some(&format!("{policy:?}")))
}

pub(crate) fn json_runtime_secondary_input_fallback_outcome(
    outcome: RuntimeSecondaryInputFallbackOutcome,
) -> String {
    json_option_string(Some(&format!("{outcome:?}")))
}

pub(crate) fn json_runtime_secondary_input_route_summary(
    summary: &RuntimeSecondaryInputRouteSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"source_kind\":{},",
            "\"source_id\":{},",
            "\"source_bus_id\":{},",
            "\"target_kind\":{},",
            "\"target_id\":{},",
            "\"target_bus_id\":{},",
            "\"attachment_policy\":{},",
            "\"fallback_outcome\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_runtime_secondary_input_source_kind(summary.source_kind),
        json_option_string(Some(summary.source_id.as_str())),
        json_option_string(summary.source_bus_id.as_deref()),
        json_runtime_secondary_input_target_kind(summary.target_kind),
        json_option_string(Some(summary.target_id.as_str())),
        json_option_string(Some(summary.target_bus_id.as_str())),
        json_runtime_secondary_input_attachment_policy(summary.attachment_policy),
        json_runtime_secondary_input_fallback_outcome(summary.fallback_outcome),
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(crate) fn json_runtime_secondary_input_route_summary_vec(
    summaries: &[RuntimeSecondaryInputRouteSummary],
) -> String {
    format!(
        "[{}]",
        summaries
            .iter()
            .map(json_runtime_secondary_input_route_summary)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_bus_role(role: RuntimeBusRole) -> String {
    json_string(&format!("{role:?}"))
}

pub(crate) fn json_runtime_auxiliary_path_kind(kind: RuntimeAuxiliaryPathKind) -> String {
    json_string(&format!("{kind:?}"))
}

pub(crate) fn json_runtime_bus_connection_attachment_class(
    class: RuntimeBusConnectionAttachmentClass,
) -> String {
    json_string(&format!("{class:?}"))
}

pub(crate) fn json_runtime_bus_connection_fallback_outcome(
    outcome: RuntimeBusConnectionFallbackOutcome,
) -> String {
    json_string(&format!("{outcome:?}"))
}

pub(crate) fn json_runtime_bus_connection_summary(summary: &RuntimeBusConnectionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"connection_id\":{},",
            "\"source_node_id\":{},",
            "\"source_bus_id\":{},",
            "\"source_bus_role\":{},",
            "\"target_node_id\":{},",
            "\"target_bus_id\":{},",
            "\"target_bus_role\":{},",
            "\"auxiliary_path_kind\":{},",
            "\"auxiliary_path_id\":{},",
            "\"attachment_class\":{},",
            "\"fallback_outcome\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(summary.connection_id.as_str())),
        json_option_string(Some(summary.source_node_id.as_str())),
        json_option_string(Some(summary.source_bus_id.as_str())),
        json_runtime_bus_role(summary.source_bus_role),
        json_option_string(Some(summary.target_node_id.as_str())),
        json_option_string(Some(summary.target_bus_id.as_str())),
        json_runtime_bus_role(summary.target_bus_role),
        summary
            .auxiliary_path_kind
            .map(json_runtime_auxiliary_path_kind)
            .unwrap_or_else(|| "null".into()),
        json_option_string(summary.auxiliary_path_id.as_deref()),
        json_runtime_bus_connection_attachment_class(summary.attachment_class),
        json_runtime_bus_connection_fallback_outcome(summary.fallback_outcome),
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(crate) fn json_runtime_bus_connection_summary_vec(
    summaries: &[RuntimeBusConnectionSummary],
) -> String {
    format!(
        "[{}]",
        summaries
            .iter()
            .map(json_runtime_bus_connection_summary)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_auxiliary_path_summary(summary: &RuntimeAuxiliaryPathSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"auxiliary_path_id\":{},",
            "\"path_kind\":{},",
            "\"bus_role\":{},",
            "\"material_bus_intent\":{},",
            "\"source_node_ids\":{},",
            "\"target_node_ids\":{},",
            "\"bus_ids\":{},",
            "\"connection_ids\":{},",
            "\"attachment_class\":{},",
            "\"fallback_outcome\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(summary.auxiliary_path_id.as_str())),
        json_runtime_auxiliary_path_kind(summary.path_kind),
        json_runtime_bus_role(summary.bus_role),
        json_runtime_bus_intent(summary.material_bus_intent),
        json_string_vec(&summary.source_node_ids),
        json_string_vec(&summary.target_node_ids),
        json_string_vec(&summary.bus_ids),
        json_string_vec(&summary.connection_ids),
        json_runtime_bus_connection_attachment_class(summary.attachment_class),
        json_runtime_bus_connection_fallback_outcome(summary.fallback_outcome),
        json_option_string(Some(summary.summary.as_str())),
    )
}

pub(crate) fn json_runtime_auxiliary_path_summary_vec(
    summaries: &[RuntimeAuxiliaryPathSummary],
) -> String {
    format!(
        "[{}]",
        summaries
            .iter()
            .map(json_runtime_auxiliary_path_summary)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_runtime_multichannel_io_summary(
    summary: &RuntimeMultichannelIoSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"input_layout\":{},",
            "\"output_layout\":{},",
            "\"input_bus_intent\":{},",
            "\"output_bus_intent\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_runtime_multichannel_layout_summary(&summary.input_layout),
        json_runtime_multichannel_layout_summary(&summary.output_layout),
        json_runtime_bus_intent(summary.input_bus_intent),
        json_runtime_bus_intent(summary.output_bus_intent),
        json_option_string(Some(summary.summary.as_str())),
    )
}
