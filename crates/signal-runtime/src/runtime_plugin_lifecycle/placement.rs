use super::state_model::RuntimePluginSandboxStateModel;
use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RuntimePluginPlacementDecision {
    outcome: RuntimePluginIsolationOutcome,
    rule_id: Option<String>,
    sandbox_group_key: String,
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

fn runtime_plugin_placement_decision(
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
                    .unwrap_or_else(|| format!("sandbox:{}", sandbox.sandbox_id)),
            };
        }
    }

    RuntimePluginPlacementDecision {
        outcome: policy.default_outcome,
        rule_id: None,
        sandbox_group_key: match policy.default_outcome {
            RuntimePluginIsolationOutcome::InProcess => "in-process:default".into(),
            RuntimePluginIsolationOutcome::SharedSandbox
            | RuntimePluginIsolationOutcome::IsolatedSandbox => {
                format!("sandbox:{}", sandbox.sandbox_id)
            }
        },
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
