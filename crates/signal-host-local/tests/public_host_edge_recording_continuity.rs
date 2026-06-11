#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_continuity_support::apply_public_capture_graph;
use signal_graph::synthetic_stereo_block;
use signal_host_local::LocalRuntimeHost;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeRecordingCaptureKind, RuntimeRecordingCaptureStartRequest, SafeModeRequest,
    SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_resumable_recording_checkpoint_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-recording".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_capture_graph(&mut runtime, "graph:host-local:recording");
    runtime.start().unwrap();
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:local:resumable".into(),
            track_id: "track:local:resumable".into(),
            start_samples: 2_048,
            capture_path: std::env::temp_dir()
                .join("signal-local-host-recording-resumable.wav")
                .display()
                .to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
        )
        .unwrap();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .recording_capture_snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );
}
