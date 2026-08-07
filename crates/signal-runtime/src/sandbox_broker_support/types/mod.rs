//! Sandbox broker support types and small helpers.

mod plugin;
mod receipt;
mod session;
mod wire;

pub use plugin::{
    SandboxEditorClosed, SandboxEditorOpened, SandboxPluginActivateOutcome,
    SandboxPluginAudioLease, SandboxPluginInventory, SandboxPluginParameter,
};
pub use receipt::{
    SandboxBrokerAttachedSession, SandboxBrokerReceiptState, SandboxBrokerTeardownReceipt,
};
pub use session::{
    PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord, SandboxBrokerClientSession,
    SandboxBrokerExecutionSummary, SandboxBrokerSession, SandboxBrokerSpawnConfig,
};

pub(crate) use plugin::{parse_parameter_inventory, user_closed_editor_instance};
pub(crate) use receipt::SandboxBrokerReceiptLine;
pub(crate) use session::{DEFAULT_BROKER_READ_TIMEOUT, STDERR_TAIL_LINES};
pub(crate) use wire::{
    decode_wire_token, io_runtime_error, parse_broker_receipt_line,
    record_broker_failure_and_convert, split_broker_args,
};
