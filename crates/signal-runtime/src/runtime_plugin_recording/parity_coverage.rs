use super::coverage::{
    runtime_plugin_host_platform_sort_key, runtime_plugin_parity_band,
    runtime_plugin_platform_scope_summary,
};
use super::*;

fn runtime_plugin_format_rule_count(
    policy: &RuntimePluginPlacementPolicy,
    format: PluginFormat,
) -> usize {
    policy
        .rules
        .iter()
        .filter(|rule| {
            matches!(
                rule.matcher,
                RuntimePluginPlacementRuleMatcher::PluginFormat(matcher) if matcher == format
            )
        })
        .count()
}

pub(crate) fn runtime_plugin_parity_coverage(
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
    sandboxes: &[RuntimePluginSandboxSnapshot],
    policy: &RuntimePluginPlacementPolicy,
    platform_coverage: &[RuntimePluginFormatPlatformCoverageRecord],
) -> Vec<RuntimePluginFormatParityRecord> {
    let mut formats = discovered_types
        .iter()
        .map(|record| record.format)
        .chain(sandboxes.iter().filter_map(|sandbox| sandbox.plugin_format))
        .chain(platform_coverage.iter().map(|record| record.format))
        .collect::<Vec<_>>();
    formats.sort_by_key(|format| plugin_format_sort_key(*format));
    formats.dedup();

    formats
        .into_iter()
        .map(|format| {
            let coverage = platform_coverage.iter().find(|record| record.format == format);
            let mut supported_platforms = coverage.map(|coverage| coverage.supported_platforms.clone()).unwrap_or_default();
            supported_platforms.sort_by_key(|platform| runtime_plugin_host_platform_sort_key(*platform));
            supported_platforms.dedup();
            let mut unsupported_platforms = coverage.map(|coverage| coverage.unsupported_platforms.clone()).unwrap_or_default();
            unsupported_platforms.sort_by_key(|platform| runtime_plugin_host_platform_sort_key(*platform));
            unsupported_platforms.dedup();

            let discovered_type_count = discovered_types.iter().filter(|record| record.format == format).count();
            let prepare_capable_type_count = discovered_types.iter().filter(|record| record.format == format && record.lifecycle_contract.supports_prepare).count();
            let activate_capable_type_count = discovered_types.iter().filter(|record| record.format == format && record.lifecycle_contract.supports_activate).count();
            let sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format)).count();
            let in_process_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.placement_outcome == RuntimePluginIsolationOutcome::InProcess).count();
            let shared_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.placement_outcome == RuntimePluginIsolationOutcome::SharedSandbox).count();
            let isolated_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.placement_outcome == RuntimePluginIsolationOutcome::IsolatedSandbox).count();
            let ready_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.state == RuntimePluginLifecycleState::Ready).count();
            let restarting_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.state == RuntimePluginLifecycleState::Restarting).count();
            let rebindable_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.rebindable).count();
            let degraded_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.state == RuntimePluginLifecycleState::Degraded).count();
            let faulted_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.state == RuntimePluginLifecycleState::Faulted).count();
            let quarantined_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.state == RuntimePluginLifecycleState::Quarantined).count();
            let terminal_sandbox_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.continuity_class == RuntimeInterruptionClass::Terminal).count();
            let active_transport_count = sandboxes.iter().filter(|sandbox| sandbox.plugin_format == Some(format) && sandbox.active_transport).count();
            let explicit_placement_rule_count = runtime_plugin_format_rule_count(policy, format);
            let parity_band = runtime_plugin_parity_band(coverage);
            let linux_parity_band = coverage.map(|coverage| coverage.linux_parity_band).unwrap_or(RuntimePluginParityBand::Guarded);
            let linux_supported = coverage.map(|coverage| coverage.supported_platforms.contains(&RuntimePluginHostPlatform::Linux)).unwrap_or(false);
            let linux_preferred_sandbox_outcome = coverage.and_then(|coverage| coverage.linux_preferred_sandbox_outcome);
            let linux_strict_sandbox_default = coverage.map(|coverage| coverage.linux_strict_sandbox_default).unwrap_or(false);

            RuntimePluginFormatParityRecord {
                format,
                parity_band,
                linux_parity_band,
                supported_platforms,
                unsupported_platforms,
                linux_supported,
                linux_preferred_sandbox_outcome,
                linux_strict_sandbox_default,
                discovered_type_count,
                prepare_capable_type_count,
                activate_capable_type_count,
                sandbox_count,
                in_process_sandbox_count,
                shared_sandbox_count,
                isolated_sandbox_count,
                ready_sandbox_count,
                restarting_sandbox_count,
                rebindable_sandbox_count,
                degraded_sandbox_count,
                faulted_sandbox_count,
                quarantined_sandbox_count,
                terminal_sandbox_count,
                active_transport_count,
                explicit_placement_rule_count,
                summary: format!(
                    "format={format:?} parity={parity_band:?} linux={linux_parity_band:?} linux_supported={linux_supported} linux_policy={linux_preferred_sandbox_outcome:?} linux_strict_default={linux_strict_sandbox_default} {} discovered_types={} prepare_capable={} activate_capable={} sandboxes={} in_process={} shared={} isolated={} ready={} restarting={} rebindable={} degraded={} faulted={} quarantined={} terminal={} active_transport={} placement_rules={}",
                    runtime_plugin_platform_scope_summary(coverage),
                    discovered_type_count,
                    prepare_capable_type_count,
                    activate_capable_type_count,
                    sandbox_count,
                    in_process_sandbox_count,
                    shared_sandbox_count,
                    isolated_sandbox_count,
                    ready_sandbox_count,
                    restarting_sandbox_count,
                    rebindable_sandbox_count,
                    degraded_sandbox_count,
                    faulted_sandbox_count,
                    quarantined_sandbox_count,
                    terminal_sandbox_count,
                    active_transport_count,
                    explicit_placement_rule_count,
                ),
            }
        })
        .collect()
}
