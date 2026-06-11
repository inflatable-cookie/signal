use super::*;
impl RuntimeLv2ExtensionSnapshot {
    /// Builds an LV2 extension snapshot by correlating discovered plugin types with sandbox lifecycle state.
    pub fn capture(
        discovery: &RuntimePluginDiscoverySnapshot,
        lifecycle: &RuntimePluginLifecycleSnapshot,
    ) -> Self {
        let mut records = Vec::new();
        let mut sandbox_count = 0usize;
        let mut worker_required_type_count = 0usize;
        let mut worker_guarded_type_count = 0usize;
        let mut urid_negotiated_type_count = 0usize;
        let mut patch_supported_type_count = 0usize;
        let mut negotiated_type_count = 0usize;
        let mut guarded_type_count = 0usize;
        let mut unavailable_type_count = 0usize;

        for record in discovery
            .discovered_types
            .iter()
            .filter(|record| record.format == PluginFormat::Lv2)
        {
            let capabilities = record
                .lv2_extension_capabilities
                .clone()
                .unwrap_or_else(RuntimeLv2ExtensionCapabilitySummary::absent);
            let sandboxes = lifecycle
                .sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.plugin_format == Some(PluginFormat::Lv2)
                        && sandbox.plugin_type_id.as_deref() == Some(record.plugin_type_id.as_str())
                })
                .collect::<Vec<_>>();
            let prepared_negotiation = sandboxes
                .iter()
                .filter_map(|sandbox| sandbox.lv2_prepared_negotiation.as_ref())
                .next();
            let active_sandbox_count = sandboxes
                .iter()
                .filter(|sandbox| sandbox.active || sandbox.active_transport)
                .count();
            let faulted_sandbox_count = sandboxes
                .iter()
                .filter(|sandbox| {
                    matches!(
                        sandbox.state,
                        RuntimePluginLifecycleState::Faulted
                            | RuntimePluginLifecycleState::Quarantined
                    )
                })
                .count();
            let strongest_lifecycle_state = sandboxes
                .iter()
                .max_by_key(|sandbox| runtime_plugin_lifecycle_state_severity(sandbox.state))
                .map(|sandbox| sandbox.state);
            let guarded_lifecycle = sandboxes.iter().any(|sandbox| {
                matches!(
                    sandbox.state,
                    RuntimePluginLifecycleState::Degraded | RuntimePluginLifecycleState::Restarting
                )
            }) || faulted_sandbox_count > 0;
            let unavailable_lifecycle = !sandboxes.is_empty()
                && sandboxes.iter().all(|sandbox| {
                    matches!(
                        sandbox.state,
                        RuntimePluginLifecycleState::Faulted
                            | RuntimePluginLifecycleState::Quarantined
                            | RuntimePluginLifecycleState::Stopped
                    )
                });

            let (
                worker_posture,
                urid_negotiation_posture,
                patch_exchange_posture,
                extension_negotiation_state,
            ) = if let Some(prepared_negotiation) = prepared_negotiation {
                (
                    prepared_negotiation.worker_posture,
                    prepared_negotiation.urid_negotiation_posture,
                    prepared_negotiation.patch_exchange_posture,
                    prepared_negotiation.extension_negotiation_state,
                )
            } else {
                let worker_posture = match capabilities.worker_capability {
                    RuntimeLv2WorkerCapability::Absent => RuntimeLv2WorkerPosture::WorkerAbsent,
                    RuntimeLv2WorkerCapability::Supported => {
                        if unavailable_lifecycle {
                            RuntimeLv2WorkerPosture::WorkerUnavailable
                        } else if guarded_lifecycle {
                            RuntimeLv2WorkerPosture::WorkerGuarded
                        } else {
                            RuntimeLv2WorkerPosture::WorkerAvailable
                        }
                    }
                    RuntimeLv2WorkerCapability::Required => {
                        if unavailable_lifecycle {
                            RuntimeLv2WorkerPosture::WorkerUnavailable
                        } else if guarded_lifecycle {
                            RuntimeLv2WorkerPosture::WorkerGuarded
                        } else {
                            RuntimeLv2WorkerPosture::WorkerRequiredAvailable
                        }
                    }
                };
                let urid_negotiation_posture = match capabilities.urid_capability {
                    RuntimeLv2UridCapability::NotRequired => {
                        RuntimeLv2UridNegotiationPosture::NotRequired
                    }
                    RuntimeLv2UridCapability::Supported | RuntimeLv2UridCapability::Required => {
                        if unavailable_lifecycle {
                            RuntimeLv2UridNegotiationPosture::Unavailable
                        } else if guarded_lifecycle {
                            RuntimeLv2UridNegotiationPosture::Guarded
                        } else {
                            RuntimeLv2UridNegotiationPosture::Negotiated
                        }
                    }
                };
                let patch_exchange_posture = match capabilities.patch_capability {
                    RuntimeLv2PatchCapability::Absent => RuntimeLv2PatchExchangePosture::Absent,
                    RuntimeLv2PatchCapability::Supported => {
                        if unavailable_lifecycle {
                            RuntimeLv2PatchExchangePosture::Unavailable
                        } else if guarded_lifecycle {
                            RuntimeLv2PatchExchangePosture::Guarded
                        } else {
                            RuntimeLv2PatchExchangePosture::Supported
                        }
                    }
                };
                let extension_needed = capabilities.negotiated_extension_count > 0
                    || capabilities.worker_capability != RuntimeLv2WorkerCapability::Absent
                    || capabilities.urid_capability != RuntimeLv2UridCapability::NotRequired
                    || capabilities.patch_capability != RuntimeLv2PatchCapability::Absent;
                let extension_negotiation_state = if !extension_needed {
                    RuntimeLv2ExtensionNegotiationState::NotRequired
                } else if unavailable_lifecycle {
                    RuntimeLv2ExtensionNegotiationState::Unavailable
                } else if matches!(
                    worker_posture,
                    RuntimeLv2WorkerPosture::WorkerGuarded
                        | RuntimeLv2WorkerPosture::WorkerUnavailable
                ) || matches!(
                    urid_negotiation_posture,
                    RuntimeLv2UridNegotiationPosture::Guarded
                        | RuntimeLv2UridNegotiationPosture::Unavailable
                ) || matches!(
                    patch_exchange_posture,
                    RuntimeLv2PatchExchangePosture::Guarded
                        | RuntimeLv2PatchExchangePosture::Unavailable
                ) {
                    RuntimeLv2ExtensionNegotiationState::PartiallySatisfied
                } else {
                    RuntimeLv2ExtensionNegotiationState::Negotiated
                };
                (
                    worker_posture,
                    urid_negotiation_posture,
                    patch_exchange_posture,
                    extension_negotiation_state,
                )
            };

            if capabilities.worker_capability == RuntimeLv2WorkerCapability::Required {
                worker_required_type_count += 1;
            }
            if worker_posture == RuntimeLv2WorkerPosture::WorkerGuarded {
                worker_guarded_type_count += 1;
            }
            if urid_negotiation_posture == RuntimeLv2UridNegotiationPosture::Negotiated {
                urid_negotiated_type_count += 1;
            }
            if patch_exchange_posture == RuntimeLv2PatchExchangePosture::Supported {
                patch_supported_type_count += 1;
            }
            match extension_negotiation_state {
                RuntimeLv2ExtensionNegotiationState::Negotiated => negotiated_type_count += 1,
                RuntimeLv2ExtensionNegotiationState::PartiallySatisfied => guarded_type_count += 1,
                RuntimeLv2ExtensionNegotiationState::Unavailable => unavailable_type_count += 1,
                RuntimeLv2ExtensionNegotiationState::NotRequired => {}
            }

            sandbox_count += sandboxes.len();
            let lv2_record = RuntimeLv2ExtensionRecord {
                plugin_type_id: record.plugin_type_id.clone(),
                plugin_id: record.plugin_id.clone(),
                worker_posture,
                urid_negotiation_posture,
                patch_exchange_posture,
                extension_negotiation_state,
                strongest_lifecycle_state,
                sandbox_count: sandboxes.len(),
                active_sandbox_count,
                faulted_sandbox_count,
            };
            records.push(lv2_record);
        }

        let plugin_type_count = records.len();
        let snapshot = Self {
            plugin_type_count,
            sandbox_count,
            worker_required_type_count,
            worker_guarded_type_count,
            urid_negotiated_type_count,
            patch_supported_type_count,
            negotiated_type_count,
            guarded_type_count,
            unavailable_type_count,
            records,
        };
        snapshot
    }
}
