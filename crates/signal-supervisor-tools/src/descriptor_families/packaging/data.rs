use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PackagingManifestInputKind {
    Document,
    Descriptor,
    ValidationTask,
    Contract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PackagingManifestInput {
    pub(super) id: &'static str,
    pub(super) kind: PackagingManifestInputKind,
    pub(super) path_or_command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PackagingReceiptSurface {
    pub(super) id: &'static str,
    pub(super) surface: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PackagingManifestValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

impl PackagingManifestInputKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Descriptor => "descriptor",
            Self::ValidationTask => "validation-task",
            Self::Contract => "contract",
        }
    }
}

pub(super) fn packaging_manifest_inputs() -> &'static [PackagingManifestInput] {
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

pub(super) fn packaging_receipt_surfaces() -> &'static [PackagingReceiptSurface] {
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

pub(super) fn packaging_manifest_validation_steps() -> &'static [PackagingManifestValidationStep] {
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

pub(super) fn packaging_manifest_unsupported_paths() -> &'static [&'static str] {
    &[
        "crates.io publication and registry upload automation",
        "signed installers, notarization, and platform distribution packaging",
        "downstream application-specific release wrappers or private CI pipelines",
        "generation closeout bundling and post-release promotion policy beyond the current g05 milestone",
    ]
}
