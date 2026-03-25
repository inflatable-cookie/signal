use super::*;

impl SignalRuntime {
    pub fn record_xrun_overload(
        &mut self,
        processing_epoch: Option<u64>,
    ) -> RuntimeSupervisionSnapshot {
        self.increment_xruns();
        if self
            .supervision
            .record_xrun_overload(processing_epoch, self.diagnostics.xruns)
        {
            self.safe_mode_enabled = true;
        }
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        self.emit(RuntimeEvent::SupervisionChanged(
            self.get_supervision_snapshot(),
        ));
        self.get_supervision_snapshot()
    }

    pub fn fail_runtime(&mut self, error: RuntimeError) -> RuntimeReadiness {
        self.safe_mode_enabled = true;
        self.control.running = false;
        self.readiness = RuntimeReadiness::Failed { fatal: error };
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        self.readiness.clone()
    }

    pub fn record_watchdog_restart(
        &mut self,
        record: WatchdogRestartRecord,
    ) -> RuntimeSupervisionSnapshot {
        if self.supervision.record_watchdog_restart(record) {
            self.safe_mode_enabled = true;
        }
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        self.emit(RuntimeEvent::SupervisionChanged(
            self.get_supervision_snapshot(),
        ));
        self.get_supervision_snapshot()
    }
}
