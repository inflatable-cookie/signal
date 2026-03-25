mod acceptance_release;
mod device_media;
mod plugin_linux;
mod runtime_boundaries;

use crate::{
    parse_args, CliArgs, CliMode, ExportDebugOptions, HostProfile, OutputFormat, Scenario,
};

fn assert_supports_describe(flag: &str, mode: CliMode) {
    assert_eq!(
        parse_args(["--format=json".into(), flag.into()]),
        Ok(CliArgs {
            format: OutputFormat::Json,
            debug: ExportDebugOptions { payload: false },
            mode,
        })
    );
}

fn assert_rejects_positionals(flag: &str) {
    let error = parse_args([flag.into(), "local".into(), "default".into()]).unwrap_err();
    assert!(error.contains("does not accept"));
}

#[test]
fn parse_args_supports_short_json_flag() {
    assert_eq!(
        parse_args(["--json".into(), "server".into(), "soak".into()]),
        Ok(CliArgs {
            format: OutputFormat::Json,
            debug: ExportDebugOptions { payload: false },
            mode: CliMode::Run {
                profile: HostProfile::Server,
                scenario: Scenario::Soak,
            },
        })
    );
}

#[test]
fn parse_args_supports_include_payload_flag() {
    assert_eq!(
        parse_args([
            "--json".into(),
            "--include-payload".into(),
            "local".into(),
            "default".into(),
        ]),
        Ok(CliArgs {
            format: OutputFormat::Json,
            debug: ExportDebugOptions { payload: true },
            mode: CliMode::Run {
                profile: HostProfile::Local,
                scenario: Scenario::Default,
            },
        })
    );
}

#[test]
fn parse_args_supports_describe_export_mode() {
    assert_supports_describe("--describe-export", CliMode::DescribeExport);
}

#[test]
fn parse_args_rejects_positionals_with_describe_export() {
    assert_rejects_positionals("--describe-export");
}

#[test]
fn parse_args_supports_describe_conformance_matrix_mode() {
    assert_supports_describe(
        "--describe-conformance-matrix",
        CliMode::DescribeConformanceMatrix,
    );
}

#[test]
fn parse_args_rejects_positionals_with_describe_conformance_matrix() {
    assert_rejects_positionals("--describe-conformance-matrix");
}

#[test]
fn parse_args_rejects_multiple_describe_modes() {
    let error = parse_args([
        "--describe-export".into(),
        "--describe-conformance-matrix".into(),
    ])
    .unwrap_err();
    assert!(error.contains("mutually exclusive"));
}
