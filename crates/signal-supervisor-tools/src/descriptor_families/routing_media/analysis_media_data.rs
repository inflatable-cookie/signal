#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MediaServiceBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl MediaServiceBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MediaServiceBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: MediaServiceBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MediaServiceBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnalysisMetadataBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl AnalysisMetadataBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AnalysisMetadataBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: AnalysisMetadataBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AnalysisMetadataBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn media_service_boundary_surfaces() -> &'static [MediaServiceBoundarySurface] {
    &[
        MediaServiceBoundarySurface {
            id: "runtime-media-service-report",
            kind: MediaServiceBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::media_pipeline_snapshot, RuntimeObservationReport::media_service_snapshot, and RuntimeSupervisorReport::observation.{media_pipeline_snapshot,media_service_snapshot}",
            runtime_anchor: "RuntimeMediaPipelineSnapshot + RuntimeMediaServiceSnapshot",
            rationale:
                "Keeps indexing, waveform readiness, preview readiness, and invalidation truth on one runtime-owned report seam instead of product-local preview or cache heuristics.",
        },
        MediaServiceBoundarySurface {
            id: "runtime-media-service-snapshot",
            kind: MediaServiceBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationApi::get_media_pipeline_snapshot() and RuntimeObservationApi::get_media_service_snapshot()",
            runtime_anchor: "RuntimeObservationApi media service accessors",
            rationale:
                "Lets downstream consumers inspect the same media indexing and service truth directly from runtime-owned snapshots instead of bespoke media-service facades.",
        },
        MediaServiceBoundarySurface {
            id: "shared-host-media-service-report",
            kind: MediaServiceBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned media readiness, invalidation, and preview state without product-local reconstruction.",
        },
    ]
}

pub(super) fn media_service_boundary_validation_steps(
) -> &'static [MediaServiceBoundaryValidationStep] {
    &[
        MediaServiceBoundaryValidationStep {
            id: "runtime-media-service-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect indexing, waveform readiness, preview state, and invalidation truth through public runtime reexports alone.",
        },
        MediaServiceBoundaryValidationStep {
            id: "local-host-media-service-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_media_service_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned media pipeline and media-service receipts on supervisor export.",
        },
        MediaServiceBoundaryValidationStep {
            id: "server-host-media-service-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_media_service_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned media readiness and invalidation receipt family on supervisor export.",
        },
        MediaServiceBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools media_service_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable media-service boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        MediaServiceBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared media indexing, waveform, preview, and invalidation seam without reading private product pipelines.",
        },
    ]
}

pub(super) fn analysis_metadata_boundary_surfaces() -> &'static [AnalysisMetadataBoundarySurface] {
    &[
        AnalysisMetadataBoundarySurface {
            id: "runtime-analysis-metadata-report",
            kind: AnalysisMetadataBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::media_library_snapshot and RuntimeSupervisorReport::observation.media_library_snapshot",
            runtime_anchor: "RuntimeMediaLibraryServiceSnapshot",
            rationale:
                "Keeps reusable loudness, character, and explicit deferred-family coverage on one runtime-owned report seam instead of product-local metadata caches.",
        },
        AnalysisMetadataBoundarySurface {
            id: "runtime-analysis-metadata-snapshot",
            kind: AnalysisMetadataBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_media_library_service_snapshot()",
            runtime_anchor: "RuntimeObservationApi media library accessor",
            rationale:
                "Lets downstream consumers inspect the same asset-analysis descriptor family directly from runtime-owned snapshots instead of reconstructing availability from media-service state alone.",
        },
        AnalysisMetadataBoundarySurface {
            id: "shared-host-analysis-metadata-report",
            kind: AnalysisMetadataBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned analysis-metadata and library-service receipts without product-local extraction or metadata forks.",
        },
    ]
}

pub(super) fn analysis_metadata_boundary_validation_steps(
) -> &'static [AnalysisMetadataBoundaryValidationStep] {
    &[
        AnalysisMetadataBoundaryValidationStep {
            id: "runtime-analysis-metadata-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors",
            rationale:
                "Proves a downstream-style runtime consumer can inspect the reusable library descriptor family, including ready and invalidated outcomes, through public runtime reexports alone.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "local-host-analysis-metadata-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_analysis_metadata_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned analysis metadata and library-service descriptors on supervisor export.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "server-host-analysis-metadata-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_analysis_metadata_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned analysis descriptor family without private metadata reconstruction.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable analysis-metadata boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared analysis-metadata and library-service seam without reading private product extraction code.",
        },
    ]
}
