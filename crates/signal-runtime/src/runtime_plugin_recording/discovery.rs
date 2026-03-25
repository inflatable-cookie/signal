use super::*;

impl SignalRuntime {
    pub fn record_plugin_scan_request(&mut self, request: &PluginScanRequest) -> ScanHandle {
        self.plugin_discovery.record_scan(request)
    }

    pub fn record_plugin_format_platform_coverage(
        &mut self,
        coverage: Vec<RuntimePluginFormatPlatformCoverageRecord>,
    ) {
        self.plugin_discovery.record_platform_coverage(coverage);
    }

    pub fn record_plugin_scan_results(
        &mut self,
        scan_handle: ScanHandle,
        discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    ) {
        let lifecycle = self.plugin_lifecycle_snapshot();
        let parity_coverage = runtime_plugin_parity_coverage(
            &discovered_types,
            &lifecycle.sandboxes,
            &self.plugin_placement_policy,
            &self.plugin_discovery.platform_coverage,
        );
        self.plugin_discovery
            .record_scan_results(scan_handle, discovered_types, parity_coverage);
    }
}
