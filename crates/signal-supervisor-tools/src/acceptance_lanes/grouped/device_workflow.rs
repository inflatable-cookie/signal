use super::super::*;

fn device_workflow_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "live-endpoint-ownership-and-protocol-continuity",
            title: "Live Endpoint Ownership And Protocol Continuity",
            required_tasks: DEVICE_WORKFLOW_LIVE_PROTOCOL_REQUIRED_TASKS,
            advisory_tasks: DEVICE_WORKFLOW_LIVE_PROTOCOL_ADVISORY_TASKS,
            rationale:
                "Keeps live endpoint graph, ownership, parity, and controller-expression truth on one required protocol lane instead of letting backend-local endpoint policy define the shared proof surface.",
        },
        IntegratedAcceptanceFamily {
            id: "control-surface-and-advanced-hardware-workflow",
            title: "Control-Surface And Advanced Hardware Workflow",
            required_tasks: DEVICE_WORKFLOW_CONTROL_REQUIRED_TASKS,
            advisory_tasks: DEVICE_WORKFLOW_CONTROL_ADVISORY_TASKS,
            rationale:
                "Requires bounded control-surface, advanced feedback, scene-mapping, and safe-action workflow posture together rather than treating device workflow as optional host-private glue.",
        },
        IntegratedAcceptanceFamily {
            id: "cross-backend-host-edge-coherence",
            title: "Cross-Backend Host-Edge Coherence",
            required_tasks: DEVICE_WORKFLOW_HOST_EDGE_REQUIRED_TASKS,
            advisory_tasks: DEVICE_WORKFLOW_HOST_EDGE_ADVISORY_TASKS,
            rationale:
                "Pins the shared lane to public runtime, supervisor, and both stable host edges so one backend or one host path cannot define a special-case workflow story.",
        },
    ]
}

fn device_workflow_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_device_workflow_acceptance_evidence",
            rationale:
                "Proves one repo-owned supervisor export carries external MIDI live ownership, controller-expression, control-surface, and advanced-hardware receipts together instead of reducing the lane to isolated boundary-local proofs.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools device_workflow_acceptance_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable device workflow acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped device workflow acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: DEVICE_WORKFLOW_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded device workflow acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of isolated boundary proofs.",
        },
    ]
}

pub(crate) fn render_device_workflow_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "device_workflow_acceptance_lane",
        lane: DEVICE_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: DEVICE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: DEVICE_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: DEVICE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: DEVICE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: device_workflow_acceptance_families(),
        validation_steps: device_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required external MIDI, controller-expression, control-surface, and advanced-hardware acceptance tasks without claiming exhaustive backend certification",
            "backend-native patchbay, reservation, session-manager, and richer repeated-run device matrices remain advisory or deferred instead of silently entering the required lane",
            "broader Linux failure-injection, immersive, preview, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}

pub(crate) fn render_device_workflow_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "device_workflow_acceptance_lane",
        lane: DEVICE_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: DEVICE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: DEVICE_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: DEVICE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: DEVICE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: device_workflow_acceptance_families(),
        validation_steps: device_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required external MIDI, controller-expression, control-surface, and advanced-hardware acceptance tasks without claiming exhaustive backend certification",
            "backend-native patchbay, reservation, session-manager, and richer repeated-run device matrices remain advisory or deferred instead of silently entering the required lane",
            "broader Linux failure-injection, immersive, preview, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}
