use signal_runtime::{
    RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass,
};

pub fn sample_host_preset_descriptor() -> RuntimePluginPresetDescriptor {
    RuntimePluginPresetDescriptor {
        preset_id: Some("preset:user:local-lead".into()),
        label: Some("Local Lead".into()),
        origin: RuntimePluginPresetOrigin::User,
        summary: "local host preset descriptor".into(),
    }
}

pub fn sample_host_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class: RuntimePluginRecallPortabilityClass::ContextOnly,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: "doc:host-local".into(),
            display_label: Some("Song".into()),
            summary: "local host ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: "source:take-01".into(),
            display_label: Some("Take 01".into()),
            summary: "local host ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: "region:chorus".into(),
            display_label: Some("Chorus".into()),
            timeline_start_samples: Some(2_048),
            duration_samples: Some(8_192),
            summary: "local host ara region".into(),
        }),
        summary: "local host ara context".into(),
    }
}
