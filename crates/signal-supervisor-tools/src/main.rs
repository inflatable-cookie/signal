use std::env;

use signal_host_local::{LocalRuntimeHost, LocalRuntimeHostSummary};
use signal_host_server::{ServerRuntimeHost, ServerRuntimeHostSummary};
use signal_runtime::{RuntimeConfig, RuntimeSupervisorReport, SignalRuntime};

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
}

const EXPORT_SCHEMA: &str = "signal.supervisor.export";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_HOST_SUMMARY_SECTIONS: &[&str] = &["execution", "transport", "faults"];
const SUPPORTED_DEBUG_SECTIONS: &[HostSummaryDebugSection] = &[HostSummaryDebugSection::Payload];

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
        "usage: signal-supervisor-tools [--format text|json] [--include-payload] [--describe-export] <local|server> <default|timeout|crash|heartbeat|soak|mixed>"
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
            "\"supervisor_report\":{}",
            "}}"
        ),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(&format!("{profile:?}")),
        json_string(&format!("{scenario:?}")),
        host_summary,
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
    report: RuntimeSupervisorReport,
) {
    match format {
        OutputFormat::Text => println!(
            "signal-supervisor-tools profile={profile:?} scenario={scenario:?}\n{summary}\nsupervisor:\n{}",
            report.render_multiline()
        ),
        OutputFormat::Json => println!(
            "{}",
            render_supervisor_export_json(profile, scenario, summary, &report)
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
    print_report(
        format,
        HostProfile::Local,
        scenario,
        match format {
            OutputFormat::Text => render_local_summary(&summary, debug),
            OutputFormat::Json => render_local_summary_json(&summary, debug),
        },
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
    print_report(
        format,
        HostProfile::Server,
        scenario,
        match format {
            OutputFormat::Text => render_server_summary(&summary, debug),
            OutputFormat::Json => render_server_summary_json(&summary, debug),
        },
        report,
    );
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut debug = ExportDebugOptions::default();
    let mut describe_export = false;
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
        if let Some(value) = arg.strip_prefix("--format=") {
            format = OutputFormat::parse(value)?;
            continue;
        }
        positional.push(arg);
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
    };

    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_args, render_export_description_json, render_export_description_text,
        render_supervisor_export_json, CliArgs, CliMode, ExportDebugOptions, HostProfile,
        HostSummaryDebugSection, OutputFormat, Scenario,
    };
    use signal_host_local::host::{
        LocalExecutionSummary, LocalFaultSummary, LocalPayloadSummary, LocalTransportSummary,
    };
    use signal_host_local::{LocalRuntimeHostSummary, RecoveryRestartIntent};
    use signal_plugin::{CompletionState, WatchdogTriggerReason};
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        HeartbeatCycleStage, PluginSandboxLifecycleStage, PluginSandboxTransportStage,
        RuntimeConfig, RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink,
        RuntimeSupervisorReport, SandboxOperationFailureStage, SignalRuntime, StopReason,
        TransportDispatchState, TransportHeartbeatFreshness, TransportSessionState,
    };

    fn sample_local_summary() -> LocalRuntimeHostSummary {
        LocalRuntimeHostSummary {
            backend_name: "coreaudio",
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
            },
            transport: LocalTransportSummary {
                sandbox_id: "sandbox-1".into(),
                shared_memory_lease_id: "lease-1".into(),
                shared_memory_region_id: "region-1".into(),
                shared_memory_path: "/tmp/signal-region-1".into(),
                shared_memory_bytes: 4096,
            },
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
    fn export_json_is_versioned() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &report,
        );
        assert!(export.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(export.contains("\"schema_version\":1"));
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
        let export =
            render_supervisor_export_json(HostProfile::Local, Scenario::Soak, "{}".into(), &report);
        assert!(export.contains("\"recovery_events\":1"));
        assert!(export.contains("\"recovery_sequence\":[{"));
        assert!(export.contains("\"intent\":\"WatchdogRecovery\""));
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
        assert!(export.contains("\"dispatch_events\":5"));
        assert!(export.contains("\"teardown_events\":3"));
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
        assert!(export.contains("\"active_block_sequence\":null"));
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
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].transport_fault_count,
            1
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[1].transport_fault_count,
            1
        );

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report,
        );
        assert!(export.contains("\"active_sessions\":[{"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-a\""));
        assert!(export.contains("\"state\":\"DetachRequested\""));
        assert!(export.contains("\"currently_attached\":true"));
        assert!(export.contains("\"heartbeat_freshness\":\"Missed\""));
        assert!(export.contains("\"dispatch_state\":\"TimedOut\""));
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
        assert!(!rendered.contains("\"payload\""));
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
