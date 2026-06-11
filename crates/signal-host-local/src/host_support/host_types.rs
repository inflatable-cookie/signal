use signal_hardware::BackendPolicyTier;
use signal_runtime::{RuntimeHostClockDomain, RuntimeHostClockFallbackState};

pub(crate) const LOCAL_DEMO_GRAPH_ID: &str = "signal.host.local.demo";
pub(crate) const LOCAL_DEMO_PLUGIN_NODE_ID: &str = "plugin-insert";

#[derive(Clone, Debug, Default)]
pub(crate) struct LocalSupervisorState {
    pub(crate) scans_started: u64,
    pub(crate) sandboxes: u64,
    pub(crate) restarts: u64,
    pub(crate) teardowns: u64,
    pub(crate) backend_policy: Option<BackendPolicyTier>,
    pub(crate) last_scan_roots: Vec<String>,
    pub(crate) last_sandbox_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LocalClockTransitionMemory {
    pub(crate) configured_stream: bool,
    pub(crate) domain: RuntimeHostClockDomain,
    pub(crate) fallback_state: RuntimeHostClockFallbackState,
    pub(crate) initialized: bool,
}

impl Default for LocalClockTransitionMemory {
    fn default() -> Self {
        Self {
            configured_stream: false,
            domain: RuntimeHostClockDomain::SameClock,
            fallback_state: RuntimeHostClockFallbackState::Unconfigured,
            initialized: false,
        }
    }
}
