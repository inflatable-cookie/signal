use super::*;

impl RuntimePluginPinMatrixSnapshot {
    /// Builds a pin matrix snapshot by cross-referencing discovered plugin types, sandbox lifecycle state, and chain stages.
    pub fn capture(
        discovery: &RuntimePluginDiscoverySnapshot,
        lifecycle: &RuntimePluginLifecycleSnapshot,
        plugin_chain: &RuntimePluginChainSnapshot,
    ) -> Self {
        let mut records = Vec::new();
        let mut negotiated_type_count = 0usize;
        let mut guarded_type_count = 0usize;
        let mut unavailable_type_count = 0usize;
        let mut dynamic_negotiated_type_count = 0usize;
        let mut dynamic_guarded_type_count = 0usize;

        for record in discovery
            .discovered_types
            .iter()
            .filter(|record| record.complex_io_summary.has_complex_topology)
        {
            let sandboxes = lifecycle
                .sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.plugin_type_id.as_deref() == Some(record.plugin_type_id.as_str())
                })
                .collect::<Vec<_>>();
            let matching_stages = plugin_chain
                .chains
                .iter()
                .flat_map(|chain| chain.stages.iter())
                .filter(|stage| {
                    stage.sandbox_id.as_deref().is_some_and(|sandbox_id| {
                        sandboxes
                            .iter()
                            .any(|sandbox| sandbox.sandbox_id == sandbox_id)
                    })
                })
                .collect::<Vec<_>>();
            let strongest_lifecycle_state = sandboxes
                .iter()
                .max_by_key(|sandbox| runtime_plugin_lifecycle_state_severity(sandbox.state))
                .map(|sandbox| sandbox.state);
            let unavailable_lifecycle = !sandboxes.is_empty()
                && sandboxes.iter().all(|sandbox| {
                    matches!(
                        sandbox.state,
                        RuntimePluginLifecycleState::Faulted
                            | RuntimePluginLifecycleState::Quarantined
                            | RuntimePluginLifecycleState::Stopped
                    )
                });
            let guarded_lifecycle = sandboxes.iter().any(|sandbox| {
                matches!(
                    sandbox.state,
                    RuntimePluginLifecycleState::Degraded
                        | RuntimePluginLifecycleState::Restarting
                        | RuntimePluginLifecycleState::Faulted
                        | RuntimePluginLifecycleState::Quarantined
                )
            }) || matching_stages
                .iter()
                .any(|stage| !stage.degraded_reasons.is_empty());
            let stage_count = matching_stages.len();
            let active_stage_count = matching_stages
                .iter()
                .filter(|stage| {
                    stage.transport_stage == Some(PluginSandboxTransportStage::Attached)
                })
                .count();
            let pin_matrix_posture = if unavailable_lifecycle {
                RuntimePluginPinMatrixPosture::Unavailable
            } else if guarded_lifecycle {
                RuntimePluginPinMatrixPosture::Guarded
            } else if active_stage_count > 0 {
                RuntimePluginPinMatrixPosture::Negotiated
            } else {
                RuntimePluginPinMatrixPosture::Declared
            };
            let dynamic_surface_declared = record.complex_io_summary.multi_output_instrument
                || record.complex_io_summary.bus_capable_fx_class.is_some()
                || record.complex_io_summary.secondary_input_group_count > 0
                || record.complex_io_summary.aux_input_group_count > 0
                || record.complex_io_summary.aux_output_group_count > 0;
            let dynamic_bus_negotiation_posture = if unavailable_lifecycle {
                RuntimeDynamicBusNegotiationPosture::Unavailable
            } else if !dynamic_surface_declared {
                RuntimeDynamicBusNegotiationPosture::Static
            } else if guarded_lifecycle {
                RuntimeDynamicBusNegotiationPosture::Guarded
            } else if active_stage_count > 0 {
                RuntimeDynamicBusNegotiationPosture::Negotiated
            } else {
                RuntimeDynamicBusNegotiationPosture::Static
            };
            let mut pin_group_identities = Vec::new();
            if record.complex_io_summary.main_input_group_count > 0
                || record.complex_io_summary.main_output_group_count > 0
            {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::PrimaryProgramPath);
            }
            if record.complex_io_summary.instrument_output_group_count > 0 {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::SecondaryProgramPath);
            }
            if record.complex_io_summary.secondary_input_group_count > 0 {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::SidechainPath);
            }
            if record.complex_io_summary.aux_input_group_count > 0
                || record.complex_io_summary.aux_output_group_count > 0
            {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::AuxReturnPath);
            }
            if record.complex_io_summary.analysis_output_group_count > 0 {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::AnalysisPath);
            }
            if active_stage_count == 0 {
                pin_group_identities.push(RuntimePluginPinGroupIdentity::InactiveDeclaredPath);
            }
            let fallback_outcome = runtime_plugin_topology_fallback_to_negotiation_outcome(
                record.complex_io_summary.fallback_outcome,
            );
            match pin_matrix_posture {
                RuntimePluginPinMatrixPosture::Negotiated => negotiated_type_count += 1,
                RuntimePluginPinMatrixPosture::Guarded => guarded_type_count += 1,
                RuntimePluginPinMatrixPosture::Unavailable => unavailable_type_count += 1,
                RuntimePluginPinMatrixPosture::Simple | RuntimePluginPinMatrixPosture::Declared => {
                }
            }
            match dynamic_bus_negotiation_posture {
                RuntimeDynamicBusNegotiationPosture::Negotiated => {
                    dynamic_negotiated_type_count += 1
                }
                RuntimeDynamicBusNegotiationPosture::Guarded => dynamic_guarded_type_count += 1,
                RuntimeDynamicBusNegotiationPosture::Static
                | RuntimeDynamicBusNegotiationPosture::Unavailable => {}
            }
            let pin_record = RuntimePluginPinMatrixRecord {
                plugin_type_id: record.plugin_type_id.clone(),
                plugin_id: record.plugin_id.clone(),
                pin_group_identities,
                pin_matrix_posture,
                dynamic_bus_negotiation_posture,
                fallback_outcome,
                strongest_lifecycle_state,
                stage_count,
                active_stage_count,
            };
            records.push(pin_record);
        }

        let plugin_type_count = records.len();
        let snapshot = Self {
            plugin_type_count,
            negotiated_type_count,
            guarded_type_count,
            unavailable_type_count,
            dynamic_negotiated_type_count,
            dynamic_guarded_type_count,
            records,
        };
        snapshot
    }
}
