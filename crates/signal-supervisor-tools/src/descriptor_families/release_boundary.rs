use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReleaseBoundaryArtifactKind {
    Document,
    ExportDescription,
    ConformanceMatrix,
    PackagingManifest,
    Example,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryArtifact {
    id: &'static str,
    kind: ReleaseBoundaryArtifactKind,
    path_or_command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl ReleaseBoundaryArtifactKind {
    fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::ExportDescription => "export-description",
            Self::ConformanceMatrix => "conformance-matrix",
            Self::PackagingManifest => "packaging-manifest",
            Self::Example => "example",
        }
    }
}

fn release_boundary_artifacts() -> &'static [ReleaseBoundaryArtifact] {
    &[
        ReleaseBoundaryArtifact {
            id: "workspace-changelog",
            kind: ReleaseBoundaryArtifactKind::Document,
            path_or_command: RELEASE_CHANGELOG_PATH,
            rationale:
                "Every consumer-facing release baseline must carry a human-readable change summary in the workspace changelog.",
        },
        ReleaseBoundaryArtifact {
            id: "supervisor-export-description",
            kind: ReleaseBoundaryArtifactKind::ExportDescription,
            path_or_command: "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
            rationale:
                "The versioned export schema remains the machine-readable release contract for automation.",
        },
        ReleaseBoundaryArtifact {
            id: "consumer-conformance-matrix",
            kind: ReleaseBoundaryArtifactKind::ConformanceMatrix,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Consumers need one inspectable list of the runnable proof surfaces included in the baseline.",
        },
        ReleaseBoundaryArtifact {
            id: "publication-packaging-manifest",
            kind: ReleaseBoundaryArtifactKind::PackagingManifest,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "Publication-grade packaging now has a repo-owned manifest descriptor instead of living only in prose around the baseline release boundary.",
        },
        ReleaseBoundaryArtifact {
            id: "runtime-supervisor-report-demo",
            kind: ReleaseBoundaryArtifactKind::Example,
            path_or_command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "The human-readable report example remains part of the first shared release baseline for manual inspection.",
        },
    ]
}

fn release_boundary_validation_steps() -> &'static [ReleaseBoundaryValidationStep] {
    &[
        ReleaseBoundaryValidationStep {
            id: "consumer-conformance",
            command: RELEASE_CONFORMANCE_TASK,
            rationale:
                "The runnable consumer conformance matrix must pass before the packaging baseline is considered valid.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-health",
            command: "effigy health",
            rationale:
                "The repo-owned build baseline must stay healthy for a release-boundary claim to be credible.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-test",
            command: "effigy test",
            rationale:
                "The shared repo-owned test surface remains part of the packaging baseline rather than downstream-only policy.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-validate",
            command: "effigy validate",
            rationale:
                "Validation must include the repo-owned configure/build/test chain before a release boundary is declared.",
        },
    ]
}

fn release_boundary_unstable_scopes() -> &'static [&'static str] {
    &[
        "backend breadth beyond the current CLAP-first plugin path",
        "host convenience APIs outside the frozen runtime/export boundary",
        "crates.io publication and downstream release orchestration",
        "publication packaging beyond the repo-owned manifest descriptor and receipt inventory",
    ]
}

pub(crate) fn render_release_boundary_text() -> String {
    let mut rendered = format!(
        "release_boundary: {RELEASE_BOUNDARY}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\nchangelog_path: {RELEASE_CHANGELOG_PATH}\nexport_schema: {EXPORT_SCHEMA}\nexport_schema_version: {EXPORT_SCHEMA_VERSION}\nconformance_task: {RELEASE_CONFORMANCE_TASK}\nartifacts:\n",
        env!("CARGO_PKG_VERSION")
    );
    for artifact in release_boundary_artifacts() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            artifact.id,
            artifact.kind.label(),
            artifact.path_or_command,
            artifact.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in release_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("intentionally_unstable:\n");
    for scope in release_boundary_unstable_scopes() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_release_boundary_json() -> String {
    let artifacts = release_boundary_artifacts()
        .iter()
        .map(|artifact| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(artifact.id),
                json_string(artifact.kind.label()),
                json_string(artifact.path_or_command),
                json_string(artifact.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = release_boundary_validation_steps()
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
    let unstable = release_boundary_unstable_scopes()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"changelog_path\":{},",
            "\"export_schema\":{},",
            "\"export_schema_version\":{},",
            "\"conformance_task\":{},",
            "\"artifacts\":[{}],",
            "\"validation_steps\":[{}],",
            "\"intentionally_unstable\":[{}]",
            "}}"
        ),
        json_string(RELEASE_BOUNDARY),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(RELEASE_CHANGELOG_PATH),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(RELEASE_CONFORMANCE_TASK),
        artifacts,
        validation_steps,
        unstable,
    )
}
