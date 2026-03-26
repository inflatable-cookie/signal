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
    use super::host_test_support::{
        assert_runtime_automation_continuity, assert_runtime_automation_values,
        assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
        prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
        temp_media_fixture_path,
    };
    use super::ServerRuntimeHost;
    use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
    use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
    use signal_plugin_clap::ClapSandboxLifecycleHarness;
    use signal_primitives::{ChannelCount, ChannelLayout};
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
        GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
        GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
        PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
        RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind, RuntimeExternalIoDeviceChangeState,
        RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
        RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
        RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
        RuntimeMediaPreviewState, RuntimeObservationApi, RuntimePluginHostPlatform,
        RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
        RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage, SignalRuntime,
        StopReason, TransportAttachIntent,
    };
    use std::{fs, path::Path};

    #[test]
    fn server_host_rolls_leases_forward_after_timeout() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_timeout_recovery()
            .expect("timeout recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 10);
        assert_eq!(summary.execution.engine_processed_blocks, 10);
        assert_eq!(summary.execution.last_block_sequence, 9);
        assert_eq!(
            summary.execution.last_engine_graph_id.as_deref(),
            Some("signal.host.server.demo")
        );
        let plugin_state = summary
            .execution
            .last_plugin_state
            .as_ref()
            .expect("plugin instance state should be projected into server summary");
        assert_eq!(plugin_state.plugin_type_id, "plugin:clap:server");
        assert_eq!(plugin_state.instance_id, "instance:server:default");
        assert_eq!(plugin_state.lifecycle_state, "Active");
        assert_eq!(plugin_state.readiness_state, "Ready");
        assert!(plugin_state.active);
        assert_eq!(plugin_state.processing_sample_rate_hz, Some(48_000));
        assert_eq!(plugin_state.processing_max_block_frames, Some(512));
        assert!(plugin_state.last_fault.is_none());
        let observed_plugin_state = supervisor
            .observation
            .observation
            .last_plugin_instance_state()
            .expect("runtime observation should retain typed plugin state");
        assert_eq!(observed_plugin_state.instance_id, "instance:server:default");
        assert_eq!(observed_plugin_state.lifecycle_state, "Active");
        assert_eq!(observed_plugin_state.readiness_state, "Ready");
        assert!(supervisor
            .render_json()
            .contains("\"plugin_instance_state_events\":"));
        assert!(
            summary
                .execution
                .last_engine_output_peak
                .unwrap_or_default()
                <= 0.7
        );
        assert!(summary.execution.last_engine_output_rms.unwrap_or_default() > 0.0);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(1)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(true)
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples)
                .unwrap_or_default()
                > 0
        );
        assert_eq!(supervisor.observation.engine_block_snapshot.node_count, 3);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_node_count,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .latency_node_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .plugin_backed_node_count,
            1
        );
        assert!(
            !supervisor
                .observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .inline_realtime_node_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_semantic_policy,
            signal_runtime::RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_bound_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_active_bound_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            0
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .planned_nodes
            .iter()
            .any(|node| node.node_id == "drive"
                && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")));
        assert_eq!(supervisor.observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_phase_count,
            0
        );
        assert_eq!(supervisor.observation.engine_block_snapshot.lane_count, 1);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_lane_count,
            0
        );
        assert_eq!(
            supervisor.observation.engine_block_snapshot.dispatch_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .dispatch_boundary_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prepared_dispatch_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .realtime_dispatch_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .dispatch_handoff_count,
            0
        );
        assert!(
            !supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_enabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_requested_mode,
            signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_mode,
            signal_runtime::RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_state,
            signal_runtime::RuntimePreworkCacheState::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            signal_runtime::RuntimePreworkFreshnessState::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_admissions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_consumptions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_retirement_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_hits,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_misses,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_output_peak,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_admission_processing_epoch,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_admission_block_sequence,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_block_sequence,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retirement_reason,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retired_unconsumed,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            None
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_realtime_input_peak
            .is_some());
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .total_latency_samples,
            32
        );
        assert_eq!(summary.last_payload.event_count, 11);
        assert_eq!(summary.last_payload.parameter_event_count, 2);
        assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
        assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
        assert_eq!(summary.last_payload.note_event_count, 1);
        assert_eq!(summary.last_payload.note_expression_event_count, 3);
        assert_eq!(summary.last_payload.midi_event_count, 1);
        assert_eq!(summary.last_payload.generated_event_bytes, 268);
        assert_eq!(summary.last_payload.first_output_sample, Some(9.0));
        assert_eq!(summary.faults.deadline_misses, 2);
        assert_eq!(summary.faults.heartbeat_misses, 0);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            summary.faults.watchdog_trigger_reason,
            Some(WatchdogTriggerReason::DeadlineMisses)
        );
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            1
        );
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-2"));
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_recovery_overlap_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_admitted_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_plugin_event_snapshot(&supervisor, 2, 2, &[2], 0);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
    }

    #[test]
    fn server_host_rolls_back_replacement_transport_when_recovery_teardown_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_teardown_failure()
            .expect_err("recovery teardown failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error
                .message
                .contains("injected old transport teardown failure"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
        assert_eq!(
            supervisor
                .observation
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(supervisor.observation.control_snapshot.restart_count, 0);
    }

    #[test]
    fn server_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_deferred_teardown_failure()
            .expect_err("deferred teardown failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error
                .message
                .contains("deferred old transport teardown during recovery retry"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::DetachFaulted
        );
    }

    #[test]
    fn server_host_recovers_after_lingering_deferred_teardown_cleanup() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_recovery_deferred_teardown_then_cleanup()
            .expect("lingering cleanup recovery should succeed");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::AttachActive
        );
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
    }

    #[test]
    fn server_host_recovers_after_lingering_cleanup_fails_once_more() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_recovery_deferred_teardown_cleanup_retry()
            .expect("cleanup retry recovery should succeed");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            2
        );
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure
                        .detail
                        .contains("injected lingering cleanup retry failure")
            }));
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::AttachActive
        );
    }

    #[test]
    fn server_host_sweeps_orphan_lingering_sessions_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
        let orphan_region = host
            .broker
            .create_region("server-orphan-lingering", 256)
            .expect("orphan region");
        let orphan_transport = orphan_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan",
                orphan_transport.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport.backing_path.clone()),
                Some(orphan_transport.total_bytes),
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan",
            orphan_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let recovered = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &run,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect("orphan lingering sweep recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(recovered.processing_epoch, 2);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .all(|session| session.lease_id != "lease-orphan"));
        assert!(!Path::new(&orphan_transport.backing_path).exists());
    }

    #[test]
    fn server_host_aborts_when_orphan_lingering_cleanup_fails_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan",
                "region-orphan-failure",
                TransportAttachIntent::RecoveryOverlap,
                None,
                None,
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan",
            "region-orphan-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let error = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &run,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect_err("orphan lingering cleanup failure should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(error.message.contains("missing backing_path metadata"));
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-orphan"));
    }

    #[test]
    fn server_host_cleans_multiple_orphan_lingering_sessions_for_same_sandbox() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let orphan_region_a = host
            .broker
            .create_region("server-orphan-a", 256)
            .expect("orphan region a");
        let orphan_transport_a = orphan_region_a.metadata().clone();
        let orphan_region_b = host
            .broker
            .create_region("server-orphan-b", 256)
            .expect("orphan region b");
        let orphan_transport_b = orphan_region_b.metadata().clone();

        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan-a",
                orphan_transport_a.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(orphan_transport_a.backing_path.clone()),
                Some(orphan_transport_a.total_bytes),
            )
            .expect("orphan session a");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan-a",
            orphan_transport_a.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan a lingering".into()),
        );
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan-b",
                orphan_transport_b.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport_b.backing_path.clone()),
                Some(orphan_transport_b.total_bytes),
            )
            .expect("orphan session b");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan-b",
            orphan_transport_b.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan b lingering".into()),
        );

        host.cleanup_orphan_lingering_sessions_for_sandbox(
            "server-default-sandbox",
            1,
            None,
            None,
            LingeringCleanupMode::StrictPreAttach,
        )
        .expect("multiple orphan cleanup");

        let supervisor = host.supervisor_report();
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .is_empty());
        assert!(!Path::new(&orphan_transport_a.backing_path).exists());
        assert!(!Path::new(&orphan_transport_b.backing_path).exists());
    }

    #[test]
    fn server_host_reconciles_late_lingering_completion_without_disturbing_active_replacement() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("server-late-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-late-origin",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-late-origin",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

        let supervisor = host.supervisor_report();
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .lease_id,
            recovered.shared_memory_lease_id
        );
        assert!(!Path::new(&late_transport.backing_path).exists());
    }

    #[test]
    fn server_host_keeps_active_replacement_running_when_late_lingering_cleanup_fails() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-late-origin",
                "region-late-origin-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-late-origin",
            "region-late-origin-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

        let supervisor = host.supervisor_report();
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == recovered.shared_memory_lease_id));
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-late-origin"));
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure.detail.contains("missing backing_path metadata")
            }));
    }

    #[test]
    fn server_host_sweeps_prior_late_lingering_before_next_overlap_recovery() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("server-adjacent-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-prior-lingering",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-prior-lingering",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");
        let recovered_transport = recovered_epoch2
            .transport
            .as_ref()
            .expect("recovered transport");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            recovered_epoch2.shared_memory_lease_id.as_str(),
            recovered_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(recovered_epoch2.processing_epoch),
            Some("current replacement became lingering before adjacent recovery".into()),
        );

        let recovered_epoch3 = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &recovered_epoch2,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect("adjacent recovery should sweep prior lingering session");
        let supervisor = host.supervisor_report();

        assert_eq!(recovered_epoch3.processing_epoch, 3);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .all(|session| session.lease_id != "lease-prior-lingering"));
        assert!(!Path::new(&late_transport.backing_path).exists());
    }

    #[test]
    fn server_host_aborts_adjacent_overlap_recovery_when_prior_late_lingering_lacks_metadata() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-prior-lingering",
                "region-prior-lingering-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-prior-lingering",
            "region-prior-lingering-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        let error = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &recovered_epoch2,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect_err("adjacent recovery should abort on stale lingering metadata");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(error.message.contains("missing backing_path metadata"));
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-prior-lingering"));
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == recovered_epoch2.shared_memory_lease_id));
    }

    #[test]
    fn server_host_rolls_back_replacement_transport_when_recovery_start_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_restart_failure()
            .expect_err("recovery start failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("injected replacement start failure"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
        assert_eq!(
            supervisor
                .observation
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(supervisor.observation.control_snapshot.restart_count, 0);
    }

    #[test]
    fn server_host_rolls_back_partial_overlap_when_competing_recovery_attach_is_rejected() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_overlap_contention()
            .expect_err("overlap contention should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("recovery overlap session limit 1"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
    }

    #[test]
    fn server_host_handles_interleaved_recovery_failures_across_retries() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_interleaved_failures()
            .expect_err("interleaved failures should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("recovery overlap session limit 1"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure.detail.contains("deferred old transport teardown")
            }));
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
    }

    #[test]
    fn server_host_shared_report_surfaces_unavailable_external_io_monitoring_state() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.external_io_snapshot.health_state,
            RuntimeExternalIoHealthState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.device_change_state,
            RuntimeExternalIoDeviceChangeState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.primary_role,
            RuntimeExternalIoPrimaryRole::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.monitoring_state,
            RuntimeExternalIoMonitoringState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.monitoring_tap_point,
            RuntimeExternalIoMonitoringTapPoint::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.loopback_state,
            RuntimeExternalIoLoopbackState::Unavailable
        );
        assert_eq!(
            report
                .observation
                .external_io_snapshot
                .linux_clocking_parity,
            signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
        );
        assert_eq!(
            report.observation.external_io_snapshot.linux_duplex_parity,
            signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
        );
        assert_eq!(
            report
                .observation
                .external_io_snapshot
                .linux_endpoint_topology_parity,
            signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
        );
        assert_eq!(
            report.observation.external_io_snapshot.endpoint_topology,
            signal_runtime::RuntimeHostEndpointTopology::Unconfigured
        );
        assert_eq!(
            report.observation.external_io_snapshot.fallback_state,
            signal_runtime::RuntimeHostClockFallbackState::Unconfigured
        );

        let rendered = report.render_json();
        assert!(rendered.contains("\"external_io_snapshot\":{"));
        assert!(rendered.contains("\"health_state\":\"Unavailable\""));
        assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
        assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
        assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.external_midi_snapshot.discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.external_midi_snapshot.graph_state,
            signal_runtime::RuntimeExternalMidiGraphState::Empty
        );
        assert_eq!(
            report.observation.external_midi_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
        assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
        assert_eq!(
            report
                .observation
                .external_midi_snapshot
                .live_ownership
                .ownership_posture,
            signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
        );
        assert_eq!(
            report
                .observation
                .external_midi_snapshot
                .live_ownership
                .backend_parity,
            signal_runtime::RuntimeExternalMidiBackendParity::Guarded
        );
        assert!(report.observation.external_midi_snapshot.devices.is_empty());
        assert!(report
            .observation
            .external_midi_snapshot
            .endpoints
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"external_midi_snapshot\":{"));
        assert!(rendered.contains("\"live_ownership\":{"));
        assert!(rendered.contains("\"discovery_state\":\"Idle\""));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_linux_backend_session_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        let snapshot = &report.observation.linux_backend_session_snapshot;
        assert_eq!(
            snapshot.backend_identity,
            signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
        );
        assert_eq!(
            snapshot.ownership,
            signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
        );
        assert_eq!(
            snapshot.lifecycle_state,
            signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Running
        );
        assert_eq!(
            snapshot.device_claim_posture,
            signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
        );
        assert_eq!(
            snapshot.session_role,
            signal_runtime::RuntimeLinuxBackendSessionRole::PrimaryAudioIo
        );
        assert_eq!(
            snapshot.ownership_fallback,
            signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
        );
        assert_eq!(snapshot.backend_name, "pipewire");
        assert_eq!(snapshot.device_id, "pipewire:default-graph");
        assert!(snapshot.simulated);

        let rendered = report.render_json();
        assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
        assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
        assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_jack_coordination_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        let snapshot = &report.observation.jack_coordination_snapshot;
        assert_eq!(
            snapshot.backend_identity,
            signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
        );
        assert_eq!(snapshot.backend_name, "jack");
        assert_eq!(
            snapshot.transport_posture,
            signal_runtime::RuntimeJackTransportPosture::Detached
        );
        assert_eq!(
            snapshot.graph_state,
            signal_runtime::RuntimeJackGraphCoordinationState::AttachedGuarded
        );
        assert_eq!(
            snapshot.client_role,
            signal_runtime::RuntimeJackClientRole::PrimaryAudioIo
        );
        assert_eq!(
            snapshot.guarded_state,
            signal_runtime::RuntimeJackGuardedCoordinationState::GraphGuarded
        );
        assert_eq!(snapshot.device_id, "jack:graph-main");
        assert!(snapshot.simulated);

        let rendered = report.render_json();
        assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
        assert!(rendered.contains("\"backend_identity\":\"Jack\""));
        assert!(rendered.contains("\"transport_posture\":\"Detached\""));
        assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_control_surface_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.control_surface_snapshot.discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.control_surface_snapshot.graph_state,
            signal_runtime::RuntimeControlSurfaceGraphState::Empty
        );
        assert_eq!(
            report.observation.control_surface_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
        assert!(report
            .observation
            .control_surface_snapshot
            .devices
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"control_surface_snapshot\":{"));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_advanced_hardware_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report
                .observation
                .advanced_hardware_snapshot
                .discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.graph_state,
            signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.device_count,
            0
        );
        assert!(report
            .observation
            .advanced_hardware_snapshot
            .devices
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.stretch_engine_snapshot.clip_count, 0);
        assert_eq!(
            report.observation.stretch_engine_snapshot.ready_clip_count,
            0
        );
        assert!(report.observation.stretch_engine_snapshot.clips.is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"sample_domain_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_marker_analysis_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 0);
        assert_eq!(
            report.observation.marker_analysis_snapshot.ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .marker_analysis_snapshot
                .tempo_assist_ready_clip_count,
            0
        );
        assert!(report.observation.marker_analysis_snapshot.clips.is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"tempo_assist_ready_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_transform_artifact_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 0);
        assert_eq!(
            report
                .observation
                .transform_artifact_snapshot
                .ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .transform_artifact_snapshot
                .reusable_clip_count,
            0
        );
        assert!(report
            .observation
            .transform_artifact_snapshot
            .clips
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"reusable_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_preview_transform_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.preview_transform_snapshot.clip_count, 0);
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .active_audition_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .artifact_backed_clip_count,
            0
        );
        assert!(report
            .observation
            .preview_transform_snapshot
            .clips
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"preview_transform_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"artifact_backed_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_media_service_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("handshake");
        host.runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("configure");

        let imported_path = temp_media_fixture_path("server-media-service");
        fs::write(&imported_path, b"signal media fixture").expect("write media fixture");
        host.runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:server-media".into(),
                content_hash: "server-media".into(),
                source_path: imported_path.display().to_string(),
                file_name: "server-media.bin".into(),
                byte_size: fs::metadata(&imported_path)
                    .expect("fixture metadata")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 12,
            }])
            .expect("media reconcile");
        host.runtime
            .start_media_preview("asset:sha256:server-media")
            .expect("start media preview");

        let report = host.supervisor_report();
        assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 1);
        assert_eq!(
            report.observation.media_pipeline_snapshot.ready_asset_count,
            1
        );
        assert_eq!(
            report
                .observation
                .media_service_snapshot
                .indexed_asset_count,
            1
        );
        assert_eq!(
            report.observation.media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Previewing
        );
        assert_eq!(
            report
                .observation
                .media_service_snapshot
                .previewing_asset_id
                .as_deref(),
            Some("asset:sha256:server-media")
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .indexed_asset_count,
            1
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .loudness_ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .character_ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .unavailable_descriptor_count,
            1
        );

        let rendered = report.render_json();
        assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
        assert!(rendered.contains("\"media_service_snapshot\":{"));
        assert!(rendered.contains("\"media_library_snapshot\":{"));
        assert!(rendered.contains("\"preview_state\":\"Previewing\""));
        assert!(rendered.contains("\"unavailable_descriptor_count\":1"));

        let _ = fs::remove_file(&imported_path);
        if let Some(path) = host
            .runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_spatial_execution_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("handshake");
        host.runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("configure");
        host.runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:host-server:spatial".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "spatial-stereo".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                    },
                    GraphNodeProjection {
                        node_id: "spatial-surround".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 20,
                        stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                    },
                ],
            })
            .expect("apply spatial graph");
        host.runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:host-server:spatial".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "spatial-stereo".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "main:in".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:stereo".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:stereo".into()),
                            bus_group_id: Some("bus:spatial:stereo".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "spatial-surround".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "main:surround-in".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:surround".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:surround".into()),
                            bus_group_id: Some("bus:spatial:surround".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply spatial contract");
        host.runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:host-server:spatial".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "spatial-stereo".into(),
                        sandbox_id: "sandbox:spatial-stereo".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "spatial-surround".into(),
                        sandbox_id: "sandbox:spatial-surround".into(),
                    },
                ],
            })
            .expect("bind spatial nodes");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .spatial_node_count,
            2
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .active_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .fallback_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .surround_bed_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .expanded_fallback_spatial_node_count,
            1
        );
        assert!(report
            .observation
            .plugin_chain_snapshot
            .chains
            .iter()
            .flat_map(|chain| chain.stages.iter())
            .any(|stage| stage.node_id == "spatial-surround"
                && stage
                    .spatial_execution
                    .as_ref()
                    .is_some_and(|spatial| {
                        spatial.fallback_outcome
                            == Some(
                                signal_runtime::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing
                            )
                            && spatial.bed_class
                                == signal_runtime::RuntimeSpatialBedClass::CanonicalSurroundBed
                            && spatial.expanded_fallback_outcome
                                == Some(
                                    signal_runtime::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                                )
                    })));

        let rendered = report.render_json();
        assert!(rendered.contains("\"spatial_node_count\":2"));
        assert!(rendered.contains("\"active_spatial_node_count\":1"));
        assert!(rendered.contains("\"fallback_spatial_node_count\":1"));
        assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
        assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
        assert!(rendered.contains("\"adapter_class\":\"Balance\""));
        assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
        assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
        assert!(rendered.contains("\"execution_mode\":\"Bypassed\""));
    }

    #[test]
    fn server_host_vst3_scan_and_sandbox_surface_linux_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
            formats: vec![PluginFormat::Vst3],
        })
        .expect("server vst3 plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-vst3-sandbox".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:linux-synth".into()),
        })
        .expect("server vst3 sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Vst3])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:vst3:linux-synth"
                && plugin.format == PluginFormat::Vst3
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:vst3:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:vst3:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-vst3-sandbox")
            .expect("server vst3 sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:vst3:linux-synth")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let vst3_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Vst3)
            .expect("server vst3 parity should be present");
        assert_eq!(
            vst3_parity.linux_parity_band,
            RuntimePluginParityBand::Portable
        );
        assert!(vst3_parity.linux_supported);
        assert_eq!(
            vst3_parity.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(vst3_parity.linux_strict_sandbox_default);
        assert!(vst3_parity.prepare_capable_type_count >= 1);
        assert!(vst3_parity.activate_capable_type_count >= 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Vst3\""));
        assert!(rendered.contains("\"formats\":[\"Vst3\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
        assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    }

    #[test]
    fn server_host_au_scan_and_sandbox_surface_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
            formats: vec![PluginFormat::Au],
        })
        .expect("server au plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-au-sandbox".into(),
            plugin_format: PluginFormat::Au,
            plugin_type_id: Some("plugin:au:instrument".into()),
        })
        .expect("server au sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Au])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:au:instrument"
                && plugin.format == PluginFormat::Au
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:au:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:au:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-au-sandbox")
            .expect("server au sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:au:instrument")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let au_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Au)
            .expect("server au parity should be present");
        assert_eq!(
            au_parity.supported_platforms,
            vec![RuntimePluginHostPlatform::MacOs]
        );
        assert_eq!(
            au_parity.unsupported_platforms,
            vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert_eq!(au_parity.discovered_type_count, 4);
        assert_eq!(au_parity.sandbox_count, 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Au\""));
        assert!(rendered.contains("\"formats\":[\"Au\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"parity_coverage\":["));
        assert!(rendered.contains("\"parity_band\":\"Guarded\""));
        assert!(rendered.contains("\"supported_platforms\":[\"MacOs\"]"));
        assert!(rendered.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
    }

    #[test]
    fn server_host_lv2_scan_and_sandbox_surface_linux_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
            formats: vec![PluginFormat::Lv2],
        })
        .expect("server lv2 plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-lv2-sandbox".into(),
            plugin_format: PluginFormat::Lv2,
            plugin_type_id: Some("plugin:lv2:linux-synth".into()),
        })
        .expect("server lv2 sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Lv2])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:lv2:linux-synth"
                && plugin.format == PluginFormat::Lv2
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:lv2:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:lv2:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-lv2-sandbox")
            .expect("server lv2 sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:lv2:linux-synth")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let lv2_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Lv2)
            .expect("server lv2 parity should be present");
        assert_eq!(
            lv2_parity.supported_platforms,
            vec![RuntimePluginHostPlatform::Linux]
        );
        assert_eq!(
            lv2_parity.unsupported_platforms,
            vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert_eq!(lv2_parity.discovered_type_count, 4);
        assert_eq!(lv2_parity.sandbox_count, 1);
        assert_eq!(
            lv2_parity.linux_parity_band,
            RuntimePluginParityBand::Portable
        );
        assert!(lv2_parity.linux_supported);
        assert_eq!(
            lv2_parity.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(lv2_parity.linux_strict_sandbox_default);
        assert!(lv2_parity.prepare_capable_type_count >= 1);
        assert!(lv2_parity.activate_capable_type_count >= 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Lv2\""));
        assert!(rendered.contains("\"formats\":[\"Lv2\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
        assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    }

    #[test]
    fn server_host_recovers_after_crash() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_crash_recovery()
            .expect("crash recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 9);
        assert_eq!(summary.last_payload.event_count, 11);
        assert_eq!(summary.last_payload.parameter_event_count, 2);
        assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
        assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
        assert_eq!(summary.last_payload.note_event_count, 1);
        assert_eq!(summary.last_payload.note_expression_event_count, 3);
        assert_eq!(summary.last_payload.midi_event_count, 1);
        assert_eq!(summary.last_payload.first_output_sample, Some(8.0));
        assert_eq!(summary.faults.deadline_misses, 0);
        assert_eq!(summary.faults.heartbeat_misses, 0);
        assert!(!summary.faults.watchdog_triggered);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            0
        );
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary
            .transport
            .shared_memory_region_id
            .starts_with("region-"));
        assert_runtime_automation_values(&supervisor, 9, 9, 3, 6, 0.1, 0.5, 0.08);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 8, 0, 1);
    }

    #[test]
    fn server_host_recovers_after_heartbeat_watchdog_trigger() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_heartbeat_miss_recovery()
            .expect("heartbeat recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 8);
        assert_eq!(summary.execution.last_block_sequence, 9);
        assert_eq!(summary.faults.heartbeat_misses, 2);
        assert_eq!(summary.faults.deadline_misses, 0);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            summary.faults.watchdog_trigger_reason,
            Some(WatchdogTriggerReason::HeartbeatMisses)
        );
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            1
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(supervisor.observation.control_snapshot.running);
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
        assert_runtime_automation_continuity(&supervisor, 2, 2, &[2], 0);
        assert_runtime_sequence_continuity(&supervisor, &[2], 2, 9, 0, 0);
    }

    #[test]
    fn server_host_enters_safe_mode_after_repeated_watchdog_restarts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_escalating_heartbeat_failures()
            .expect("escalating heartbeat recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 3);
        assert_eq!(summary.execution.restart_count, 2);
        assert_eq!(summary.execution.teardown_count, 2);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 10);
        assert_eq!(summary.execution.last_block_sequence, 13);
        assert_eq!(summary.faults.heartbeat_misses, 4);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            2
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(matches!(
            supervisor.observation.readiness,
            signal_runtime::RuntimeReadiness::Degraded { .. }
        ));
        assert_eq!(supervisor.observation.control_snapshot.start_count, 3);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 2);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_runtime_automation_values(&supervisor, 10, 10, 2, 8, 0.2, 0.75, 0.18);
        assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 13, 0, 1);
    }

    #[test]
    fn server_host_soak_path_rolls_across_multiple_lease_generations() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host.boot_with_watchdog_soak().expect("watchdog soak boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 4);
        assert_eq!(summary.execution.restart_count, 3);
        assert_eq!(summary.execution.teardown_count, 3);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 12);
        assert_eq!(summary.execution.last_block_sequence, 17);
        assert_eq!(summary.faults.heartbeat_misses, 6);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            3
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
        assert_eq!(summary.last_payload.first_output_sample, Some(17.0));
        assert!(matches!(
            supervisor.observation.readiness,
            signal_runtime::RuntimeReadiness::Degraded { .. }
        ));
        assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.recovery_event_count(), 3);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::RecoveryCycle {
                        intent: RecoveryRestartIntent::WatchdogRecovery,
                        stop_reason: StopReason::DegradedModeRecovery,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceDeactivated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceReset,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceDestroyed,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::DetachRequested,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::Detached,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::HeartbeatCycle {
                        stage: HeartbeatCycleStage::Missed,
                        ..
                    }
                ))
                .count(),
            6
        );
        assert_eq!(supervisor.block_dispatch_event_count(), 24);
        assert_eq!(supervisor.lease_rollover_event_count(), 2);
        assert_eq!(supervisor.invalidation_event_count(), 6);
        assert_eq!(supervisor.completion_slot_event_count(), 39);
        assert_eq!(supervisor.broker_failure_event_count(), 0);
        assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
        assert_eq!(supervisor.transport_fault_event_count(), 0);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Requested,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::LeaseEpochInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::ReadyForProcessing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Processing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Invalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_runtime_automation_values(&supervisor, 12, 12, 2, 10, 0.2, 0.95, 0.26);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
    }

    #[test]
    fn server_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_mixed_watchdog_soak()
            .expect("mixed watchdog soak boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 4);
        assert_eq!(summary.execution.restart_count, 3);
        assert_eq!(summary.execution.teardown_count, 3);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 14);
        assert_eq!(summary.execution.last_block_sequence, 17);
        assert_eq!(summary.faults.deadline_misses, 2);
        assert_eq!(summary.faults.heartbeat_misses, 4);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            3
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.recovery_event_count(), 3);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::RecoveryCycle {
                        intent: RecoveryRestartIntent::WatchdogRecovery,
                        stop_reason: StopReason::DegradedModeRecovery,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::TransportTornDown,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::SandboxRestarted,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::DetachRequested,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::Detached,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::HeartbeatCycle {
                        stage: HeartbeatCycleStage::Missed,
                        ..
                    }
                ))
                .count(),
            4
        );
        assert_eq!(supervisor.block_dispatch_event_count(), 28);
        assert_eq!(supervisor.lease_rollover_event_count(), 2);
        assert_eq!(supervisor.invalidation_event_count(), 6);
        assert_eq!(supervisor.completion_slot_event_count(), 45);
        assert_eq!(supervisor.broker_failure_event_count(), 0);
        assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
        assert_eq!(supervisor.transport_fault_event_count(), 0);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Requested,
                        ..
                    }
                ))
                .count(),
            14
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::TimedOut,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::LeaseEpochInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::SandboxOperationFailure {
                        stage: SandboxOperationFailureStage::ProcessAttach,
                        ..
                    }
                ))
                .count(),
            0
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerFailure {
                        stage: BrokerFailureStage::PayloadRead,
                        ..
                    }
                ))
                .count(),
            0
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::ReadyForProcessing,
                        ..
                    }
                ))
                .count(),
            14
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Processing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::TimedOut,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::FallbackApplied,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Invalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_runtime_automation_values(&supervisor, 14, 14, 2, 12, 0.2, 0.95, 0.26);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
        assert!(supervisor.event_count() > 24);
        assert_eq!(supervisor.supervision_update_count(), 3);
        assert_eq!(supervisor.plugin_fault_count(), 3);
        assert_eq!(
            supervisor
                .observation
                .observation
                .fault_detail_count_containing("heartbeat watchdog"),
            2
        );
        assert_eq!(
            supervisor
                .observation
                .observation
                .fault_detail_count_containing("block deadline"),
            1
        );
        assert_eq!(
            host.runtime()
                .get_supervision_snapshot()
                .last_watchdog_trigger,
            Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
        let rendered = supervisor.render_compact();
        assert!(rendered.contains("readiness=Degraded"));
        assert!(rendered.contains("supervision_updates=3"));
        assert!(rendered.contains("plugin_faults=3"));
        assert!(rendered.contains("last_watchdog=HeartbeatMisses"));
        assert!(rendered.contains(&format!("event_stream={}", supervisor.event_count())));
    }
}
