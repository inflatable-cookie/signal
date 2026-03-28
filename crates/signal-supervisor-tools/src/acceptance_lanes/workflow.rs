use super::*;

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
