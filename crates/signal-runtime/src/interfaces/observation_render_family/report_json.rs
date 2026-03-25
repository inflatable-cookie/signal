use super::*;

pub(crate) fn render_runtime_supervisor_report_json(report: &RuntimeSupervisorReport) -> String {
    let automation = &report.observation.automation_snapshot;
    let automation = if automation.parameter_id == 0
        && automation.lane_count == 0
        && automation.last_batch_epoch.is_none()
    {
        "null".into()
    } else {
        json_runtime_automation_snapshot(automation)
    };
    let plugin_events = &report.observation.plugin_event_snapshot;
    let plugin_events =
        if plugin_events.total_events == 0 && plugin_events.last_processing_epoch.is_none() {
            "null".into()
        } else {
            json_runtime_plugin_event_snapshot(plugin_events)
        };
    let deferred_service = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .map(RuntimeDeferredServiceReceipt::render_json)
        .unwrap_or_else(|| "null".into());
    let core = render_runtime_supervisor_report_core_json(
        report,
        &deferred_service,
        &automation,
        &plugin_events,
    );
    let events = render_runtime_supervisor_report_event_history_json(report);
    format!("{{{core},{events}}}")
}
