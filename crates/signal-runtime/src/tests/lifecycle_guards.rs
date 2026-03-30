use super::*;

#[test]
fn configure_requires_prior_handshake() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let error = runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap_err();

    assert_eq!(
        error.kind,
        crate::interfaces::RuntimeErrorKind::InvalidState
    );
}

#[test]
fn start_requires_prior_configuration() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    let error = runtime.start().unwrap_err();

    assert_eq!(
        error.kind,
        crate::interfaces::RuntimeErrorKind::InvalidState
    );
}

#[test]
fn control_snapshot_tracks_handshake_configure_and_restart_history() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();
    runtime.start().unwrap();
    runtime
        .restart(RestartRequest {
            reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
        })
        .unwrap();

    let control = runtime.get_control_snapshot();
    assert!(control.handshaken);
    assert!(control.configured);
    assert!(control.running);
    assert_eq!(control.handshake_count, 1);
    assert_eq!(control.configure_count, 2);
    assert_eq!(control.start_count, 2);
    assert_eq!(control.stop_count, 1);
    assert_eq!(control.restart_count, 1);
    assert_eq!(control.last_client_version.as_deref(), Some("runtime-test"));
    assert_eq!(
        control.last_stop_reason,
        Some(StopReason::DeviceReconfigure)
    );
    assert_eq!(
        control
            .last_reconfigure
            .map(|request| request.sample_rate.0),
        Some(44_100)
    );
}
