use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdvancedHardwareBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

struct AdvancedHardwareBoundarySurface {
    id: &'static str,
    kind: AdvancedHardwareBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

struct AdvancedHardwareBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl AdvancedHardwareBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn advanced_hardware_boundary_surfaces() -> &'static [AdvancedHardwareBoundarySurface] {
    &[
        AdvancedHardwareBoundarySurface {
            id: "runtime-advanced-hardware-report",
            kind: AdvancedHardwareBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::advanced_hardware_snapshot and RuntimeSupervisorReport::observation.advanced_hardware_snapshot",
            runtime_anchor:
                "RuntimeAdvancedHardwareSnapshot::{display_transport_device_count,motor_transport_device_count,haptic_transport_device_count,scene_mapping_device_count,feedback_page_device_count,safe_action_graph_device_count} + RuntimeAdvancedHardwareDeviceDescriptor::{display_transport_posture,display_content_class,motor_transport_posture,haptic_transport_posture,feedback_authority,feedback_outcome,scene_mapping_posture,feedback_page_posture,feedback_page_class,safe_action_graph_posture,action_authority,safe_action_outcome}",
            rationale:
                "Keeps advanced-hardware graph state, scripting-safe device policy posture, guarded feedback-channel posture, typed display or motor or haptic transport posture, bounded scene or page workflow posture, and safe-action outcome on one runtime-owned report seam instead of host-local hardware or controller-workflow policy.",
        },
        AdvancedHardwareBoundarySurface {
            id: "runtime-advanced-hardware-control-surface-anchor",
            kind: AdvancedHardwareBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot",
            runtime_anchor: "RuntimeControlSurfaceSnapshot",
            rationale:
                "Keeps the widened advanced control-feedback baseline explicitly derived from the closed control-surface substrate instead of creating a second hardware shell.",
        },
        AdvancedHardwareBoundarySurface {
            id: "shared-host-advanced-hardware-report",
            kind: AdvancedHardwareBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward the same runtime-owned advanced control-feedback baseline without host-local hardware or controller-policy reconstruction.",
        },
    ]
}

fn advanced_hardware_boundary_validation_steps() -> &'static [AdvancedHardwareBoundaryValidationStep]
{
    &[
        AdvancedHardwareBoundaryValidationStep {
            id: "runtime-advanced-hardware-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect advanced-hardware graph state, scripting-safe policy posture, typed display or motor or haptic transport posture, bounded scene or page workflow posture, and safe-action outcomes through public runtime surfaces.",
        },
        AdvancedHardwareBoundaryValidationStep {
            id: "local-host-advanced-hardware-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_advanced_hardware_truth",
            rationale:
                "Proves the stable local host edge forwards the runtime-owned advanced control-feedback and controller-workflow baseline instead of rebuilding local hardware or workflow policy.",
        },
        AdvancedHardwareBoundaryValidationStep {
            id: "server-host-advanced-hardware-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_advanced_hardware_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned advanced control-feedback and controller-workflow baseline instead of inventing server-local hardware or workflow policy.",
        },
        AdvancedHardwareBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools advanced_hardware_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable advanced control-feedback and controller-workflow boundary aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        AdvancedHardwareBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared advanced control-feedback and controller-workflow proof seam without reading host-local hardware policy or device-private workflow implementation detail.",
        },
    ]
}

pub(crate) fn render_advanced_hardware_boundary_text() -> String {
    let mut rendered = format!(
        "advanced_hardware_boundary: {ADVANCED_HARDWARE_BOUNDARY}\ncontract_path: {ADVANCED_HARDWARE_CONTRACT_PATH}\nacceptance_task: {ADVANCED_HARDWARE_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in advanced_hardware_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in advanced_hardware_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves a bounded advanced control-feedback and controller-workflow baseline, not richer vendor protocol payloads, page-aware display depth beyond the bounded classes, real motor transport, real haptic transport, or executable scripting depth",
        "the current seam keeps runtime-owned scripting-safe policy, guarded feedback posture, typed display or motor or haptic transport posture, bounded scene or page workflow posture, and safe-action outcomes consumable through runtime and stable host edges, but fuller device execution workflows remain later work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_advanced_hardware_boundary_json() -> String {
    let surfaces = advanced_hardware_boundary_surfaces()
        .iter()
        .map(|surface| {
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
                json_string(surface.kind.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = advanced_hardware_boundary_validation_steps()
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
        "the shared boundary now proves a bounded advanced control-feedback and controller-workflow baseline, not richer vendor protocol payloads, page-aware display depth beyond the bounded classes, real motor transport, real haptic transport, or executable scripting depth",
        "the current seam keeps runtime-owned scripting-safe policy, guarded feedback posture, typed display or motor or haptic transport posture, bounded scene or page workflow posture, and safe-action outcomes consumable through runtime and stable host edges, but fuller device execution workflows remain later work",
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
        json_string(ADVANCED_HARDWARE_BOUNDARY),
        json_string(ADVANCED_HARDWARE_CONTRACT_PATH),
        json_string(ADVANCED_HARDWARE_ACCEPTANCE_TASK),
        advanced_hardware_boundary_surfaces().len(),
        surfaces,
        advanced_hardware_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
