use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OfflineRenderContinuityBoundarySurfaceKind {
    RuntimeSnapshot,
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OfflineRenderContinuityBoundarySurface {
    id: &'static str,
    kind: OfflineRenderContinuityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OfflineRenderContinuityValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn offline_render_continuity_boundary_surfaces() -> &'static [OfflineRenderContinuityBoundarySurface]
{
    &[
        OfflineRenderContinuityBoundarySurface {
            id: "runtime-offline-render-session-snapshot",
            kind: OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::offline_render_session_snapshot and RuntimeSupervisorReport::observation.offline_render_session_snapshot",
            runtime_anchor: "RuntimeOfflineRenderSessionSnapshot",
            rationale:
                "Carries active and last render-session continuity, checkpoints, cancellation, and purge truth directly on public runtime reports.",
        },
        OfflineRenderContinuityBoundarySurface {
            id: "runtime-offline-render-observation-api",
            kind: OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_offline_render_session_snapshot()",
            runtime_anchor: "RuntimeOfflineRenderSessionSnapshot",
            rationale:
                "Keeps render continuity inspectable without forcing consumers through filesystem artifacts or supervisor-only JSON parsing.",
        },
        OfflineRenderContinuityBoundarySurface {
            id: "shared-host-offline-render-supervisor-report",
            kind: OfflineRenderContinuityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges forward resumable, restartable, and terminal render-session truth without host-local retry policy.",
        },
    ]
}

fn offline_render_continuity_validation_steps() -> &'static [OfflineRenderContinuityValidationStep]
{
    &[
        OfflineRenderContinuityValidationStep {
            id: "runtime-resumable-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_preserves_checkpoint_through_pause_and_recoverable_states",
            rationale:
                "Proves checkpoints survive pause and recoverable interruption under the same runtime-owned render-session identity.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-restartable-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_reports_restartable_state_across_stop_restart_and_resume",
            rationale:
                "Proves runtime stop and restart preserve render-session continuity as a restartable path instead of silently dropping active work.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-terminal-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_reports_failed_terminal_state_on_delivery_error",
            rationale:
                "Proves failed render delivery is exported as typed terminal session state rather than disappearing into a raw I/O error.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-public-boundary-proof",
            command:
                "cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states",
            rationale:
                "Proves a downstream-style runtime consumer can distinguish all three render continuity outcomes through public reexports.",
        },
        OfflineRenderContinuityValidationStep {
            id: "local-host-render-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_resumable_offline_render_session_truth",
            rationale:
                "Proves the local shared host edge preserves resumable render-session truth on supervisor export.",
        },
        OfflineRenderContinuityValidationStep {
            id: "server-host-render-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth",
            rationale:
                "Proves the server shared host edge preserves restartable and terminal render-session truth on supervisor export.",
        },
        OfflineRenderContinuityValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-offline-render-continuity-boundary --format=json",
            rationale:
                "Lets consumers inspect the render continuity proof boundary without reading private runtime or host implementation detail.",
        },
    ]
}

pub(crate) fn render_offline_render_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "offline_render_continuity_boundary: {OFFLINE_RENDER_CONTINUITY_BOUNDARY}\ncontract_path: {OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in offline_render_continuity_boundary_surfaces() {
        let kind = match surface.kind {
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
            OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
        };
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, kind, surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in offline_render_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_offline_render_continuity_boundary_json() -> String {
    let surfaces = offline_render_continuity_boundary_surfaces()
        .iter()
        .map(|surface| {
            let kind = match surface.kind {
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
                OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
            };
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(kind),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = offline_render_continuity_validation_steps()
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
                json_string(step.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"surface_count\":{},",
            "\"surfaces\":[{}],",
            "\"validation_step_count\":{},",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(OFFLINE_RENDER_CONTINUITY_BOUNDARY),
        json_string(OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH),
        json_string(OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK),
        offline_render_continuity_boundary_surfaces().len(),
        surfaces,
        offline_render_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
