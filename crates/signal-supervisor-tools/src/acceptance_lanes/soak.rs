use super::*;

fn g06_soak_lane_records() -> &'static [G06SoakLaneScenarioRecord] {
    &[
        G06SoakLaneScenarioRecord {
            id: "required-local-soak-export",
            status: "required",
            command: "cargo run -p signal-supervisor-tools -- --format=json local soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Provides one bounded long-session local-host soak artifact carrying runtime profiling, soak, and supervisor receipts together.",
        },
        G06SoakLaneScenarioRecord {
            id: "required-local-mixed-soak-export",
            status: "required",
            command: "cargo run -p signal-supervisor-tools -- --format=json local mixed",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Keeps mixed watchdog and recovery churn inside the bounded soak lane without depending on deferred server-host overlap behavior.",
        },
        G06SoakLaneScenarioRecord {
            id: "advisory-integrated-acceptance-base",
            status: "advisory",
            command: INTEGRATED_ACCEPTANCE_TASK,
            typed_output:
                "machine-readable integrated acceptance descriptors plus boundary proof outputs",
            rationale:
                "The bounded soak lane still depends on the fast integrated lane staying green, but that fast path remains a separate required base rather than the soak lane itself.",
        },
        G06SoakLaneScenarioRecord {
            id: "deferred-server-soak-export",
            status: "deferred",
            command: "cargo run -p signal-supervisor-tools -- --format=json server soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "The broader server-host soak path remains outside the bounded lane because the recovery-overlap attach limit still trips that scenario.",
        },
    ]
}

fn g06_soak_lane_validation_steps() -> &'static [G06SoakLaneValidationStep] {
    &[
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-proof",
            command:
                "cargo test -p signal-supervisor-tools g06_soak_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable bounded soak descriptor aligned with the required, advisory, and deferred policy frozen in the closeout contract.",
        },
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json",
            rationale:
                "Lets maintainers inspect the bounded soak lane without reading closeout contract prose or Effigy internals.",
        },
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-task",
            command: G06_SOAK_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded soak lane is runnable as one repo-owned Effigy task instead of a loose list of scenario commands.",
        },
    ]
}

pub(crate) fn render_g06_soak_lane_text() -> String {
    let mut rendered = format!(
        "g06_soak_lane: {G06_SOAK_LANE}\ncontract_path: {G06_SOAK_CONTRACT_PATH}\nacceptance_task: {G06_SOAK_ACCEPTANCE_TASK}\nrecords:\n"
    );
    for record in g06_soak_lane_records() {
        rendered.push_str(&format!(
            "- id: {}\n  status: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            record.id, record.status, record.command, record.typed_output, record.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in g06_soak_lane_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the broader server-host soak path remains deferred because the recovery-overlap attach limit still trips that scenario",
        "wider rerun counts and promotion thresholds still belong to later g06.020 closeout review work",
        "remote or distributed soak orchestration remains outside the shared bounded lane",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_g06_soak_lane_json() -> String {
    let records = g06_soak_lane_records()
        .iter()
        .map(|record| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"status\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(record.id),
                json_string(record.status),
                json_string(record.command),
                json_string(record.typed_output),
                json_string(record.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = g06_soak_lane_validation_steps()
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "the broader server-host soak path remains deferred because the recovery-overlap attach limit still trips that scenario",
        "wider rerun counts and promotion thresholds still belong to later g06.020 closeout review work",
        "remote or distributed soak orchestration remains outside the shared bounded lane",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        concat!(
            "{{",
            "\"lane\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"record_count\":{},",
            "\"records\":[{}],",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(G06_SOAK_LANE),
        json_string(G06_SOAK_CONTRACT_PATH),
        json_string(G06_SOAK_ACCEPTANCE_TASK),
        g06_soak_lane_records().len(),
        records,
        validation_steps,
        deferred_scope,
    )
}
