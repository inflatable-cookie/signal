//! Sandbox broker session attachment, receipts, and lifecycle helpers.

mod client_session;
mod ops;
mod types;

pub use ops::{
    ensure_prepared_sandbox_session, record_broker_attached_execution_summary,
    record_broker_sandbox_prepared, record_protocol_violation_prepare_failure,
    teardown_broker_sandbox_session,
};
pub use types::{
    PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord, SandboxBrokerAttachedSession,
    SandboxBrokerClientSession, SandboxBrokerReceiptState, SandboxBrokerSession,
    SandboxBrokerSpawnConfig, SandboxEditorClosed, SandboxEditorOpened,
    SandboxPluginActivateOutcome, SandboxPluginAudioLease, SandboxPluginInventory,
    SandboxPluginParameter,
};

// Crate-internal helpers used outside this module via historical paths.
#[allow(unused_imports)]
pub(crate) use ops::{
    ensure_broker_sandbox_session, record_broker_sandbox_detached,
    record_broker_transport_detach_requested,
};
#[allow(unused_imports)]
pub(crate) use types::{SandboxBrokerExecutionSummary, SandboxBrokerTeardownReceipt};

#[cfg(test)]
mod tests;
