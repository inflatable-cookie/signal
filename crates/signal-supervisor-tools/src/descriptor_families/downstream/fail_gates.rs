use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamFailGateRule {
    id: &'static str,
    gate: &'static str,
    command: &'static str,
    blocks_release: bool,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamDeferredDepthRecord {
    id: &'static str,
    command: &'static str,
    status: &'static str,
    rationale: &'static str,
}

fn downstream_fail_gate_rules() -> &'static [DownstreamFailGateRule] {
    &[
        DownstreamFailGateRule {
            id: "mandatory-release-gate",
            gate: "required",
            command: DOWNSTREAM_AUTOMATION_MANDATORY_TASK,
            blocks_release: true,
            rationale:
                "The bounded downstream release task is the current mandatory gate for widened consumer and packaging claims.",
        },
        DownstreamFailGateRule {
            id: "automation-boundary-descriptor",
            gate: "required",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json",
            blocks_release: true,
            rationale:
                "The fail-gate policy itself must remain inspectable as a machine-readable repo-owned surface.",
        },
        DownstreamFailGateRule {
            id: "optional-depth-lane",
            gate: "advisory",
            command: DOWNSTREAM_AUTOMATION_OPTIONAL_TASK,
            blocks_release: false,
            rationale:
                "Optional depth broadens confidence, but it does not currently block the fast release path.",
        },
    ]
}

fn downstream_deferred_depth_records() -> &'static [DownstreamDeferredDepthRecord] {
    &[
        DownstreamDeferredDepthRecord {
            id: "server-soak-export",
            command: "cargo run -p signal-supervisor-tools -- --format=json server soak",
            status: "deferred",
            rationale:
                "The current server-host soak path is not yet stable enough to gate release because the recovery-overlap attach limit still trips this fixture.",
        },
        DownstreamDeferredDepthRecord {
            id: "analysis-acceptance-promotion",
            command: "effigy acceptance:analysis",
            status: "deferred",
            rationale:
                "Analysis acceptance remains useful optional depth, but it is not yet part of the bounded shared release gate.",
        },
    ]
}

pub(crate) fn render_downstream_fail_gates_text() -> String {
    let mut rendered = format!(
        "downstream_fail_gates: {DOWNSTREAM_FAIL_GATES}\ncontract_path: {DOWNSTREAM_AUTOMATION_CONTRACT_PATH}\nfail_gate_task: {DOWNSTREAM_FAIL_GATE_TASK}\nmandatory_release_task: {DOWNSTREAM_AUTOMATION_MANDATORY_TASK}\noptional_depth_task: {DOWNSTREAM_AUTOMATION_OPTIONAL_TASK}\nrules:\n"
    );
    for rule in downstream_fail_gate_rules() {
        rendered.push_str(&format!(
            "- id: {}\n  gate: {}\n  command: {}\n  blocks_release: {}\n  rationale: {}\n",
            rule.id, rule.gate, rule.command, rule.blocks_release, rule.rationale,
        ));
    }
    rendered.push_str("deferred_depth:\n");
    for record in downstream_deferred_depth_records() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  status: {}\n  rationale: {}\n",
            record.id, record.command, record.status, record.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_downstream_fail_gates_json() -> String {
    let rules = downstream_fail_gate_rules()
        .iter()
        .map(|rule| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"gate\":{},",
                    "\"command\":{},",
                    "\"blocks_release\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(rule.id),
                json_string(rule.gate),
                json_string(rule.command),
                rule.blocks_release,
                json_string(rule.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred = downstream_deferred_depth_records()
        .iter()
        .map(|record| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"status\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(record.id),
                json_string(record.command),
                json_string(record.status),
                json_string(record.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"fail_gate_task\":{},",
            "\"mandatory_release_task\":{},",
            "\"optional_depth_task\":{},",
            "\"rules\":[{}],",
            "\"deferred_depth\":[{}]",
            "}}"
        ),
        json_string(DOWNSTREAM_FAIL_GATES),
        json_string(DOWNSTREAM_AUTOMATION_CONTRACT_PATH),
        json_string(DOWNSTREAM_FAIL_GATE_TASK),
        json_string(DOWNSTREAM_AUTOMATION_MANDATORY_TASK),
        json_string(DOWNSTREAM_AUTOMATION_OPTIONAL_TASK),
        rules,
        deferred,
    )
}
