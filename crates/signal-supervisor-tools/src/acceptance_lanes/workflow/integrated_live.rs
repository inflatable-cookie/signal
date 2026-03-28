use super::super::*;

fn integrated_live_workflow_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "linux-live-and-device-workflow-continuity",
            title: "Linux Live And Device Workflow Continuity",
            required_tasks: INTEGRATED_LIVE_AND_DEVICE_REQUIRED_TASKS,
            advisory_tasks: INTEGRATED_LIVE_AND_DEVICE_ADVISORY_TASKS,
            rationale:
                "Keeps Linux live ownership, backend-native coordination, external MIDI live ownership, and bounded device workflow posture on one required family instead of letting backend-local or device-private glue define the integrated proof surface.",
        },
        IntegratedAcceptanceFamily {
            id: "immersive-and-preview-workflow-continuity",
            title: "Immersive And Preview Workflow Continuity",
            required_tasks: INTEGRATED_IMMERSIVE_AND_PREVIEW_REQUIRED_TASKS,
            advisory_tasks: INTEGRATED_IMMERSIVE_AND_PREVIEW_ADVISORY_TASKS,
            rationale:
                "Requires immersive render and monitoring posture to stay coherent with preview-device and preview-workflow truth instead of splitting monitoring and workflow evidence into renderer-private or browser-local lanes.",
        },
        IntegratedAcceptanceFamily {
            id: "cross-surface-integrated-coherence",
            title: "Cross-Surface Integrated Coherence",
            required_tasks: INTEGRATED_CROSS_SURFACE_REQUIRED_TASKS,
            advisory_tasks: INTEGRATED_CROSS_SURFACE_ADVISORY_TASKS,
            rationale:
                "Pins the grouped integrated seam to public runtime, supervisor export, and both stable host edges by requiring the already-closed shared lanes together instead of allowing one host path or one family to define a special-case story.",
        },
        IntegratedAcceptanceFamily {
            id: "shared-grouped-integrated-acceptance-export",
            title: "Shared Grouped Integrated Acceptance Export",
            required_tasks: INTEGRATED_GROUPED_EXPORT_REQUIRED_TASKS,
            advisory_tasks: INTEGRATED_GROUPED_EXPORT_ADVISORY_TASKS,
            rationale:
                "Requires one repo-owned grouped descriptor and runnable lane to span Linux live, device workflow, immersive, and control-preview acceptance instead of leaving the integrated claim as four unrelated checklists.",
        },
    ]
}

fn integrated_live_workflow_acceptance_validation_steps(
) -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence",
            rationale:
                "Proves one supervisor export can carry Linux live ownership, device workflow, immersive render and monitoring, and control-preview workflow evidence together instead of leaving the integrated lane as a grouped descriptor over four separate acceptance seams.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable integrated live-ownership and workflow descriptor aligned with the frozen required, advisory, and deferred policy before the later grouped consumer proof lands.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped integrated acceptance lane without reading contract prose or manually composing the four grouped acceptance descriptors.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded integrated live-ownership and workflow acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of already-closed grouped lanes.",
        },
    ]
}

pub(crate) fn render_integrated_live_workflow_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "integrated_live_workflow_acceptance_lane",
        lane: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: integrated_live_workflow_acceptance_families(),
        validation_steps: integrated_live_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required Linux live, device workflow, immersive, and control-preview workflow acceptance tasks without claiming repeated-run certification or environment-specific exhaustiveness",
            "broader repeated-run confidence passes, richer host-profile or environment-specific permutations, and closer-to-closeout reruns remain advisory or deferred instead of silently entering the required lane",
            "broader environment certification, repeated-run stress matrices, and closeout-adjacent downstream workflow depth remain outside the bounded integrated lane until later closeout work promotes them explicitly",
        ],
    })
}

pub(crate) fn render_integrated_live_workflow_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "integrated_live_workflow_acceptance_lane",
        lane: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_LANE,
        contract_path: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK,
        required_tasks: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS,
        families: integrated_live_workflow_acceptance_families(),
        validation_steps: integrated_live_workflow_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required Linux live, device workflow, immersive, and control-preview workflow acceptance tasks without claiming repeated-run certification or environment-specific exhaustiveness",
            "broader repeated-run confidence passes, richer host-profile or environment-specific permutations, and closer-to-closeout reruns remain advisory or deferred instead of silently entering the required lane",
            "broader environment certification, repeated-run stress matrices, and closeout-adjacent downstream workflow depth remain outside the bounded integrated lane until later closeout work promotes them explicitly",
        ],
    })
}
