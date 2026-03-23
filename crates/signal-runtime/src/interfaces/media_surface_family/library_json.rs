use super::*;

fn json_runtime_media_analysis_descriptor_state(
    state: RuntimeMediaAnalysisDescriptorState,
) -> String {
    json_option_string(Some(&format!("{state:?}")))
}

fn json_runtime_media_analysis_family_state(state: RuntimeMediaAnalysisFamilyState) -> String {
    json_option_string(Some(&format!("{state:?}")))
}

fn json_runtime_media_loudness_descriptor(descriptor: &RuntimeMediaLoudnessDescriptor) -> String {
    format!(
        concat!(
            "{{",
            "\"integrated_lufs\":{},",
            "\"loudness_range_lu\":{},",
            "\"true_peak_dbtp\":{},",
            "\"target_offset_lu\":{},",
            "\"peak_to_loudness_lu\":{},",
            "\"confidence\":{},",
            "\"summary\":{}",
            "}}"
        ),
        descriptor.integrated_lufs,
        descriptor.loudness_range_lu,
        descriptor.true_peak_dbtp,
        descriptor.target_offset_lu,
        descriptor.peak_to_loudness_lu,
        descriptor.confidence,
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_media_character_descriptor(descriptor: &RuntimeMediaCharacterDescriptor) -> String {
    format!(
        concat!(
            "{{",
            "\"centroid_hz\":{},",
            "\"rolloff_95_hz\":{},",
            "\"flatness\":{},",
            "\"contrast_db\":{},",
            "\"onset_density\":{},",
            "\"transient_density\":{},",
            "\"sustain_ratio\":{},",
            "\"rms_energy\":{},",
            "\"dynamic_range\":{},",
            "\"confidence\":{},",
            "\"summary\":{}",
            "}}"
        ),
        descriptor.centroid_hz,
        descriptor.rolloff_95_hz,
        descriptor.flatness,
        descriptor.contrast_db,
        descriptor.onset_density,
        descriptor.transient_density,
        descriptor.sustain_ratio,
        descriptor.rms_energy,
        descriptor.dynamic_range,
        descriptor.confidence,
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_media_library_asset_descriptor(
    descriptor: &RuntimeMediaLibraryAssetDescriptor,
) -> String {
    format!(
        concat!(
            "{{",
            "\"asset_id\":{},",
            "\"content_hash\":{},",
            "\"file_name\":{},",
            "\"asset_state\":{},",
            "\"metadata_state\":{},",
            "\"loudness_state\":{},",
            "\"character_state\":{},",
            "\"rhythm_state\":{},",
            "\"tonal_state\":{},",
            "\"embedding_state\":{},",
            "\"loudness\":{},",
            "\"character\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(descriptor.asset_id.as_str())),
        json_option_string(Some(descriptor.content_hash.as_str())),
        json_option_string(Some(descriptor.file_name.as_str())),
        json_option_string(
            descriptor
                .asset_state
                .map(|state| format!("{state:?}"))
                .as_deref()
        ),
        json_runtime_media_analysis_descriptor_state(descriptor.metadata_state),
        json_runtime_media_analysis_family_state(descriptor.loudness_state),
        json_runtime_media_analysis_family_state(descriptor.character_state),
        json_runtime_media_analysis_family_state(descriptor.rhythm_state),
        json_runtime_media_analysis_family_state(descriptor.tonal_state),
        json_runtime_media_analysis_family_state(descriptor.embedding_state),
        descriptor
            .loudness
            .as_ref()
            .map(json_runtime_media_loudness_descriptor)
            .unwrap_or_else(|| "null".into()),
        descriptor
            .character
            .as_ref()
            .map(json_runtime_media_character_descriptor)
            .unwrap_or_else(|| "null".into()),
        json_option_string(descriptor.last_error.as_deref()),
        json_option_string(Some(descriptor.summary.as_str())),
    )
}

fn json_runtime_media_library_asset_descriptor_vec(
    descriptors: &[RuntimeMediaLibraryAssetDescriptor],
) -> String {
    let entries = descriptors
        .iter()
        .map(json_runtime_media_library_asset_descriptor)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{entries}]")
}

pub(crate) fn json_runtime_media_library_service_snapshot(
    snapshot: &RuntimeMediaLibraryServiceSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"indexed_asset_count\":{},",
            "\"ready_descriptor_count\":{},",
            "\"pending_descriptor_count\":{},",
            "\"invalidated_descriptor_count\":{},",
            "\"unavailable_descriptor_count\":{},",
            "\"loudness_ready_descriptor_count\":{},",
            "\"character_ready_descriptor_count\":{},",
            "\"rhythm_deferred_descriptor_count\":{},",
            "\"tonal_deferred_descriptor_count\":{},",
            "\"embedding_deferred_descriptor_count\":{},",
            "\"descriptors\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.indexed_asset_count,
        snapshot.ready_descriptor_count,
        snapshot.pending_descriptor_count,
        snapshot.invalidated_descriptor_count,
        snapshot.unavailable_descriptor_count,
        snapshot.loudness_ready_descriptor_count,
        snapshot.character_ready_descriptor_count,
        snapshot.rhythm_deferred_descriptor_count,
        snapshot.tonal_deferred_descriptor_count,
        snapshot.embedding_deferred_descriptor_count,
        json_runtime_media_library_asset_descriptor_vec(&snapshot.descriptors),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
