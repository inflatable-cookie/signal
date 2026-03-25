use signal_host_local::LocalRuntimeHost;
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeProfilingReceipt, RuntimeSoakReceipt, RuntimeSupervisorReport,
    SignalRuntime,
};

use crate::{
    render_local_summary, render_local_summary_json, render_server_summary,
    render_server_summary_json, render_supervisor_export_json, ExportDebugOptions, HostProfile,
    OutputFormat, Scenario,
};

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

pub(crate) fn run_local(
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

pub(crate) fn run_server(
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
