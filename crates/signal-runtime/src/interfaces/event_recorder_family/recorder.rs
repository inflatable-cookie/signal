use super::*;

#[derive(Clone, Default)]
pub struct RuntimeEventRecorder {
    pub(super) events: Arc<Mutex<Vec<RuntimeEvent>>>,
}

impl RuntimeEventRecorder {
    pub fn count(&self) -> usize {
        self.events
            .lock()
            .map(|events| events.len())
            .unwrap_or_default()
    }

    pub fn snapshot(&self) -> Vec<RuntimeEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    pub fn supervision_updates(&self) -> Vec<RuntimeSupervisionSnapshot> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::SupervisionChanged(snapshot) => Some(snapshot),
                _ => None,
            })
            .collect()
    }

    pub fn plugin_faults(&self) -> Vec<PluginFaultRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxFault {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                } => Some(PluginFaultRecord {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn plugin_instance_states(&self) -> Vec<PluginSandboxInstanceStateRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxInstanceState { state } => Some(state),
                _ => None,
            })
            .collect()
    }

    pub fn recovery_events(&self) -> Vec<RecoveryRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::RecoveryCycle {
                    sandbox_id,
                    intent,
                    stop_reason,
                    processing_epoch,
                } => Some(RecoveryRecord {
                    sandbox_id,
                    intent,
                    stop_reason,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn lifecycle_events(&self) -> Vec<PluginSandboxLifecycleRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxLifecycle {
                    sandbox_id,
                    stage,
                    processing_epoch,
                } => Some(PluginSandboxLifecycleRecord {
                    sandbox_id,
                    stage,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn sandbox_operation_failure_events(&self) -> Vec<SandboxOperationFailureRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::SandboxOperationFailure {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    operation,
                    error_kind,
                    stage,
                    detail,
                } => Some(SandboxOperationFailureRecord {
                    sandbox_id,
                    lease_id,
                    processing_epoch,
                    operation,
                    error_kind,
                    stage,
                    detail,
                }),
                _ => None,
            })
            .collect()
    }
}
