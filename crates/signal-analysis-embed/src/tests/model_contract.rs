use super::*;

#[test]
fn built_in_model_spec_is_explicit() {
    let embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
    let spec = embedder.model_spec();

    assert_eq!(spec.model_id, BUILTIN_DESCRIPTOR_MODEL_ID);
    assert_eq!(spec.version, SemanticModelVersion::new(1, 0, 0));
    assert_eq!(spec.resources.embedding_dimensions, EMBEDDING_DIMENSIONS);
    assert!(spec.resources.deterministic);
    assert!(!spec.resources.requires_network);
}

#[test]
fn unknown_model_fails_closed_when_requested() {
    let error = SemanticEmbedder::new(SemanticEmbedderConfig {
        requested_model_id: Some("signal:missing-model".to_string()),
        fallback_behavior: ModelFallbackBehavior::FailClosed,
        ..SemanticEmbedderConfig::default()
    })
    .unwrap_err();

    assert_eq!(error.requested_model_id, "signal:missing-model");
    assert_eq!(error.fallback_behavior, ModelFallbackBehavior::FailClosed);
}

#[test]
fn unknown_model_can_fallback_to_builtin() {
    let embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
        requested_model_id: Some("signal:missing-model".to_string()),
        fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
        ..SemanticEmbedderConfig::default()
    })
    .unwrap();

    assert_eq!(embedder.model_spec().model_id, BUILTIN_DESCRIPTOR_MODEL_ID);
}
