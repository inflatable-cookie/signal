use std::env;

use signal_host_local::{LocalRuntimeHost, LocalRuntimeHostSummary};
use signal_host_server::{ServerRuntimeHost, ServerRuntimeHostSummary};
use signal_runtime::{
    RuntimeConfig, RuntimeProfilingReceipt, RuntimeSoakReceipt, RuntimeSupervisorReport,
    SignalRuntime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Default,
    Timeout,
    Crash,
    Heartbeat,
    Soak,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostSummaryDebugSection {
    Payload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CliMode {
    Run {
        profile: HostProfile,
        scenario: Scenario,
    },
    DescribeExport,
    DescribeConformanceMatrix,
    DescribeReleaseBoundary,
    DescribeGenerationCloseout,
}

const EXPORT_SCHEMA: &str = "signal.supervisor.export";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_HOST_SUMMARY_SECTIONS: &[&str] = &["execution", "transport", "faults"];
const SUPPORTED_DEBUG_SECTIONS: &[HostSummaryDebugSection] = &[HostSummaryDebugSection::Payload];
const RELEASE_BOUNDARY: &str = "signal.release.boundary";
const RELEASE_VERSION_SOURCE: &str = "workspace.package.version";
const RELEASE_CHANGELOG_PATH: &str = "CHANGELOG.md";
const RELEASE_CONFORMANCE_TASK: &str = "effigy acceptance:conformance --repo .";
const GENERATION_CLOSEOUT: &str = "signal.generation.closeout";
const GENERATION_CLOSEOUT_GENERATION: &str = "g04";
const GENERATION_CLOSEOUT_TASK: &str = "effigy acceptance:g04-closeout --repo .";
const POST_G04_QUEUE_PATH: &str =
    "docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConformanceMatrixEntryKind {
    PublicBoundaryTest,
    ExportConsumerTest,
    Example,
    Introspection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConformanceMatrixEntry {
    id: &'static str,
    kind: ConformanceMatrixEntryKind,
    crate_name: &'static str,
    surface: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReleaseBoundaryArtifactKind {
    Document,
    ExportDescription,
    ConformanceMatrix,
    Example,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryArtifact {
    id: &'static str,
    kind: ReleaseBoundaryArtifactKind,
    path_or_command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationCloseoutValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExportDebugOptions {
    payload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CliArgs {
    format: OutputFormat,
    debug: ExportDebugOptions,
    mode: CliMode,
}

impl HostProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "local" => Ok(Self::Local),
            "server" => Ok(Self::Server),
            _ => Err(format!(
                "unknown profile {value:?}; expected one of: local, server"
            )),
        }
    }
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "timeout" => Ok(Self::Timeout),
            "crash" => Ok(Self::Crash),
            "heartbeat" => Ok(Self::Heartbeat),
            "soak" => Ok(Self::Soak),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!(
                "unknown scenario {value:?}; expected one of: default, timeout, crash, heartbeat, soak, mixed"
            )),
        }
    }
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "unknown format {value:?}; expected one of: text, json"
            )),
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: signal-supervisor-tools [--format text|json] [--include-payload] [--describe-export|--describe-conformance-matrix|--describe-release-boundary|--describe-generation-closeout] <local|server> <default|timeout|crash|heartbeat|soak|mixed>"
    );
}

impl HostSummaryDebugSection {
    fn label(self) -> &'static str {
        match self {
            Self::Payload => "payload",
        }
    }
}

impl ExportDebugOptions {
    fn supports(self, section: HostSummaryDebugSection) -> bool {
        match section {
            HostSummaryDebugSection::Payload => self.payload,
        }
    }
}

impl ConformanceMatrixEntryKind {
    fn label(self) -> &'static str {
        match self {
            Self::PublicBoundaryTest => "public-boundary-test",
            Self::ExportConsumerTest => "export-consumer-test",
            Self::Example => "example",
            Self::Introspection => "introspection",
        }
    }
}

impl ReleaseBoundaryArtifactKind {
    fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::ExportDescription => "export-description",
            Self::ConformanceMatrix => "conformance-matrix",
            Self::Example => "example",
        }
    }
}

fn conformance_matrix_entries() -> &'static [ConformanceMatrixEntry] {
    &[
        ConformanceMatrixEntry {
            id: "runtime-public-contract-boundary",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-runtime",
            surface: "SignalRuntime, RuntimeObservationReport, RuntimeSupervisorReport public reexports",
            command:
                "cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style consumer can capture runtime/export/plugin receipts without private internals.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-discovery-consumer",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-supervisor-tools",
            surface: "signal.supervisor.export JSON carrying runtime-owned plugin discovery catalog",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog",
            rationale:
                "Proves the versioned supervisor export carries the widened discovery boundary without host-local reconstruction.",
        },
        ConformanceMatrixEntry {
            id: "runtime-supervisor-report-demo",
            kind: ConformanceMatrixEntryKind::Example,
            crate_name: "signal-runtime",
            surface: "supervisor_report_demo example",
            command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "Provides a host-free runnable example that emits the stabilized supervisor report surface.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-schema-description",
            kind: ConformanceMatrixEntryKind::Introspection,
            crate_name: "signal-supervisor-tools",
            surface: "signal-supervisor-tools export/conformance schema description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Lets consumers inspect the runnable conformance matrix without reading private implementation detail.",
        },
    ]
}

fn release_boundary_artifacts() -> &'static [ReleaseBoundaryArtifact] {
    &[
        ReleaseBoundaryArtifact {
            id: "workspace-changelog",
            kind: ReleaseBoundaryArtifactKind::Document,
            path_or_command: RELEASE_CHANGELOG_PATH,
            rationale:
                "Every consumer-facing release baseline must carry a human-readable change summary in the workspace changelog.",
        },
        ReleaseBoundaryArtifact {
            id: "supervisor-export-description",
            kind: ReleaseBoundaryArtifactKind::ExportDescription,
            path_or_command: "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
            rationale:
                "The versioned export schema remains the machine-readable release contract for automation.",
        },
        ReleaseBoundaryArtifact {
            id: "consumer-conformance-matrix",
            kind: ReleaseBoundaryArtifactKind::ConformanceMatrix,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Consumers need one inspectable list of the runnable proof surfaces included in the baseline.",
        },
        ReleaseBoundaryArtifact {
            id: "runtime-supervisor-report-demo",
            kind: ReleaseBoundaryArtifactKind::Example,
            path_or_command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "The human-readable report example remains part of the first shared release baseline for manual inspection.",
        },
    ]
}

fn release_boundary_validation_steps() -> &'static [ReleaseBoundaryValidationStep] {
    &[
        ReleaseBoundaryValidationStep {
            id: "consumer-conformance",
            command: RELEASE_CONFORMANCE_TASK,
            rationale:
                "The runnable consumer conformance matrix must pass before the packaging baseline is considered valid.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-health",
            command: "effigy health --repo .",
            rationale:
                "The repo-owned build baseline must stay healthy for a release-boundary claim to be credible.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-test",
            command: "effigy test --repo .",
            rationale:
                "The shared repo-owned test surface remains part of the packaging baseline rather than downstream-only policy.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-validate",
            command: "effigy validate --repo .",
            rationale:
                "Validation must include the repo-owned configure/build/test chain before a release boundary is declared.",
        },
    ]
}

fn release_boundary_unstable_scopes() -> &'static [&'static str] {
    &[
        "backend breadth beyond the current CLAP-first plugin path",
        "host convenience APIs outside the frozen runtime/export boundary",
        "crates.io publication and downstream release orchestration",
        "artifact packaging beyond changelog plus host-free boundary descriptions",
    ]
}

fn generation_closeout_validation_steps() -> &'static [GenerationCloseoutValidationStep] {
    &[
        GenerationCloseoutValidationStep {
            id: "release-boundary-baseline",
            command: "effigy acceptance:release-boundary --repo .",
            rationale:
                "The combined closeout must include the full conformance matrix plus the explicit release-packaging baseline.",
        },
        GenerationCloseoutValidationStep {
            id: "generation-closeout-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json",
            rationale:
                "Consumers and maintainers need one host-free machine-readable closeout record for the completed generation.",
        },
        GenerationCloseoutValidationStep {
            id: "repo-validation",
            command: "effigy validate --repo .",
            rationale:
                "Generation closure still requires the repo-owned configure/build/test chain to stay green.",
        },
    ]
}

fn generation_closeout_residual_risks() -> &'static [&'static str] {
    &[
        "non-CLAP plugin backend breadth remains deferred beyond the current conformance boundary",
        "host convenience APIs and downstream orchestration remain outside the first stable release promise",
        "publication-grade packaging beyond changelog plus host-free boundary descriptions still needs a later queue",
    ]
}

fn generation_closeout_next_queue_summary() -> &'static str {
    "Promote the post-g04 queue when maintainers want broader backend-neutral consumer breadth, publication-ready packaging, or longer-running downstream conformance automation."
}

fn render_host_summary_sections_text(debug: ExportDebugOptions) -> String {
    let mut sections = DEFAULT_HOST_SUMMARY_SECTIONS.join(",");
    if debug.supports(HostSummaryDebugSection::Payload) {
        sections.push(',');
        sections.push_str(HostSummaryDebugSection::Payload.label());
    }
    format!("sections: {sections}\n")
}

fn render_supported_debug_sections_text() -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .map(|section| section.label())
        .collect::<Vec<_>>()
        .join(",");
    format!("debug_sections_supported: {sections}\n")
}

fn render_enabled_debug_sections_text(debug: ExportDebugOptions) -> String {
    let enabled = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .copied()
        .filter(|section| debug.supports(*section))
        .map(|section| section.label())
        .collect::<Vec<_>>();
    if enabled.is_empty() {
        "debug_sections_enabled: none\n".into()
    } else {
        format!("debug_sections_enabled: {}\n", enabled.join(","))
    }
}

fn render_host_summary_sections_json(debug: ExportDebugOptions) -> String {
    let mut sections: Vec<String> = DEFAULT_HOST_SUMMARY_SECTIONS
        .iter()
        .map(|section| json_string(section))
        .collect();
    if debug.supports(HostSummaryDebugSection::Payload) {
        sections.push(json_string(HostSummaryDebugSection::Payload.label()));
    }
    format!("[{}]", sections.join(","))
}

fn render_supported_debug_sections_json() -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .map(|section| json_string(section.label()))
        .collect::<Vec<_>>();
    format!("[{}]", sections.join(","))
}

fn render_enabled_debug_sections_json(debug: ExportDebugOptions) -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .copied()
        .filter(|section| debug.supports(*section))
        .map(|section| json_string(section.label()))
        .collect::<Vec<_>>();
    format!("[{}]", sections.join(","))
}

fn render_local_payload_text(summary: &LocalRuntimeHostSummary) -> String {
    format!(
        "\npayload: events={} parameter_events={} parameter_gestures={} parameter_modulations={} note_events={} note_expression_events={} midi_events={} generated_event_bytes={} first_output_sample={:?}",
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        summary.last_payload.first_output_sample,
    )
}

fn render_server_payload_text(summary: &ServerRuntimeHostSummary) -> String {
    format!(
        "\npayload: events={} parameter_events={} parameter_gestures={} parameter_modulations={} note_events={} note_expression_events={} midi_events={} generated_event_bytes={} first_output_sample={:?}",
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        summary.last_payload.first_output_sample,
    )
}

fn render_local_summary(summary: &LocalRuntimeHostSummary, debug: ExportDebugOptions) -> String {
    let mut rendered = format!(
        "profile=Local backend={}\n{}{}{}execution: sandbox={:?} processed_blocks={} completion={:?} last_block={} control_requests={} control_responses={} heartbeat_responses={} last_control_message={:?} epoch={} restarts={} teardowns={} last_recovery_intent={:?} last_stop_reason={:?}\ntransport: lease_id={:?} region_id={:?} shared_memory_bytes={}\nfaults: deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?}",
        summary.backend_name,
        render_host_summary_sections_text(debug),
        render_supported_debug_sections_text(),
        render_enabled_debug_sections_text(debug),
        summary.transport.sandbox_id,
        summary.execution.processed_blocks,
        summary.execution.last_completion_state,
        summary.execution.last_block_sequence,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.last_control_message,
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        summary.execution.last_recovery_intent,
        summary.execution.last_stop_reason,
        summary.transport.shared_memory_lease_id,
        summary.transport.shared_memory_region_id,
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        summary.faults.watchdog_trigger_reason,
    );
    rendered.push_str(&format!(
        "\nengine: processed_blocks={} graph_id={:?} output_peak={:?} output_rms={:?}",
        summary.execution.engine_processed_blocks,
        summary.execution.last_engine_graph_id,
        summary.execution.last_engine_output_peak,
        summary.execution.last_engine_output_rms,
    ));
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push_str(&render_local_payload_text(summary));
    }
    rendered
}

fn render_server_summary(summary: &ServerRuntimeHostSummary, debug: ExportDebugOptions) -> String {
    let mut rendered = format!(
        "profile=Server\n{}{}{}execution: sandbox={:?} processed_blocks={} completion={:?} last_block={} control_requests={} control_responses={} heartbeat_responses={} last_control_message={:?} epoch={} restarts={} teardowns={} last_recovery_intent={:?} last_stop_reason={:?}\ntransport: lease_id={:?} region_id={:?} shared_memory_bytes={}\nfaults: deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?}",
        render_host_summary_sections_text(debug),
        render_supported_debug_sections_text(),
        render_enabled_debug_sections_text(debug),
        summary.transport.sandbox_id,
        summary.execution.processed_blocks,
        summary.execution.last_completion_state,
        summary.execution.last_block_sequence,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.last_control_message,
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        summary.execution.last_recovery_intent,
        summary.execution.last_stop_reason,
        summary.transport.shared_memory_lease_id,
        summary.transport.shared_memory_region_id,
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        summary.faults.watchdog_trigger_reason,
    );
    rendered.push_str(&format!(
        "\nengine: processed_blocks={} graph_id={:?} output_peak={:?} output_rms={:?}",
        summary.execution.engine_processed_blocks,
        summary.execution.last_engine_graph_id,
        summary.execution.last_engine_output_peak,
        summary.execution.last_engine_output_rms,
    ));
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push_str(&render_server_payload_text(summary));
    }
    rendered
}

fn render_local_payload_json(summary: &LocalRuntimeHostSummary) -> String {
    format!(
        concat!(
            "\"payload\":{{",
            "\"events\":{},",
            "\"parameter_events\":{},",
            "\"parameter_gestures\":{},",
            "\"parameter_modulations\":{},",
            "\"note_events\":{},",
            "\"note_expression_events\":{},",
            "\"midi_events\":{},",
            "\"generated_event_bytes\":{},",
            "\"first_output_sample\":{}",
            "}}"
        ),
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        json_option_f32(summary.last_payload.first_output_sample),
    )
}

fn render_server_payload_json(summary: &ServerRuntimeHostSummary) -> String {
    format!(
        concat!(
            "\"payload\":{{",
            "\"events\":{},",
            "\"parameter_events\":{},",
            "\"parameter_gestures\":{},",
            "\"parameter_modulations\":{},",
            "\"note_events\":{},",
            "\"note_expression_events\":{},",
            "\"midi_events\":{},",
            "\"generated_event_bytes\":{},",
            "\"first_output_sample\":{}",
            "}}"
        ),
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        json_option_f32(summary.last_payload.first_output_sample),
    )
}

fn render_local_summary_json(
    summary: &LocalRuntimeHostSummary,
    debug: ExportDebugOptions,
) -> String {
    let mut rendered = format!(
        concat!(
            "{{",
            "\"profile\":\"Local\",",
            "\"backend\":{},",
            "\"sections\":{},",
            "\"debug_sections_supported\":{},",
            "\"debug_sections_enabled\":{},",
            "\"execution\":{{",
            "\"sandbox_id\":{},",
            "\"control_requests\":{},",
            "\"control_responses\":{},",
            "\"heartbeat_responses\":{},",
            "\"processed_blocks\":{},",
            "\"engine_processed_blocks\":{},",
            "\"last_completion_state\":{},",
            "\"last_block_sequence\":{},",
            "\"last_control_message\":{},",
            "\"last_engine_graph_id\":{},",
            "\"last_engine_output_peak\":{},",
            "\"last_engine_output_rms\":{},",
            "\"processing_epoch\":{},",
            "\"restart_count\":{},",
            "\"teardown_count\":{},",
            "\"last_recovery_intent\":{},",
            "\"last_stop_reason\":{}",
            "}},",
            "\"transport\":{{",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"shared_memory_path\":{},",
            "\"shared_memory_bytes\":{}",
            "}},",
            "\"faults\":{{",
            "\"deadline_misses\":{},",
            "\"heartbeat_misses\":{},",
            "\"watchdog_triggered\":{},",
            "\"watchdog_trigger_reason\":{}",
            "}}"
        ),
        json_string(summary.backend_name),
        render_host_summary_sections_json(debug),
        render_supported_debug_sections_json(),
        render_enabled_debug_sections_json(debug),
        json_string(&summary.transport.sandbox_id),
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.engine_processed_blocks,
        json_string(&format!("{:?}", summary.execution.last_completion_state)),
        summary.execution.last_block_sequence,
        json_string(&summary.execution.last_control_message),
        summary
            .execution
            .last_engine_graph_id
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_option_f32(summary.execution.last_engine_output_peak),
        json_option_f32(summary.execution.last_engine_output_rms),
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        json_option_debug(summary.execution.last_recovery_intent),
        json_option_debug(summary.execution.last_stop_reason),
        json_string(&summary.transport.shared_memory_lease_id),
        json_string(&summary.transport.shared_memory_region_id),
        json_string(&summary.transport.shared_memory_path),
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        json_option_debug(summary.faults.watchdog_trigger_reason),
    );
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push(',');
        rendered.push_str(&render_local_payload_json(summary));
    }
    rendered.push('}');
    rendered
}

fn render_server_summary_json(
    summary: &ServerRuntimeHostSummary,
    debug: ExportDebugOptions,
) -> String {
    let mut rendered = format!(
        concat!(
            "{{",
            "\"profile\":\"Server\",",
            "\"sections\":{},",
            "\"debug_sections_supported\":{},",
            "\"debug_sections_enabled\":{},",
            "\"execution\":{{",
            "\"sandbox_id\":{},",
            "\"control_requests\":{},",
            "\"control_responses\":{},",
            "\"heartbeat_responses\":{},",
            "\"processed_blocks\":{},",
            "\"engine_processed_blocks\":{},",
            "\"last_completion_state\":{},",
            "\"last_block_sequence\":{},",
            "\"last_control_message\":{},",
            "\"last_engine_graph_id\":{},",
            "\"last_engine_output_peak\":{},",
            "\"last_engine_output_rms\":{},",
            "\"processing_epoch\":{},",
            "\"restart_count\":{},",
            "\"teardown_count\":{},",
            "\"last_recovery_intent\":{},",
            "\"last_stop_reason\":{}",
            "}},",
            "\"transport\":{{",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"shared_memory_path\":{},",
            "\"shared_memory_bytes\":{}",
            "}},",
            "\"faults\":{{",
            "\"deadline_misses\":{},",
            "\"heartbeat_misses\":{},",
            "\"watchdog_triggered\":{},",
            "\"watchdog_trigger_reason\":{}",
            "}}"
        ),
        render_host_summary_sections_json(debug),
        render_supported_debug_sections_json(),
        render_enabled_debug_sections_json(debug),
        json_string(&summary.transport.sandbox_id),
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.engine_processed_blocks,
        json_string(&format!("{:?}", summary.execution.last_completion_state)),
        summary.execution.last_block_sequence,
        json_string(&summary.execution.last_control_message),
        summary
            .execution
            .last_engine_graph_id
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_option_f32(summary.execution.last_engine_output_peak),
        json_option_f32(summary.execution.last_engine_output_rms),
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        json_option_debug(summary.execution.last_recovery_intent),
        json_option_debug(summary.execution.last_stop_reason),
        json_string(&summary.transport.shared_memory_lease_id),
        json_string(&summary.transport.shared_memory_region_id),
        json_string(&summary.transport.shared_memory_path),
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        json_option_debug(summary.faults.watchdog_trigger_reason),
    );
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push(',');
        rendered.push_str(&render_server_payload_json(summary));
    }
    rendered.push('}');
    rendered
}

fn json_option_f32(value: Option<f32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn render_supervisor_export_json(
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

fn render_export_description_text() -> String {
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

fn render_export_description_json() -> String {
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

fn print_export_description(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_export_description_text()),
        OutputFormat::Json => println!("{}", render_export_description_json()),
    }
}

fn render_conformance_matrix_text() -> String {
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

fn render_conformance_matrix_json() -> String {
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

fn print_conformance_matrix(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_conformance_matrix_text()),
        OutputFormat::Json => println!("{}", render_conformance_matrix_json()),
    }
}

fn render_release_boundary_text() -> String {
    let mut rendered = format!(
        "release_boundary: {RELEASE_BOUNDARY}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\nchangelog_path: {RELEASE_CHANGELOG_PATH}\nexport_schema: {EXPORT_SCHEMA}\nexport_schema_version: {EXPORT_SCHEMA_VERSION}\nconformance_task: {RELEASE_CONFORMANCE_TASK}\nartifacts:\n",
        env!("CARGO_PKG_VERSION")
    );
    for artifact in release_boundary_artifacts() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            artifact.id,
            artifact.kind.label(),
            artifact.path_or_command,
            artifact.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in release_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("intentionally_unstable:\n");
    for scope in release_boundary_unstable_scopes() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_release_boundary_json() -> String {
    let artifacts = release_boundary_artifacts()
        .iter()
        .map(|artifact| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(artifact.id),
                json_string(artifact.kind.label()),
                json_string(artifact.path_or_command),
                json_string(artifact.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = release_boundary_validation_steps()
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let unstable = release_boundary_unstable_scopes()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"changelog_path\":{},",
            "\"export_schema\":{},",
            "\"export_schema_version\":{},",
            "\"conformance_task\":{},",
            "\"artifacts\":[{}],",
            "\"validation_steps\":[{}],",
            "\"intentionally_unstable\":[{}]",
            "}}"
        ),
        json_string(RELEASE_BOUNDARY),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(RELEASE_CHANGELOG_PATH),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(RELEASE_CONFORMANCE_TASK),
        artifacts,
        validation_steps,
        unstable,
    )
}

fn print_release_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_release_boundary_text()),
        OutputFormat::Json => println!("{}", render_release_boundary_json()),
    }
}

fn render_generation_closeout_text() -> String {
    let mut rendered = format!(
        "generation_closeout: {GENERATION_CLOSEOUT}\ngeneration: {GENERATION_CLOSEOUT_GENERATION}\ncloseout_task: {GENERATION_CLOSEOUT_TASK}\nconformance_matrix_command: cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json\nrelease_boundary_command: cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json\npost_g04_queue_path: {POST_G04_QUEUE_PATH}\nvalidation_steps:\n"
    );
    for step in generation_closeout_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("residual_risks:\n");
    for risk in generation_closeout_residual_risks() {
        rendered.push_str(&format!("- {risk}\n"));
    }
    rendered.push_str(&format!(
        "next_queue_summary: {}\n",
        generation_closeout_next_queue_summary()
    ));
    rendered
}

fn render_generation_closeout_json() -> String {
    let validation_steps = generation_closeout_validation_steps()
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let residual_risks = generation_closeout_residual_risks()
        .iter()
        .map(|risk| json_string(risk))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"closeout\":{},",
            "\"generation\":{},",
            "\"closeout_task\":{},",
            "\"conformance_matrix_command\":{},",
            "\"release_boundary_command\":{},",
            "\"post_g04_queue_path\":{},",
            "\"validation_steps\":[{}],",
            "\"residual_risks\":[{}],",
            "\"next_queue_summary\":{}",
            "}}"
        ),
        json_string(GENERATION_CLOSEOUT),
        json_string(GENERATION_CLOSEOUT_GENERATION),
        json_string(GENERATION_CLOSEOUT_TASK),
        json_string(
            "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
        ),
        json_string(
            "cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json",
        ),
        json_string(POST_G04_QUEUE_PATH),
        validation_steps,
        residual_risks,
        json_string(generation_closeout_next_queue_summary()),
    )
}

fn print_generation_closeout(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_generation_closeout_text()),
        OutputFormat::Json => println!("{}", render_generation_closeout_json()),
    }
}

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    match value {
        Some(value) => json_string(&format!("{value:?}")),
        None => "null".into(),
    }
}

fn print_report(
    format: OutputFormat,
    profile: HostProfile,
    scenario: Scenario,
    summary: String,
    profiling: &RuntimeProfilingReceipt,
    soak: &RuntimeSoakReceipt,
    report: RuntimeSupervisorReport,
) {
    match format {
        OutputFormat::Text => println!(
            "signal-supervisor-tools profile={profile:?} scenario={scenario:?}\n{summary}\nprofiling:\n{}\nsoak:\n{}\nsupervisor:\n{}",
            profiling.render_multiline(),
            soak.render_multiline(),
            report.render_multiline()
        ),
        OutputFormat::Json => println!(
            "{}",
            render_supervisor_export_json(profile, scenario, summary, profiling, soak, &report)
        ),
    }
}

fn run_local(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let host_report = host.host_supervisor_report();
    let profiling = host_report.profiling_receipt();
    let soak = host_report.soak_receipt();
    print_report(
        format,
        HostProfile::Local,
        scenario,
        match format {
            OutputFormat::Text => render_local_summary(&summary, debug),
            OutputFormat::Json => render_local_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn run_server(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let profiling = report.profiling_receipt();
    let soak = report.soak_receipt();
    print_report(
        format,
        HostProfile::Server,
        scenario,
        match format {
            OutputFormat::Text => render_server_summary(&summary, debug),
            OutputFormat::Json => render_server_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut debug = ExportDebugOptions::default();
    let mut describe_export = false;
    let mut describe_conformance_matrix = false;
    let mut describe_release_boundary = false;
    let mut describe_generation_closeout = false;
    let mut positional = Vec::new();

    for arg in args {
        if arg == "--json" {
            format = OutputFormat::Json;
            continue;
        }
        if arg == "--text" {
            format = OutputFormat::Text;
            continue;
        }
        if arg == "--include-payload" {
            debug.payload = true;
            continue;
        }
        if arg == "--describe-export" {
            describe_export = true;
            continue;
        }
        if arg == "--describe-conformance-matrix" {
            describe_conformance_matrix = true;
            continue;
        }
        if arg == "--describe-release-boundary" {
            describe_release_boundary = true;
            continue;
        }
        if arg == "--describe-generation-closeout" {
            describe_generation_closeout = true;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = OutputFormat::parse(value)?;
            continue;
        }
        positional.push(arg);
    }

    let describe_mode_count = [
        describe_export,
        describe_conformance_matrix,
        describe_release_boundary,
        describe_generation_closeout,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if describe_mode_count > 1 {
        return Err("describe modes are mutually exclusive".into());
    }

    if describe_export {
        if !positional.is_empty() {
            return Err(
                "`--describe-export` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExport,
        });
    }

    if describe_conformance_matrix {
        if !positional.is_empty() {
            return Err(
                "`--describe-conformance-matrix` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeConformanceMatrix,
        });
    }

    if describe_release_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-release-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeReleaseBoundary,
        });
    }

    if describe_generation_closeout {
        if !positional.is_empty() {
            return Err(
                "`--describe-generation-closeout` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeGenerationCloseout,
        });
    }

    if positional.len() != 2 {
        return Err("expected <profile> <scenario>".into());
    }

    Ok(CliArgs {
        format,
        debug,
        mode: CliMode::Run {
            profile: HostProfile::parse(&positional[0])?,
            scenario: Scenario::parse(&positional[1])?,
        },
    })
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            std::process::exit(2);
        }
    };

    let result = match args.mode {
        CliMode::Run { profile, scenario } => match profile {
            HostProfile::Local => run_local(args.format, args.debug, scenario),
            HostProfile::Server => run_server(args.format, args.debug, scenario),
        },
        CliMode::DescribeExport => {
            print_export_description(args.format);
            Ok(())
        }
        CliMode::DescribeConformanceMatrix => {
            print_conformance_matrix(args.format);
            Ok(())
        }
        CliMode::DescribeReleaseBoundary => {
            print_release_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeGenerationCloseout => {
            print_generation_closeout(args.format);
            Ok(())
        }
    };

    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_args, render_conformance_matrix_json, render_conformance_matrix_text,
        render_export_description_json, render_export_description_text,
        render_generation_closeout_json, render_generation_closeout_text,
        render_release_boundary_json, render_release_boundary_text, render_supervisor_export_json,
        CliArgs, CliMode, ExportDebugOptions, HostProfile, HostSummaryDebugSection, OutputFormat,
        Scenario,
    };
    use signal_hardware::{
        AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract,
        HardwareLifecycleOwnership, HardwareRestartPolicy,
    };
    use signal_host_local::host::{
        LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy,
        LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary,
        LocalTransportSummary,
    };
    use signal_host_local::{LocalRuntimeHostSummary, RecoveryRestartIntent};
    use signal_plugin::{
        CompletionState, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
        PluginProcessingContract, PluginStateContract, WatchdogTriggerReason,
    };
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        HeartbeatCycleStage, PluginSandboxLifecycleStage, PluginSandboxSpec,
        PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeEvent,
        RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionTopologySummary,
        RuntimeLifecycleApi, RuntimeOfflineRenderPurgeRequest, RuntimePluginDiscoveredTypeRecord,
        RuntimeSupervisorReport, SafeModeRequest, SandboxOperationFailureStage, SignalRuntime,
        StopReason, TransportDispatchState, TransportHeartbeatFreshness, TransportSessionState,
    };

    fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:export-consumer".into(),
            plugin_id: "com.signal.export-consumer".into(),
            vendor: "Signal".into(),
            name: "Signal Export Consumer".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
            default_io_layout: PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            audio_bus_count: 2,
            parameter_count: 12,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 4096,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                produces_midi: true,
                silence_aware: true,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            summary: "supervisor export discovered plugin".into(),
        }
    }

    fn sample_local_summary() -> LocalRuntimeHostSummary {
        LocalRuntimeHostSummary {
            backend_name: "coreaudio",
            hardware: LocalHardwareSummary {
                device_id: "coreaudio:default-output".into(),
                device_name: "CoreAudio Default Output".into(),
                sample_rate: 48_000,
                buffer_size: 512,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                lifecycle: HardwareLifecycleContract {
                    ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                    restart_policy: HardwareRestartPolicy::HostMustRestart,
                },
                simulated: false,
                backend_diagnostics: HardwareDiagnosticsSnapshot::healthy(),
            },
            audio_pump: LocalAudioPumpSummary {
                stream_state: LocalAudioStreamState::Running,
                transfer_policy: LocalAudioTransferPolicy {
                    max_callback_frames: 512,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 3,
                last_callback_index: Some(2),
                total_callback_frames: 1536,
                total_runtime_output_frames: 1536,
                copied_output_samples: 3072,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.8),
                last_runtime_graph_id: Some("signal.host.local.demo".into()),
            },
            scan_roots: vec!["/plugins".into()],
            execution: LocalExecutionSummary {
                control_requests: 4,
                control_responses: 4,
                heartbeat_responses: 2,
                processed_blocks: 3,
                engine_processed_blocks: 3,
                last_control_message: "activateInstance".into(),
                last_completion_state: CompletionState::Completed,
                last_block_sequence: 7,
                last_engine_graph_id: Some("signal.host.local.demo".into()),
                last_engine_output_peak: Some(0.8),
                last_engine_output_rms: Some(0.42),
                processing_epoch: 2,
                restart_count: 1,
                teardown_count: 1,
                last_recovery_intent: Some(RecoveryRestartIntent::WatchdogRecovery),
                last_stop_reason: Some(StopReason::DegradedModeRecovery),
                last_plugin_state: None,
            },
            transport: LocalTransportSummary {
                sandbox_id: "sandbox-1".into(),
                shared_memory_lease_id: "lease-1".into(),
                shared_memory_region_id: "region-1".into(),
                shared_memory_path: "/tmp/signal-region-1".into(),
                shared_memory_bytes: 4096,
            },
            topology: RuntimeExecutionTopologySummary::default(),
            plugin_dispatch: None,
            last_payload: LocalPayloadSummary {
                event_count: 6,
                parameter_event_count: 2,
                parameter_gesture_event_count: 2,
                parameter_modulation_event_count: 1,
                note_event_count: 1,
                note_expression_event_count: 1,
                midi_event_count: 1,
                generated_event_bytes: 128,
                first_output_sample: Some(0.5),
            },
            faults: LocalFaultSummary {
                deadline_misses: 1,
                heartbeat_misses: 0,
                watchdog_triggered: true,
                watchdog_trigger_reason: Some(WatchdogTriggerReason::DeadlineMisses),
            },
        }
    }

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
            parse_args(["--format=json".into(), "local".into(), "mixed".into(),]),
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
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-export".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExport,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_export() {
        let error =
            parse_args(["--describe-export".into(), "local".into(), "default".into()]).unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_conformance_matrix_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-conformance-matrix".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeConformanceMatrix,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_conformance_matrix() {
        let error = parse_args([
            "--describe-conformance-matrix".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
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

    #[test]
    fn parse_args_supports_describe_release_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-release-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeReleaseBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_release_boundary() {
        let error = parse_args([
            "--describe-release-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_generation_closeout_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-generation-closeout".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeGenerationCloseout,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_generation_closeout() {
        let error = parse_args([
            "--describe-generation-closeout".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
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
        assert!(rendered.contains(
            "\"default_host_summary_sections\":[\"execution\",\"transport\",\"faults\"]"
        ));
        assert!(rendered.contains("\"supported_debug_sections\":[\"payload\"]"));
    }

    #[test]
    fn conformance_matrix_text_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_text();
        assert!(rendered.contains("consumer_conformance_matrix:"));
        assert!(rendered.contains("runtime-public-contract-boundary"));
        assert!(rendered.contains("supervisor-export-discovery-consumer"));
        assert!(rendered.contains("runtime-supervisor-report-demo"));
        assert!(rendered.contains("supervisor-export-schema-description"));
        assert!(rendered.contains("cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json"
        ));
    }

    #[test]
    fn conformance_matrix_json_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_json();
        assert!(rendered.contains("\"matrix\":\"signal.consumer.conformance\""));
        assert!(rendered.contains("\"entry_count\":4"));
        assert!(rendered.contains("\"id\":\"runtime-public-contract-boundary\""));
        assert!(rendered.contains("\"kind\":\"export-consumer-test\""));
        assert!(rendered.contains("\"crate\":\"signal-supervisor-tools\""));
        assert!(rendered.contains(
            "\"command\":\"cargo run -p signal-runtime --example supervisor_report_demo\""
        ));
    }

    #[test]
    fn release_boundary_text_reports_packaging_baseline() {
        let rendered = render_release_boundary_text();
        assert!(rendered.contains("release_boundary: signal.release.boundary"));
        assert!(rendered.contains("release_version: 0.1.0"));
        assert!(rendered.contains("version_source: workspace.package.version"));
        assert!(rendered.contains("changelog_path: CHANGELOG.md"));
        assert!(rendered.contains("conformance_task: effigy acceptance:conformance --repo ."));
        assert!(rendered
            .contains("cargo run -p signal-supervisor-tools -- --describe-export --format=json"));
        assert!(rendered
            .contains("artifact packaging beyond changelog plus host-free boundary descriptions"));
    }

    #[test]
    fn release_boundary_json_reports_packaging_baseline() {
        let rendered = render_release_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.release.boundary\""));
        assert!(rendered.contains("\"release_version\":\"0.1.0\""));
        assert!(rendered.contains("\"version_source\":\"workspace.package.version\""));
        assert!(rendered.contains("\"changelog_path\":\"CHANGELOG.md\""));
        assert!(
            rendered.contains("\"conformance_task\":\"effigy acceptance:conformance --repo .\"")
        );
        assert!(rendered.contains("\"id\":\"workspace-changelog\""));
        assert!(rendered.contains("\"id\":\"consumer-conformance\""));
        assert!(rendered.contains("\"id\":\"supervisor-export-description\""));
    }

    #[test]
    fn generation_closeout_text_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_text();
        assert!(rendered.contains("generation_closeout: signal.generation.closeout"));
        assert!(rendered.contains("generation: g04"));
        assert!(rendered.contains("closeout_task: effigy acceptance:g04-closeout --repo ."));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json"
        ));
        assert!(rendered.contains(
            "post_g04_queue_path: docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md"
        ));
        assert!(rendered.contains(
            "Promote the post-g04 queue when maintainers want broader backend-neutral consumer breadth"
        ));
    }

    #[test]
    fn generation_closeout_json_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_json();
        assert!(rendered.contains("\"closeout\":\"signal.generation.closeout\""));
        assert!(rendered.contains("\"generation\":\"g04\""));
        assert!(rendered.contains("\"closeout_task\":\"effigy acceptance:g04-closeout --repo .\""));
        assert!(rendered.contains(
            "\"post_g04_queue_path\":\"docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md\""
        ));
        assert!(rendered.contains("\"id\":\"release-boundary-baseline\""));
        assert!(rendered.contains("\"id\":\"generation-closeout-description\""));
        assert!(rendered.contains(
            "\"non-CLAP plugin backend breadth remains deferred beyond the current conformance boundary\""
        ));
    }

    #[test]
    fn export_json_is_versioned() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(export.contains("\"schema_version\":1"));
        assert!(export.contains("\"profiling_receipt\":{"));
        assert!(export.contains("\"soak_receipt\":{"));
    }

    #[test]
    fn export_json_carries_last_deferred_service_receipt() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");
        let purge_receipt = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: "purge:export-proof".into(),
                artifact_root_path: Some("/tmp/nonexistent-artifacts".into()),
                report_path: Some("/tmp/nonexistent-report.json".into()),
            })
            .expect("safe mode should defer purge export proof");
        assert!(!purge_receipt.purged_report);
        assert!(!purge_receipt.purged_artifact_root);

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"last_deferred_service\":{"));
        assert!(export.contains("\"work_class\":\"OfflineRenderPurge\""));
        assert!(export.contains("\"decision\":\"Defer\""));
        assert!(export.contains("\"reason\":\"SafeMode\""));
    }

    #[test]
    fn export_json_carries_runtime_owned_plugin_discovery_catalog() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        });
        runtime.record_plugin_scan_results(scan_handle, vec![sample_discovered_type_record()]);
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "export-consumer-sandbox".into(),
            plugin_format: PluginFormat::Clap,
        });
        runtime.record_plugin_sandbox_lifecycle(
            "export-consumer-sandbox",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"host_summary\":{}"));
        assert!(export.contains("\"supervisor_report\":{"));
        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"discovered_type_count\":1"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"format\":\"Clap\""));
        assert!(export.contains("\"supports_snapshot\":true"));
        assert!(export.contains("\"supports_activate\":true"));
    }

    #[test]
    fn export_json_carries_runtime_recovery_sequence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-1".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-1".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-1".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(9),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-1".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 9,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(9),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(9),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Soak,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"recovery_events\":1"));
        assert!(export.contains("\"recovery_sequence\":[{"));
        assert!(export.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"lifecycle_events\":1"));
        assert!(export.contains("\"lifecycle_sequence\":[{"));
        assert!(export.contains("\"stage\":\"TransportAttached\""));
        assert!(export.contains("\"transport_events\":4"));
        assert!(export.contains("\"transport_sequence\":[{"));
        assert!(export.contains("\"region_id\":\"region-4\""));
        assert!(export.contains("\"heartbeat_events\":1"));
        assert!(export.contains("\"heartbeat_sequence\":[{"));
        assert!(export.contains("\"block_sequence\":9"));
        assert!(export.contains("\"block_dispatch_events\":1"));
        assert!(export.contains("\"block_dispatch_sequence\":[{"));
        assert!(export.contains("\"completion_state\":\"Completed\""));
        assert!(export.contains("\"lease_rollover_events\":1"));
        assert!(export.contains("\"lease_rollover_sequence\":[{"));
        assert!(export.contains("\"previous_lease_id\":\"lease-3\""));
        assert!(export.contains("\"invalidation_events\":1"));
        assert!(export.contains("\"invalidation_sequence\":[{"));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"completion_slot_events\":2"));
        assert!(export.contains("\"completion_slot_sequence\":[{"));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_events\":8"));
        assert!(export.contains("\"last_transport_fault\":{"));
        assert!(export.contains("\"transport_fault_sequence\":[{"));
        assert!(export.contains("\"source\":\"HostBroker\""));
        assert!(export.contains("\"source\":\"SandboxOperation\""));
        assert!(export.contains("\"source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"phase\":\"Dispatch\""));
        assert!(export.contains("\"phase\":\"Teardown\""));
        assert!(export.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(export.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(export.contains("\"resource\":\"CompletionSlot\""));
        assert!(export.contains("\"operation\":\"block_payload.read\""));
        assert!(export.contains("\"operation\":\"transport.detach_request\""));
        assert!(export.contains("\"operation\":\"transport.detached\""));
        assert!(export.contains("\"operation\":\"transport.detach_fault\""));
        assert!(export.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(export.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(export.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(export.contains("\"operation\":\"processBlock\""));
        assert!(export.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(export.contains("\"stage\":\"TransportDetached\""));
        assert!(export.contains("\"stage\":\"DetachFault\""));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(export.contains("\"host_broker_events\":4"));
        assert!(export.contains("\"sandbox_operation_events\":1"));
        assert!(export.contains("\"runtime_dispatch_events\":3"));
        assert!(export.contains("\"dispatch_events\":"));
        assert!(export.contains("\"teardown_events\":"));
        assert!(export.contains("\"transport_concurrency_snapshot\":{"));
        assert!(export.contains("\"steady_session_limit\":1"));
        assert!(export.contains("\"recovery_session_limit\":2"));
        assert!(export.contains("\"current_attached_sessions\":0"));
        assert!(export.contains("\"current_lingering_sessions\":0"));
        assert!(export.contains("\"peak_lingering_sessions\":0"));
        assert!(export.contains("\"current_detach_requested_sessions\":0"));
        assert!(export.contains("\"current_detach_faulted_sessions\":0"));
        assert!(export.contains("\"transport_session_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(export.contains("\"current_state\":\"DetachFaulted\""));
        assert!(export.contains("\"currently_attached\":false"));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"current_attached_session_count\":0"));
        assert!(export.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(export.contains("\"attach_events\":1"));
        assert!(export.contains("\"detach_requested_events\":1"));
        assert!(export.contains("\"detached_events\":1"));
        assert!(export.contains("\"detach_fault_events\":1"));
        assert!(export.contains("\"heartbeat_responded_events\":1"));
        assert!(export.contains("\"dispatch_completed_events\":1"));
        assert!(export.contains("\"active_sandbox_id\":null"));
        assert!(export.contains("\"active_lease_id\":null"));
        assert!(export.contains("\"active_region_id\":null"));
        assert!(export.contains("\"active_block_sequence\":"));
        assert!(export.contains("\"active_sessions\":[]"));
        assert!(export.contains("\"last_region_id\":\"region-4\""));
        assert!(export.contains("\"broker_failure_events\":1"));
        assert!(export.contains("\"broker_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"PayloadRead\""));
        assert!(export.contains("\"sandbox_operation_failure_events\":1"));
        assert!(export.contains("\"sandbox_operation_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"ProcessAttach\""));
    }

    #[test]
    fn export_json_serializes_per_session_transport_liveness() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].state,
            TransportSessionState::DetachRequested
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].dispatch_state,
            TransportDispatchState::TimedOut
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[1].heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[0].transport_fault_count
                >= 1
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[1].transport_fault_count
                >= 1
        );

        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"active_sessions\":[{"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-a\""));
        assert!(export.contains("\"state\":\"DetachRequested\""));
        assert!(export.contains("\"currently_attached\":true"));
        assert!(export.contains("\"heartbeat_freshness\":\"Missed\""));
        assert!(export.contains("\"dispatch_state\":\"TimedOut\""));
        assert!(export.contains("\"peak_attached_sessions\":"));
        assert!(export.contains("\"active_block_sequence\":11"));
        assert!(export.contains("\"transport_fault_count\":1"));
        assert!(export.contains("\"last_transport_fault_source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"last_transport_fault_phase\":\"Dispatch\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":4"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":11"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-b\""));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"active_block_sequence\":12"));
        assert!(export.contains("\"last_transport_fault_source\":\"HostBroker\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"PayloadRead\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":5"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":12"));
    }

    #[test]
    fn local_summary_json_excludes_payload_by_default() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: false });
        assert!(!rendered.contains("\"payload\":{"));
        assert!(rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\"]"));
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[]"));
        assert!(rendered.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(rendered.contains("\"last_stop_reason\":\"DegradedModeRecovery\""));
    }

    #[test]
    fn local_summary_json_includes_payload_when_requested() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: true });
        assert!(rendered.contains("\"payload\""));
        assert!(rendered.contains("\"generated_event_bytes\""));
        assert!(
            rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\",\"payload\"]")
        );
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[\"payload\"]"));
    }

    #[test]
    fn local_summary_text_reports_section_list() {
        let summary = sample_local_summary();
        let default_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: false });
        let payload_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: true });
        assert!(default_rendered.contains("sections: execution,transport,faults"));
        assert!(default_rendered.contains("debug_sections_supported: payload"));
        assert!(default_rendered.contains("debug_sections_enabled: none"));
        assert!(default_rendered.contains("last_recovery_intent=Some(WatchdogRecovery)"));
        assert!(default_rendered.contains("last_stop_reason=Some(DegradedModeRecovery)"));
        assert!(payload_rendered.contains("sections: execution,transport,faults,payload"));
        assert!(payload_rendered.contains("debug_sections_supported: payload"));
        assert!(payload_rendered.contains("debug_sections_enabled: payload"));
    }
}
