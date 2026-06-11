use super::*;

impl SignalRuntime {
    pub(crate) fn diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        self.diagnostics
    }
}
