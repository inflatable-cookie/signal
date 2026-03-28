use super::super::*;

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
