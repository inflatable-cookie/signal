// Tests for signal-plugin-clap
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        classify_sandbox_failure, sandbox_failure_event, ClapBlockProtocol, ClapEvent,
        ClapHostExtension, ClapNoteExpressionEvent, ClapNoteExpressionKind, ClapParamGestureEvent,
        ClapParamGesturePhase, ClapPluginHostAdapter, ClapSandboxFailureInput,
        ClapSandboxFailureStage, ClapSandboxLifecycleHarness,
    };
    use signal_ipc::{
        PluginDescriptorPayload, PluginMessageName, PluginMessagePayload, SharedMemoryBroker,
        SharedMemoryTransportKind,
    };
    use signal_plugin::{CompletionState, EventPacket, PluginFormat, PluginIoLayout};
    use std::{
        fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_broker_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-plugin-clap-tests-{}-{name}-{timestamp}",
            process::id()
        ))
    }

    mod adapter;
    mod block_processing;
    mod failures;
    mod lifecycle;
}
