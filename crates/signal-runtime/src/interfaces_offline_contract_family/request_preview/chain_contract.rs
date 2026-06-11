use super::super::*;

impl RuntimeOfflineRenderContractPreview {
    /// Builds the chain dependency preview from a live execution topology and recall handoff snapshot.
    pub fn chain_contract_from_runtime_state(
        topology: &RuntimeExecutionTopologySummary,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<RuntimeOfflineRenderChainDependencyPreview, RuntimeError> {
        let plugin_chain = &topology.plugin_chain;
        if plugin_chain.stage_count != recall_handoff.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                format!(
                    "offline render contract requires aligned plugin chain and recall handoff stages (chain={} recall={})",
                    plugin_chain.stage_count, recall_handoff.stage_count
                ),
            ));
        }

        let complex_io_stages = plugin_chain
            .chains
            .iter()
            .flat_map(|chain| {
                chain.stages.iter().filter_map(|stage| {
                    if stage.complex_io_summary.has_complex_topology {
                        Some(RuntimeOfflineRenderComplexIoStageSummary {
                            chain_id: chain.chain_id.clone(),
                            node_id: stage.node_id.clone(),
                            stage_index: stage.stage_index,
                            plugin_type_id: stage.recall.payload.plugin_type_id.clone(),
                            topology: stage.complex_io_summary.clone(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        let multi_output_instrument_stage_count = complex_io_stages
            .iter()
            .filter(|stage| stage.topology.multi_output_instrument)
            .count();
        let complex_io_stage_count = complex_io_stages.len();
        let bus_capable_fx_stage_count = complex_io_stages
            .iter()
            .filter(|stage| stage.topology.bus_capable_fx_class.is_some())
            .count();
        let sidechain_capable_fx_stage_count = complex_io_stages
            .iter()
            .filter(|stage| {
                stage.topology.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SidechainCapableFx)
                    || stage.topology.bus_capable_fx_class
                        == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            })
            .count();
        let spatial_stages = plugin_chain
            .chains
            .iter()
            .flat_map(|chain| {
                chain.stages.iter().filter_map(|stage| {
                    stage.spatial_execution.as_ref().map(|spatial| {
                        RuntimeOfflineRenderSpatialStageSummary {
                            chain_id: chain.chain_id.clone(),
                            node_id: stage.node_id.clone(),
                            stage_index: stage.stage_index,
                            plugin_type_id: stage.recall.payload.plugin_type_id.clone(),
                            spatial: spatial.clone(),
                        }
                    })
                })
            })
            .collect::<Vec<_>>();
        let spatial_stage_count = spatial_stages.len();
        let active_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.execution_mode != RuntimeSpatialExecutionMode::Bypassed)
            .count();
        let bypassed_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.execution_mode == RuntimeSpatialExecutionMode::Bypassed)
            .count();
        let fallback_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.fallback_outcome.is_some())
            .count();
        let surround_bed_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed)
            .count();
        let object_aware_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.object_count > 0 || stage.spatial.object_role.is_some())
            .count();
        let expanded_fallback_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.expanded_fallback_outcome.is_some())
            .count();
        let immersive_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.immersive_room_policy.is_some())
            .count();
        let room_policy_aware_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .immersive_room_policy
                    .as_ref()
                    .is_some_and(|immersive| {
                        immersive.object_rendering_posture
                            == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
                    })
            })
            .count();
        let fallback_room_policy_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .immersive_room_policy
                    .as_ref()
                    .is_some_and(|immersive| {
                        immersive.room_policy_class == RuntimeRoomPolicyClass::FallbackRoom
                    })
            })
            .count();
        let renderer_capability_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.renderer_export.is_some())
            .count();
        let negotiated_renderer_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.renderer_capability_posture
                            == RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
                    })
            })
            .count();
        let immersive_export_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.immersive_export_class
                            != RuntimeImmersiveExportClass::NoImmersiveExport
                    })
            })
            .count();
        let fallback_immersive_export_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.immersive_export_class
                            == RuntimeImmersiveExportClass::FallbackExport
                    })
            })
            .count();
        let deployment_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.deployment_monitoring.is_some())
            .count();
        let folded_down_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .deployment_monitoring
                    .as_ref()
                    .is_some_and(|monitoring| {
                        matches!(
                            monitoring.fold_down_policy,
                            RuntimeFoldDownPolicy::FoldDownToReferenceBed
                                | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
                                | RuntimeFoldDownPolicy::FoldDownToPortablePreview
                        )
                    })
            })
            .count();
        let fallback_monitoring_scene_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .deployment_monitoring
                    .as_ref()
                    .is_some_and(|monitoring| {
                        monitoring.monitoring_scene_class
                            == RuntimeMonitoringSceneClass::FallbackScene
                    })
            })
            .count();

        Ok(RuntimeOfflineRenderChainDependencyPreview {
            chain_count: plugin_chain.chain_count,
            stage_count: plugin_chain.stage_count,
            pending_render_stage_count: plugin_chain.pending_render_stage_count,
            settling_stage_count: plugin_chain.settling_stage_count,
            compensated_stage_count: plugin_chain.compensated_stage_count,
            degraded_stage_count: plugin_chain.degraded_stage_count,
            bypassed_stage_count: plugin_chain.bypassed_stage_count,
            missing_binding_stage_count: plugin_chain.missing_binding_stage_count,
            total_planned_latency_samples: plugin_chain.total_planned_latency_samples,
            total_realized_latency_samples: plugin_chain.total_realized_latency_samples,
            total_tail_samples: plugin_chain.total_tail_samples,
            recall_stage_count: recall_handoff.stage_count,
            unbound_recall_stage_count: recall_handoff.unbound_stage_count,
            cold_recall_stage_count: recall_handoff.cold_stage_count,
            warm_recall_stage_count: recall_handoff.warm_stage_count,
            recovered_recall_stage_count: recall_handoff.recovered_stage_count,
            unavailable_recall_stage_count: recall_handoff.unavailable_stage_count,
            secondary_input_count: topology.secondary_input_count,
            required_secondary_input_count: topology.required_secondary_input_count,
            optional_secondary_input_count: topology.optional_secondary_input_count,
            disabled_secondary_input_count: topology.disabled_secondary_input_count,
            terminal_fallback_secondary_input_count: topology
                .terminal_fallback_secondary_input_count,
            bus_connection_count: topology.bus_connection_count,
            auxiliary_path_count: topology.auxiliary_path_count,
            complex_io_stage_count,
            multi_output_instrument_stage_count,
            bus_capable_fx_stage_count,
            sidechain_capable_fx_stage_count,
            spatial_stage_count,
            active_spatial_stage_count,
            bypassed_spatial_stage_count,
            fallback_spatial_stage_count,
            surround_bed_spatial_stage_count,
            object_aware_spatial_stage_count,
            expanded_fallback_spatial_stage_count,
            immersive_spatial_stage_count,
            room_policy_aware_spatial_stage_count,
            fallback_room_policy_spatial_stage_count,
            deployment_spatial_stage_count,
            folded_down_spatial_stage_count,
            fallback_monitoring_scene_spatial_stage_count,
            renderer_capability_spatial_stage_count,
            negotiated_renderer_spatial_stage_count,
            immersive_export_spatial_stage_count,
            fallback_immersive_export_spatial_stage_count,
            secondary_inputs: topology
                .secondary_inputs
                .iter()
                .cloned()
                .map(|mut route| {
                    route.target_kind = RuntimeSecondaryInputTargetKind::RenderInput;
                    route.target_id = "offline-render".into();
                    route
                })
                .collect(),
            bus_connections: topology.bus_connections.clone(),
            auxiliary_paths: topology.auxiliary_paths.clone(),
            complex_io_stages,
            spatial_stages,
        })
    }
}
