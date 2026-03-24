use signal_runtime::{
    RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext,
    RuntimePluginRecallPortabilityClass,
};

pub fn sample_server_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class: RuntimePluginRecallPortabilityClass::ContextOnly,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: "doc:host-server".into(),
            display_label: Some("Server Session".into()),
            summary: "server host ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: "source:stem-bus".into(),
            display_label: Some("Stem Bus".into()),
            summary: "server host ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: "region:bridge".into(),
            display_label: Some("Bridge".into()),
            timeline_start_samples: Some(16_384),
            duration_samples: Some(4_096),
            summary: "server host ara region".into(),
        }),
        summary: "server host ara context".into(),
    }
}
