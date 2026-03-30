use super::super::super::*;

pub(super) fn assert_complex_io_and_pin_matrix_receipts(runtime: &SignalRuntime) {
    let snapshot = runtime.get_plugin_chain_snapshot();
    assert_eq!(snapshot.chain_count, 1);
    assert_eq!(snapshot.stage_count, 2);
    assert_eq!(snapshot.compensated_stage_count, 1);
    assert_eq!(snapshot.bypassed_stage_count, 1);
    assert_eq!(snapshot.total_realized_latency_samples, 48);
    assert_eq!(snapshot.total_tail_samples, 72);
    assert_eq!(snapshot.chains[0].chain_id, "track:lead");
    assert_eq!(snapshot.chains[0].stages[0].node_id, "plugin-a");
    assert_eq!(snapshot.chains[0].stages[1].node_id, "plugin-b");
    assert!(
        snapshot.chains[0].stages[0]
            .complex_io_summary
            .has_complex_topology
    );
    assert!(
        snapshot.chains[0].stages[0]
            .complex_io_summary
            .multi_output_instrument
    );
    assert_eq!(
        snapshot.chains[0].stages[0]
            .complex_io_summary
            .instrument_output_group_count,
        2
    );
    assert_eq!(
        snapshot.chains[0].stages[1]
            .complex_io_summary
            .bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert_eq!(
        snapshot.chains[0].stages[1]
            .complex_io_summary
            .secondary_input_group_count,
        1
    );

    let observation =
        crate::RuntimeObservationReport::capture(runtime, &crate::RuntimeEventRecorder::default());
    assert_eq!(observation.plugin_pin_matrix_snapshot.plugin_type_count, 2);
    assert_eq!(
        observation.plugin_pin_matrix_snapshot.negotiated_type_count,
        2
    );
    assert_eq!(
        observation
            .plugin_pin_matrix_snapshot
            .dynamic_negotiated_type_count,
        2
    );
    let multiout_pin_matrix = observation
        .plugin_pin_matrix_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:multiout-instrument")
        .expect("multi-output pin matrix record should exist");
    assert_eq!(
        multiout_pin_matrix.pin_matrix_posture,
        crate::RuntimePluginPinMatrixPosture::Negotiated
    );
    assert_eq!(
        multiout_pin_matrix.dynamic_bus_negotiation_posture,
        crate::RuntimeDynamicBusNegotiationPosture::Negotiated
    );
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&crate::RuntimePluginPinGroupIdentity::PrimaryProgramPath));
    assert!(multiout_pin_matrix
        .pin_group_identities
        .contains(&crate::RuntimePluginPinGroupIdentity::SecondaryProgramPath));
    let bus_fx_pin_matrix = observation
        .plugin_pin_matrix_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:vst3:bus-fx")
        .expect("bus-fx pin matrix record should exist");
    assert_eq!(
        bus_fx_pin_matrix.fallback_outcome,
        crate::RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
    );
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&crate::RuntimePluginPinGroupIdentity::SidechainPath));
    assert!(bus_fx_pin_matrix
        .pin_group_identities
        .contains(&crate::RuntimePluginPinGroupIdentity::AuxReturnPath));
}
