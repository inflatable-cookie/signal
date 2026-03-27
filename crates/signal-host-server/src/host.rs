use signal_hardware::BackendPolicyTier;
use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginFormat;
use signal_plugin_au::{AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_lv2::{Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin_vst3::{Vst3HostAdapter, Vst3HostPlatform};
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RuntimeClipProcessingRegistration, RuntimeError, RuntimeEventRecorder,
    RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimePluginDiscoveredTypeRecord,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureStartRequest,
    RuntimeSupervisorApi, RuntimeWarpClipRegistration, SignalRuntime, StopReason,
};

#[path = "host_support.rs"]
mod host_support;
use host_support::{
    runtime_au_discovered_type_record, runtime_lv2_discovered_type_record,
    runtime_plugin_discovered_type_record, runtime_plugin_format_platform_coverage,
    runtime_vst3_discovered_type_record,
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

    fn discovered_plugins_for_scan(
        &self,
        request: &PluginScanRequest,
    ) -> Vec<RuntimePluginDiscoveredTypeRecord> {
        let mut discovered = Vec::new();
        let include_clap =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
        if include_clap {
            let clap = ClapPluginHostAdapter::default();
            discovered.extend(
                ["plugin:clap:server", "plugin:clap:sandbox"]
                    .into_iter()
                    .filter_map(|plugin_type_id| clap.discover_plugin_type(plugin_type_id))
                    .map(runtime_plugin_discovered_type_record),
            );
        }

        let include_vst3 =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3);
        if include_vst3 {
            discovered.extend(
                self.vst3
                    .discover_plugins_for_roots(Vst3HostPlatform::Linux, &request.roots)
                    .into_iter()
                    .map(runtime_vst3_discovered_type_record),
            );
        }

        let include_lv2 =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Lv2);
        if include_lv2 {
            discovered.extend(
                self.lv2
                    .discover_plugins_for_roots(Lv2HostPlatform::Linux, &request.roots)
                    .into_iter()
                    .map(runtime_lv2_discovered_type_record),
            );
        }

        let include_au = request.formats.is_empty() || request.formats.contains(&PluginFormat::Au);
        if include_au {
            discovered.extend(
                self.au
                    .discover_plugins_for_roots(AuHostPlatform::MacOs, &request.roots)
                    .into_iter()
                    .map(runtime_au_discovered_type_record),
            );
        }

        discovered
    }

    fn ensure_au_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.au.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.au.instantiate_plugin(
            &discovered,
            &format!("instance:server:au:{}", request.sandbox_id),
        );
        let session = self.au.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
    }

    fn ensure_lv2_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.lv2.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.lv2.instantiate_plugin(
            &discovered,
            &format!("instance:server:lv2:{}", request.sandbox_id),
        );
        let session = self.lv2.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
    }

    fn ensure_vst3_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.vst3.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.vst3.instantiate_plugin(
            &discovered,
            &format!("instance:server:vst3:{}", request.sandbox_id),
        );
        let session = self.vst3.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
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
        let discovered_types = self.discovered_plugins_for_scan(&request);
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
            self.ensure_au_sandbox_session(&request);
        }
        if request.plugin_format == PluginFormat::Lv2 {
            self.ensure_lv2_sandbox_session(&request);
        }
        if request.plugin_format == PluginFormat::Vst3 {
            self.ensure_vst3_sandbox_session(&request);
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
