use super::super::*;

#[test]
fn runtime_shared_sandbox_blast_radius_stays_boundary_local_across_recovery_and_terminal_states() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![RuntimePluginPlacementRule {
                rule_id: "share-verified-clap".into(),
                matcher: RuntimePluginPlacementRuleMatcher::PluginTypeId(
                    "plugin://shared-verified".into(),
                ),
                outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                sandbox_group_key: Some("shared:verified".into()),
            }],
        })
        .expect("apply plugin continuity placement policy");
    apply_plugin_continuity_graph(
        &mut runtime,
        "graph:runtime:plugin-continuity:shared-boundary",
        &[
            ("plugin-a", "sandbox-shared"),
            ("plugin-b", "sandbox-shared"),
            ("plugin-c", "sandbox-shared"),
            ("plugin-d", "sandbox-steady"),
        ],
    );
    record_ready_plugin_sandbox(
        &mut runtime,
        "sandbox-shared",
        PluginFormat::Clap,
        "plugin://shared-verified",
        1,
    );
    record_ready_plugin_sandbox(
        &mut runtime,
        "sandbox-steady",
        PluginFormat::Clap,
        "plugin://steady-utility",
        1,
    );

    let steady = runtime.get_plugin_chain_snapshot();
    assert_eq!(steady.shared_sandbox_stage_count, 3);
    assert_eq!(steady.isolated_sandbox_stage_count, 1);
    assert_eq!(steady.rebindable_stage_count, 0);
    assert_eq!(steady.terminal_stage_count, 0);

    runtime.record_plugin_sandbox_transport(
        "sandbox-shared",
        "lease-sandbox-shared",
        "region-sandbox-shared",
        PluginSandboxTransportStage::DetachRequested,
        Some(2),
        Some("shared boundary rebind".into()),
    );

    let restartable = runtime.get_plugin_lifecycle_snapshot();
    let shared = restartable
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared boundary should remain exported");
    assert_eq!(shared.shared_boundary_member_count, 3);
    assert_eq!(
        shared.continuity_class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(shared.rebindable);
    let steady_boundary = restartable
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-steady")
        .expect("steady boundary should remain exported");
    assert_eq!(
        steady_boundary.continuity_class,
        RuntimeInterruptionClass::Steady
    );
    assert!(!steady_boundary.rebindable);

    let restartable_chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(restartable_chain.rebindable_stage_count, 3);
    assert_eq!(restartable_chain.terminal_stage_count, 0);
    assert_eq!(
        restartable_chain.chains[0]
            .stages
            .iter()
            .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
            .count(),
        3
    );
    assert!(restartable_chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
        .all(|stage| {
            stage.continuity_class == RuntimeInterruptionClass::Restartable
                && stage.rebindable
                && stage.shared_boundary_member_count == 3
        }));
    assert!(restartable_chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-steady"))
        .all(|stage| {
            stage.continuity_class == RuntimeInterruptionClass::Steady && !stage.rebindable
        }));

    runtime.record_plugin_sandbox_transport(
        "sandbox-shared",
        "lease-sandbox-shared",
        "region-sandbox-shared",
        PluginSandboxTransportStage::Attached,
        Some(3),
        None,
    );

    let recovered = runtime.get_plugin_lifecycle_snapshot();
    let shared = recovered
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared boundary should recover");
    assert_eq!(shared.state, RuntimePluginLifecycleState::Ready);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Steady);
    assert!(!shared.rebindable);

    let recovered_chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(recovered_chain.rebindable_stage_count, 0);
    assert_eq!(recovered_chain.terminal_stage_count, 0);
    assert!(recovered_chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
        .all(|stage| {
            stage.continuity_class == RuntimeInterruptionClass::Steady && !stage.rebindable
        }));

    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Crash,
        "shared boundary crash",
        Some(4),
    );
    runtime.record_plugin_sandbox_fault(
        "sandbox-shared",
        PluginFaultKind::Timeout,
        "shared boundary timeout",
        Some(5),
    );

    let terminal = runtime.get_plugin_lifecycle_snapshot();
    let shared = terminal
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-shared")
        .expect("shared boundary should remain visible after terminal fault");
    assert_eq!(shared.state, RuntimePluginLifecycleState::Quarantined);
    assert_eq!(shared.continuity_class, RuntimeInterruptionClass::Terminal);
    assert!(!shared.rebindable);
    let steady_boundary = terminal
        .sandboxes
        .iter()
        .find(|sandbox| sandbox.sandbox_id == "sandbox-steady")
        .expect("steady boundary should remain visible after sibling failure");
    assert_eq!(
        steady_boundary.continuity_class,
        RuntimeInterruptionClass::Steady
    );

    let terminal_chain = runtime.get_plugin_chain_snapshot();
    assert_eq!(terminal_chain.terminal_stage_count, 3);
    assert_eq!(terminal_chain.rebindable_stage_count, 0);
    assert!(terminal_chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-shared"))
        .all(|stage| {
            stage.continuity_class == RuntimeInterruptionClass::Terminal
                && !stage.rebindable
                && stage.shared_boundary_member_count == 3
        }));
    assert!(terminal_chain.chains[0]
        .stages
        .iter()
        .filter(|stage| stage.sandbox_id.as_deref() == Some("sandbox-steady"))
        .all(|stage| stage.continuity_class == RuntimeInterruptionClass::Steady));
}
