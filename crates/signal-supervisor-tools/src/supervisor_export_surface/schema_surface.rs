use signal_runtime::{RuntimeProfilingReceipt, RuntimeSoakReceipt, RuntimeSupervisorReport};

use super::super::{
    conformance_matrix_entries, json_string, HostProfile, OutputFormat, Scenario,
    DEFAULT_HOST_SUMMARY_SECTIONS, EXPORT_SCHEMA, EXPORT_SCHEMA_VERSION, SUPPORTED_DEBUG_SECTIONS,
};

pub(crate) fn render_supervisor_export_json(
    profile: HostProfile,
    scenario: Scenario,
    host_summary: String,
    profiling: &RuntimeProfilingReceipt,
    soak: &RuntimeSoakReceipt,
    supervisor_report: &RuntimeSupervisorReport,
) -> String {
    format!(
        concat!(
            "{{",
            "\"schema\":{},",
            "\"schema_version\":{},",
            "\"profile\":{},",
            "\"scenario\":{},",
            "\"host_summary\":{},",
            "\"profiling_receipt\":{},",
            "\"soak_receipt\":{},",
            "\"supervisor_report\":{}",
            "}}"
        ),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(&format!("{profile:?}")),
        json_string(&format!("{scenario:?}")),
        host_summary,
        profiling.render_json(),
        soak.render_json(),
        supervisor_report.render_json(),
    )
}

pub(crate) fn render_export_description_text() -> String {
    format!(
        "schema: {EXPORT_SCHEMA}\nschema_version: {EXPORT_SCHEMA_VERSION}\ndefault_host_summary_sections: {}\nsupported_debug_sections: {}",
        DEFAULT_HOST_SUMMARY_SECTIONS.join(","),
        SUPPORTED_DEBUG_SECTIONS
            .iter()
            .map(|section| section.label())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_supported_debug_sections_json() -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .map(|section| json_string(section.label()))
        .collect::<Vec<_>>();
    format!("[{}]", sections.join(","))
}

pub(crate) fn render_export_description_json() -> String {
    format!(
        concat!(
            "{{",
            "\"schema\":{},",
            "\"schema_version\":{},",
            "\"default_host_summary_sections\":{},",
            "\"supported_debug_sections\":{}",
            "}}"
        ),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        format!(
            "[{}]",
            DEFAULT_HOST_SUMMARY_SECTIONS
                .iter()
                .map(|section| json_string(section))
                .collect::<Vec<_>>()
                .join(",")
        ),
        render_supported_debug_sections_json(),
    )
}

pub(crate) fn print_export_description(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_export_description_text()),
        OutputFormat::Json => println!("{}", render_export_description_json()),
    }
}

pub(crate) fn render_conformance_matrix_text() -> String {
    let mut rendered = String::from("consumer_conformance_matrix:\n");
    for entry in conformance_matrix_entries() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  command: {}\n  rationale: {}\n",
            entry.id,
            entry.kind.label(),
            entry.crate_name,
            entry.surface,
            entry.command,
            entry.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_conformance_matrix_json() -> String {
    let entries = conformance_matrix_entries()
        .iter()
        .map(|entry| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(entry.id),
                json_string(entry.kind.label()),
                json_string(entry.crate_name),
                json_string(entry.surface),
                json_string(entry.command),
                json_string(entry.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"matrix\":\"signal.consumer.conformance\",",
            "\"entry_count\":{},",
            "\"entries\":[{}]",
            "}}"
        ),
        conformance_matrix_entries().len(),
        entries,
    )
}
