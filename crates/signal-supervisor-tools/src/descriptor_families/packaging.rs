use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackagingManifestInputKind {
    Document,
    Descriptor,
    ValidationTask,
    Contract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingManifestInput {
    id: &'static str,
    kind: PackagingManifestInputKind,
    path_or_command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingReceiptSurface {
    id: &'static str,
    surface: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingManifestValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl PackagingManifestInputKind {
    fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Descriptor => "descriptor",
            Self::ValidationTask => "validation-task",
            Self::Contract => "contract",
        }
    }
}

fn packaging_manifest_inputs() -> &'static [PackagingManifestInput] {
    &[
        PackagingManifestInput {
            id: "workspace-changelog",
            kind: PackagingManifestInputKind::Document,
            path_or_command: RELEASE_CHANGELOG_PATH,
            rationale:
                "The publication bundle still anchors human-readable release notes in the workspace changelog.",
        },
        PackagingManifestInput {
            id: "export-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command: "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
            rationale:
                "The versioned supervisor export descriptor remains the canonical machine-readable schema source.",
        },
        PackagingManifestInput {
            id: "consumer-conformance-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "The packaging manifest must include the repo-owned consumer-proof boundary rather than a private release matrix.",
        },
        PackagingManifestInput {
            id: "host-edge-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json",
            rationale:
                "Stable shared host edges must remain explicit in the publication bundle instead of being inferred from host crate internals.",
        },
        PackagingManifestInput {
            id: "release-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json",
            rationale:
                "The publication manifest aggregates the existing host-free release boundary rather than replacing it.",
        },
        PackagingManifestInput {
            id: "plugin-backend-breadth-acceptance",
            kind: PackagingManifestInputKind::ValidationTask,
            path_or_command: "effigy acceptance:plugin-backend-breadth",
            rationale:
                "Release packaging claims about backend-neutral breadth must point back to the repo-owned acceptance task that proves them.",
        },
        PackagingManifestInput {
            id: "host-edge-consumer-acceptance",
            kind: PackagingManifestInputKind::ValidationTask,
            path_or_command: "effigy acceptance:host-edge-consumer",
            rationale:
                "The manifest includes the stable shared host-edge proof rather than assuming it from release prose.",
        },
        PackagingManifestInput {
            id: "packaging-contract",
            kind: PackagingManifestInputKind::Contract,
            path_or_command: PACKAGING_MANIFEST_CONTRACT_PATH,
            rationale:
                "The packaging manifest stays anchored to the frozen contract instead of an ad hoc release script.",
        },
    ]
}

fn packaging_receipt_surfaces() -> &'static [PackagingReceiptSurface] {
    &[
        PackagingReceiptSurface {
            id: "manifest-generation-receipt",
            surface:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "The packaging manifest descriptor is the repo-owned receipt for what Signal currently considers packageable.",
        },
        PackagingReceiptSurface {
            id: "validation-receipt",
            surface: PACKAGING_MANIFEST_ACCEPTANCE_TASK,
            rationale:
                "The packaging acceptance task is the repo-owned receipt that the declared bundle and validation spine stay runnable together.",
        },
    ]
}

fn packaging_manifest_validation_steps() -> &'static [PackagingManifestValidationStep] {
    &[
        PackagingManifestValidationStep {
            id: "release-boundary-baseline",
            command: "effigy acceptance:release-boundary",
            rationale:
                "Publication packaging builds on the existing release-boundary baseline instead of replacing it.",
        },
        PackagingManifestValidationStep {
            id: "packaging-manifest-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "Consumers and automation need one machine-readable publication manifest descriptor.",
        },
        PackagingManifestValidationStep {
            id: "workspace-health",
            command: "effigy health",
            rationale:
                "Publication packaging claims still depend on the repo-owned build baseline staying healthy.",
        },
        PackagingManifestValidationStep {
            id: "workspace-docs",
            command: "effigy qa:docs",
            rationale:
                "The publication manifest depends on docs and index surfaces staying aligned with the declared release bundle.",
        },
    ]
}

fn packaging_manifest_unsupported_paths() -> &'static [&'static str] {
    &[
        "crates.io publication and registry upload automation",
        "signed installers, notarization, and platform distribution packaging",
        "downstream application-specific release wrappers or private CI pipelines",
        "generation closeout bundling and post-release promotion policy beyond the current g05 milestone",
    ]
}

pub(crate) fn render_packaging_manifest_text() -> String {
    let mut rendered = format!(
        "packaging_manifest: {PACKAGING_MANIFEST}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\ncontract_path: {PACKAGING_MANIFEST_CONTRACT_PATH}\nacceptance_task: {PACKAGING_MANIFEST_ACCEPTANCE_TASK}\ninputs:\n",
        env!("CARGO_PKG_VERSION")
    );
    for input in packaging_manifest_inputs() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            input.id,
            input.kind.label(),
            input.path_or_command,
            input.rationale,
        ));
    }
    rendered.push_str("receipt_surfaces:\n");
    for receipt in packaging_receipt_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  surface: {}\n  rationale: {}\n",
            receipt.id, receipt.surface, receipt.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in packaging_manifest_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("unsupported_publication_paths:\n");
    for scope in packaging_manifest_unsupported_paths() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_packaging_manifest_json() -> String {
    let inputs = packaging_manifest_inputs()
        .iter()
        .map(|input| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(input.id),
                json_string(input.kind.label()),
                json_string(input.path_or_command),
                json_string(input.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let receipts = packaging_receipt_surfaces()
        .iter()
        .map(|receipt| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"surface\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(receipt.id),
                json_string(receipt.surface),
                json_string(receipt.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = packaging_manifest_validation_steps()
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
    let unsupported = packaging_manifest_unsupported_paths()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"manifest\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"inputs\":[{}],",
            "\"receipt_surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"unsupported_publication_paths\":[{}]",
            "}}"
        ),
        json_string(PACKAGING_MANIFEST),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(PACKAGING_MANIFEST_CONTRACT_PATH),
        json_string(PACKAGING_MANIFEST_ACCEPTANCE_TASK),
        inputs,
        receipts,
        validation_steps,
        unsupported,
    )
}
