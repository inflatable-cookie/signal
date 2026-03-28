use super::*;

fn integrated_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "recovery-and-fault-attribution",
            title: "Recovery And Fault Attribution",
            required_tasks: RECOVERY_AND_FAULT_REQUIRED_TASKS,
            advisory_tasks: RECOVERY_AND_FAULT_ADVISORY_TASKS,
            rationale:
                "Keeps interruption, fault diagnostics, and device supervision in the bounded lane while leaving broader continuity depth explicit but non-blocking.",
        },
        IntegratedAcceptanceFamily {
            id: "scheduling-and-execution-pressure",
            title: "Scheduling And Execution Pressure",
            required_tasks: SCHEDULING_AND_PRESSURE_REQUIRED_TASKS,
            advisory_tasks: SCHEDULING_AND_PRESSURE_ADVISORY_TASKS,
            rationale:
                "Pins execution pressure to bounded hot-path and deferred-work policy receipts without forcing every timing-adjacent proof into the required lane.",
        },
        IntegratedAcceptanceFamily {
            id: "adapter-and-portability-breadth",
            title: "Adapter And Portability Breadth",
            required_tasks: ADAPTER_AND_PORTABILITY_REQUIRED_TASKS,
            advisory_tasks: ADAPTER_AND_PORTABILITY_ADVISORY_TASKS,
            rationale:
                "Requires one shared plugin continuity and portability lane while keeping richer per-format and event-depth checks visible as advisory breadth.",
        },
        IntegratedAcceptanceFamily {
            id: "hardware-and-external-io-continuity",
            title: "Hardware And External-I/O Continuity",
            required_tasks: HARDWARE_AND_EXTERNAL_IO_REQUIRED_TASKS,
            advisory_tasks: HARDWARE_AND_EXTERNAL_IO_ADVISORY_TASKS,
            rationale:
                "Makes hardware restart, topology, and external-I/O truth part of the integrated lane instead of leaving them as isolated subsystem proofs.",
        },
        IntegratedAcceptanceFamily {
            id: "media-and-library-service-continuity",
            title: "Media And Library-Service Continuity",
            required_tasks: MEDIA_AND_LIBRARY_REQUIRED_TASKS,
            advisory_tasks: MEDIA_AND_LIBRARY_ADVISORY_TASKS,
            rationale:
                "Keeps reusable media readiness and analysis-metadata descriptors in the shared lane without expanding into product-local browser workflows.",
        },
    ]
}

fn integrated_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence",
            rationale:
                "Proves one repo-owned supervisor export carries recovery, deferred-work, adapter breadth, hardware, and media/library evidence together instead of reducing the lane to a checklist of isolated boundary tasks.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools integrated_acceptance_lane_json_reports_required_and_advisory_policy",
            rationale:
                "Keeps the machine-readable integrated acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the integrated acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: INTEGRATED_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded required acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist.",
        },
    ]
}

pub(crate) fn render_integrated_acceptance_lane_text() -> String {
    render_acceptance_lane_text(&AcceptanceLaneRender {
        lane_label: "integrated_acceptance_lane",
        lane: INTEGRATED_ACCEPTANCE_LANE,
        contract_path: INTEGRATED_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: INTEGRATED_ACCEPTANCE_TASK,
        required_tasks: INTEGRATED_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: INTEGRATED_ACCEPTANCE_ADVISORY_TASKS,
        families: integrated_acceptance_families(),
        validation_steps: integrated_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane now groups one required cross-family acceptance path, but long-session soak thresholds and promotion policy still belong to g06.020",
            "unstable broader server-host recovery-overlap scenarios remain explicitly deferred until the integrated lane is real and bounded",
            "product-local QA dashboards, browser workflows, and exhaustive environment certification remain outside the shared Signal acceptance lane",
        ],
    })
}

pub(crate) fn render_integrated_acceptance_lane_json() -> String {
    render_acceptance_lane_json(&AcceptanceLaneRender {
        lane_label: "integrated_acceptance_lane",
        lane: INTEGRATED_ACCEPTANCE_LANE,
        contract_path: INTEGRATED_ACCEPTANCE_CONTRACT_PATH,
        acceptance_task: INTEGRATED_ACCEPTANCE_TASK,
        required_tasks: INTEGRATED_ACCEPTANCE_REQUIRED_TASKS,
        advisory_tasks: INTEGRATED_ACCEPTANCE_ADVISORY_TASKS,
        families: integrated_acceptance_families(),
        validation_steps: integrated_acceptance_validation_steps(),
        deferred_scope: &[
            "the bounded lane now groups one required cross-family acceptance path, but long-session soak thresholds and promotion policy still belong to g06.020",
            "unstable broader server-host recovery-overlap scenarios remain explicitly deferred until the integrated lane is real and bounded",
            "product-local QA dashboards, browser workflows, and exhaustive environment certification remain outside the shared Signal acceptance lane",
        ],
    })
}
