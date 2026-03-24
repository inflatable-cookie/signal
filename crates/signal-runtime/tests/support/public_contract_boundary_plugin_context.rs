use signal_runtime::{
    RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass,
};

pub fn sample_public_preset_descriptor() -> RuntimePluginPresetDescriptor {
    RuntimePluginPresetDescriptor {
        preset_id: Some("preset:factory:init".into()),
        label: Some("Init".into()),
        origin: RuntimePluginPresetOrigin::Factory,
        summary: "public runtime preset descriptor".into(),
    }
}

pub fn sample_public_ara_context(
    portability_class: RuntimePluginRecallPortabilityClass,
    document_id: &str,
    source_id: &str,
    region_id: &str,
    timeline_start_samples: i64,
    duration_samples: u32,
) -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: document_id.into(),
            display_label: Some("Session".into()),
            summary: "public runtime ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: source_id.into(),
            display_label: Some("Lead Vocal".into()),
            summary: "public runtime ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: region_id.into(),
            display_label: Some("Verse".into()),
            timeline_start_samples: Some(timeline_start_samples),
            duration_samples: Some(duration_samples),
            summary: "public runtime ara region".into(),
        }),
        summary: "public runtime ara context".into(),
    }
}
