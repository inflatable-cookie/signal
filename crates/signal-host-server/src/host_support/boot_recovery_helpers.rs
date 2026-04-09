use signal_plugin::CompletionState;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RuntimeError, RuntimeErrorKind};

use super::super::INTER_EPISODE_CONTINUITY_BLOCKS;
use super::super::{
    FaultInjection, RecoveryFailureInjection, ServerRuntimeHost, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
use super::{
    build_fault_envelope, record_runtime_fault, LifecycleRunSummary, RepeatedWatchdogRecoveryPlan,
    ServerDemoPluginSandboxAssembly, TimeoutRecoveryRetryPlan,
};

signal_runtime::impl_host_boot_recovery_helpers!(
    ServerRuntimeHost,
    ServerDemoPluginSandboxAssembly,
    TimeoutRecoveryRetryPlan<'_>,
    "instance:server:default"
);
