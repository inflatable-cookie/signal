use super::*;
use crate::interfaces::RuntimePluginScanDiagnosticRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeSupervisionPolicy {
    pub(crate) safe_mode_restart_threshold: u32,
    pub(crate) safe_mode_xrun_threshold: u64,
}

impl Default for RuntimeSupervisionPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
            safe_mode_xrun_threshold: 3,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimeSupervisionState {
    pub(crate) policy: RuntimeSupervisionPolicy,
    pub(crate) watchdog_restart_count: u32,
    pub(crate) xrun_overload_active: bool,
    pub(crate) last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub(crate) last_sandbox_id: Option<String>,
    pub(crate) last_processing_epoch: Option<u64>,
}

impl RuntimeSupervisionState {
    pub(super) fn snapshot(&self, safe_mode_enabled: bool) -> RuntimeSupervisionSnapshot {
        RuntimeSupervisionSnapshot {
            watchdog_restart_count: self.watchdog_restart_count,
            safe_mode_enabled,
            xrun_overload_active: self.xrun_overload_active,
            last_watchdog_trigger: self.last_watchdog_trigger,
            last_sandbox_id: self.last_sandbox_id.clone(),
            last_processing_epoch: self.last_processing_epoch,
        }
    }

    pub(super) fn record_watchdog_restart(&mut self, record: WatchdogRestartRecord) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        self.last_watchdog_trigger = Some(record.trigger);
        self.last_sandbox_id = Some(record.sandbox_id);
        self.last_processing_epoch = Some(record.processing_epoch);
        self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold
    }

    pub(super) fn record_xrun_overload(
        &mut self,
        processing_epoch: Option<u64>,
        xruns: u64,
    ) -> bool {
        if let Some(processing_epoch) = processing_epoch {
            self.last_processing_epoch = Some(processing_epoch);
        }
        if xruns >= self.policy.safe_mode_xrun_threshold {
            self.xrun_overload_active = true;
        }
        self.xrun_overload_active
    }

    pub(super) fn clear_xrun_overload_recovery(&mut self) {
        self.xrun_overload_active = false;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimePluginDiscoveryStateModel {
    pub(crate) scan_count: usize,
    pub(crate) format_filtered_scan_count: usize,
    pub(crate) next_scan_handle: u64,
    pub(crate) last_scan: Option<RuntimePluginScanReceipt>,
    pub(crate) discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    pub(crate) platform_coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
}

impl RuntimePluginDiscoveryStateModel {
    pub(super) fn record_platform_coverage(
        &mut self,
        coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
    ) {
        self.platform_coverage = coverage;
    }

    pub(super) fn record_scan(&mut self, request: &PluginScanRequest) -> ScanHandle {
        self.next_scan_handle = self.next_scan_handle.saturating_add(1);
        self.scan_count = self.scan_count.saturating_add(1);
        if !request.formats.is_empty() {
            self.format_filtered_scan_count = self.format_filtered_scan_count.saturating_add(1);
        }
        let scan_handle = ScanHandle(self.next_scan_handle);
        self.last_scan = Some(RuntimePluginScanReceipt {
            scan_handle,
            roots: request.roots.clone(),
            formats: request.formats.clone(),
            targeted_format_count: request.formats.len(),
            discovered_type_count: 0,
            discovered_format_count: 0,
            discovery_diagnostic_count: 0,
            discovery_diagnostics: Vec::new(),
            format_coverage: Vec::new(),
            parity_coverage: Vec::new(),
            capability_coverage: RuntimePluginCapabilityCoverageSummary {
                ..RuntimePluginCapabilityCoverageSummary::default()
            },
        });
        scan_handle
    }

    pub(super) fn record_scan_results(
        &mut self,
        scan_handle: ScanHandle,
        discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
        discovery_diagnostics: Vec<RuntimePluginScanDiagnosticRecord>,
        parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    ) {
        let format_coverage = runtime_plugin_format_coverage(&discovered_types);
        let capability_coverage = runtime_plugin_capability_coverage(&discovered_types);
        if let Some(last_scan) = self.last_scan.as_mut() {
            if last_scan.scan_handle == scan_handle {
                last_scan.discovered_type_count = discovered_types.len();
                last_scan.discovered_format_count = format_coverage.len();
                last_scan.discovery_diagnostic_count = discovery_diagnostics.len();
                last_scan.discovery_diagnostics = discovery_diagnostics;
                last_scan.format_coverage = format_coverage;
                last_scan.parity_coverage = parity_coverage;
                last_scan.capability_coverage = capability_coverage;
                self.discovered_types = discovered_types;
            }
        }
    }

    pub(super) fn snapshot(
        &self,
        parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    ) -> RuntimePluginDiscoverySnapshot {
        let format_coverage = runtime_plugin_format_coverage(&self.discovered_types);
        let capability_coverage = runtime_plugin_capability_coverage(&self.discovered_types);
        RuntimePluginDiscoverySnapshot {
            scan_count: self.scan_count,
            format_filtered_scan_count: self.format_filtered_scan_count,
            discovered_type_count: self.discovered_types.len(),
            discovered_format_count: format_coverage.len(),
            last_scan: self.last_scan.clone(),
            format_coverage,
            parity_coverage,
            capability_coverage,
            discovered_types: self.discovered_types.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeRecordingCapturePolicy {
    pub(crate) pressure_threshold_frames: u64,
}

impl Default for RuntimeRecordingCapturePolicy {
    fn default() -> Self {
        Self {
            pressure_threshold_frames: 16_384,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeMediaPipelinePolicy {
    pub(crate) cache_root: PathBuf,
}

impl Default for RuntimeMediaPipelinePolicy {
    fn default() -> Self {
        Self {
            cache_root: std::env::temp_dir().join("loophole-signal-media-cache"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeMediaAnalysisStateModel {
    pub(crate) descriptor_state: RuntimeMediaAnalysisDescriptorState,
    pub(crate) loudness: Option<RuntimeMediaLoudnessDescriptor>,
    pub(crate) character: Option<RuntimeMediaCharacterDescriptor>,
    pub(crate) last_error: Option<String>,
}

impl Default for RuntimeMediaAnalysisStateModel {
    fn default() -> Self {
        Self {
            descriptor_state: RuntimeMediaAnalysisDescriptorState::Missing,
            loudness: None,
            character: None,
            last_error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeMediaPipelineAsset {
    pub(crate) registration: RuntimeMediaAssetRegistration,
    pub(crate) state: RuntimeMediaAssetState,
    pub(crate) cache_path: Option<String>,
    pub(crate) cache_byte_size: Option<u64>,
    pub(crate) rebuild_count: u32,
    pub(crate) last_error: Option<String>,
    pub(crate) analysis: RuntimeMediaAnalysisStateModel,
}

pub(crate) fn runtime_plugin_chain_id(
    track_lane_id: Option<&str>,
    bus_group_id: Option<&str>,
    console_group_id: Option<&str>,
    send_return_id: Option<&str>,
) -> String {
    track_lane_id
        .map(str::to_string)
        .or_else(|| bus_group_id.map(str::to_string))
        .or_else(|| console_group_id.map(str::to_string))
        .or_else(|| send_return_id.map(str::to_string))
        .unwrap_or_else(|| "global".into())
}

pub(crate) fn runtime_plugin_discovered_type_for_recall<'a>(
    plugin_type_id: Option<&str>,
    discovered_types: &'a [RuntimePluginDiscoveredTypeRecord],
) -> Option<&'a RuntimePluginDiscoveredTypeRecord> {
    let plugin_type_id = plugin_type_id?;
    discovered_types
        .iter()
        .find(|record| record.plugin_type_id == plugin_type_id)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginCompensationObservation {
    pub(crate) state: RuntimePluginCompensationState,
    pub(crate) realized_latency_samples: Option<u32>,
    pub(crate) tail_samples: Option<u32>,
}
