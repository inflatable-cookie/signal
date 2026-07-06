use super::*;

impl RuntimeSupervisorApi for LocalRuntimeHost {
    fn start_plugin_scan(
        &mut self,
        request: PluginScanRequest,
    ) -> Result<signal_runtime::ScanHandle, RuntimeError> {
        let handle = self.runtime.record_plugin_scan_request(&request);
        let discoveries = discovered_plugins_for_scan(&self.clap, &self.au, &self.vst3, &request);
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
        if request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3) {
            self.discovered_vst3_types = discoveries
                .vst3
                .iter()
                .cloned()
                .map(|plugin| (plugin.plugin_type_id.0.clone(), plugin))
                .collect();
        }
        self.runtime
            .record_plugin_scan_results(handle, discoveries.runtime_records);
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
        self.ensure_sandbox_session_for_request(&request)?;
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

    fn reconcile_offline_stretch_artifact_plans(
        &mut self,
        plans: Vec<RuntimeOfflineStretchArtifactPlanRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_offline_stretch_artifact_plans(plans)
    }

    fn reconcile_offline_stretch_artifact_materializations(
        &mut self,
        artifacts: Vec<RuntimeOfflineStretchArtifactMaterializationRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime
            .reconcile_offline_stretch_artifact_materializations(artifacts)
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
        self.ensure_sandbox_session_for_request(&request)?;
        Ok(())
    }

    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError> {
        self.supervisor.backend_policy = Some(request.tier);
        Ok(())
    }
}

impl LocalRuntimeHost {
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

impl LocalRuntimeHost {
    fn ensure_sandbox_session_for_request(
        &mut self,
        request: &PluginSandboxSpec,
    ) -> Result<(), RuntimeError> {
        let session = if request.plugin_format == PluginFormat::Clap {
            let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_clap_types.get(plugin_type_id))
                .cloned()
            else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    request,
                    "plugin type was not discovered in the last local CLAP scan",
                ));
            };
            ensure_discovered_sandbox_session(
                &mut self.runtime,
                request,
                "clap",
                discovered.plugin_type_id.0.as_str(),
                discovered.default_io_layout,
                vec![(
                    "SIGNAL_PLUGIN_SANDBOX_CLAP_LIBRARY_PATH".into(),
                    discovered.library_path.clone(),
                )],
            )?
        } else if request.plugin_format == PluginFormat::Au {
            let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_au_types.get(plugin_type_id))
                .cloned()
            else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    request,
                    "plugin type was not discovered in the last local AU scan",
                ));
            };
            ensure_discovered_sandbox_session(
                &mut self.runtime,
                request,
                "au",
                discovered.plugin_type_id.0.as_str(),
                discovered.default_io_layout,
                vec![(
                    "SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT".into(),
                    discovered.bundle_root.clone(),
                )],
            )?
        } else if request.plugin_format == PluginFormat::Vst3 {
            let Some(discovered) = request
                .plugin_type_id
                .as_deref()
                .and_then(|plugin_type_id| self.discovered_vst3_types.get(plugin_type_id))
                .cloned()
            else {
                return Err(self.unsupported_or_missing_sandbox_error(
                    request,
                    "plugin type was not discovered in the last local VST3 scan",
                ));
            };
            ensure_discovered_sandbox_session(
                &mut self.runtime,
                request,
                "vst3",
                discovered.plugin_type_id.0.as_str(),
                discovered.default_io_layout,
                vec![(
                    "SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT".into(),
                    discovered.module_root.clone(),
                )],
            )?
        } else {
            return Err(self.unsupported_or_missing_sandbox_error(
                request,
                &format!(
                    "plugin format {:?} is not supported here yet on the local host sandbox path",
                    request.plugin_format
                ),
            ));
        };
        if let Some(session) = session {
            self.sandbox_broker_sessions
                .insert(request.sandbox_id.clone(), session);
        }
        Ok(())
    }
}
