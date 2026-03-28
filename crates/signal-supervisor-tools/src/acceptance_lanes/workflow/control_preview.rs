use super::super::*;

fn control_preview_workflow_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "control-surface-workflow-coherence",
            title: "Control-Surface Workflow Coherence",
            required_tasks: CONTROL_WORKFLOW_REQUIRED_TASKS,
            advisory_tasks: CONTROL_WORKFLOW_ADVISORY_TASKS,
            rationale:
                "Keeps scene-mapping, feedback-page, safe-action, and bounded advanced-feedback workflow truth on one required lane instead of letting device-private page logic define the shared proof surface.",
        },
        IntegratedAcceptanceFamily {
            id: "preview-workflow-coherence",
            title: "Preview Workflow Coherence",
            required_tasks: PREVIEW_WORKFLOW_REQUIRED_TASKS,
            advisory_tasks: PREVIEW_WORKFLOW_ADVISORY_TASKS,
            rationale:
                "Requires preview-device policy, queue posture, audition continuity, and transform-scheduling truth to stay on the same shared seam instead of drifting into browser-local queue policy.",
        },
        IntegratedAcceptanceFamily {
            id: "cross-surface-workflow-coherence",
            title: "Cross-Surface Workflow Coherence",
            required_tasks: CONTROL_PREVIEW_HOST_EDGE_REQUIRED_TASKS,
            advisory_tasks: CONTROL_PREVIEW_HOST_EDGE_ADVISORY_TASKS,
            rationale:
                "Pins the shared lane to public runtime, supervisor, and both stable host edges so one device path or one preview path cannot define a special-case workflow story.",
        },
    ]
}

fn control_preview_workflow_acceptance_validation_steps(
) -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence",
            rationale:
                "Proves one supervisor export can carry control-surface workflow, advanced-feedback, preview-device policy, and preview-workflow truth together instead of only listing a grouped descriptor over isolated boundary seams.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable control and preview workflow acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped control-surface and preview workflow acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded control-surface and preview workflow acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of isolated workflow proofs.",
        },
    ]
}

pub(crate) fn render_control_preview_workflow_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "control_preview_workflow_acceptance_lane",
        lane: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: control_preview_workflow_acceptance_families(),
        validation_steps: control_preview_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required advanced-hardware and preview-transform proof without claiming exhaustive device-vendor or browser UX certification",
            "device-native page, display, motor, haptic, and browser-native queue reruns remain advisory or deferred instead of silently entering the required lane",
            "cross-family grouped export proof and broader integrated live or immersive acceptance still belong to later g08 batches",
        ],
    })
}

pub(crate) fn render_control_preview_workflow_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "control_preview_workflow_acceptance_lane",
        lane: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: control_preview_workflow_acceptance_families(),
        validation_steps: control_preview_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required advanced-hardware and preview-transform proof without claiming exhaustive device-vendor or browser UX certification",
            "device-native page, display, motor, haptic, and browser-native queue reruns remain advisory or deferred instead of silently entering the required lane",
            "cross-family grouped export proof and broader integrated live or immersive acceptance still belong to later g08 batches",
        ],
    })
}
