use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DownstreamAutomationFixtureKind {
    AcceptanceTask,
    Descriptor,
    ScenarioExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamAutomationFixture {
    id: &'static str,
    kind: DownstreamAutomationFixtureKind,
    command: &'static str,
    typed_output: &'static str,
    rationale: &'static str,
}

impl DownstreamAutomationFixtureKind {
    fn label(self) -> &'static str {
        match self {
            Self::AcceptanceTask => "acceptance-task",
            Self::Descriptor => "descriptor",
            Self::ScenarioExport => "scenario-export",
        }
    }
}

fn downstream_automation_mandatory_fixtures() -> &'static [DownstreamAutomationFixture] {
    &[
        DownstreamAutomationFixture {
            id: "consumer-conformance",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: RELEASE_CONFORMANCE_TASK,
            typed_output:
                "conformance matrix descriptor plus task-local test/example receipts",
            rationale:
                "The bounded release fast path still starts from the shared consumer conformance matrix.",
        },
        DownstreamAutomationFixture {
            id: "release-packaging-consumer",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: PACKAGING_MANIFEST_ACCEPTANCE_TASK,
            typed_output:
                "release-boundary and packaging-manifest descriptors plus public binary-facing proof",
            rationale:
                "The mandatory release path must prove packaging claims remain consumable without private scripts.",
        },
        DownstreamAutomationFixture {
            id: "downstream-automation-descriptor",
            kind: DownstreamAutomationFixtureKind::Descriptor,
            command:
                "cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json",
            typed_output: "machine-readable downstream automation boundary descriptor",
            rationale:
                "Mandatory release automation must stay inspectable as one repo-owned boundary description.",
        },
    ]
}

fn downstream_automation_optional_fixtures() -> &'static [DownstreamAutomationFixture] {
    &[
        DownstreamAutomationFixture {
            id: "local-mixed-watchdog-export",
            kind: DownstreamAutomationFixtureKind::ScenarioExport,
            command: "cargo run -p signal-supervisor-tools -- --format=json local mixed",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Optional depth should exercise richer mixed watchdog/fault scenarios through typed export rather than log-only review.",
        },
        DownstreamAutomationFixture {
            id: "local-soak-export",
            kind: DownstreamAutomationFixtureKind::ScenarioExport,
            command: "cargo run -p signal-supervisor-tools -- --format=json local soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Optional depth should include a broader watchdog-soak path while keeping the output typed and inspectable.",
        },
        DownstreamAutomationFixture {
            id: "analysis-acceptance",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: "effigy acceptance:analysis",
            typed_output: "analysis harness task receipts across the shared analysis crates",
            rationale:
                "Longer-running shared confidence can extend into broader analysis acceptance without becoming a release prerequisite yet.",
        },
    ]
}

pub(crate) fn render_downstream_automation_text() -> String {
    let mut rendered = format!(
        "downstream_automation_boundary: {DOWNSTREAM_AUTOMATION_BOUNDARY}\ncontract_path: {DOWNSTREAM_AUTOMATION_CONTRACT_PATH}\nmandatory_release_task: {DOWNSTREAM_AUTOMATION_MANDATORY_TASK}\noptional_depth_task: {DOWNSTREAM_AUTOMATION_OPTIONAL_TASK}\ncombined_task: {DOWNSTREAM_AUTOMATION_COMBINED_TASK}\nmandatory_release_acceptance:\n"
    );
    for fixture in downstream_automation_mandatory_fixtures() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            fixture.id,
            fixture.kind.label(),
            fixture.command,
            fixture.typed_output,
            fixture.rationale,
        ));
    }
    rendered.push_str("optional_confidence_depth:\n");
    for fixture in downstream_automation_optional_fixtures() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            fixture.id,
            fixture.kind.label(),
            fixture.command,
            fixture.typed_output,
            fixture.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_downstream_automation_json() -> String {
    let mandatory = downstream_automation_mandatory_fixtures()
        .iter()
        .map(|fixture| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(fixture.id),
                json_string(fixture.kind.label()),
                json_string(fixture.command),
                json_string(fixture.typed_output),
                json_string(fixture.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let optional = downstream_automation_optional_fixtures()
        .iter()
        .map(|fixture| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(fixture.id),
                json_string(fixture.kind.label()),
                json_string(fixture.command),
                json_string(fixture.typed_output),
                json_string(fixture.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"mandatory_release_task\":{},",
            "\"optional_depth_task\":{},",
            "\"combined_task\":{},",
            "\"mandatory_release_acceptance\":[{}],",
            "\"optional_confidence_depth\":[{}]",
            "}}"
        ),
        json_string(DOWNSTREAM_AUTOMATION_BOUNDARY),
        json_string(DOWNSTREAM_AUTOMATION_CONTRACT_PATH),
        json_string(DOWNSTREAM_AUTOMATION_MANDATORY_TASK),
        json_string(DOWNSTREAM_AUTOMATION_OPTIONAL_TASK),
        json_string(DOWNSTREAM_AUTOMATION_COMBINED_TASK),
        mandatory,
        optional,
    )
}
