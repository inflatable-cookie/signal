use super::*;
use crate::interfaces::RuntimePluginScanDiagnosticRecord;

impl SignalRuntime {
    /// Registers a plugin scan request and returns a handle used to deliver results.
    pub fn record_plugin_scan_request(&mut self, request: &PluginScanRequest) -> ScanHandle {
        self.plugin_discovery.record_scan(request)
    }

    /// Records the set of plugin format platform coverage entries for parity classification.
    pub fn record_plugin_format_platform_coverage(
        &mut self,
        coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
    ) {
        self.plugin_discovery.record_platform_coverage(coverage);
    }

    /// Records plugin scan results without diagnostic data.
    pub fn record_plugin_scan_results(
        &mut self,
        scan_handle: ScanHandle,
        discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    ) {
        self.record_plugin_scan_results_with_diagnostics(scan_handle, discovered_types, Vec::new());
    }

    /// Records plugin scan results together with per-plugin diagnostic records.
    pub fn record_plugin_scan_results_with_diagnostics(
        &mut self,
        scan_handle: ScanHandle,
        discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
        discovery_diagnostics: Vec<RuntimePluginScanDiagnosticRecord>,
    ) {
        let lifecycle = self.plugin_lifecycle_snapshot();
        let parity_coverage = runtime_plugin_parity_coverage(
            &discovered_types,
            &lifecycle.sandboxes,
            &self.plugin_placement_policy,
            &self.plugin_discovery.platform_coverage,
        );
        self.plugin_discovery.record_scan_results(
            scan_handle,
            discovered_types,
            discovery_diagnostics,
            parity_coverage,
        );
    }
}
