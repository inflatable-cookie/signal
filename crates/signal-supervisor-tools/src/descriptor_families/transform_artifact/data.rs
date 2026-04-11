#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TransformArtifactBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TransformArtifactBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: TransformArtifactBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TransformArtifactBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

impl TransformArtifactBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

pub(super) fn transform_artifact_boundary_surfaces() -> &'static [TransformArtifactBoundarySurface]
{
    &[
        TransformArtifactBoundarySurface {
            id: "runtime-transform-artifact-observation-report",
            kind: TransformArtifactBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::transform_artifact_snapshot and RuntimeSupervisorReport::observation.transform_artifact_snapshot",
            runtime_anchor:
                "RuntimeTransformArtifactSnapshot::{clip_count, ready_clip_count, pending_media_clip_count, degraded_clip_count, invalidated_clip_count, unsupported_clip_count, cached_media_ready_clip_count, reusable_clip_count, requires_render_clip_count, guarded_reuse_clip_count, transform_persistence} + RuntimeTransformPersistenceSummary::{persistence_posture, retention_policy_class, retention_authority, retention_outcome, cache_placement_posture, cache_placement_authority, cache_placement_outcome}",
            rationale:
                "Keeps post-warp render, cache readiness, invalidation, reuse, retention, and cache-placement truth on one runtime-owned report seam instead of host-local preview-cache or browser-local persistence policy.",
        },
        TransformArtifactBoundarySurface {
            id: "runtime-transform-artifact-render-preview-snapshot",
            kind: TransformArtifactBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeClipRenderResult::transform_artifact_snapshot and RuntimeOfflineRenderContractPreview::transform_artifact_snapshot",
            runtime_anchor:
                "RuntimeClipRenderResult::transform_artifact_snapshot + RuntimeOfflineRenderContractPreview::transform_artifact_snapshot",
            rationale:
                "Proves clip render and offline preview carry the same runtime-owned transform-artifact and transform-persistence posture instead of splitting render, cache, and retention truth across private paths.",
        },
        TransformArtifactBoundarySurface {
            id: "shared-host-transform-artifact-report",
            kind: TransformArtifactBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned transform-artifact and transform-persistence receipts without host-local preview-cache or cache-policy reconstruction.",
        },
    ]
}

pub(super) fn transform_artifact_boundary_validation_steps(
) -> &'static [TransformArtifactBoundaryValidationStep] {
    &[
        TransformArtifactBoundaryValidationStep {
            id: "runtime-transform-artifact-public-proof",
            command:
                "cargo test -p signal-runtime --test public_contract_boundary_transform_artifact public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned transform-artifact readiness, invalidation, reuse, retention, and cache-placement truth through public runtime surfaces alone.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "local-host-transform-artifact-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_transform_artifact local_shared_host_edge_exports_runtime_transform_artifact_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned transform-artifact and transform-persistence receipts instead of rebuilding local preview-cache or persistence posture.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "server-host-transform-artifact-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_transform_artifact server_shared_host_edge_exports_runtime_transform_artifact_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned transform-artifact and transform-persistence receipts without server-local cache heuristics.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable transform-artifact and transform-persistence boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only closure notes.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared transform-artifact and transform-persistence proof seam without reading host-local preview-cache or cache-policy glue.",
        },
    ]
}
