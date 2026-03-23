use super::*;

pub(super) fn json_runtime_spatial_execution_summary(
    summary: &RuntimeSpatialExecutionSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"node_id\":{},",
            "\"adapter_class\":\"{:?}\",",
            "\"execution_mode\":\"{:?}\",",
            "\"target_environment\":\"{:?}\",",
            "\"control_family\":\"{:?}\",",
            "\"activation_policy\":\"{:?}\",",
            "\"fallback_outcome\":{},",
            "\"bed_class\":\"{:?}\",",
            "\"object_role\":{},",
            "\"object_count\":{},",
            "\"mix_policy\":\"{:?}\",",
            "\"render_scope\":\"{:?}\",",
            "\"expanded_fallback_outcome\":{},",
            "\"immersive_room_policy\":{},",
            "\"deployment_monitoring\":{},",
            "\"renderer_export\":{},",
            "\"balance\":{},",
            "\"input_layout\":{},",
            "\"output_layout\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(summary.node_id.as_str())),
        summary.adapter_class,
        summary.execution_mode,
        summary.target_environment,
        summary.control_family,
        summary.activation_policy,
        json_option_string(
            summary
                .fallback_outcome
                .map(|outcome| format!("{outcome:?}"))
                .as_deref()
        ),
        summary.bed_class,
        json_option_string(
            summary
                .object_role
                .map(|role| format!("{role:?}"))
                .as_deref()
        ),
        summary.object_count,
        summary.mix_policy,
        summary.render_scope,
        json_option_string(
            summary
                .expanded_fallback_outcome
                .map(|outcome| format!("{outcome:?}"))
                .as_deref()
        ),
        summary
            .immersive_room_policy
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_immersive_room_policy_summary),
        summary
            .deployment_monitoring
            .as_ref()
            .map_or_else(|| "null".into(), json_runtime_deployment_monitoring_summary),
        summary.renderer_export.as_ref().map_or_else(
            || "null".into(),
            json_runtime_renderer_immersive_export_summary
        ),
        json_option_string(summary.balance.as_deref()),
        json_runtime_multichannel_layout_summary(&summary.input_layout),
        json_runtime_multichannel_layout_summary(&summary.output_layout),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_renderer_immersive_export_summary(
    summary: &RuntimeRendererImmersiveExportSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"renderer_capability_posture\":\"{:?}\",",
            "\"capability_authority\":\"{:?}\",",
            "\"immersive_export_class\":\"{:?}\",",
            "\"export_authority\":\"{:?}\",",
            "\"export_outcome\":\"{:?}\",",
            "\"summary\":{}",
            "}}"
        ),
        summary.renderer_capability_posture,
        summary.capability_authority,
        summary.immersive_export_class,
        summary.export_authority,
        summary.export_outcome,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_deployment_monitoring_summary(
    summary: &RuntimeDeploymentMonitoringSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"deployment_class\":\"{:?}\",",
            "\"fold_down_policy\":\"{:?}\",",
            "\"monitoring_scene_class\":\"{:?}\",",
            "\"monitoring_scene_authority\":\"{:?}\",",
            "\"monitoring_outcome\":\"{:?}\",",
            "\"summary\":{}",
            "}}"
        ),
        summary.deployment_class,
        summary.fold_down_policy,
        summary.monitoring_scene_class,
        summary.monitoring_scene_authority,
        summary.monitoring_outcome,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_immersive_room_policy_summary(
    summary: &RuntimeImmersiveRoomPolicySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"object_rendering_posture\":\"{:?}\",",
            "\"room_policy_class\":\"{:?}\",",
            "\"room_policy_authority\":\"{:?}\",",
            "\"room_outcome\":\"{:?}\",",
            "\"summary\":{}",
            "}}"
        ),
        summary.object_rendering_posture,
        summary.room_policy_class,
        summary.room_policy_authority,
        summary.room_outcome,
        json_option_string(Some(summary.summary.as_str())),
    )
}
