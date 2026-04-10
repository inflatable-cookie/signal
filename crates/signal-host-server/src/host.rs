use std::collections::HashMap;

use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_lv2::Lv2HostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest,
    RuntimeClipProcessingRegistration, RuntimeError, RuntimeEventRecorder,
    RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeWarpClipRegistration,
    SignalRuntime,
};

#[path = "host_support.rs"]
mod host_support;
use host_support::{
    discovered_plugins_for_scan, ensure_au_sandbox_session, ensure_clap_sandbox_session,
    ensure_lv2_sandbox_session, ensure_vst3_sandbox_session,
    runtime_plugin_format_platform_coverage, teardown_broker_sandbox_session, SandboxBrokerSession,
    ServerSupervisorState,
};
pub use host_support::{
    ensure_default_demo_plugin_override, ServerExecutionSummary, ServerFaultSummary,
    ServerPayloadSummary, ServerRuntimeHostSummary, ServerTransportSummary,
};
pub(crate) use host_support::{
    samples_to_ms, FaultInjection, RecoveryFailureInjection, INTER_EPISODE_CONTINUITY_BLOCKS,
    SOAK_RESTART_EPISODES, STEADY_STATE_BLOCKS, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};

pub struct ServerRuntimeHost {
    runtime: SignalRuntime,
    broker: SharedMemoryBroker,
    clap: ClapPluginHostAdapter,
    au: AuHostAdapter,
    lv2: Lv2HostAdapter,
    vst3: Vst3HostAdapter,
    discovered_clap_types: HashMap<String, signal_plugin_clap::ClapDiscoveredPluginType>,
    discovered_au_types: HashMap<String, signal_plugin_au::AuDiscoveredPluginType>,
    discovered_lv2_types: HashMap<String, signal_plugin_lv2::Lv2DiscoveredPluginType>,
    discovered_vst3_types: HashMap<String, signal_plugin_vst3::Vst3DiscoveredPluginType>,
    active_sandbox_specs: HashMap<String, PluginSandboxSpec>,
    sandbox_broker_sessions: HashMap<String, SandboxBrokerSession>,
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
            clap: ClapPluginHostAdapter::default(),
            au: AuHostAdapter::default(),
            lv2: Lv2HostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            discovered_clap_types: HashMap::new(),
            discovered_au_types: HashMap::new(),
            discovered_lv2_types: HashMap::new(),
            discovered_vst3_types: HashMap::new(),
            active_sandbox_specs: HashMap::new(),
            sandbox_broker_sessions: HashMap::new(),
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
        let discoveries =
            discovered_plugins_for_scan(&self.clap, &self.au, &self.lv2, &self.vst3, &request);
        if request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap) {
            self.discovered_clap_types = discoveries
                .clap
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin))
                .collect();
        }
        if request.formats.is_empty() || request.formats.contains(&PluginFormat::Au) {
            self.discovered_au_types = discoveries
                .au
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin))
                .collect();
        }
        if request.formats.is_empty() || request.formats.contains(&PluginFormat::Lv2) {
            self.discovered_lv2_types = discoveries
                .lv2
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin))
                .collect();
        }
        if request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3) {
            self.discovered_vst3_types = discoveries
                .vst3
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin))
                .collect();
        }
        self.runtime.record_plugin_scan_results_with_diagnostics(
            handle,
            discoveries.runtime_records,
            discoveries.runtime_diagnostics,
        );
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
        self.active_sandbox_specs
            .insert(request.sandbox_id.clone(), request.clone());
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        if request.plugin_format == PluginFormat::Clap {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_clap_types.get(plugin_type_id))
                .cloned()
                .or_else(|| {
                    request
                        .plugin_type_id
                        .as_deref()
                        .and_then(|plugin_type_id| self.clap.discover_plugin_type(plugin_type_id))
                })
            {
                if let Some(session) = ensure_clap_sandbox_session(
                    &mut self.runtime,
                    &self.broker,
                    &self.clap,
                    &discovered,
                    &request,
                )? {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server CLAP scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Au {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_au_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) =
                    ensure_au_sandbox_session(&mut self.runtime, &self.au, &discovered, &request)?
                {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server AU scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Lv2 {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_lv2_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) =
                    ensure_lv2_sandbox_session(&mut self.runtime, &self.lv2, &discovered, &request)?
                {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server LV2 scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Vst3 {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_vst3_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) = ensure_vst3_sandbox_session(
                    &mut self.runtime,
                    &self.vst3,
                    &discovered,
                    &request,
                )? {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server VST3 scan",
                ));
            }
        } else {
            return Err(self.unsupported_or_missing_sandbox_error(
                &request,
                &format!(
                    "plugin format {:?} is not supported here yet on the server host sandbox path",
                    request.plugin_format
                ),
            ));
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
        if let Some(session) = self.sandbox_broker_sessions.remove(sandbox_id) {
            teardown_broker_sandbox_session(&mut self.runtime, sandbox_id, session)?;
        }
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
        let Some(request) = self.active_sandbox_specs.get(sandbox_id).cloned() else {
            return Ok(());
        };
        if request.plugin_format == PluginFormat::Clap {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_clap_types.get(plugin_type_id))
                .cloned()
                .or_else(|| {
                    request
                        .plugin_type_id
                        .as_deref()
                        .and_then(|plugin_type_id| self.clap.discover_plugin_type(plugin_type_id))
                })
            {
                if let Some(session) = ensure_clap_sandbox_session(
                    &mut self.runtime,
                    &self.broker,
                    &self.clap,
                    &discovered,
                    &request,
                )? {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server CLAP scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Au {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_au_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) =
                    ensure_au_sandbox_session(&mut self.runtime, &self.au, &discovered, &request)?
                {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server AU scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Lv2 {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_lv2_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) =
                    ensure_lv2_sandbox_session(&mut self.runtime, &self.lv2, &discovered, &request)?
                {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server LV2 scan",
                ));
            }
        } else if request.plugin_format == PluginFormat::Vst3 {
            if let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_vst3_types.get(plugin_type_id))
                .cloned()
            {
                if let Some(session) = ensure_vst3_sandbox_session(
                    &mut self.runtime,
                    &self.vst3,
                    &discovered,
                    &request,
                )? {
                    self.sandbox_broker_sessions
                        .insert(request.sandbox_id.clone(), session);
                }
            } else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    &request,
                    "plugin type was not discovered in the last server VST3 scan",
                ));
            }
        } else {
            return Err(self.unsupported_or_missing_sandbox_error(
                &request,
                &format!(
                    "plugin format {:?} is not supported here yet on the server host sandbox path",
                    request.plugin_format
                ),
            ));
        }
        Ok(())
    }

    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError> {
        self.supervisor.backend_policy = Some(request.tier);
        Ok(())
    }
}

impl ServerRuntimeHost {
    fn unsupported_or_missing_sandbox_error(
        &mut self,
        request: &PluginSandboxSpec,
        detail: &str,
    ) -> RuntimeError {
        self.runtime.record_plugin_sandbox_fault(
            request.sandbox_id.as_str(),
            signal_runtime::PluginFaultKind::ProtocolViolation,
            detail,
            None,
        );
        RuntimeError::new(signal_runtime::RuntimeErrorKind::InvalidRequest, detail)
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
