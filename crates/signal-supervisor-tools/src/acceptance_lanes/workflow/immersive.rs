use super::super::*;

fn immersive_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "room-policy-and-render-continuity",
            title: "Room-Policy And Render Continuity",
            required_tasks: IMMERSIVE_RENDER_REQUIRED_TASKS,
            advisory_tasks: IMMERSIVE_RENDER_ADVISORY_TASKS,
            rationale:
                "Keeps immersive room-policy, object-rendering fallback, and renderer-export posture on one required lane instead of letting renderer-private capability shells define the shared proof surface.",
        },
        IntegratedAcceptanceFamily {
            id: "deployment-fold-down-and-monitoring-coherence",
            title: "Deployment Fold-Down And Monitoring Coherence",
            required_tasks: IMMERSIVE_MONITORING_REQUIRED_TASKS,
            advisory_tasks: IMMERSIVE_MONITORING_ADVISORY_TASKS,
            rationale:
                "Requires deployment-aware, fold-down, and fallback-monitoring truth to stay on the same shared seam instead of drifting into product-local monitoring workflows.",
        },
        IntegratedAcceptanceFamily {
            id: "cross-surface-immersive-coherence",
            title: "Cross-Surface Immersive Coherence",
            required_tasks: IMMERSIVE_HOST_EDGE_REQUIRED_TASKS,
            advisory_tasks: IMMERSIVE_HOST_EDGE_ADVISORY_TASKS,
            rationale:
                "Pins the shared lane to public runtime, supervisor, and both stable host edges so one renderer posture or one host path cannot define a special-case immersive story.",
        },
    ]
}

fn immersive_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence",
            rationale:
                "Proves one supervisor export can carry immersive room-policy, deployment-monitoring, and renderer-export truth together instead of only listing a grouped descriptor over the broader spatial seam.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools immersive_acceptance_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable immersive acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped immersive render and monitoring acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: IMMERSIVE_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded immersive render and monitoring acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of isolated spatial proofs.",
        },
    ]
}

pub(crate) fn render_immersive_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "immersive_acceptance_lane",
        lane: IMMERSIVE_ACCEPTANCE_LANE,
        contract_path: IMMERSIVE_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: IMMERSIVE_ACCEPTANCE_TASK,
        required_tasks: IMMERSIVE_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: IMMERSIVE_ACCEPTANCE_ADVISORY_TASKS,
        families: immersive_acceptance_families(),
        validation_steps: immersive_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required room-policy, deployment-monitoring, and renderer-export proof through the existing spatial boundary without claiming exhaustive renderer certification",
            "renderer-native reruns, richer monitoring-scene variants, and immersive authoring or export-adjacent confidence passes remain advisory or deferred instead of silently entering the required lane",
            "broader preview, device-workflow, Linux live, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}

pub(crate) fn render_immersive_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "immersive_acceptance_lane",
        lane: IMMERSIVE_ACCEPTANCE_LANE,
        contract_path: IMMERSIVE_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: IMMERSIVE_ACCEPTANCE_TASK,
        required_tasks: IMMERSIVE_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: IMMERSIVE_ACCEPTANCE_ADVISORY_TASKS,
        families: immersive_acceptance_families(),
        validation_steps: immersive_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required room-policy, deployment-monitoring, and renderer-export proof through the existing spatial boundary without claiming exhaustive renderer certification",
            "renderer-native reruns, richer monitoring-scene variants, and immersive authoring or export-adjacent confidence passes remain advisory or deferred instead of silently entering the required lane",
            "broader preview, device-workflow, Linux live, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}
