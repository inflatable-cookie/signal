use signal_hardware::BackendPolicyTier;
use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_lv2::Lv2HostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest,
    RecoveryRestartIntent, RuntimeClipProcessingRegistration, RuntimeError,
    RuntimeEventRecorder, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeWarpClipRegistration,
    SignalRuntime, StopReason,
};

#[path = "host_support.rs"]
mod host_support;
use host_support::{
    discovered_plugins_for_scan, ensure_au_sandbox_session, ensure_lv2_sandbox_session,
    ensure_vst3_sandbox_session, runtime_plugin_format_platform_coverage,
};
pub use host_support::{
    ServerExecutionSummary, ServerFaultSummary, ServerPayloadSummary, ServerRuntimeHostSummary,
    ServerTransportSummary,
};

const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
const STEADY_STATE_BLOCKS: u64 = 8;
const SOAK_RESTART_EPISODES: u32 = 3;
const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;

fn samples_to_ms(samples: u32, sample_rate_hz: u32) -> f32 {
    if sample_rate_hz == 0 {
        0.0
    } else {
        samples as f32 * 1_000.0 / sample_rate_hz as f32
    }
}

#[derive(Clone, Debug, Default)]
struct ServerSupervisorState {
    scans_started: u64,
    sandboxes: u64,
    restarts: u64,
    teardowns: u64,
    backend_policy: Option<BackendPolicyTier>,
    last_scan_roots: Vec<String>,
    last_sandbox_id: Option<String>,
    last_recovery_intent: Option<RecoveryRestartIntent>,
    last_stop_reason: Option<StopReason>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultInjection {
    Timeout,
    Crash,
    HeartbeatMiss,
    RecoveryDeferredTeardownFailure,
    RecoveryDeferredTeardownThenCleanup,
    RecoveryDeferredTeardownCleanupRetry,
    RecoveryTeardownFailure,
    RecoveryRestartFailure,
    RecoveryOverlapContention,
    RecoveryInterleavedFailures,
    EscalatingHeartbeatMisses { restart_episodes: u32 },
    MixedWatchdogEpisodes { restart_episodes: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecoveryFailureInjection {
    OldTransportTeardown,
    DeferredOldTransportTeardown,
    LingeringCleanupTeardown,
    ReplacementStart,
    CompetingOverlapAttach,
}

pub struct ServerRuntimeHost {
    runtime: SignalRuntime,
    broker: SharedMemoryBroker,
    au: AuHostAdapter,
    lv2: Lv2HostAdapter,
    vst3: Vst3HostAdapter,
    supervisor: ServerSupervisorState,
    events: RuntimeEventRecorder,
}

impl ServerRuntimeHost {
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));
        runtime.record_plugin_format_platform_coverage(runtime_plugin_format_platform_coverage());

        Self {
            runtime,
            broker: SharedMemoryBroker::default(),
            au: AuHostAdapter::default(),
            lv2: Lv2HostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            supervisor: ServerSupervisorState::default(),
            events,
        }
    }

    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }
}

impl RuntimeSupervisorApi for ServerRuntimeHost {
    fn start_plugin_scan(
        &mut self,
        request: PluginScanRequest,
    ) -> Result<signal_runtime::ScanHandle, RuntimeError> {
        let handle = self.runtime.record_plugin_scan_request(&request);
        let discovered_types =
            discovered_plugins_for_scan(&self.au, &self.lv2, &self.vst3, &request);
        self.runtime
            .record_plugin_scan_results(handle, discovered_types);
        self.supervisor.scans_started = handle.0;
        self.supervisor.last_scan_roots = request.roots;
        Ok(handle)
    }

    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<signal_runtime::SandboxHandle, RuntimeError> {
        self.supervisor.sandboxes = self.supervisor.sandboxes.saturating_add(1);
        self.runtime.record_plugin_sandbox_spec(&request);
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        if request.plugin_format == PluginFormat::Au {
            ensure_au_sandbox_session(&mut self.runtime, &self.au, &request);
        }
        if request.plugin_format == PluginFormat::Lv2 {
            ensure_lv2_sandbox_session(&mut self.runtime, &self.lv2, &request);
        }
        if request.plugin_format == PluginFormat::Vst3 {
            ensure_vst3_sandbox_session(&mut self.runtime, &self.vst3, &request);
        }
        self.supervisor.last_sandbox_id = Some(request.sandbox_id);
        Ok(signal_runtime::SandboxHandle(self.supervisor.sandboxes))
    }

    fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError> {
        self.runtime.start_recording_capture(request)
    }

    fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        self.runtime.finish_recording_capture()
    }

    fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError> {
        self.runtime.cancel_recording_capture()
    }

    fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_media_assets(assets)
    }

    fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError> {
        self.runtime.start_media_preview(asset_id)
    }

    fn stop_media_preview(&mut self) -> Result<(), RuntimeError> {
        self.runtime.stop_media_preview()
    }

    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_warp_clips(clips)
    }

    fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_clip_processing_clips(clips)
    }

    fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        self.runtime.render_offline(request)
    }

    fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError> {
        self.runtime.render_offline_with_checkpoints(request)
    }

    fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.begin_offline_render_execution(request)
    }

    fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.pause_offline_render_execution(request_id)
    }

    fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.resume_offline_render_execution(request_id)
    }

    fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime
            .interrupt_offline_render_execution(request_id, reason)
    }

    fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.advance_offline_render_execution(request_id)
    }

    fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError> {
        self.runtime.cancel_offline_render_execution(request_id)
    }

    fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError> {
        self.runtime.render_offline_queue(requests)
    }

    fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError> {
        self.runtime.purge_offline_render_artifacts(request)
    }

    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.teardowns = self.supervisor.teardowns.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::SandboxTeardown,
            None,
        );
        Ok(())
    }

    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.restarts = self.supervisor.restarts.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::SandboxRestarted,
            None,
        );
        Ok(())
    }

    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError> {
        self.supervisor.backend_policy = Some(request.tier);
        Ok(())
    }
}

#[cfg(test)]
#[path = "host_test_support.rs"]
mod host_test_support;

#[cfg(test)]
mod tests {
    mod lingering_sessions {
        include!("host_tests/lingering_sessions.rs");
    }
    mod plugin_scan {
        include!("host_tests/plugin_scan.rs");
    }
    mod report_surfacing {
        include!("host_tests/report_surfacing.rs");
    }
    mod soak {
        include!("host_tests/soak.rs");
    }
    mod timeout_recovery {
        include!("host_tests/timeout_recovery.rs");
    }
    mod watchdog_recovery {
        include!("host_tests/watchdog_recovery.rs");
    }
}
