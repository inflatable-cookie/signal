use signal_plugin::WatchdogOutcome;
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    HeartbeatCycleStage, RuntimeError, RuntimePreworkServicePressure, RuntimeProjectionApi,
    WatchdogRestartRecord,
};

use super::super::LocalRuntimeHost;
use super::{
    plugin_instance_state_record_from_response, record_runtime_fault, runtime_error_from_failure,
    runtime_watchdog_trigger, LifecycleRunSummary,
};

signal_runtime::impl_host_runtime_cycle_support!(LocalRuntimeHost);
