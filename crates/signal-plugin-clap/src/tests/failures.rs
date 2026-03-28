use super::*;

#[test]
fn sandbox_failure_event_exposes_timeout_kind() {
    let failure = sandbox_failure_event(ClapSandboxFailureInput {
        sandbox_id: "sandbox-a".to_string(),
        instance_id: Some("instance-a".into()),
        stage: "processBlock".to_string(),
        error_kind: "timeout".to_string(),
        detail: "sandbox exceeded block deadline".to_string(),
        processing_epoch: Some(3),
        shared_memory_lease_id: Some("lease-a".into()),
        correlation_id: None,
        instance_state: None,
    });

    match failure.payload {
        PluginMessagePayload::SandboxFailure { error_kind, .. } => {
            assert_eq!(error_kind, "timeout");
        }
        other => panic!("expected sandbox failure, got {other:?}"),
    }
}

#[test]
fn classify_sandbox_failure_maps_process_attach_errors() {
    let failure = sandbox_failure_event(ClapSandboxFailureInput {
        sandbox_id: "sandbox-a".to_string(),
        instance_id: Some("instance-a".into()),
        stage: "processBlock".to_string(),
        error_kind: "resourceUnavailable".to_string(),
        detail: "failed to attach shared-memory region: stale mapping".to_string(),
        processing_epoch: Some(3),
        shared_memory_lease_id: Some("lease-a".into()),
        correlation_id: None,
        instance_state: None,
    });

    let classification = classify_sandbox_failure(&failure).expect("classification");
    assert_eq!(classification.stage, ClapSandboxFailureStage::ProcessAttach);
    assert_eq!(classification.operation, "processBlock");
    assert_eq!(classification.lease_id.as_deref(), Some("lease-a"));
}
