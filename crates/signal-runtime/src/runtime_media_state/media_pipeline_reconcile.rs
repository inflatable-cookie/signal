use super::*;
use crate::runtime::runtime_utils::sanitize_asset_id;

impl RuntimeMediaPipelineStateModel {
    pub(crate) fn start_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError> {
        let asset = self.assets.get(asset_id).ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!("media asset not indexed for preview: {asset_id}"),
            )
        })?;
        if asset.state != RuntimeMediaAssetState::Ready {
            let message = format!(
                "media asset not ready for preview: {asset_id} state={:?}",
                asset.state
            );
            self.previewing_asset_id = None;
            self.last_preview_error = Some(message.clone());
            return Err(RuntimeError::new(RuntimeErrorKind::InvalidState, message));
        }
        self.previewing_asset_id = Some(asset_id.to_string());
        self.last_preview_error = None;
        Ok(())
    }

    pub(crate) fn stop_preview(&mut self) {
        self.previewing_asset_id = None;
        self.last_preview_error = None;
    }

    pub(crate) fn reconcile_preview_state(&mut self) {
        if let Some(asset_id) = self.previewing_asset_id.clone() {
            match self.assets.get(&asset_id) {
                Some(asset) if asset.state == RuntimeMediaAssetState::Ready => {}
                Some(asset) => {
                    self.previewing_asset_id = None;
                    self.last_preview_error = Some(format!(
                        "preview invalidated for asset {} state={:?}",
                        asset.registration.asset_id, asset.state
                    ));
                }
                None => {
                    self.previewing_asset_id = None;
                    self.last_preview_error = Some(format!(
                        "preview asset removed from runtime index: {asset_id}"
                    ));
                }
            }
        }
    }

    pub(crate) fn reconcile_assets(
        &mut self,
        registrations: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        fs::create_dir_all(&self.policy.cache_root).map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "failed to create media cache root {}: {error}",
                    self.policy.cache_root.display()
                ),
            )
        })?;

        let retained_ids = registrations
            .iter()
            .map(|asset| asset.asset_id.clone())
            .collect::<BTreeSet<_>>();
        self.assets
            .retain(|asset_id, _| retained_ids.contains(asset_id));

        for registration in registrations {
            let cache_path = self.cache_path_for(&registration);
            let cache_exists = cache_path.is_file();
            let rebuild = self
                .assets
                .get(&registration.asset_id)
                .map(|existing| {
                    existing.registration.content_hash != registration.content_hash
                        || existing.registration.source_path != registration.source_path
                        || !cache_exists
                })
                .unwrap_or(false);
            let mut asset =
                self.assets
                    .remove(&registration.asset_id)
                    .unwrap_or(RuntimeMediaPipelineAsset {
                        registration: registration.clone(),
                        state: RuntimeMediaAssetState::Ingesting,
                        cache_path: None,
                        cache_byte_size: None,
                        rebuild_count: 0,
                        last_error: None,
                        analysis: RuntimeMediaAnalysisStateModel::default(),
                    });
            asset.registration = registration;
            if rebuild {
                asset.rebuild_count = asset.rebuild_count.saturating_add(1);
                asset.state = RuntimeMediaAssetState::Rebuilding;
            } else if asset.cache_path.is_none() {
                asset.state = RuntimeMediaAssetState::Ingesting;
            }
            self.materialize_asset(&mut asset, &cache_path);
            self.assets
                .insert(asset.registration.asset_id.clone(), asset);
        }

        self.reconcile_preview_state();

        Ok(())
    }

    fn materialize_asset(&self, asset: &mut RuntimeMediaPipelineAsset, cache_path: &Path) {
        if asset.registration.source_path.trim().is_empty() {
            asset.state = RuntimeMediaAssetState::Invalid;
            asset.cache_path = None;
            asset.cache_byte_size = None;
            let message = "source path must not be empty".to_string();
            asset.last_error = Some(message.clone());
            asset.analysis.descriptor_state = RuntimeMediaAnalysisDescriptorState::Invalidated;
            asset.analysis.last_error = Some(message);
            return;
        }
        let source_path = Path::new(&asset.registration.source_path);
        if !source_path.is_file() {
            asset.state = RuntimeMediaAssetState::Invalid;
            asset.cache_path = None;
            asset.cache_byte_size = None;
            let message = format!("source media missing at {}", source_path.display());
            asset.last_error = Some(message.clone());
            asset.analysis.descriptor_state = RuntimeMediaAnalysisDescriptorState::Invalidated;
            asset.analysis.last_error = Some(message);
            return;
        }
        asset.state = if asset.rebuild_count > 0 {
            RuntimeMediaAssetState::Rebuilding
        } else {
            RuntimeMediaAssetState::Ingesting
        };
        asset.analysis.descriptor_state = RuntimeMediaAnalysisDescriptorState::Pending;
        asset.analysis.last_error = None;
        asset.state = RuntimeMediaAssetState::Conforming;
        match fs::copy(source_path, cache_path) {
            Ok(_) => match fs::metadata(cache_path) {
                Ok(metadata) => {
                    asset.state = RuntimeMediaAssetState::Ready;
                    asset.cache_path = Some(cache_path.display().to_string());
                    asset.cache_byte_size = Some(metadata.len());
                    asset.last_error = None;
                    match analyze_runtime_media_asset(cache_path, &asset.registration) {
                        Ok(analysis) => asset.analysis = analysis,
                        Err(error) => {
                            asset.analysis = RuntimeMediaAnalysisStateModel {
                                descriptor_state: RuntimeMediaAnalysisDescriptorState::Unavailable,
                                loudness: None,
                                character: None,
                                last_error: Some(error),
                            };
                        }
                    }
                }
                Err(error) => {
                    asset.state = RuntimeMediaAssetState::Invalid;
                    asset.cache_path = None;
                    asset.cache_byte_size = None;
                    let message =
                        format!("cached media written but metadata lookup failed: {error}");
                    asset.last_error = Some(message.clone());
                    asset.analysis.descriptor_state =
                        RuntimeMediaAnalysisDescriptorState::Invalidated;
                    asset.analysis.last_error = Some(message);
                }
            },
            Err(error) => {
                asset.state = RuntimeMediaAssetState::Invalid;
                asset.cache_path = None;
                asset.cache_byte_size = None;
                let message = format!("cache conform failed: {error}");
                asset.last_error = Some(message.clone());
                asset.analysis.descriptor_state = RuntimeMediaAnalysisDescriptorState::Invalidated;
                asset.analysis.last_error = Some(message);
            }
        }
    }

    pub(crate) fn cache_path_for(&self, registration: &RuntimeMediaAssetRegistration) -> PathBuf {
        let extension = Path::new(&registration.file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("wav");
        self.policy.cache_root.join(format!(
            "{}-{}.{}",
            sanitize_asset_id(&registration.asset_id),
            registration.content_hash,
            extension
        ))
    }
}

impl Default for RuntimeMediaPipelineStateModel {
    fn default() -> Self {
        Self {
            policy: RuntimeMediaPipelinePolicy::default(),
            assets: BTreeMap::new(),
            previewing_asset_id: None,
            last_preview_error: None,
        }
    }
}
