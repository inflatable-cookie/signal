use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerCapability {
    Absent,
    Supported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridCapability {
    NotRequired,
    Supported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchCapability {
    Absent,
    Supported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionCapabilitySummary {
    pub worker_capability: RuntimeLv2WorkerCapability,
    pub urid_capability: RuntimeLv2UridCapability,
    pub patch_capability: RuntimeLv2PatchCapability,
    pub negotiated_extension_count: usize,
    pub summary: String,
}

impl RuntimeLv2ExtensionCapabilitySummary {
    pub fn absent() -> Self {
        Self {
            worker_capability: RuntimeLv2WorkerCapability::Absent,
            urid_capability: RuntimeLv2UridCapability::NotRequired,
            patch_capability: RuntimeLv2PatchCapability::Absent,
            negotiated_extension_count: 0,
            summary: "worker=Absent urid=NotRequired patch=Absent extensions=0".into(),
        }
    }

    pub fn from_lv2_feature_uris(
        required_features: &[String],
        supported_extensions: &[String],
    ) -> Self {
        let supports_worker = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/worker#"));
        let requires_worker = required_features
            .iter()
            .any(|feature| feature == "http://lv2plug.in/ns/ext/worker#schedule");
        let supports_urid = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/urid#"));
        let requires_urid = required_features
            .iter()
            .any(|feature| feature == "http://lv2plug.in/ns/ext/urid#map");
        let supports_patch = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/patch#"));
        let negotiated_extension_count = required_features
            .iter()
            .chain(supported_extensions.iter())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let worker_capability = if requires_worker {
            RuntimeLv2WorkerCapability::Required
        } else if supports_worker {
            RuntimeLv2WorkerCapability::Supported
        } else {
            RuntimeLv2WorkerCapability::Absent
        };
        let urid_capability = if requires_urid {
            RuntimeLv2UridCapability::Required
        } else if supports_urid {
            RuntimeLv2UridCapability::Supported
        } else {
            RuntimeLv2UridCapability::NotRequired
        };
        let patch_capability = if supports_patch {
            RuntimeLv2PatchCapability::Supported
        } else {
            RuntimeLv2PatchCapability::Absent
        };
        let mut summary = Self {
            worker_capability,
            urid_capability,
            patch_capability,
            negotiated_extension_count,
            summary: String::new(),
        };
        summary.summary = format!(
            "worker={:?} urid={:?} patch={:?} extensions={}",
            summary.worker_capability,
            summary.urid_capability,
            summary.patch_capability,
            summary.negotiated_extension_count,
        );
        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerPosture {
    WorkerAbsent,
    WorkerAvailable,
    WorkerRequiredAvailable,
    WorkerGuarded,
    WorkerUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridNegotiationPosture {
    NotRequired,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchExchangePosture {
    Absent,
    Supported,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2ExtensionNegotiationState {
    NotRequired,
    Negotiated,
    PartiallySatisfied,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub worker_posture: RuntimeLv2WorkerPosture,
    pub urid_negotiation_posture: RuntimeLv2UridNegotiationPosture,
    pub patch_exchange_posture: RuntimeLv2PatchExchangePosture,
    pub extension_negotiation_state: RuntimeLv2ExtensionNegotiationState,
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub sandbox_count: usize,
    pub active_sandbox_count: usize,
    pub faulted_sandbox_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionSnapshot {
    pub plugin_type_count: usize,
    pub sandbox_count: usize,
    pub worker_required_type_count: usize,
    pub worker_guarded_type_count: usize,
    pub urid_negotiated_type_count: usize,
    pub patch_supported_type_count: usize,
    pub negotiated_type_count: usize,
    pub guarded_type_count: usize,
    pub unavailable_type_count: usize,
    pub records: Vec<RuntimeLv2ExtensionRecord>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCapabilityCoverageSummary {
    pub discovered_format_count: usize,
    pub multi_format_catalog: bool,
    pub complex_io_type_count: usize,
    pub multi_output_instrument_count: usize,
    pub bus_capable_fx_count: usize,
    pub sidechain_capable_fx_count: usize,
    pub instrument_count: usize,
    pub audio_effect_count: usize,
    pub analyzer_count: usize,
    pub utility_count: usize,
    pub note_effect_count: usize,
    pub supports_snapshot_count: usize,
    pub supports_reset_count: usize,
    pub supports_bypass_count: usize,
    pub exposes_latency_count: usize,
    pub exposes_tail_count: usize,
    pub sample_accurate_automation_count: usize,
    pub accepts_midi_count: usize,
    pub accepts_note_events_count: usize,
    pub supports_note_expression_count: usize,
    pub produces_midi_count: usize,
    pub silence_aware_count: usize,
    pub requires_main_thread_for_state_count: usize,
    pub supports_prepare_count: usize,
    pub supports_activate_count: usize,
    pub supports_reset_while_active_count: usize,
    pub max_complex_io_port_group_count: usize,
    pub max_audio_bus_count: usize,
    pub max_parameter_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginDiscoverySnapshot {
    pub scan_count: usize,
    pub format_filtered_scan_count: usize,
    pub discovered_type_count: usize,
    pub discovered_format_count: usize,
    pub last_scan: Option<RuntimePluginScanReceipt>,
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    pub discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    pub summary: String,
}

impl RuntimePluginPinMatrixSnapshot {
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
            let mut pin_record = RuntimePluginPinMatrixRecord {
                plugin_type_id: record.plugin_type_id.clone(),
                plugin_id: record.plugin_id.clone(),
                pin_group_identities,
                pin_matrix_posture,
                dynamic_bus_negotiation_posture,
                fallback_outcome,
                strongest_lifecycle_state,
                stage_count,
                active_stage_count,
                summary: String::new(),
            };
            pin_record.summary = format!(
                "plugin_type={} pin_groups={:?} matrix={:?} dynamic={:?} fallback={:?} stages={}/active={} lifecycle={:?}",
                pin_record.plugin_type_id,
                pin_record.pin_group_identities,
                pin_record.pin_matrix_posture,
                pin_record.dynamic_bus_negotiation_posture,
                pin_record.fallback_outcome,
                pin_record.stage_count,
                pin_record.active_stage_count,
                pin_record.strongest_lifecycle_state,
            );
            records.push(pin_record);
        }

        let plugin_type_count = records.len();
        let mut snapshot = Self {
            plugin_type_count,
            negotiated_type_count,
            guarded_type_count,
            unavailable_type_count,
            dynamic_negotiated_type_count,
            dynamic_guarded_type_count,
            records,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "plugin_types={} negotiated={} guarded={} unavailable={} dynamic_negotiated={} dynamic_guarded={}",
            snapshot.plugin_type_count,
            snapshot.negotiated_type_count,
            snapshot.guarded_type_count,
            snapshot.unavailable_type_count,
            snapshot.dynamic_negotiated_type_count,
            snapshot.dynamic_guarded_type_count,
        );
        snapshot
    }
}
