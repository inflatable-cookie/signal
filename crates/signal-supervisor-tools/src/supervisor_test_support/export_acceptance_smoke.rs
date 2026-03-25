use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_runtime::{RuntimeConfig, RuntimeEventRecorder, RuntimeSupervisorReport, SignalRuntime};

use super::{
    sample_control_preview_workflow_external_midi_snapshot, sample_g07_acceptance_host_io,
    sample_g07_external_midi_snapshot,
};

pub(crate) fn verify_export_json_carries_cross_family_device_workflow_acceptance_evidence() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_g07_acceptance_host_io())
        .with_external_midi_snapshot(sample_g07_external_midi_snapshot());
    let report = RuntimeSupervisorReport {
        observation,
        ..report
    };

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"external_midi_snapshot\":{"));
    assert!(export.contains("\"live_ownership\":{"));
    assert!(export.contains("\"backend_parity\":\""));
    assert!(export.contains("\"attach_continuity\":\""));
    assert!(export.contains("\"supports_widened_expression\":true"));
    assert!(export.contains("\"control_surface_snapshot\":{"));
    assert!(export.contains("\"graph_state\":\"Guarded\""));
    assert!(export.contains("\"advanced_hardware_snapshot\":{"));
}

pub(crate) fn verify_export_json_carries_cross_family_linux_live_acceptance_evidence() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_g07_acceptance_host_io())
        .with_external_midi_snapshot(sample_control_preview_workflow_external_midi_snapshot());
    let report = RuntimeSupervisorReport {
        observation,
        ..report
    };

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"linux_backend_session_snapshot\":{"));
    assert!(export.contains("\"jack_coordination_snapshot\":{"));
    assert!(export.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(export.contains("\"transport_posture\":\"Unavailable\""));
    assert!(export.contains("\"session_role\":\"Unavailable\""));
    assert!(export.contains("\"clock_domain\":\"SameClock\""));
    assert!(export.contains("\"linux_clocking_parity\":\"Portable\""));
}

pub(crate) fn verify_export_json_carries_cross_family_control_preview_workflow_acceptance_evidence()
{
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let observation = report
        .observation
        .clone()
        .with_host_external_io(&sample_g07_acceptance_host_io());
    let report = RuntimeSupervisorReport {
        observation,
        ..report
    };

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"control_surface_snapshot\":{"));
    assert!(export.contains("\"advanced_hardware_snapshot\":{"));
    assert!(export.contains("\"preview_transform_snapshot\":{"));
    assert!(export.contains("\"preview_device_policy\":{"));
    assert!(export.contains("\"routing_posture\":\""));
    assert!(export.contains("\"low_latency_device_policy_outcome\":\""));
    assert!(export.contains("\"preview_workflow\":{"));
    assert!(export.contains("\"queue_posture\":\""));
    assert!(export.contains("\"audition_continuity_outcome\":\""));
    assert!(export.contains("\"transform_scheduling_outcome\":\""));
}
