use crate::{
    parse_args, render_conformance_matrix_json, render_conformance_matrix_text,
    render_export_description_json, render_export_description_text, CliArgs, CliMode,
    ExportDebugOptions, HostProfile, HostSummaryDebugSection, OutputFormat, Scenario,
};

#[test]
fn parses_profiles() {
    assert_eq!(HostProfile::parse("local"), Ok(HostProfile::Local));
    assert_eq!(HostProfile::parse("server"), Ok(HostProfile::Server));
}

#[test]
fn parses_scenarios() {
    assert_eq!(Scenario::parse("default"), Ok(Scenario::Default));
    assert_eq!(Scenario::parse("mixed"), Ok(Scenario::Mixed));
    assert_eq!(Scenario::parse("soak"), Ok(Scenario::Soak));
}

#[test]
fn parses_json_flag_and_positionals() {
    assert_eq!(
        parse_args(["--format=json".into(), "local".into(), "mixed".into()]),
        Ok(CliArgs {
            format: OutputFormat::Json,
            debug: ExportDebugOptions { payload: false },
            mode: CliMode::Run {
                profile: HostProfile::Local,
                scenario: Scenario::Mixed,
            },
        })
    );
}

#[test]
fn rejects_missing_positionals() {
    let error = parse_args(["local".into()]).unwrap_err();
    assert!(error.contains("expected"));
}

#[test]
fn only_payload_is_currently_supported_as_debug_section() {
    assert!(ExportDebugOptions { payload: true }.supports(HostSummaryDebugSection::Payload));
    assert_eq!(HostSummaryDebugSection::Payload.label(), "payload");
}

#[test]
fn export_description_text_reports_frozen_policy() {
    let rendered = render_export_description_text();
    assert!(rendered.contains("schema: signal.supervisor.export"));
    assert!(rendered.contains("schema_version: 1"));
    assert!(rendered.contains("default_host_summary_sections: execution,transport,faults"));
    assert!(rendered.contains("supported_debug_sections: payload"));
}

#[test]
fn export_description_json_reports_frozen_policy() {
    let rendered = render_export_description_json();
    assert!(rendered.contains("\"schema\":\"signal.supervisor.export\""));
    assert!(rendered.contains("\"schema_version\":1"));
    assert!(rendered
        .contains("\"default_host_summary_sections\":[\"execution\",\"transport\",\"faults\"]"));
    assert!(rendered.contains("\"supported_debug_sections\":[\"payload\"]"));
}

#[test]
fn conformance_matrix_text_reports_runnable_consumer_boundary() {
    let rendered = render_conformance_matrix_text();
    assert!(rendered.contains("consumer_conformance_matrix:"));
    assert!(rendered.contains("runtime-public-contract-boundary"));
    assert!(rendered.contains("supervisor-export-discovery-consumer"));
    assert!(rendered.contains("plugin-backend-breadth-coverage"));
    assert!(rendered.contains("shared-host-edge-consumer"));
    assert!(rendered.contains("runtime-supervisor-report-demo"));
    assert!(rendered.contains("supervisor-export-schema-description"));
    assert!(rendered.contains("cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports"));
    assert!(rendered.contains("effigy acceptance:plugin-backend-breadth"));
    assert!(rendered.contains("effigy acceptance:host-edge-consumer"));
    assert!(rendered.contains(
        "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json"
    ));
}

#[test]
fn conformance_matrix_json_reports_runnable_consumer_boundary() {
    let rendered = render_conformance_matrix_json();
    assert!(rendered.contains("\"matrix\":\"signal.consumer.conformance\""));
    assert!(rendered.contains("\"entry_count\":6"));
    assert!(rendered.contains("\"id\":\"runtime-public-contract-boundary\""));
    assert!(rendered.contains("\"kind\":\"export-consumer-test\""));
    assert!(rendered.contains("\"crate\":\"signal-supervisor-tools\""));
    assert!(rendered.contains("\"id\":\"plugin-backend-breadth-coverage\""));
    assert!(rendered.contains("\"id\":\"shared-host-edge-consumer\""));
    assert!(rendered
        .contains("\"command\":\"cargo run -p signal-runtime --example supervisor_report_demo\""));
}
