use super::state_model::RuntimePluginSandboxStateModel;
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginPlacementDecision {
    pub(crate) outcome: RuntimePluginIsolationOutcome,
    pub(crate) rule_id: Option<String>,
    pub(crate) sandbox_group_key: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginBoundaryCounts {
    pub(crate) sandbox_stage_counts: HashMap<String, usize>,
}

pub(crate) fn runtime_plugin_boundary_counts(
    planned_nodes: &[crate::interfaces::RuntimePlannedGraphNode],
) -> RuntimePluginBoundaryCounts {
    let mut counts = RuntimePluginBoundaryCounts::default();
    for node in planned_nodes
        .iter()
        .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
    {
        if let Some(sandbox_id) = node.plugin_sandbox_id.as_ref() {
            *counts
                .sandbox_stage_counts
                .entry(sandbox_id.clone())
                .or_insert(0) += 1;
        }
    }
    counts
}

fn runtime_plugin_placement_matches(
    matcher: &RuntimePluginPlacementRuleMatcher,
    sandbox: &RuntimePluginSandboxStateModel,
) -> bool {
    match matcher {
        RuntimePluginPlacementRuleMatcher::Any => true,
        RuntimePluginPlacementRuleMatcher::PluginFormat(format) => {
            sandbox.plugin_format == Some(*format)
        }
        RuntimePluginPlacementRuleMatcher::PluginTypeId(plugin_type_id) => sandbox
            .plugin_type_id
            .as_deref()
            .is_some_and(|value| value == plugin_type_id),
    }
}

pub(crate) fn runtime_plugin_placement_decision(
    sandbox: &RuntimePluginSandboxStateModel,
    policy: &RuntimePluginPlacementPolicy,
) -> RuntimePluginPlacementDecision {
    for rule in &policy.rules {
        if runtime_plugin_placement_matches(&rule.matcher, sandbox) {
            return RuntimePluginPlacementDecision {
                outcome: rule.outcome,
                rule_id: Some(rule.rule_id.clone()),
                sandbox_group_key: rule
                    .sandbox_group_key
                    .clone()
                    .unwrap_or_else(|| runtime_plugin_default_group_key(sandbox, rule.outcome)),
            };
        }
    }

    RuntimePluginPlacementDecision {
        outcome: policy.default_outcome,
        rule_id: None,
        sandbox_group_key: runtime_plugin_default_group_key(sandbox, policy.default_outcome),
    }
}

fn runtime_plugin_default_group_key(
    sandbox: &RuntimePluginSandboxStateModel,
    outcome: RuntimePluginIsolationOutcome,
) -> String {
    match outcome {
        RuntimePluginIsolationOutcome::InProcess => "in-process:default".into(),
        RuntimePluginIsolationOutcome::SharedSandbox => sandbox
            .plugin_type_id
            .as_deref()
            .map(|plugin_type_id| format!("plugin:{plugin_type_id}"))
            .unwrap_or_else(|| format!("sandbox:{}", sandbox.sandbox_id)),
        RuntimePluginIsolationOutcome::IsolatedSandbox => {
            format!("sandbox:{}", sandbox.sandbox_id)
        }
    }
}

fn runtime_plugin_boundary_continuity_class(
    sandbox_id_present: bool,
    state: RuntimePluginLifecycleState,
    transport_stage: Option<PluginSandboxTransportStage>,
) -> RuntimeInterruptionClass {
    if !sandbox_id_present {
        return RuntimeInterruptionClass::Steady;
    }
    match state {
        RuntimePluginLifecycleState::Quarantined => RuntimeInterruptionClass::Terminal,
        RuntimePluginLifecycleState::Faulted | RuntimePluginLifecycleState::Restarting => {
            RuntimeInterruptionClass::Restartable
        }
        RuntimePluginLifecycleState::Booting => RuntimeInterruptionClass::Recoverable,
        RuntimePluginLifecycleState::Degraded => match transport_stage {
            Some(
                PluginSandboxTransportStage::DetachRequested
                | PluginSandboxTransportStage::Detached
                | PluginSandboxTransportStage::DetachFault,
            ) => RuntimeInterruptionClass::Restartable,
            _ => RuntimeInterruptionClass::Recoverable,
        },
        RuntimePluginLifecycleState::Stopped => RuntimeInterruptionClass::Restartable,
        RuntimePluginLifecycleState::Ready => RuntimeInterruptionClass::Steady,
    }
}

pub(super) fn runtime_plugin_sandbox_snapshot(
    sandbox: &RuntimePluginSandboxStateModel,
    policy: &RuntimePluginPlacementPolicy,
    shared_boundary_member_count: usize,
) -> RuntimePluginSandboxSnapshot {
    let mut snapshot = sandbox.snapshot();
    let placement = runtime_plugin_placement_decision(sandbox, policy);
    let continuity_class =
        runtime_plugin_boundary_continuity_class(true, snapshot.state, snapshot.transport_stage);
    snapshot.sandbox_group_key = placement.sandbox_group_key;
    snapshot.placement_outcome = placement.outcome;
    snapshot.placement_rule_id = placement.rule_id;
    snapshot.shared_boundary_member_count = shared_boundary_member_count.max(1);
    snapshot.continuity_class = continuity_class;
    snapshot.rebindable = matches!(
        continuity_class,
        RuntimeInterruptionClass::Resumable
            | RuntimeInterruptionClass::Restartable
            | RuntimeInterruptionClass::Recoverable
    );
    snapshot
}

pub(crate) fn runtime_plugin_stage_assignment(
    sandbox_id: Option<&str>,
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
) -> (
    RuntimePluginIsolationOutcome,
    Option<String>,
    Option<String>,
    usize,
    RuntimeInterruptionClass,
    bool,
) {
    match (sandbox_id, sandbox) {
        (Some(_), Some(sandbox)) => (
            sandbox.placement_outcome,
            Some(sandbox.sandbox_group_key.clone()),
            sandbox.placement_rule_id.clone(),
            sandbox.shared_boundary_member_count,
            sandbox.continuity_class,
            sandbox.rebindable,
        ),
        (Some(sandbox_id), None) => (
            RuntimePluginIsolationOutcome::IsolatedSandbox,
            Some(format!("sandbox:{sandbox_id}")),
            None,
            1,
            RuntimeInterruptionClass::Restartable,
            true,
        ),
        (None, None) | (None, Some(_)) => (
            RuntimePluginIsolationOutcome::InProcess,
            None,
            None,
            1,
            RuntimeInterruptionClass::Steady,
            false,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        PluginSandboxSpec, RuntimeConfig, RuntimeObservationApi, RuntimePluginIsolationOutcome,
        SignalRuntime,
    };
    use signal_plugin::PluginFormat;

    #[test]
    fn shared_sandbox_default_group_key_uses_plugin_type_id() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.ensure_shared_sandbox_placement("com.signal.fixture");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "member-a".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.fixture".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "member-b".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.fixture".into()),
        });
        let snapshot = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(snapshot.sandboxes.len(), 2);
        for sandbox in &snapshot.sandboxes {
            assert_eq!(
                sandbox.placement_outcome,
                RuntimePluginIsolationOutcome::SharedSandbox
            );
            assert_eq!(sandbox.sandbox_group_key, "plugin:com.signal.fixture");
            assert_eq!(sandbox.shared_boundary_member_count, 2);
        }
    }

    #[test]
    fn isolated_sandbox_default_group_key_stays_per_sandbox() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-a".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.fixture".into()),
        });
        let snapshot = runtime.get_plugin_lifecycle_snapshot();
        let sandbox = &snapshot.sandboxes[0];
        assert_eq!(
            sandbox.placement_outcome,
            RuntimePluginIsolationOutcome::IsolatedSandbox
        );
        assert_eq!(sandbox.sandbox_group_key, "sandbox:sandbox-a");
        assert_eq!(sandbox.shared_boundary_member_count, 1);
    }

    #[test]
    fn shared_sandbox_fault_fans_out_to_every_group_member() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.ensure_shared_sandbox_placement("com.signal.fixture");
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "member-a".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.fixture".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "member-b".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.fixture".into()),
        });
        runtime.record_plugin_sandbox_fault(
            "member-a",
            crate::PluginFaultKind::Crash,
            "shared_boundary_child_dead",
            None,
        );
        let snapshot = runtime.get_plugin_lifecycle_snapshot();
        assert_eq!(snapshot.sandboxes.len(), 2);
        let classes: Vec<_> = snapshot
            .sandboxes
            .iter()
            .map(|sandbox| format!("{:?}", sandbox.continuity_class))
            .collect();
        assert!(
            classes.iter().all(|class| class == &classes[0]),
            "shared-boundary members must share one continuity class, got {classes:?}"
        );
        assert_eq!(classes[0], "Restartable");
        for sandbox in &snapshot.sandboxes {
            assert_eq!(sandbox.shared_boundary_member_count, 2);
            assert_eq!(sandbox.state, crate::RuntimePluginLifecycleState::Faulted);
        }
    }

    #[test]
    fn isolated_sandbox_fault_does_not_fan_out() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-a".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.a".into()),
        });
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "sandbox-b".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: Some("com.signal.b".into()),
        });
        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::PluginFaultKind::Crash,
            "dedicated_child_dead",
            None,
        );
        let snapshot = runtime.get_plugin_lifecycle_snapshot();
        let a = snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-a")
            .expect("sandbox-a");
        let b = snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "sandbox-b")
            .expect("sandbox-b");
        assert_eq!(a.state, crate::RuntimePluginLifecycleState::Faulted);
        assert_eq!(format!("{:?}", a.continuity_class), "Restartable");
        assert_eq!(b.state, crate::RuntimePluginLifecycleState::Stopped);
        assert!(a.last_fault_kind.is_some());
        assert!(b.last_fault_kind.is_none());
    }
}
