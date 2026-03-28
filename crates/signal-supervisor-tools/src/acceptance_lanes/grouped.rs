use super::*;

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

fn linux_live_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "live-ownership-and-guarded-continuity",
            title: "Live Ownership And Guarded Continuity",
            required_tasks: LINUX_LIVE_OWNERSHIP_REQUIRED_TASKS,
            advisory_tasks: LINUX_LIVE_OWNERSHIP_ADVISORY_TASKS,
            rationale:
                "Keeps live Linux ownership, guarded continuity, and clock-topology impact on one required lane instead of letting backend-local recovery policy define the shared proof surface.",
        },
        IntegratedAcceptanceFamily {
            id: "backend-native-coordination-and-parity",
            title: "Backend-Native Coordination And Parity",
            required_tasks: LINUX_LIVE_BACKEND_PROTOCOL_REQUIRED_TASKS,
            advisory_tasks: LINUX_LIVE_BACKEND_PROTOCOL_ADVISORY_TASKS,
            rationale:
                "Requires JACK coordination and PipeWire/ALSA parity truth together rather than treating backend-native coordination as optional daemon-local depth.",
        },
        IntegratedAcceptanceFamily {
            id: "cross-backend-host-edge-coherence",
            title: "Cross-Backend Host-Edge Coherence",
            required_tasks: LINUX_LIVE_HOST_EDGE_REQUIRED_TASKS,
            advisory_tasks: LINUX_LIVE_HOST_EDGE_ADVISORY_TASKS,
            rationale:
                "Pins the shared lane to public runtime, supervisor, and both stable host edges so one backend or one host path cannot define a special-case Linux live story.",
        },
    ]
}

fn linux_live_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence",
            rationale:
                "Proves one supervisor export can carry Linux live ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology truth together instead of only listing separate boundary-local descriptors.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools linux_live_acceptance_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable Linux live acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the grouped Linux live acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: LINUX_LIVE_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded Linux live acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist of isolated boundary proofs.",
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

pub(crate) fn render_linux_live_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "linux_live_acceptance_lane",
        lane: LINUX_LIVE_ACCEPTANCE_LANE,
        contract_path: LINUX_LIVE_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: LINUX_LIVE_ACCEPTANCE_TASK,
        required_tasks: LINUX_LIVE_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: LINUX_LIVE_ACCEPTANCE_ADVISORY_TASKS,
        families: linux_live_acceptance_families(),
        validation_steps: linux_live_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required Linux live ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology acceptance tasks without claiming exhaustive distro or daemon certification",
            "backend-native daemon policy, session-manager glue, and richer repeated-run Linux recovery matrices remain advisory or deferred instead of silently entering the required lane",
            "broader immersive, preview, device-workflow, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}

pub(crate) fn render_linux_live_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "linux_live_acceptance_lane",
        lane: LINUX_LIVE_ACCEPTANCE_LANE,
        contract_path: LINUX_LIVE_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: LINUX_LIVE_ACCEPTANCE_TASK,
        required_tasks: LINUX_LIVE_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: LINUX_LIVE_ACCEPTANCE_ADVISORY_TASKS,
        families: linux_live_acceptance_families(),
        validation_steps: linux_live_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane groups required Linux live ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology acceptance tasks without claiming exhaustive distro or daemon certification",
            "backend-native daemon policy, session-manager glue, and richer repeated-run Linux recovery matrices remain advisory or deferred instead of silently entering the required lane",
            "broader immersive, preview, device-workflow, and generation-level integrated acceptance still belong to later g08 milestones",
        ],
    })
}
