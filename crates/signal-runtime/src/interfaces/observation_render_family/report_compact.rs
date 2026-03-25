use super::*;

pub(crate) fn render_runtime_observation_report_compact(
    report: &RuntimeObservationReport,
) -> String {
    let sections = build_runtime_observation_report_compact_sections(report);
    render_runtime_observation_report_compact_engine(report, &sections)
}
