use super::super::*;

fn g07_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "routing-and-multichannel-coherence",
            title: "Routing And Multichannel Coherence",
            required_tasks: G07_ROUTING_REQUIRED_TASKS,
            advisory_tasks: G07_ROUTING_ADVISORY_TASKS,
            rationale:
                "Keeps the bounded lane anchored on shared multichannel, sidechain, multi-bus, and spatial routing truth while leaving richer complex plugin-I/O breadth visible as advisory depth.",
        },
        IntegratedAcceptanceFamily {
            id: "linux-plugin-and-backend-continuity",
            title: "Linux Plugin And Backend Continuity",
            required_tasks: G07_LINUX_REQUIRED_TASKS,
            advisory_tasks: G07_LINUX_ADVISORY_TASKS,
            rationale:
                "Pins Linux-native plugin parity and backend portability to one required lane while keeping narrower LV2-specific breadth explicit as advisory depth.",
        },
        IntegratedAcceptanceFamily {
            id: "external-control-and-advanced-hardware",
            title: "External Control And Advanced Hardware",
            required_tasks: G07_CONTROL_REQUIRED_TASKS,
            advisory_tasks: G07_CONTROL_ADVISORY_TASKS,
            rationale:
                "Requires endpoint, controller-expression, control-surface, and advanced hardware policy truth together instead of treating the control stack as separate optional seams.",
        },
        IntegratedAcceptanceFamily {
            id: "stretch-analysis-artifact-and-preview",
            title: "Stretch Analysis Artifact And Preview",
            required_tasks: G07_STRETCH_REQUIRED_TASKS,
            advisory_tasks: G07_STRETCH_ADVISORY_TASKS,
            rationale:
                "Makes stretch, marker-analysis, transform-artifact, and preview-transform continuity part of one bounded media lane instead of isolated post-warp checks.",
        },
    ]
}

fn g07_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_g07_acceptance_evidence",
            rationale:
                "Proves one repo-owned supervisor export carries routing, Linux, controller, and stretch receipts together instead of reducing the lane to a checklist of isolated boundary tasks.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools g07_acceptance_lane_json_reports_required_and_advisory_policy",
            rationale:
                "Keeps the machine-readable g07 integrated acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-g07-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped g07 acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: G07_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded g07 required acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of milestone-local proofs.",
        },
    ]
}

pub(crate) fn render_g07_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "g07_acceptance_lane",
        lane: G07_ACCEPTANCE_LANE,
        contract_path: G07_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: G07_ACCEPTANCE_TASK,
        required_tasks: G07_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: G07_ACCEPTANCE_ADVISORY_TASKS,
        families: g07_acceptance_families(),
        validation_steps: g07_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane now groups required routing, Linux, controller, and stretch acceptance tasks and includes one repo-owned cross-family supervisor export proof",
            "broader repeated-run confidence passes, richer local or server permutations, and exhaustive environment matrices remain advisory or deferred instead of silently entering the required lane",
            "the Loophole-facing closeout and promotion verdict remains outside this lane and still belongs to g07.020",
        ],
    })
}

pub(crate) fn render_g07_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "g07_acceptance_lane",
        lane: G07_ACCEPTANCE_LANE,
        contract_path: G07_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: G07_ACCEPTANCE_TASK,
        required_tasks: G07_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: G07_ACCEPTANCE_ADVISORY_TASKS,
        families: g07_acceptance_families(),
        validation_steps: g07_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane now groups required routing, Linux, controller, and stretch acceptance tasks and includes one repo-owned cross-family supervisor export proof",
            "broader repeated-run confidence passes, richer local or server permutations, and exhaustive environment matrices remain advisory or deferred instead of silently entering the required lane",
            "the Loophole-facing closeout and promotion verdict remains outside this lane and still belongs to g07.020",
        ],
    })
}
