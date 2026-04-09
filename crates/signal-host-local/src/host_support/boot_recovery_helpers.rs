use signal_plugin::CompletionState;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{RuntimeError, RuntimeErrorKind};

use super::super::INTER_EPISODE_CONTINUITY_BLOCKS;
use super::super::{
    FaultInjection, LocalRuntimeHost, RecoveryFailureInjection, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
use super::demo::LocalDemoPluginSandboxAssembly;
use super::lifecycle_run::LifecycleRunSummary;
use super::{
    build_fault_envelope, record_runtime_fault, RepeatedWatchdogRecoveryPlan,
    TimeoutRecoveryRetryPlan,
};
signal_runtime::impl_host_boot_recovery_helpers!(
    LocalRuntimeHost,
    LocalDemoPluginSandboxAssembly,
    TimeoutRecoveryRetryPlan<'_>,
    "instance:local:default"
);
