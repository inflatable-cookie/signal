use std::cell::RefCell;

use signal_hardware::HardwareStreamConfig;
use signal_hardware_coreaudio::CoreAudioBackend;
use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_clap::{ClapBlockProtocol, ClapPluginHostAdapter, ClapSandboxLifecycleHarness};
use signal_plugin_vst3::Vst3HostAdapter;
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount};
use signal_runtime::{
    BackendPolicyOverride, HandshakeRequest, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginScanRequest, RuntimeClipProcessingRegistration, RuntimeConfigRequest, RuntimeError,
    RuntimeEventRecorder, RuntimeHostObservationReport, RuntimeHostSupervisorReport,
    RuntimeLifecycleApi, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimeProjectionApi, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeWarpClipRegistration,
    SignalRuntime,
};

#[path = "host_support.rs"]
mod host_support;
use host_support::{
    discovered_plugins_for_scan, ensure_au_sandbox_session, ensure_vst3_sandbox_session,
    local_demo_runtime_assembly, runtime_plugin_format_platform_coverage, LifecycleRunSummary,
    LocalAudioPumpState, LocalClockTransitionMemory, LocalSupervisorState, STEADY_STATE_BLOCKS,
};
pub use host_support::{
    LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy, LocalExecutionSummary,
    LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary, LocalPluginDispatchSummary,
    LocalRuntimeHostSummary, LocalTransportSummary,
};
pub(crate) use host_support::{
    FaultInjection, INTER_EPISODE_CONTINUITY_BLOCKS, LOCAL_DEMO_GRAPH_ID,
    LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES, LOCAL_DEMO_PLUGIN_NODE_ID,
    LOCAL_DEMO_PLUGIN_TAIL_SAMPLES, RecoveryFailureInjection, SOAK_RESTART_EPISODES,
    WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};

pub struct LocalRuntimeHost {
    runtime: SignalRuntime,
    coreaudio: CoreAudioBackend,
    clap: ClapPluginHostAdapter,
    au: AuHostAdapter,
    vst3: Vst3HostAdapter,
    broker: SharedMemoryBroker,
    active_output_stream: Option<HardwareStreamConfig>,
    clock_transition_memory: RefCell<LocalClockTransitionMemory>,
    audio_pump: LocalAudioPumpState,
    supervisor: LocalSupervisorState,
    events: RuntimeEventRecorder,
}

impl LocalRuntimeHost {
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));
        runtime.record_plugin_format_platform_coverage(runtime_plugin_format_platform_coverage());

        Self {
            runtime,
            coreaudio: CoreAudioBackend::default(),
            clap: ClapPluginHostAdapter::default(),
            au: AuHostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            broker: SharedMemoryBroker::default(),
            active_output_stream: None,
            clock_transition_memory: RefCell::new(LocalClockTransitionMemory::default()),
            audio_pump: LocalAudioPumpState::default(),
            supervisor: LocalSupervisorState::default(),
            events,
        }
    }

    fn boot_with_fault_recovery(
        &mut self,
        fault: Option<FaultInjection>,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        let runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;
        let assembly = local_demo_runtime_assembly();
        self.runtime
            .apply_graph_projection(assembly.graph.clone())?;
        self.runtime
            .apply_graph_contract_projection(assembly.graph_contracts.clone())?;

        let hardware_stream = self.prepare_default_output_hardware()?;

        self.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        })?;

        for sandbox in &assembly.plugin_sandboxes {
            self.ensure_plugin_sandbox(sandbox.spec())?;
        }
        self.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())?;
        self.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        let sandbox = assembly.primary_sandbox();

        self.runtime.set_cpu_load_percent(4.5);
        self.runtime.set_graph_latency_ms(2.7);
        self.runtime.start()?;

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:default",
            "instance:local:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let mut run = self.run_lifecycle(
            &protocol,
            sandbox.request.sandbox_id.as_str(),
            1,
            &mut lifecycle,
        )?;
        let executed_steady_state_tail = if let Some(fault) = fault {
            let executed_steady_state_tail = self.apply_boot_fault_recovery(
                &protocol,
                sandbox,
                &mut run,
                &mut lifecycle,
                fault,
            )?;
            if !executed_steady_state_tail {
                self.execute_block_sequence(
                    &protocol,
                    &mut run,
                    STEADY_STATE_BLOCKS,
                    &mut lifecycle,
                    false,
                )?;
            }
            executed_steady_state_tail
        } else {
            self.execute_block_sequence(
                &protocol,
                &mut run,
                STEADY_STATE_BLOCKS,
                &mut lifecycle,
                false,
            )?;
            false
        };
        let _ = executed_steady_state_tail;
        Ok(self.summarize_boot_outcome(
            &hardware_stream,
            &sandbox.request.sandbox_id,
            &protocol,
            run,
        ))
    }

    fn process_engine_block_through_output_pump(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<signal_runtime::RuntimeEngineBlockResult, RuntimeError> {
        let Some(stream) = self.active_output_stream.clone() else {
            self.audio_pump.fault();
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "local host audio pump has no negotiated output stream",
            ));
        };
        let input = AudioBuffer::new(
            self.runtime.config().sample_rate,
            ChannelLayout::Count(ChannelCount(stream.output_channels as usize)),
            FrameCount(stream.buffer_size),
        );
        let result = self
            .runtime
            .process_engine_block(processing_epoch, block_sequence, input)
            .inspect_err(|_| self.audio_pump.fault())?;
        self.audio_pump.record_callback(
            &stream,
            block_sequence,
            &result.output,
            result.snapshot.graph_id.as_deref(),
        );
        Ok(result)
    }

    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }

    pub fn clap_supported(&self) -> bool {
        self.clap.supports_format(PluginFormat::Clap)
    }

    pub fn host_observation_report(&self) -> RuntimeHostObservationReport {
        let (observation, host_io) = self.observation_with_host_io();
        RuntimeHostObservationReport::new(observation, host_io)
    }

    pub fn host_supervisor_report(&self) -> RuntimeHostSupervisorReport {
        let (supervisor, host_io) = self.supervisor_with_host_io();
        RuntimeHostSupervisorReport::new(supervisor, host_io)
    }
}

impl RuntimeSupervisorApi for LocalRuntimeHost {
    fn start_plugin_scan(
        &mut self,
        request: PluginScanRequest,
    ) -> Result<signal_runtime::ScanHandle, RuntimeError> {
        let handle = self.runtime.record_plugin_scan_request(&request);
        let discovered_types =
            discovered_plugins_for_scan(&self.clap, &self.au, &self.vst3, &request);
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
    include!("host_tests.rs");
}
