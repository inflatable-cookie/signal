//! Runtime configuration and shell implementation for Signal.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};
use signal_graph::{
    synthetic_stereo_block, ExecutableGraph, GraphBlockReport, GraphConfig, GraphExecutionContext,
    GraphNodeBufferContract, GraphNodeExecutionClass, GraphNodeRenderOverride, GraphNodeSpec,
    GraphNodeTopologyMetadata, GraphNodeTopologyRole, GraphPreparedDispatch,
};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    AutomationContinuityReport, BlockSequenceContinuityReport, CompletionState,
    ParameterAutomationSummary,
};
use signal_primitives::{AudioBuffer, FrameCount, SampleRate};

use crate::interfaces::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    DegradedReason, EffectiveRuntimeConfig, GraphContractProjection, GraphProjection,
    HandshakeRequest, HandshakeResponse, HeartbeatCycleStage, LeaseRolloverRecord,
    LingeringCleanupMode, LingeringCleanupQueueReceipt, LingeringCleanupTrigger, ParameterBatch,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginNodeRenderBatch,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxTransportStage,
    ProjectionReceipt, RecoveryRestartIntent, RestartRequest, RuntimeAutomationSnapshot,
    RuntimeConfigRequest, RuntimeControlSnapshot, RuntimeDiagnosticsSnapshot,
    RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventSink, RuntimeExecutionPhase, RuntimeLifecycleApi,
    RuntimeMediaAssetRegistration, RuntimeMediaAssetSnapshot, RuntimeMediaAssetState,
    RuntimeMediaPipelineSnapshot, RuntimeObservationApi, RuntimePluginDispatchState,
    RuntimePreworkBacklogClass, RuntimePreworkCacheState, RuntimePreworkForecastMode,
    RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureSnapshot,
    RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState, RuntimeSchedulerSnapshot,
    RuntimeSchedulerState, RuntimeSchedulerTopologyIssue, RuntimeSchedulerTopologySummary,
    RuntimeSupervisionSnapshot, RuntimeTimelineSnapshot, RuntimeTransportConcurrencySnapshot,
    RuntimeTransportObservationSnapshot, RuntimeTransportTransitionKind, RuntimeWarpClipRegistration,
    RuntimeWarpClipSnapshot, RuntimeWarpMode, RuntimeWarpPipelineSnapshot,
    RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest,
    SandboxOperationFailureStage, ScheduleProjection, StopReason, SubscriptionHandle,
    TransportAttachIntent, TransportProjection, TransportSessionProvenance,
    TransportSessionState, WatchdogRestartRecord,
};

const PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub sample_rate: SampleRate,
    pub graph: GraphConfig,
    pub profile: RuntimeProfile,
}

impl RuntimeConfig {
    pub fn local(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Local,
        }
    }

    pub fn server(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Server,
        }
    }
}

pub struct SignalRuntime {
    config: RuntimeConfig,
    readiness: RuntimeReadiness,
    safe_mode_enabled: bool,
    anticipative_enabled: bool,
    active_output_device: Option<String>,
    applied_graph: Option<GraphProjection>,
    applied_schedule: Option<ScheduleProjection>,
    applied_transport: Option<TransportProjection>,
    prework_forecast_requested_mode: RuntimePreworkForecastMode,
    prework_forecast_mode: RuntimePreworkForecastMode,
    prework_forecast_policy: Option<RuntimePreworkForecastPolicy>,
    prework_forecast_profile: Option<RuntimePreworkForecastProfileSelection>,
    prework_forecast_profile_source: Option<RuntimePreworkForecastProfileSource>,
    latest_parameter_epoch: u64,
    projection_epoch: u64,
    control: RuntimeControlSnapshot,
    timeline: RuntimeTimelineState,
    automation: RuntimeAutomationState,
    engine: RuntimeEngineState,
    transport_concurrency: RuntimeTransportConcurrencyState,
    recording_capture: RuntimeRecordingCaptureStateModel,
    media_pipeline: RuntimeMediaPipelineStateModel,
    warp_pipeline: RuntimeWarpPipelineStateModel,
    diagnostics: RuntimeDiagnosticsSnapshot,
    supervision: RuntimeSupervisionState,
    next_subscription: u64,
    sinks: Vec<Box<dyn RuntimeEventSink>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeSupervisionPolicy {
    safe_mode_restart_threshold: u32,
}

impl Default for RuntimeSupervisionPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeSupervisionState {
    policy: RuntimeSupervisionPolicy,
    watchdog_restart_count: u32,
    last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    last_sandbox_id: Option<String>,
    last_processing_epoch: Option<u64>,
}

impl RuntimeSupervisionState {
    fn snapshot(&self, safe_mode_enabled: bool) -> RuntimeSupervisionSnapshot {
        RuntimeSupervisionSnapshot {
            watchdog_restart_count: self.watchdog_restart_count,
            safe_mode_enabled,
            last_watchdog_trigger: self.last_watchdog_trigger,
            last_sandbox_id: self.last_sandbox_id.clone(),
            last_processing_epoch: self.last_processing_epoch,
        }
    }

    fn record_watchdog_restart(&mut self, record: WatchdogRestartRecord) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        self.last_watchdog_trigger = Some(record.trigger);
        self.last_sandbox_id = Some(record.sandbox_id);
        self.last_processing_epoch = Some(record.processing_epoch);
        self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold
    }
}

impl Default for RuntimeSupervisionState {
    fn default() -> Self {
        Self {
            policy: RuntimeSupervisionPolicy::default(),
            watchdog_restart_count: 0,
            last_watchdog_trigger: None,
            last_sandbox_id: None,
            last_processing_epoch: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeRecordingCapturePolicy {
    pressure_threshold_frames: u64,
}

impl Default for RuntimeRecordingCapturePolicy {
    fn default() -> Self {
        Self {
            pressure_threshold_frames: 16_384,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeMediaPipelinePolicy {
    cache_root: PathBuf,
}

impl Default for RuntimeMediaPipelinePolicy {
    fn default() -> Self {
        Self {
            cache_root: std::env::temp_dir().join("loophole-signal-media-cache"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeMediaPipelineAsset {
    registration: RuntimeMediaAssetRegistration,
    state: RuntimeMediaAssetState,
    cache_path: Option<String>,
    cache_byte_size: Option<u64>,
    rebuild_count: u32,
    last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeMediaPipelineStateModel {
    policy: RuntimeMediaPipelinePolicy,
    assets: BTreeMap<String, RuntimeMediaPipelineAsset>,
}

impl RuntimeMediaPipelineStateModel {
    fn snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        let assets = self
            .assets
            .values()
            .map(|asset| RuntimeMediaAssetSnapshot {
                asset_id: asset.registration.asset_id.clone(),
                content_hash: asset.registration.content_hash.clone(),
                source_path: asset.registration.source_path.clone(),
                file_name: asset.registration.file_name.clone(),
                byte_size: asset.registration.byte_size,
                sample_rate_hz: asset.registration.sample_rate_hz,
                channel_count: asset.registration.channel_count,
                duration_samples: asset.registration.duration_samples,
                waveform_bin_count: asset.registration.waveform_bin_count,
                state: Some(asset.state),
                cache_path: asset.cache_path.clone(),
                cache_byte_size: asset.cache_byte_size,
                rebuild_count: asset.rebuild_count,
                last_error: asset.last_error.clone(),
                summary: format!(
                    "state={:?} cache={} rebuilds={} error={}",
                    asset.state,
                    asset.cache_path.as_deref().unwrap_or("none"),
                    asset.rebuild_count,
                    asset.last_error.as_deref().unwrap_or("none"),
                ),
            })
            .collect::<Vec<_>>();
        let ready_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Ready))
            .count();
        let invalid_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Invalid))
            .count();
        let ingesting_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Ingesting))
            .count();
        let conforming_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Conforming))
            .count();
        let rebuilding_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Rebuilding))
            .count();

        RuntimeMediaPipelineSnapshot {
            cache_root_path: self.policy.cache_root.display().to_string(),
            asset_count: assets.len(),
            ready_asset_count,
            invalid_asset_count,
            ingesting_asset_count,
            conforming_asset_count,
            rebuilding_asset_count,
            assets,
            summary: format!(
                "assets={} ready={} invalid={} rebuilding={} cache_root={}",
                self.assets.len(),
                ready_asset_count,
                invalid_asset_count,
                rebuilding_asset_count,
                self.policy.cache_root.display(),
            ),
        }
    }

    fn reconcile_assets(
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

        Ok(())
    }

    fn materialize_asset(&self, asset: &mut RuntimeMediaPipelineAsset, cache_path: &Path) {
        if asset.registration.source_path.trim().is_empty() {
            asset.state = RuntimeMediaAssetState::Invalid;
            asset.cache_path = None;
            asset.cache_byte_size = None;
            asset.last_error = Some("source path must not be empty".to_string());
            return;
        }
        let source_path = Path::new(&asset.registration.source_path);
        if !source_path.is_file() {
            asset.state = RuntimeMediaAssetState::Invalid;
            asset.cache_path = None;
            asset.cache_byte_size = None;
            asset.last_error = Some(format!("source media missing at {}", source_path.display()));
            return;
        }
        asset.state = if asset.rebuild_count > 0 {
            RuntimeMediaAssetState::Rebuilding
        } else {
            RuntimeMediaAssetState::Ingesting
        };
        asset.state = RuntimeMediaAssetState::Conforming;
        match fs::copy(source_path, cache_path) {
            Ok(_) => match fs::metadata(cache_path) {
                Ok(metadata) => {
                    asset.state = RuntimeMediaAssetState::Ready;
                    asset.cache_path = Some(cache_path.display().to_string());
                    asset.cache_byte_size = Some(metadata.len());
                    asset.last_error = None;
                }
                Err(error) => {
                    asset.state = RuntimeMediaAssetState::Invalid;
                    asset.cache_path = None;
                    asset.cache_byte_size = None;
                    asset.last_error = Some(format!(
                        "cached media written but metadata lookup failed: {error}"
                    ));
                }
            },
            Err(error) => {
                asset.state = RuntimeMediaAssetState::Invalid;
                asset.cache_path = None;
                asset.cache_byte_size = None;
                asset.last_error = Some(format!("cache conform failed: {error}"));
            }
        }
    }

    fn cache_path_for(&self, registration: &RuntimeMediaAssetRegistration) -> PathBuf {
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
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeWarpPipelineStateModel {
    clips: BTreeMap<String, RuntimeWarpClipRegistration>,
}

impl RuntimeWarpPipelineStateModel {
    fn snapshot(
        &self,
        project_tempo_bpm: f64,
        media_pipeline: &RuntimeMediaPipelineStateModel,
    ) -> RuntimeWarpPipelineSnapshot {
        let project_tempo_bpm = if project_tempo_bpm.is_finite() && project_tempo_bpm > 0.0 {
            project_tempo_bpm
        } else {
            120.0
        };
        let clips = self
            .clips
            .values()
            .map(|registration| {
                self.snapshot_clip(registration, project_tempo_bpm, &media_pipeline.assets)
            })
            .collect::<Vec<_>>();
        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Ready)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Degraded)
            .count();
        let bypassed_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Bypassed)
            .count();
        let active_warp_count = clips
            .iter()
            .filter(|clip| clip.mode != RuntimeWarpMode::Off)
            .count();

        RuntimeWarpPipelineSnapshot {
            clip_count: clips.len(),
            ready_clip_count,
            degraded_clip_count,
            bypassed_clip_count,
            active_warp_count,
            clips,
            summary: format!(
                "warp clips={} active={} ready={} degraded={} bypassed={} project_tempo={project_tempo_bpm:.2}",
                self.clips.len(),
                active_warp_count,
                ready_clip_count,
                degraded_clip_count,
                bypassed_clip_count,
            ),
        }
    }

    fn snapshot_clip(
        &self,
        registration: &RuntimeWarpClipRegistration,
        project_tempo_bpm: f64,
        media_assets: &BTreeMap<String, RuntimeMediaPipelineAsset>,
    ) -> RuntimeWarpClipSnapshot {
        let mut realized_ratio = 1.0;
        let (readiness, last_error) = match registration.mode {
            RuntimeWarpMode::Off => (RuntimeWarpReadiness::Bypassed, None),
            RuntimeWarpMode::Repitch | RuntimeWarpMode::ElastiqueDraft => {
                match registration.source_tempo_bpm {
                    Some(source_tempo_bpm)
                        if source_tempo_bpm.is_finite() && source_tempo_bpm > 0.0 =>
                    {
                        realized_ratio = project_tempo_bpm / source_tempo_bpm;
                        if !realized_ratio.is_finite() || realized_ratio <= 0.0 {
                            (
                                RuntimeWarpReadiness::Degraded,
                                Some("warp ratio is invalid".to_string()),
                            )
                        } else if let Some(media_asset_id) = registration.media_asset_id.as_deref()
                        {
                            match media_assets.get(media_asset_id) {
                                Some(asset) if asset.state == RuntimeMediaAssetState::Ready => {
                                    if registration.mode == RuntimeWarpMode::ElastiqueDraft
                                        && !(0.5..=2.0).contains(&realized_ratio)
                                    {
                                        (
                                            RuntimeWarpReadiness::Degraded,
                                            Some(format!(
                                                "elastique draft ratio {realized_ratio:.3} outside baseline support"
                                            )),
                                        )
                                    } else {
                                        (RuntimeWarpReadiness::Ready, None)
                                    }
                                }
                                Some(asset) => (
                                    RuntimeWarpReadiness::Degraded,
                                    Some(format!("media asset not ready: {:?}", asset.state)),
                                ),
                                None => (
                                    RuntimeWarpReadiness::Degraded,
                                    Some(format!(
                                        "media asset `{media_asset_id}` missing from runtime cache"
                                    )),
                                ),
                            }
                        } else {
                            (
                                RuntimeWarpReadiness::Degraded,
                                Some("warp clip missing media asset".to_string()),
                            )
                        }
                    }
                    Some(_) => (
                        RuntimeWarpReadiness::Degraded,
                        Some("warp source tempo must be positive".to_string()),
                    ),
                    None => (
                        RuntimeWarpReadiness::Degraded,
                        Some("warp source tempo missing".to_string()),
                    ),
                }
            }
        };

        RuntimeWarpClipSnapshot {
            clip_id: registration.clip_id.clone(),
            media_asset_id: registration.media_asset_id.clone(),
            mode: registration.mode,
            source_tempo_bpm: registration.source_tempo_bpm,
            project_tempo_bpm,
            realized_ratio,
            anchor_timeline_samples: registration.anchor_timeline_samples,
            start_samples: registration.start_samples,
            duration_samples: registration.duration_samples,
            readiness,
            last_error: last_error.clone(),
            summary: format!(
                "clip={} mode={:?} readiness={:?} ratio={realized_ratio:.3} source_tempo={} project_tempo={project_tempo_bpm:.2} error={}",
                registration.clip_id,
                registration.mode,
                readiness,
                registration
                    .source_tempo_bpm
                    .map(|tempo| format!("{tempo:.2}"))
                    .unwrap_or_else(|| "none".to_string()),
                last_error.as_deref().unwrap_or("none"),
            ),
        }
    }

    fn reconcile_clips(&mut self, clips: Vec<RuntimeWarpClipRegistration>) {
        let retained_ids = clips
            .iter()
            .map(|clip| clip.clip_id.clone())
            .collect::<BTreeSet<_>>();
        self.clips.retain(|clip_id, _| retained_ids.contains(clip_id));
        for clip in clips {
            self.clips.insert(clip.clip_id.clone(), clip);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeRecordingCaptureActiveSession {
    take_id: String,
    track_id: String,
    start_samples: i64,
    capture_path: String,
    sample_rate_hz: u32,
    channel_count: usize,
    samples: Vec<f32>,
    buffered_block_count: u64,
    buffered_frame_count: u64,
    peak_level: f32,
    pressure_event_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeRecordingCaptureStateModel {
    policy: RuntimeRecordingCapturePolicy,
    active: Option<RuntimeRecordingCaptureActiveSession>,
    last_committed_take_id: Option<String>,
    last_committed_path: Option<String>,
    last_committed_duration_samples: Option<u32>,
    last_error: Option<String>,
}

impl RuntimeRecordingCaptureStateModel {
    fn capture_ready(&self, configured: bool, readiness: &RuntimeReadiness) -> bool {
        configured
            && !matches!(
                readiness,
                RuntimeReadiness::Stopped | RuntimeReadiness::Failed { .. }
            )
    }

    fn snapshot(
        &self,
        configured: bool,
        readiness: &RuntimeReadiness,
    ) -> RuntimeRecordingCaptureSnapshot {
        let state = if self.last_error.is_some() {
            Some(RuntimeRecordingCaptureState::Failed)
        } else if self.active.is_some() {
            Some(RuntimeRecordingCaptureState::Capturing)
        } else {
            Some(RuntimeRecordingCaptureState::Idle)
        };
        let summary = if let Some(active) = self.active.as_ref() {
            format!(
                "state=capturing ready={} take={} track={} frames={} blocks={} pressure={} path={}",
                self.capture_ready(configured, readiness),
                active.take_id,
                active.track_id,
                active.buffered_frame_count,
                active.buffered_block_count,
                active.pressure_event_count,
                active.capture_path
            )
        } else {
            format!(
                "state={} ready={} last_take={} last_path={} duration={} error={}",
                if self.last_error.is_some() {
                    "failed"
                } else {
                    "idle"
                },
                self.capture_ready(configured, readiness),
                self.last_committed_take_id.as_deref().unwrap_or("none"),
                self.last_committed_path.as_deref().unwrap_or("none"),
                self.last_committed_duration_samples.unwrap_or(0),
                self.last_error.as_deref().unwrap_or("none"),
            )
        };

        RuntimeRecordingCaptureSnapshot {
            capture_ready: self.capture_ready(configured, readiness),
            state,
            active_take_id: self.active.as_ref().map(|active| active.take_id.clone()),
            active_track_id: self.active.as_ref().map(|active| active.track_id.clone()),
            capture_start_samples: self.active.as_ref().map(|active| active.start_samples),
            active_capture_path: self
                .active
                .as_ref()
                .map(|active| active.capture_path.clone()),
            buffered_block_count: self
                .active
                .as_ref()
                .map(|active| active.buffered_block_count)
                .unwrap_or(0),
            buffered_frame_count: self
                .active
                .as_ref()
                .map(|active| active.buffered_frame_count)
                .unwrap_or(0),
            captured_channel_count: self
                .active
                .as_ref()
                .map(|active| active.channel_count)
                .unwrap_or(0),
            peak_level: self.active.as_ref().map(|active| active.peak_level),
            pressure_event_count: self
                .active
                .as_ref()
                .map(|active| active.pressure_event_count)
                .unwrap_or(0),
            last_committed_take_id: self.last_committed_take_id.clone(),
            last_committed_path: self.last_committed_path.clone(),
            last_committed_duration_samples: self.last_committed_duration_samples,
            last_error: self.last_error.clone(),
            summary,
        }
    }

    fn start_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
        sample_rate_hz: u32,
        configured: bool,
        readiness: &RuntimeReadiness,
    ) -> Result<(), RuntimeError> {
        if !self.capture_ready(configured, readiness) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is not ready to begin recording capture",
            ));
        }
        if self.active.is_some() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is already active",
            ));
        }
        self.last_error = None;
        self.active = Some(RuntimeRecordingCaptureActiveSession {
            take_id: request.take_id,
            track_id: request.track_id,
            start_samples: request.start_samples,
            capture_path: request.capture_path,
            sample_rate_hz,
            channel_count: 0,
            samples: Vec::new(),
            buffered_block_count: 0,
            buffered_frame_count: 0,
            peak_level: 0.0,
            pressure_event_count: 0,
        });
        Ok(())
    }

    fn record_output_block(&mut self, output: &AudioBuffer) {
        let Some(active) = self.active.as_mut() else {
            return;
        };

        let channel_count = output.channel_count().0;
        let frame_count = output.frames().0 as u64;
        if active.channel_count == 0 {
            active.channel_count = channel_count;
        } else if active.channel_count != channel_count {
            self.last_error = Some(format!(
                "capture channel-count mismatch: expected {} got {}",
                active.channel_count, channel_count
            ));
            return;
        }

        active.samples.extend_from_slice(output.samples());
        active.buffered_block_count = active.buffered_block_count.saturating_add(1);
        active.buffered_frame_count = active.buffered_frame_count.saturating_add(frame_count);
        let block_peak = output
            .samples()
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        active.peak_level = active.peak_level.max(block_peak);
        if active.buffered_frame_count >= self.policy.pressure_threshold_frames {
            active.pressure_event_count = active.pressure_event_count.saturating_add(1);
        }
    }

    fn finish_capture(&mut self) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        let active = self.active.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is not active",
            )
        })?;
        if active.channel_count == 0 || active.samples.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture has no buffered audio to commit",
            ));
        }

        let duration_samples = active.buffered_frame_count.min(u32::MAX as u64) as u32;
        let capture_path = active.capture_path.clone();
        if let Some(parent) = Path::new(&capture_path).parent() {
            fs::create_dir_all(parent).map_err(|error| {
                self.last_error = Some(error.to_string());
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!("failed to create recording capture directory: {error}"),
                )
            })?;
        }

        let spec = WavSpec {
            channels: active.channel_count as u16,
            sample_rate: active.sample_rate_hz,
            bits_per_sample: 32,
            sample_format: HoundSampleFormat::Float,
        };
        let mut writer = WavWriter::create(&capture_path, spec).map_err(|error| {
            self.last_error = Some(error.to_string());
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!("failed to create recording capture wav: {error}"),
            )
        })?;
        for sample in &active.samples {
            writer.write_sample(*sample).map_err(|error| {
                self.last_error = Some(error.to_string());
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!("failed to write recording capture sample: {error}"),
                )
            })?;
        }
        writer.finalize().map_err(|error| {
            self.last_error = Some(error.to_string());
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!("failed to finalize recording capture wav: {error}"),
            )
        })?;

        let receipt = RuntimeRecordingCaptureCommitReceipt {
            take_id: active.take_id.clone(),
            track_id: active.track_id.clone(),
            start_samples: active.start_samples,
            duration_samples,
            channel_count: active.channel_count,
            peak_level: active.peak_level,
            capture_path,
        };

        self.last_error = None;
        self.last_committed_take_id = Some(receipt.take_id.clone());
        self.last_committed_path = Some(receipt.capture_path.clone());
        self.last_committed_duration_samples = Some(receipt.duration_samples);
        self.active = None;
        Ok(receipt)
    }

    fn cancel_capture(&mut self) -> Result<(), RuntimeError> {
        if self.active.is_none() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is not active",
            ));
        }
        self.last_error = None;
        self.active = None;
        Ok(())
    }
}

impl Default for RuntimeRecordingCaptureStateModel {
    fn default() -> Self {
        Self {
            policy: RuntimeRecordingCapturePolicy::default(),
            active: None,
            last_committed_take_id: None,
            last_committed_path: None,
            last_committed_duration_samples: None,
            last_error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencyPolicy {
    steady_session_limit: usize,
    recovery_session_limit: usize,
}

impl Default for RuntimeTransportConcurrencyPolicy {
    fn default() -> Self {
        Self {
            steady_session_limit: 1,
            recovery_session_limit: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencySession {
    sandbox_id: String,
    lease_id: String,
    region_id: String,
    intent: TransportAttachIntent,
    provenance: TransportSessionProvenance,
    attach_sequence: u64,
    attach_processing_epoch: Option<u64>,
    state: TransportSessionState,
    backing_path: Option<String>,
    total_bytes: Option<u32>,
    cleanup_attempt_count: u32,
    last_cleanup_mode: Option<LingeringCleanupMode>,
    last_cleanup_wave: Option<u64>,
    cleanup_in_progress: bool,
    last_cleanup_epoch: Option<u64>,
    last_cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencyState {
    policy: RuntimeTransportConcurrencyPolicy,
    active_sessions: BTreeMap<(String, String, String), RuntimeTransportConcurrencySession>,
    pending_cleanup_work: VecDeque<RuntimeLingeringCleanupWorkItem>,
    peak_attached_sessions: usize,
    peak_recovery_overlap_sessions: usize,
    peak_lingering_sessions: usize,
    next_attach_sequence: u64,
    next_cleanup_work_id: u64,
    next_cleanup_epoch: u64,
    next_cleanup_wave_by_sandbox: BTreeMap<String, u64>,
    last_admitted_sandbox_id: Option<String>,
    last_rejected_sandbox_id: Option<String>,
    last_rejection_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeLingeringCleanupWorkItem {
    work_id: u64,
    cleanup_epoch: u64,
    cleanup_wave: u64,
    sandbox_id: String,
    mode: LingeringCleanupMode,
    trigger: LingeringCleanupTrigger,
    retry_count: u32,
    processing_epoch: u64,
    ready_at_processing_epoch: u64,
    exclude_lease_id: Option<String>,
    exclude_region_id: Option<String>,
}

impl RuntimeTransportConcurrencyState {
    fn active_session_view(
        session: &RuntimeTransportConcurrencySession,
    ) -> crate::interfaces::ActiveTransportConcurrencySession {
        crate::interfaces::ActiveTransportConcurrencySession {
            sandbox_id: session.sandbox_id.clone(),
            lease_id: session.lease_id.clone(),
            region_id: session.region_id.clone(),
            intent: session.intent,
            provenance: session.provenance,
            attach_sequence: session.attach_sequence,
            attach_processing_epoch: session.attach_processing_epoch,
            state: session.state,
            backing_path: session.backing_path.clone(),
            total_bytes: session.total_bytes,
            cleanup_attempt_count: session.cleanup_attempt_count,
            last_cleanup_mode: session.last_cleanup_mode,
            last_cleanup_wave: session.last_cleanup_wave,
            cleanup_in_progress: session.cleanup_in_progress,
            last_cleanup_epoch: session.last_cleanup_epoch,
            last_cleanup_error: session.last_cleanup_error.clone(),
        }
    }

    fn next_cleanup_wave_for_sandbox(&mut self, sandbox_id: &str) -> u64 {
        let next = self
            .next_cleanup_wave_by_sandbox
            .entry(sandbox_id.to_string())
            .or_insert(1);
        let cleanup_wave = *next;
        *next = next.saturating_add(1);
        cleanup_wave
    }

    fn has_lingering_candidates(
        &self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> bool {
        self.active_sessions.values().any(|session| {
            session.sandbox_id == sandbox_id
                && matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
                && !matches!(
                    (exclude_lease_id, exclude_region_id),
                    (Some(exclude_lease_id), Some(exclude_region_id))
                        if session.lease_id == exclude_lease_id
                            && session.region_id == exclude_region_id
                )
        })
    }

    fn enqueue_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        retry_count: u32,
        processing_epoch: u64,
        cleanup_wave: Option<u64>,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        if !self.has_lingering_candidates(sandbox_id, exclude_lease_id, exclude_region_id) {
            return None;
        }

        let work_id = self.next_cleanup_work_id;
        self.next_cleanup_work_id = self.next_cleanup_work_id.saturating_add(1);
        let cleanup_epoch = self.next_cleanup_epoch;
        self.next_cleanup_epoch = self.next_cleanup_epoch.saturating_add(1);
        let cleanup_wave =
            cleanup_wave.unwrap_or_else(|| self.next_cleanup_wave_for_sandbox(sandbox_id));
        let backoff = match trigger {
            LingeringCleanupTrigger::DeferredRetry => retry_count.max(1) as u64,
            LingeringCleanupTrigger::RecoveryPreAttach
            | LingeringCleanupTrigger::PostStartReconciliation => 0,
        };
        self.pending_cleanup_work
            .push_back(RuntimeLingeringCleanupWorkItem {
                work_id,
                cleanup_epoch,
                cleanup_wave,
                sandbox_id: sandbox_id.to_string(),
                mode,
                trigger,
                retry_count,
                processing_epoch,
                ready_at_processing_epoch: processing_epoch.saturating_add(backoff),
                exclude_lease_id: exclude_lease_id.map(ToOwned::to_owned),
                exclude_region_id: exclude_region_id.map(ToOwned::to_owned),
            });
        Some(LingeringCleanupQueueReceipt {
            work_id,
            cleanup_epoch,
            cleanup_wave,
        })
    }

    fn cleanup_attempt_count(&self, sandbox_id: &str, lease_id: &str, region_id: &str) -> u32 {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .map(|session| session.cleanup_attempt_count)
            .unwrap_or(0)
    }

    fn cleanup_wave_for_session(
        &self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> Option<u64> {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .and_then(|session| session.last_cleanup_wave)
    }

    fn pending_deferred_retry_work_count(&self) -> usize {
        self.pending_cleanup_work
            .iter()
            .filter(|item| item.trigger == LingeringCleanupTrigger::DeferredRetry)
            .count()
    }

    fn oldest_pending_cleanup_ready_epoch(&self) -> Option<u64> {
        self.pending_cleanup_work
            .iter()
            .map(|item| item.ready_at_processing_epoch)
            .min()
    }

    fn pending_cleanup_waves(&self) -> Vec<crate::interfaces::PendingLingeringCleanupWaveSummary> {
        let mut by_wave: BTreeMap<
            (String, u64),
            crate::interfaces::PendingLingeringCleanupWaveSummary,
        > = BTreeMap::new();
        for item in &self.pending_cleanup_work {
            let key = (item.sandbox_id.clone(), item.cleanup_wave);
            let entry = by_wave.entry(key).or_insert_with(|| {
                crate::interfaces::PendingLingeringCleanupWaveSummary {
                    sandbox_id: item.sandbox_id.clone(),
                    cleanup_wave: item.cleanup_wave,
                    mode: item.mode,
                    first_trigger: item.trigger,
                    latest_trigger: item.trigger,
                    pending_work_items: 0,
                    deferred_retry_work_items: 0,
                    first_cleanup_epoch: item.cleanup_epoch,
                    latest_cleanup_epoch: item.cleanup_epoch,
                    first_processing_epoch: item.processing_epoch,
                    latest_processing_epoch: item.processing_epoch,
                    oldest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                    newest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                }
            });
            entry.latest_trigger = item.trigger;
            entry.pending_work_items = entry.pending_work_items.saturating_add(1);
            if item.trigger == LingeringCleanupTrigger::DeferredRetry {
                entry.deferred_retry_work_items = entry.deferred_retry_work_items.saturating_add(1);
            }
            entry.first_cleanup_epoch = entry.first_cleanup_epoch.min(item.cleanup_epoch);
            entry.latest_cleanup_epoch = entry.latest_cleanup_epoch.max(item.cleanup_epoch);
            entry.first_processing_epoch = entry.first_processing_epoch.min(item.processing_epoch);
            entry.latest_processing_epoch =
                entry.latest_processing_epoch.max(item.processing_epoch);
            entry.oldest_ready_at_processing_epoch = entry
                .oldest_ready_at_processing_epoch
                .min(item.ready_at_processing_epoch);
            entry.newest_ready_at_processing_epoch = entry
                .newest_ready_at_processing_epoch
                .max(item.ready_at_processing_epoch);
        }
        by_wave.into_values().collect()
    }

    fn dequeue_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<crate::interfaces::LingeringCleanupPlan> {
        let position = self.pending_cleanup_work.iter().position(|item| {
            item.sandbox_id == sandbox_id
                && item.ready_at_processing_epoch <= current_processing_epoch
        })?;
        let work = self.pending_cleanup_work.remove(position)?;
        let candidates = self.lingering_cleanup_candidates(
            work.sandbox_id.as_str(),
            work.exclude_lease_id.as_deref(),
            work.exclude_region_id.as_deref(),
            work.mode,
            work.processing_epoch,
            work.cleanup_wave,
        );
        if candidates.is_empty() {
            return None;
        }
        Some(crate::interfaces::LingeringCleanupPlan {
            work_id: work.work_id,
            cleanup_epoch: work.cleanup_epoch,
            cleanup_wave: work.cleanup_wave,
            sandbox_id: work.sandbox_id,
            mode: work.mode,
            trigger: work.trigger,
            retry_count: work.retry_count,
            processing_epoch: work.processing_epoch,
            ready_at_processing_epoch: work.ready_at_processing_epoch,
            exclude_lease_id: work.exclude_lease_id,
            exclude_region_id: work.exclude_region_id,
            candidates,
        })
    }

    fn lingering_cleanup_candidates(
        &mut self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        cleanup_wave: u64,
    ) -> Vec<crate::interfaces::ActiveTransportConcurrencySession> {
        let mut session_keys: Vec<_> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| {
                session.sandbox_id == sandbox_id
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
                    && !matches!(
                        (exclude_lease_id, exclude_region_id),
                        (Some(exclude_lease_id), Some(exclude_region_id))
                            if session.lease_id == exclude_lease_id
                                && session.region_id == exclude_region_id
                    )
            })
            .map(|(key, _)| key.clone())
            .collect();

        session_keys.sort_by(|left, right| {
            let left = self
                .active_sessions
                .get(left)
                .expect("missing left session");
            let right = self
                .active_sessions
                .get(right)
                .expect("missing right session");
            let left_key = (
                match left.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                left.attach_sequence,
                match left.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                left.lease_id.as_str(),
                left.region_id.as_str(),
            );
            let right_key = (
                match right.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                right.attach_sequence,
                match right.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                right.lease_id.as_str(),
                right.region_id.as_str(),
            );
            left_key.cmp(&right_key)
        });

        let mut sessions = Vec::with_capacity(session_keys.len());
        for key in session_keys {
            if let Some(session) = self.active_sessions.get_mut(&key) {
                session.cleanup_attempt_count = session.cleanup_attempt_count.saturating_add(1);
                session.last_cleanup_mode = Some(mode);
                session.last_cleanup_wave = Some(cleanup_wave);
                session.cleanup_in_progress = true;
                session.last_cleanup_epoch = Some(processing_epoch);
                session.last_cleanup_error = None;
                sessions.push(Self::active_session_view(session));
            }
        }

        sessions
    }

    fn steady_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::SteadyState)
            .count()
    }

    fn recovery_overlap_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::RecoveryOverlap)
            .count()
    }

    fn lingering_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| {
                matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            })
            .count()
    }

    fn detach_requested_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachRequested)
            .count()
    }

    fn detach_faulted_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachFaulted)
            .count()
    }

    fn recovery_overlap_limit(&self) -> usize {
        self.policy
            .recovery_session_limit
            .saturating_sub(self.policy.steady_session_limit)
            .max(1)
    }

    fn lingering_reason_suffix(&self, intent: TransportAttachIntent) -> String {
        let lingering = self
            .active_sessions
            .values()
            .filter(|session| {
                session.intent == intent
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
            })
            .count();
        if lingering == 0 {
            String::new()
        } else {
            format!(" ({lingering} lingering session(s) pending detach)")
        }
    }

    fn snapshot(&self) -> RuntimeTransportConcurrencySnapshot {
        RuntimeTransportConcurrencySnapshot {
            steady_session_limit: self.policy.steady_session_limit,
            recovery_session_limit: self.policy.recovery_session_limit,
            current_attached_sessions: self.active_sessions.len(),
            peak_attached_sessions: self.peak_attached_sessions,
            current_recovery_overlap_sessions: self.recovery_overlap_session_count(),
            peak_recovery_overlap_sessions: self.peak_recovery_overlap_sessions,
            current_lingering_sessions: self.lingering_session_count(),
            peak_lingering_sessions: self.peak_lingering_sessions,
            current_detach_requested_sessions: self.detach_requested_session_count(),
            current_detach_faulted_sessions: self.detach_faulted_session_count(),
            pending_cleanup_work_items: self.pending_cleanup_work.len(),
            pending_deferred_retry_work_items: self.pending_deferred_retry_work_count(),
            next_cleanup_epoch: self.next_cleanup_epoch,
            oldest_pending_cleanup_ready_epoch: self.oldest_pending_cleanup_ready_epoch(),
            pending_cleanup_waves: self.pending_cleanup_waves(),
            active_sessions: self
                .active_sessions
                .values()
                .map(Self::active_session_view)
                .collect(),
            last_admitted_sandbox_id: self.last_admitted_sandbox_id.clone(),
            last_rejected_sandbox_id: self.last_rejected_sandbox_id.clone(),
            last_rejection_reason: self.last_rejection_reason.clone(),
        }
    }

    fn begin_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        provenance: TransportSessionProvenance,
        attach_processing_epoch: Option<u64>,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let key = (
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        );
        if self.active_sessions.contains_key(&key) {
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some("transport session is already attached".to_string());
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "transport session is already attached",
            ));
        }

        let steady_sessions = self.steady_session_count();
        let recovery_sessions = self.recovery_overlap_session_count();

        if matches!(intent, TransportAttachIntent::SteadyState)
            && steady_sessions >= self.policy.steady_session_limit
        {
            let reason = format!(
                "steady-state transport session limit {} is already attached{}",
                self.policy.steady_session_limit,
                self.lingering_reason_suffix(TransportAttachIntent::SteadyState)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        if matches!(intent, TransportAttachIntent::RecoveryOverlap)
            && recovery_sessions >= self.recovery_overlap_limit()
        {
            let reason = format!(
                "recovery overlap session limit {} is already attached{}",
                self.recovery_overlap_limit(),
                self.lingering_reason_suffix(TransportAttachIntent::RecoveryOverlap)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        let limit = match intent {
            TransportAttachIntent::SteadyState => self.policy.steady_session_limit,
            TransportAttachIntent::RecoveryOverlap => self.policy.recovery_session_limit,
        };
        if self.active_sessions.len() >= limit {
            let reason = format!(
                "transport session admission exceeds {:?} limit {}",
                intent, limit
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        self.active_sessions.insert(
            key,
            RuntimeTransportConcurrencySession {
                sandbox_id: sandbox_id.to_string(),
                lease_id: lease_id.to_string(),
                region_id: region_id.to_string(),
                intent,
                provenance,
                attach_sequence: self.next_attach_sequence,
                attach_processing_epoch,
                state: TransportSessionState::AttachActive,
                backing_path,
                total_bytes,
                cleanup_attempt_count: 0,
                last_cleanup_mode: None,
                last_cleanup_wave: None,
                cleanup_in_progress: false,
                last_cleanup_epoch: None,
                last_cleanup_error: None,
            },
        );
        self.next_attach_sequence = self.next_attach_sequence.saturating_add(1);
        self.peak_attached_sessions = self.peak_attached_sessions.max(self.active_sessions.len());
        let recovery_sessions = self.recovery_overlap_session_count();
        self.peak_recovery_overlap_sessions =
            self.peak_recovery_overlap_sessions.max(recovery_sessions);
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.last_admitted_sandbox_id = Some(sandbox_id.to_string());
        self.last_rejected_sandbox_id = None;
        self.last_rejection_reason = None;
        Ok(self.snapshot())
    }

    fn mark_session_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        state: TransportSessionState,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.state = state;
        }
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.snapshot()
    }

    fn promote_session_to_steady_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.intent = TransportAttachIntent::SteadyState;
        }
        self.snapshot()
    }

    fn record_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.last_cleanup_mode = Some(mode);
            session.cleanup_in_progress = false;
            session.last_cleanup_epoch = Some(processing_epoch);
            session.last_cleanup_error = Some(error.to_string());
        }
        self.snapshot()
    }

    fn clear_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.cleanup_in_progress = false;
            session.last_cleanup_error = None;
        }
        self.snapshot()
    }

    fn end_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.active_sessions.remove(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        ));
        self.snapshot()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for RuntimeTransportConcurrencyState {
    fn default() -> Self {
        Self {
            policy: RuntimeTransportConcurrencyPolicy::default(),
            active_sessions: BTreeMap::new(),
            pending_cleanup_work: VecDeque::new(),
            peak_attached_sessions: 0,
            peak_recovery_overlap_sessions: 0,
            peak_lingering_sessions: 0,
            next_attach_sequence: 1,
            next_cleanup_work_id: 1,
            next_cleanup_epoch: 1,
            next_cleanup_wave_by_sandbox: BTreeMap::new(),
            last_admitted_sandbox_id: None,
            last_rejected_sandbox_id: None,
            last_rejection_reason: None,
        }
    }
}

fn transport_session_provenance(intent: TransportAttachIntent) -> TransportSessionProvenance {
    match intent {
        TransportAttachIntent::SteadyState => TransportSessionProvenance::SteadyOrigin,
        TransportAttachIntent::RecoveryOverlap => TransportSessionProvenance::RecoveryReplacement,
    }
}

fn hash_audio_buffer(buffer: &AudioBuffer) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for sample in buffer.samples() {
        hash ^= u64::from(sample.to_bits());
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= buffer.frames().0 as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash ^= buffer.channel_count().0 as u64;
    hash
}

fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

fn classify_transport_transition(
    previous: Option<TransportProjection>,
    next: TransportProjection,
) -> Option<RuntimeTransportTransitionKind> {
    let Some(previous) = previous else {
        return Some(if next.playing {
            RuntimeTransportTransitionKind::Started
        } else {
            RuntimeTransportTransitionKind::Initial
        });
    };
    if previous.playing != next.playing {
        return Some(if next.playing {
            RuntimeTransportTransitionKind::Started
        } else {
            RuntimeTransportTransitionKind::Stopped
        });
    }
    if previous.timeline_position_samples != next.timeline_position_samples {
        return Some(RuntimeTransportTransitionKind::Seeked);
    }
    if previous.tempo_bpm != next.tempo_bpm {
        return Some(RuntimeTransportTransitionKind::TempoChanged);
    }
    if previous.loop_state != next.loop_state {
        return Some(RuntimeTransportTransitionKind::LoopStateChanged);
    }
    None
}

fn classify_transport_invalidation_reason(
    previous: Option<TransportProjection>,
    next: TransportProjection,
) -> RuntimePreworkInvalidationReason {
    match classify_transport_transition(previous, next) {
        Some(RuntimeTransportTransitionKind::Started)
        | Some(RuntimeTransportTransitionKind::Initial) => {
            RuntimePreworkInvalidationReason::TransportStarted
        }
        Some(RuntimeTransportTransitionKind::Stopped) => {
            RuntimePreworkInvalidationReason::TransportStopped
        }
        Some(RuntimeTransportTransitionKind::Seeked) => {
            RuntimePreworkInvalidationReason::TransportSeeked
        }
        Some(RuntimeTransportTransitionKind::TempoChanged) => {
            RuntimePreworkInvalidationReason::TransportTempoChanged
        }
        Some(RuntimeTransportTransitionKind::LoopStateChanged) => {
            RuntimePreworkInvalidationReason::TransportLoopStateChanged
        }
        Some(RuntimeTransportTransitionKind::LoopWrapped) => {
            RuntimePreworkInvalidationReason::TransportLoopWrapped
        }
        None => RuntimePreworkInvalidationReason::TransportSeeked,
    }
}

fn transport_projection_from_context(context: &GraphExecutionContext) -> TransportProjection {
    TransportProjection {
        playing: context.transport_playing,
        timeline_position_samples: context.timeline_position_samples,
        tempo_bpm: context.transport_tempo_bpm,
        loop_state: None,
    }
}

const PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW: u64 = 2;
const PREWORK_QUEUE_CAPACITY: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimePendingTransportTransition {
    kind: RuntimeTransportTransitionKind,
    effective_block_sequence: Option<u64>,
    transport_epoch: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RuntimeEngineTransportAdvance {
    start_samples: Option<i64>,
    end_samples: Option<i64>,
    loop_wrapped: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeTimelineState {
    next_block_sequence: u64,
    continuity: BlockSequenceContinuityReport,
    transport_epoch: u64,
    last_transport_transition: Option<RuntimeTransportTransitionKind>,
    last_transport_transition_processing_epoch: Option<u64>,
    last_transport_transition_block_sequence: Option<u64>,
    pending_transport_transition: Option<RuntimePendingTransportTransition>,
    last_transport_playing: Option<bool>,
    last_transport_tempo_bpm: Option<f64>,
    last_transport_timeline_position_samples: Option<i64>,
    last_transport_loop_start_samples: Option<i64>,
    last_transport_loop_end_samples: Option<i64>,
    last_engine_block_start_samples: Option<i64>,
    last_engine_block_end_samples: Option<i64>,
    loop_wrap_count: u64,
}

impl RuntimeTimelineState {
    fn allocate_block_sequence(&mut self) -> u64 {
        let block_sequence = self.next_block_sequence;
        self.next_block_sequence = self.next_block_sequence.saturating_add(1);
        block_sequence
    }

    fn record_block_sequence(
        &mut self,
        sandbox_id: &str,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) -> Option<LeaseRolloverRecord> {
        let lease_id = lease_id.into();
        let previous = self.continuity.segments.last().cloned();
        self.continuity
            .record(processing_epoch, lease_id.clone(), block_sequence);
        previous.and_then(|segment| {
            (segment.lease_id != lease_id).then(|| LeaseRolloverRecord {
                sandbox_id: sandbox_id.to_string(),
                previous_lease_id: segment.lease_id,
                lease_id,
                processing_epoch,
                first_block_sequence: block_sequence,
            })
        })
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn record_transport_projection(
        &mut self,
        kind: RuntimeTransportTransitionKind,
        effective_block_sequence: Option<u64>,
        processing_epoch: Option<u64>,
        projection: TransportProjection,
    ) -> u64 {
        self.transport_epoch = self.transport_epoch.saturating_add(1);
        self.last_transport_transition = Some(kind);
        self.last_transport_transition_processing_epoch = processing_epoch;
        self.last_transport_transition_block_sequence = effective_block_sequence;
        self.pending_transport_transition = Some(RuntimePendingTransportTransition {
            kind,
            effective_block_sequence,
            transport_epoch: self.transport_epoch,
        });
        self.update_transport_state(projection);
        self.transport_epoch
    }

    fn consume_pending_transport_transition(
        &mut self,
        block_sequence: u64,
    ) -> Option<RuntimePendingTransportTransition> {
        match self.pending_transport_transition {
            Some(pending)
                if pending
                    .effective_block_sequence
                    .map_or(true, |effective| effective == block_sequence) =>
            {
                self.pending_transport_transition = None;
                Some(pending)
            }
            _ => None,
        }
    }

    fn record_loop_wrap(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        projection: TransportProjection,
    ) -> u64 {
        self.transport_epoch = self.transport_epoch.saturating_add(1);
        self.loop_wrap_count = self.loop_wrap_count.saturating_add(1);
        self.last_transport_transition = Some(RuntimeTransportTransitionKind::LoopWrapped);
        self.last_transport_transition_processing_epoch = Some(processing_epoch);
        self.last_transport_transition_block_sequence = Some(block_sequence);
        self.update_transport_state(projection);
        self.transport_epoch
    }

    fn record_engine_block_window(&mut self, start_samples: Option<i64>, end_samples: Option<i64>) {
        self.last_engine_block_start_samples = start_samples;
        self.last_engine_block_end_samples = end_samples;
    }

    fn update_transport_state(&mut self, projection: TransportProjection) {
        self.last_transport_playing = Some(projection.playing);
        self.last_transport_tempo_bpm = Some(projection.tempo_bpm);
        self.last_transport_timeline_position_samples = Some(projection.timeline_position_samples);
        self.last_transport_loop_start_samples = projection
            .loop_state
            .map(|loop_region| loop_region.start_samples);
        self.last_transport_loop_end_samples = projection
            .loop_state
            .map(|loop_region| loop_region.end_samples);
    }

    fn snapshot(&self) -> RuntimeTimelineSnapshot {
        RuntimeTimelineSnapshot {
            next_block_sequence: self.next_block_sequence,
            block_sequence_continuity: self.continuity.clone(),
            transport_epoch: self.transport_epoch,
            last_transport_transition: self.last_transport_transition,
            last_transport_transition_processing_epoch: self
                .last_transport_transition_processing_epoch,
            last_transport_transition_block_sequence: self.last_transport_transition_block_sequence,
            last_transport_playing: self.last_transport_playing,
            last_transport_tempo_bpm: self.last_transport_tempo_bpm,
            last_transport_timeline_position_samples: self.last_transport_timeline_position_samples,
            last_transport_loop_start_samples: self.last_transport_loop_start_samples,
            last_transport_loop_end_samples: self.last_transport_loop_end_samples,
            last_engine_block_start_samples: self.last_engine_block_start_samples,
            last_engine_block_end_samples: self.last_engine_block_end_samples,
            loop_wrap_count: self.loop_wrap_count,
        }
    }
}

fn sanitize_asset_id(asset_id: &str) -> String {
    asset_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeAutomationState {
    continuity: AutomationContinuityReport,
}

impl RuntimeAutomationState {
    fn record_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        self.continuity.record(processing_epoch, lease_id, summary);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn snapshot(&self) -> RuntimeAutomationSnapshot {
        let aggregate = self.continuity.aggregate();
        RuntimeAutomationSnapshot {
            parameter_id: aggregate.parameter_id,
            value_events: aggregate.value_events,
            modulation_events: aggregate.modulation_events,
            gesture_begin_events: aggregate.gesture_begin_events,
            gesture_end_events: aggregate.gesture_end_events,
            first_value: aggregate.first_value,
            last_value: aggregate.last_value,
            last_modulation: aggregate.last_modulation,
            first_epoch: self.continuity.first_epoch(),
            last_epoch: self.continuity.last_epoch(),
            segment_count: self.continuity.segment_count(),
            segment_epochs: self.continuity.segment_epochs(),
            lease_rollovers: self.continuity.lease_rollovers,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeEngineState {
    graph: Option<ExecutableGraph>,
    snapshot: RuntimeEngineBlockSnapshot,
    plugin_node_bindings: HashMap<String, String>,
    pending_plugin_node_renders: BTreeMap<(u64, u64), PluginNodeRenderBatch>,
    prework_queue: VecDeque<RuntimeEnginePreworkCache>,
    pending_prework_targets: VecDeque<RuntimePendingPreworkTarget>,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeEnginePreworkCache {
    graph_id: String,
    projection_epoch: u64,
    parameter_epoch: u64,
    transport: TransportProjection,
    block_size: usize,
    frame_count: usize,
    channel_count: usize,
    input_signature: u64,
    prepared: GraphPreparedDispatch,
    valid_until_processing_epoch: u64,
    valid_until_block_sequence: u64,
    source_processing_epoch: u64,
    source_block_sequence: u64,
    admitted_from_block_sequence: u64,
    consumption_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimePendingPreworkTarget {
    target_block_sequence: u64,
    admitted_from_block_sequence: u64,
    buffer: AudioBuffer,
    input_signature: u64,
    backlog_class: RuntimePreworkBacklogClass,
    parameter_epoch_override: Option<u64>,
    transport_override: Option<TransportProjection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RuntimePluginBackedBindingSummary {
    bound_sandbox_ids: Vec<String>,
    active_bound_sandboxes: usize,
    degraded_bound_sandboxes: usize,
    missing_bound_sandboxes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RuntimePreworkTransportCondition {
    recovery_overlap_sessions: usize,
    lingering_sessions: usize,
    detach_faulted_sessions: usize,
}

impl RuntimePreworkTransportCondition {
    fn gate_active(self, pressure: RuntimePreworkServicePressure) -> bool {
        pressure != RuntimePreworkServicePressure::Normal
            && (self.lingering_sessions > 0 || self.detach_faulted_sessions > 0)
    }

    fn reduce_service_scope(
        self,
        effective_cycles: usize,
        effective_budget_per_cycle: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> (usize, usize, RuntimePreworkBacklogClass) {
        if self.detach_faulted_sessions > 0 || self.lingering_sessions > 0 {
            (
                effective_cycles.min(1),
                effective_budget_per_cycle.min(1),
                RuntimePreworkBacklogClass::Immediate,
            )
        } else if self.recovery_overlap_sessions > 0 {
            (
                effective_cycles.min(1),
                effective_budget_per_cycle.min(1),
                max_backlog_class.min(RuntimePreworkBacklogClass::NearTerm),
            )
        } else {
            (
                effective_cycles,
                effective_budget_per_cycle,
                max_backlog_class,
            )
        }
    }
}

impl RuntimeEngineState {
    fn classify_prework_service_semantic_policy(
        graph: &ExecutableGraph,
        anticipative_enabled: bool,
        active_plugin_sandboxes: u32,
    ) -> RuntimePreworkServiceSemanticPolicy {
        if !anticipative_enabled {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        let planning = graph.planning_summary(anticipative_enabled);
        if planning.anticipative_eligible_node_count == 0 {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        if graph.plugin_backed_node_count() > 0 && active_plugin_sandboxes > 0 {
            return RuntimePreworkServiceSemanticPolicy::PluginConstrained;
        }

        if graph.total_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
            || graph.max_node_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
        {
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        } else {
            RuntimePreworkServiceSemanticPolicy::Balanced
        }
    }

    fn classify_prework_backlog_class(
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
    ) -> RuntimePreworkBacklogClass {
        match target_block_sequence.saturating_sub(admitted_from_block_sequence) {
            0 | 1 => RuntimePreworkBacklogClass::Immediate,
            2 => RuntimePreworkBacklogClass::NearTerm,
            _ => RuntimePreworkBacklogClass::Deferred,
        }
    }

    fn set_prework_service_pressure(&mut self, pressure: RuntimePreworkServicePressure) {
        self.snapshot.prework_service_pressure = pressure;
    }

    fn set_prework_service_plugin_state(
        &mut self,
        active_plugin_sandboxes: u32,
        bound_plugin_sandboxes: usize,
        active_bound_plugin_sandboxes: usize,
        degraded_bound_plugin_sandboxes: usize,
        missing_bound_plugin_sandboxes: usize,
        plugin_gate_active: bool,
    ) {
        self.snapshot.prework_service_active_plugin_sandboxes = active_plugin_sandboxes;
        self.snapshot.prework_service_bound_plugin_sandboxes = bound_plugin_sandboxes;
        self.snapshot.prework_service_active_bound_plugin_sandboxes = active_bound_plugin_sandboxes;
        self.snapshot
            .prework_service_degraded_bound_plugin_sandboxes = degraded_bound_plugin_sandboxes;
        self.snapshot.prework_service_missing_bound_plugin_sandboxes =
            missing_bound_plugin_sandboxes;
        self.snapshot.prework_service_plugin_gate_active = plugin_gate_active;
    }

    fn set_prework_service_transport_state(
        &mut self,
        recovery_overlap_sessions: usize,
        lingering_sessions: usize,
        detach_faulted_sessions: usize,
        transport_gate_active: bool,
    ) {
        self.snapshot.prework_service_recovery_overlap_sessions = recovery_overlap_sessions;
        self.snapshot.prework_service_lingering_sessions = lingering_sessions;
        self.snapshot.prework_service_detach_faulted_sessions = detach_faulted_sessions;
        self.snapshot.prework_service_transport_gate_active = transport_gate_active;
    }

    fn transition_prework_service_state(
        &mut self,
        state: RuntimePreworkServiceState,
        processing_epoch: Option<u64>,
    ) {
        let previous = self.snapshot.prework_service_state;
        if previous == state {
            return;
        }
        if state == RuntimePreworkServiceState::Paused {
            self.snapshot.prework_service_pause_count =
                self.snapshot.prework_service_pause_count.saturating_add(1);
        }
        if previous == RuntimePreworkServiceState::Paused
            && state == RuntimePreworkServiceState::Servicing
        {
            self.snapshot.prework_service_resume_count =
                self.snapshot.prework_service_resume_count.saturating_add(1);
        }
        if state == RuntimePreworkServiceState::Starved {
            self.snapshot.prework_service_starvation_count = self
                .snapshot
                .prework_service_starvation_count
                .saturating_add(1);
        }
        self.snapshot.prework_service_state = state;
        if let Some(processing_epoch) = processing_epoch {
            self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        }
    }

    fn update_prework_queue_snapshot(
        &mut self,
        current_block_sequence: Option<u64>,
        preserve_invalidated: bool,
    ) {
        self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
        self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
        self.snapshot.prework_cache_peak_queue_depth = self
            .snapshot
            .prework_cache_peak_queue_depth
            .max(self.prework_queue.len());
        self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
        self.snapshot.prework_pending_immediate_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Immediate)
            .count();
        self.snapshot.prework_pending_near_term_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::NearTerm)
            .count();
        self.snapshot.prework_pending_deferred_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Deferred)
            .count();
        self.snapshot.prework_next_pending_target_block_sequence = self
            .pending_prework_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .min();
        let mut target_block_sequences = self
            .prework_queue
            .iter()
            .map(|cache| cache.source_block_sequence)
            .chain(
                self.pending_prework_targets
                    .iter()
                    .map(|target| target.target_block_sequence),
            )
            .collect::<Vec<_>>();
        target_block_sequences.sort_unstable();
        target_block_sequences.dedup();
        self.snapshot.prework_cache_window_target_count = target_block_sequences.len();
        self.snapshot.prework_cache_window_target_block_sequences = target_block_sequences;

        let latest = self.prework_queue.back();
        self.snapshot.prework_cache_freshness_state =
            self.prework_freshness_state(latest, current_block_sequence);
        self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
            cache
                .valid_until_block_sequence
                .saturating_sub(current_block_sequence.unwrap_or(cache.source_block_sequence))
        });
        self.snapshot.prework_cache_valid_until_processing_epoch =
            latest.map(|cache| cache.valid_until_processing_epoch);
        self.snapshot.prework_cache_valid_until_block_sequence =
            latest.map(|cache| cache.valid_until_block_sequence);

        if latest.is_none() && !preserve_invalidated {
            self.snapshot.prework_cache_state = if self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
        }
    }

    fn record_prework_service_cycle(
        &mut self,
        processing_epoch: u64,
        cycle_count: usize,
        budget_per_cycle: usize,
        prepared_targets: usize,
    ) {
        self.snapshot.prework_service_cycle_count = self
            .snapshot
            .prework_service_cycle_count
            .saturating_add(cycle_count as u64);
        self.snapshot.prework_service_prepared_targets = self
            .snapshot
            .prework_service_prepared_targets
            .saturating_add(prepared_targets as u64);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_cycle_count = cycle_count;
        self.snapshot.last_prework_service_budget_per_cycle = Some(budget_per_cycle);
        self.snapshot.last_prework_service_prepared_targets = prepared_targets;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn record_prework_service_request(
        &mut self,
        requested_cycles: usize,
        effective_cycles: usize,
        requested_budget_per_cycle: usize,
        effective_budget_per_cycle: usize,
    ) {
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = effective_cycles;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(effective_budget_per_cycle);
        if effective_cycles < requested_cycles
            || effective_budget_per_cycle < requested_budget_per_cycle
        {
            self.snapshot.prework_service_throttle_count = self
                .snapshot
                .prework_service_throttle_count
                .saturating_add(1);
        }
    }

    fn record_prework_service_yield(
        &mut self,
        processing_epoch: u64,
        requested_cycles: usize,
        requested_budget_per_cycle: usize,
    ) {
        self.snapshot.prework_service_yield_count =
            self.snapshot.prework_service_yield_count.saturating_add(1);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = 0;
        self.snapshot.last_prework_service_cycle_count = 0;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(0);
        self.snapshot.last_prework_service_prepared_targets = 0;
    }

    fn record_last_serviced_pending_target(&mut self, target: &RuntimePendingPreworkTarget) {
        self.snapshot.last_prework_serviced_target_block_sequence =
            Some(target.target_block_sequence);
        self.snapshot.last_prework_serviced_backlog_class = Some(target.backlog_class);
    }

    fn retire_prework_entry(
        &mut self,
        cache: RuntimeEnginePreworkCache,
        reason: RuntimePreworkInvalidationReason,
    ) {
        self.snapshot.prework_cache_invalidation_count = self
            .snapshot
            .prework_cache_invalidation_count
            .saturating_add(1);
        self.snapshot.last_prework_invalidation_reason = Some(reason);
        let retirement_reason = self.retirement_reason_from_invalidation(reason);
        let retired_unconsumed = cache.consumption_count == 0;
        self.snapshot.prework_cache_retirement_count = self
            .snapshot
            .prework_cache_retirement_count
            .saturating_add(1);
        if retired_unconsumed {
            self.snapshot.prework_cache_unconsumed_retirement_count = self
                .snapshot
                .prework_cache_unconsumed_retirement_count
                .saturating_add(1);
        } else {
            self.snapshot.prework_cache_consumed_retirement_count = self
                .snapshot
                .prework_cache_consumed_retirement_count
                .saturating_add(1);
        }
        self.snapshot.last_prework_retirement_reason = Some(retirement_reason);
        self.snapshot.last_prework_retired_unconsumed = Some(retired_unconsumed);
        self.snapshot.last_prework_retirement_processing_epoch =
            Some(cache.source_processing_epoch);
        self.snapshot.last_prework_retirement_block_sequence = Some(cache.source_block_sequence);
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
    }

    fn retire_prework_entries_matching(
        &mut self,
        mut should_retire: impl FnMut(&RuntimeEnginePreworkCache) -> bool,
        reason: RuntimePreworkInvalidationReason,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            if should_retire(&self.prework_queue[index]) {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn pending_target_matches(
        pending: &RuntimePendingPreworkTarget,
        target: &RuntimePreworkWindowTarget,
    ) -> bool {
        pending.target_block_sequence == target.target_block_sequence
            && pending.admitted_from_block_sequence == target.admitted_from_block_sequence
            && pending.parameter_epoch_override == target.parameter_epoch_override
            && pending.transport_override == target.transport_override
            && pending.input_signature == hash_audio_buffer(&target.buffer)
            && pending.buffer.frames() == target.buffer.frames()
            && pending.buffer.channel_count() == target.buffer.channel_count()
    }

    fn prepared_target_matches(
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        target: &RuntimePreworkWindowTarget,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) -> bool {
        let transport = target.transport_override.or(applied_transport);
        cache.graph_id == graph_id
            && cache.source_block_sequence == target.target_block_sequence
            && cache.admitted_from_block_sequence == target.admitted_from_block_sequence
            && cache.projection_epoch == projection_epoch
            && cache.parameter_epoch
                == target
                    .parameter_epoch_override
                    .unwrap_or(latest_parameter_epoch)
            && cache.transport.playing == transport.map(|t| t.playing).unwrap_or(false)
            && cache.transport.tempo_bpm == transport.map(|t| t.tempo_bpm).unwrap_or(0.0)
            && cache.transport.timeline_position_samples
                == transport.map(|t| t.timeline_position_samples).unwrap_or(0)
            && cache.block_size == block_size
            && cache.frame_count == target.buffer.frames().0
            && cache.channel_count == target.buffer.channel_count().0
            && cache.input_signature == hash_audio_buffer(&target.buffer)
    }

    fn reconcile_pending_prework_targets(
        &mut self,
        targets: &[RuntimePreworkWindowTarget],
        graph_id: Option<&str>,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) {
        self.pending_prework_targets.retain(|pending| {
            targets
                .iter()
                .any(|target| Self::pending_target_matches(pending, target))
        });

        for target in targets {
            let already_prepared = graph_id.is_some_and(|graph_id| {
                self.prework_queue.iter().any(|cache| {
                    Self::prepared_target_matches(
                        cache,
                        graph_id,
                        target,
                        projection_epoch,
                        latest_parameter_epoch,
                        applied_transport,
                        block_size,
                    )
                })
            });
            let already_pending = self
                .pending_prework_targets
                .iter()
                .any(|pending| Self::pending_target_matches(pending, target));
            if !already_prepared && !already_pending {
                self.pending_prework_targets
                    .push_back(RuntimePendingPreworkTarget {
                        target_block_sequence: target.target_block_sequence,
                        admitted_from_block_sequence: target.admitted_from_block_sequence,
                        input_signature: hash_audio_buffer(&target.buffer),
                        buffer: target.buffer.clone(),
                        backlog_class: Self::classify_prework_backlog_class(
                            target.target_block_sequence,
                            target.admitted_from_block_sequence,
                        ),
                        parameter_epoch_override: target.parameter_epoch_override,
                        transport_override: target.transport_override,
                    });
            }
        }

        self.pending_prework_targets
            .make_contiguous()
            .sort_by_key(|target| (target.backlog_class, target.target_block_sequence));
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn take_pending_prework_targets(
        &mut self,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Vec<RuntimePendingPreworkTarget> {
        let mut drained = Vec::with_capacity(budget.min(self.pending_prework_targets.len()));
        let mut retained = VecDeque::with_capacity(self.pending_prework_targets.len());
        while let Some(target) = self.pending_prework_targets.pop_front() {
            if drained.len() < budget && target.backlog_class <= max_backlog_class {
                drained.push(target);
            } else {
                retained.push_back(target);
            }
        }
        self.pending_prework_targets = retained;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        drained
    }

    fn matching_prework_index(
        &self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<usize> {
        self.prework_queue
            .iter()
            .enumerate()
            .rev()
            .find(|(_, cache)| {
                self.prework_cache_matches(cache, graph_id, context, buffer, input_signature)
            })
            .map(|(index, _)| index)
    }

    fn retire_unready_or_mismatched_prework_for_current_block(
        &mut self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            let maybe_reason = {
                let cache = &self.prework_queue[index];
                if context.processing_epoch > cache.valid_until_processing_epoch {
                    Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
                } else if cache.source_block_sequence > context.block_sequence {
                    None
                } else {
                    self.prework_cache_mismatch_reason(
                        cache,
                        graph_id,
                        context,
                        buffer,
                        input_signature,
                    )
                }
            };

            if let Some(reason) = maybe_reason {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn apply_graph_projection(
        &mut self,
        projection: &GraphProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        if projection.node_count != projection.nodes.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph node_count must match node projection count",
            ));
        }
        if projection
            .nodes
            .iter()
            .any(|node| node.node_id.is_empty() || node.stages.is_empty())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph nodes must have non-empty ids and at least one stage",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(node.execution_class, GraphNodeExecutionClass::PureTransform)
                && node.latency_samples != 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "pure-transform graph nodes must report zero latency",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(
                node.execution_class,
                GraphNodeExecutionClass::LatencyBearing
            ) && node.latency_samples == 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "latency-bearing graph nodes must report non-zero latency",
            ));
        }

        self.graph = Some(ExecutableGraph::new(
            projection.graph_id.clone(),
            projection
                .nodes
                .iter()
                .map(|node| GraphNodeSpec {
                    node_id: node.node_id.clone(),
                    execution_class: node.execution_class,
                    latency_samples: node.latency_samples,
                    tail_samples: 0,
                    buffer_contract: GraphNodeBufferContract::default(),
                    topology: GraphNodeTopologyMetadata::default(),
                    stages: node.stages.clone(),
                })
                .collect(),
        ));
        self.plugin_node_bindings.clear();
        self.pending_plugin_node_renders.clear();
        self.invalidate_prework_cache(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    fn apply_graph_contract_projection(
        &mut self,
        projection: &GraphContractProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot apply graph node contracts before a graph is applied",
            ));
        };
        if projection.contract_count != projection.nodes.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph contract_count must match node contract projection count",
            ));
        }
        if projection.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph contract projection graph_id must match the active graph",
            ));
        }
        if projection
            .nodes
            .iter()
            .any(|node| node.node_id.trim().is_empty())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph node contracts must reference non-empty node ids",
            ));
        }

        let plan = graph.plan().clone();
        let known_node_ids = plan
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut seen_contract_nodes = BTreeSet::new();
        for node in &projection.nodes {
            if !known_node_ids.contains(node.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "graph node contract references unknown node '{}'",
                        node.node_id
                    ),
                ));
            }
            if !seen_contract_nodes.insert(node.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("graph node contract repeats node '{}'", node.node_id),
                ));
            }
        }

        let contract_by_node = projection
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect::<HashMap<_, _>>();

        self.graph = Some(ExecutableGraph::new(
            plan.graph_id,
            plan.nodes
                .into_iter()
                .map(|mut node| {
                    if let Some(contract) = contract_by_node.get(node.node_id.as_str()) {
                        node.buffer_contract = GraphNodeBufferContract {
                            input: signal_graph::GraphNodeBusEndpoint::new(
                                contract.buffer_contract.input.bus_id.clone(),
                                contract.buffer_contract.input.channels,
                            ),
                            output: signal_graph::GraphNodeBusEndpoint::new(
                                contract.buffer_contract.output.bus_id.clone(),
                                contract.buffer_contract.output.channels,
                            ),
                            scratch_buffers: contract.buffer_contract.scratch_buffers,
                            silence_policy: contract.buffer_contract.silence_policy,
                            channel_adaptation: contract.buffer_contract.channel_adaptation,
                            reset_policy: contract.buffer_contract.reset_policy,
                        };
                        node.topology = GraphNodeTopologyMetadata {
                            role: contract.topology.role,
                            lane_id: contract.topology.lane_id.clone(),
                            bus_group_id: contract.topology.bus_group_id.clone(),
                        };
                    }
                    node
                })
                .collect(),
        ));
        self.pending_plugin_node_renders.clear();
        self.invalidate_prework_cache(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: &PluginBackedNodeBindingProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot bind plugin-backed nodes before a graph is applied",
            ));
        };
        if projection.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "plugin-backed node bindings must target the currently applied graph",
            ));
        }

        let planning = graph.planning_summary(anticipative_enabled);
        let mut bindings = HashMap::new();
        for binding in &projection.bindings {
            if !planning.planned_nodes.iter().any(|node| {
                node.node_id == binding.node_id
                    && matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            }) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin-backed binding node '{}' does not resolve to a plugin-backed node",
                        binding.node_id
                    ),
                ));
            }
            if bindings
                .insert(binding.node_id.clone(), binding.sandbox_id.clone())
                .is_some()
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "duplicate plugin-backed binding provided for node '{}'",
                        binding.node_id
                    ),
                ));
            }
        }

        self.plugin_node_bindings = bindings;
        self.pending_plugin_node_renders.clear();
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    fn apply_plugin_node_render_batch(
        &mut self,
        batch: PluginNodeRenderBatch,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot apply plugin node renders before a graph is applied",
            ));
        };
        if batch.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "plugin node render batch must target the currently applied graph",
            ));
        }

        let planning = graph.planning_summary(true);
        let mut seen_node_ids = BTreeSet::new();
        for render in &batch.renders {
            if !planning.planned_nodes.iter().any(|node| {
                node.node_id == render.node_id
                    && matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            }) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin node render '{}' does not resolve to a plugin-backed node",
                        render.node_id
                    ),
                ));
            }
            if !seen_node_ids.insert(render.node_id.as_str()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("plugin node render batch repeats node '{}'", render.node_id),
                ));
            }
            if let Some(bound_sandbox_id) = self.plugin_node_bindings.get(&render.node_id) {
                if bound_sandbox_id != &render.sandbox_id {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "plugin node render '{}' is bound to sandbox '{}' not '{}'",
                            render.node_id, bound_sandbox_id, render.sandbox_id
                        ),
                    ));
                }
            } else {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin node render '{}' has no active plugin-backed binding",
                        render.node_id
                    ),
                ));
            }
        }

        self.pending_plugin_node_renders
            .insert((batch.processing_epoch, batch.block_sequence), batch);
        Ok(())
    }

    fn take_plugin_node_render_batch(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Option<PluginNodeRenderBatch> {
        self.pending_plugin_node_renders
            .remove(&(processing_epoch, block_sequence))
    }

    fn retire_stale_plugin_node_renders(&mut self, processing_epoch: u64, block_sequence: u64) {
        self.pending_plugin_node_renders
            .retain(|(render_epoch, render_block_sequence), _| {
                *render_epoch > processing_epoch
                    || (*render_epoch == processing_epoch
                        && *render_block_sequence >= block_sequence)
            });
    }

    fn refresh_planning(&mut self, anticipative_enabled: bool) {
        if !anticipative_enabled {
            self.invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if let Some(graph) = self.graph.as_ref() {
            let planning = graph.planning_summary(anticipative_enabled);
            let contract = graph.contract_summary();
            let contract_by_node = contract
                .node_contracts
                .iter()
                .map(|node| (node.node_id.as_str(), node))
                .collect::<BTreeMap<_, _>>();
            self.snapshot.graph_id = Some(graph.graph_id().to_string());
            self.snapshot.node_count = graph.node_count();
            self.snapshot.stateful_node_count = graph.stateful_node_count();
            self.snapshot.latency_node_count = graph.latency_node_count();
            self.snapshot.plugin_backed_node_count = graph.plugin_backed_node_count();
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = planning.inline_realtime_node_count;
            self.snapshot.stateful_realtime_node_count = planning.stateful_realtime_node_count;
            self.snapshot.anticipative_eligible_node_count =
                planning.anticipative_eligible_node_count;
            self.snapshot.phase_count = planning.phase_count;
            self.snapshot.anticipative_phase_count = planning.anticipative_phase_count;
            self.snapshot.phase_order = planning.phase_order.clone();
            self.snapshot.lane_count = planning.lane_count;
            self.snapshot.anticipative_lane_count = planning.anticipative_lane_count;
            self.snapshot.lane_order = planning.lane_order.clone();
            self.snapshot.dispatch_count = planning.dispatch_count;
            self.snapshot.dispatch_boundary_count = planning.dispatch_boundary_count;
            self.snapshot.dispatch_order = planning
                .dispatches
                .iter()
                .map(|dispatch| dispatch.lane)
                .collect();
            self.snapshot.prepared_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
                .count();
            self.snapshot.realtime_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Realtime)
                .count();
            self.snapshot.dispatch_handoff_count = usize::from(
                self.snapshot.prepared_dispatch_count > 0
                    && self.snapshot.realtime_dispatch_count > 0,
            );
            self.snapshot.prework_cache_enabled = self.snapshot.prepared_dispatch_count > 0;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
            self.snapshot.prework_cache_state = if !self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Disabled
            } else if !self.prework_queue.is_empty() {
                match self.snapshot.prework_cache_state {
                    RuntimePreworkCacheState::Consumed => RuntimePreworkCacheState::Consumed,
                    RuntimePreworkCacheState::Admitted => RuntimePreworkCacheState::Admitted,
                    _ => RuntimePreworkCacheState::Admitted,
                }
            } else if matches!(
                self.snapshot.prework_cache_state,
                RuntimePreworkCacheState::Invalidated
            ) {
                RuntimePreworkCacheState::Invalidated
            } else {
                RuntimePreworkCacheState::Empty
            };
            self.snapshot.last_prework_cache_hit = false;
            let latest = self.prework_queue.back();
            self.snapshot.prework_cache_freshness_state =
                self.prework_freshness_state(latest, None);
            self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
                cache
                    .valid_until_block_sequence
                    .saturating_sub(cache.source_block_sequence)
            });
            self.snapshot.prework_cache_valid_until_processing_epoch =
                latest.map(|cache| cache.valid_until_processing_epoch);
            self.snapshot.prework_cache_valid_until_block_sequence =
                latest.map(|cache| cache.valid_until_block_sequence);
            self.snapshot.last_prework_source_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                latest.map(|cache| cache.admitted_from_block_sequence);
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
            self.snapshot.prework_cache_peak_queue_depth = self
                .snapshot
                .prework_cache_peak_queue_depth
                .max(self.prework_queue.len());
            self.snapshot.planned_nodes = planning
                .planned_nodes
                .into_iter()
                .map(|node| crate::interfaces::RuntimePlannedGraphNode {
                    topology_role: contract_by_node
                        .get(node.node_id.as_str())
                        .map(|contract| contract.topology_role)
                        .unwrap_or(GraphNodeTopologyRole::Utility),
                    lane_id: contract_by_node
                        .get(node.node_id.as_str())
                        .and_then(|contract| contract.lane_id.clone()),
                    bus_group_id: contract_by_node
                        .get(node.node_id.as_str())
                        .and_then(|contract| contract.bus_group_id.clone()),
                    input_bus_id: contract_by_node
                        .get(node.node_id.as_str())
                        .map(|contract| contract.input_bus_id.clone())
                        .unwrap_or_else(|| "main:in".into()),
                    output_bus_id: contract_by_node
                        .get(node.node_id.as_str())
                        .map(|contract| contract.output_bus_id.clone())
                        .unwrap_or_else(|| "main:out".into()),
                    plugin_sandbox_id: self.plugin_node_bindings.get(&node.node_id).cloned(),
                    node_id: node.node_id,
                    execution_class: node.execution_class,
                    group: node.group,
                    latency_samples: node.latency_samples,
                })
                .collect();
            self.snapshot.stage_count = graph.stage_count();
            self.snapshot.dynamic_kernel_stage_count = graph.dynamic_kernel_stage_count();
            self.snapshot.dynamic_stage_state_model = graph.dynamic_stage_state_model();
            self.snapshot.total_latency_samples = graph.total_latency_samples();
            self.snapshot.max_node_latency_samples = graph.max_node_latency_samples();
            self.snapshot.total_tail_samples = graph.total_tail_samples();
            self.snapshot.max_node_tail_samples = graph.max_node_tail_samples();
        } else {
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = 0;
            self.snapshot.stateful_realtime_node_count = 0;
            self.snapshot.anticipative_eligible_node_count = 0;
            self.snapshot.plugin_backed_node_count = 0;
            self.snapshot.phase_count = 0;
            self.snapshot.anticipative_phase_count = 0;
            self.snapshot.phase_order.clear();
            self.snapshot.lane_count = 0;
            self.snapshot.anticipative_lane_count = 0;
            self.snapshot.lane_order.clear();
            self.snapshot.dispatch_count = 0;
            self.snapshot.dispatch_boundary_count = 0;
            self.snapshot.dispatch_order.clear();
            self.snapshot.prepared_dispatch_count = 0;
            self.snapshot.realtime_dispatch_count = 0;
            self.snapshot.dispatch_handoff_count = 0;
            self.snapshot.stage_count = 0;
            self.snapshot.dynamic_kernel_stage_count = 0;
            self.snapshot.dynamic_stage_state_model =
                signal_graph::GraphDynamicStageStateModel::RebuiltPerBlock;
            self.snapshot.total_latency_samples = 0;
            self.snapshot.max_node_latency_samples = 0;
            self.snapshot.total_tail_samples = 0;
            self.snapshot.max_node_tail_samples = 0;
            self.snapshot.output_tail_samples = 0;
            self.snapshot.max_bus_tail_samples = 0;
            self.snapshot.prework_cache_enabled = false;
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
            self.snapshot.last_prework_cache_hit = false;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Disabled;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_cache_queue_depth = 0;
            self.snapshot.prework_pending_target_count = 0;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            self.snapshot.last_prework_consumption_processing_epoch = None;
            self.snapshot.last_prework_consumption_block_sequence = None;
            self.snapshot.last_prework_consumed_from_block_sequence = None;
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.planned_nodes.clear();
            self.plugin_node_bindings.clear();
            self.prework_queue.clear();
            self.pending_prework_targets.clear();
        }
    }

    fn process_block(
        &mut self,
        context: GraphExecutionContext,
        transport: Option<TransportProjection>,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        let graph_id = self
            .graph
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?
            .graph_id()
            .to_string();
        self.retire_stale_plugin_node_renders(context.processing_epoch, context.block_sequence);

        let input_signature = hash_audio_buffer(&buffer);
        self.retire_unready_or_mismatched_prework_for_current_block(
            graph_id.as_str(),
            &context,
            &buffer,
            input_signature,
        );

        let cache_hit_index =
            self.matching_prework_index(graph_id.as_str(), &context, &buffer, input_signature);
        let cache_hit = cache_hit_index.is_some();

        let prepared = if cache_hit {
            self.snapshot.prework_cache_hits = self.snapshot.prework_cache_hits.saturating_add(1);
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            if self.prework_queue[cache_hit_index.expect("cache hit index present")]
                .admitted_from_block_sequence
                < context.block_sequence
            {
                self.snapshot.prework_cache_queued_consumptions = self
                    .snapshot
                    .prework_cache_queued_consumptions
                    .saturating_add(1);
            }
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            let cache = &mut self.prework_queue[cache_hit_index.expect("cache hit index present")];
            self.snapshot.last_prework_source_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            self.snapshot.last_prework_consumed_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            cache.consumption_count = cache.consumption_count.saturating_add(1);
            Some(cache.prepared.clone())
        } else {
            let anticipative_dispatches_present = self
                .graph
                .as_ref()
                .map(|graph| {
                    graph
                        .planning_summary(context.anticipative_enabled)
                        .dispatches
                        .iter()
                        .any(|dispatch| {
                            dispatch.lane == signal_graph::GraphExecutionLane::Anticipative
                        })
                })
                .unwrap_or(false);
            let planning = self
                .graph
                .as_ref()
                .map(|graph| graph.planning_summary(context.anticipative_enabled))
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "no executable graph has been applied",
                    )
                })?;
            if anticipative_dispatches_present {
                self.snapshot.prework_cache_misses =
                    self.snapshot.prework_cache_misses.saturating_add(1);
            }
            let admitted = self.admit_prework_for_block(
                context.clone(),
                transport,
                context.block_sequence,
                buffer.clone(),
            )?;
            let prepared = if admitted {
                self.prework_queue
                    .iter()
                    .rev()
                    .find(|cache| cache.source_block_sequence == context.block_sequence)
                    .map(|cache| cache.prepared.clone())
            } else {
                None
            };
            self.snapshot.prework_cache_state = if !self.prework_queue.is_empty() {
                RuntimePreworkCacheState::Admitted
            } else if planning
                .dispatches
                .iter()
                .any(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
            {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            prepared
        };
        let prepared_was_used = prepared.is_some();
        let plugin_node_renders = self
            .take_plugin_node_render_batch(context.processing_epoch, context.block_sequence)
            .map(|batch| {
                batch
                    .renders
                    .into_iter()
                    .map(|render| GraphNodeRenderOverride {
                        node_id: render.node_id,
                        buffer: render.output,
                        latency_samples: render.latency_samples,
                        tail_samples: render.tail_samples,
                        bypassed: render.bypassed,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let planning = graph.planning_summary(context.anticipative_enabled);
        let contract = graph.contract_summary();
        let routing = graph.routing_summary();

        let (
            output,
            GraphBlockReport {
                graph_id,
                context,
                node_count,
                stateful_node_count,
                latency_node_count,
                plugin_backed_node_count,
                phase_count,
                anticipative_phase_count,
                phase_order,
                lane_count,
                anticipative_lane_count,
                lane_order,
                dispatch_count,
                dispatch_boundary_count,
                dispatch_order,
                prepared_dispatch_count,
                realtime_dispatch_count,
                dispatch_handoff_count,
                stage_count,
                dynamic_kernel_stage_count,
                dynamic_stage_state_model,
                total_latency_samples,
                max_node_latency_samples,
                total_tail_samples,
                max_node_tail_samples,
                output_tail_samples,
                max_bus_tail_samples,
                frame_count,
                channel_count,
                input_peak,
                prework_output_peak,
                realtime_input_peak,
                output_peak,
                output_rms,
                first_output_sample,
                ..
            },
        ) = graph.execute_realtime_from_prepared_with_node_overrides(
            &buffer,
            peak_abs(buffer.samples()),
            prepared,
            context,
            None,
            &planning,
            &contract,
            &routing,
            &plugin_node_renders,
        );

        self.snapshot.graph_id = Some(graph_id);
        self.snapshot.node_count = node_count;
        self.snapshot.stateful_node_count = stateful_node_count;
        self.snapshot.latency_node_count = latency_node_count;
        self.snapshot.plugin_backed_node_count = plugin_backed_node_count;
        self.snapshot.phase_count = phase_count;
        self.snapshot.anticipative_phase_count = anticipative_phase_count;
        self.snapshot.phase_order = phase_order;
        self.snapshot.lane_count = lane_count;
        self.snapshot.anticipative_lane_count = anticipative_lane_count;
        self.snapshot.lane_order = lane_order;
        self.snapshot.dispatch_count = dispatch_count;
        self.snapshot.dispatch_boundary_count = dispatch_boundary_count;
        self.snapshot.dispatch_order = dispatch_order;
        self.snapshot.prepared_dispatch_count = prepared_dispatch_count;
        self.snapshot.realtime_dispatch_count = realtime_dispatch_count;
        self.snapshot.dispatch_handoff_count = dispatch_handoff_count;
        self.snapshot.prework_cache_enabled = prepared_dispatch_count > 0;
        if !self.snapshot.prework_cache_enabled {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
        } else if !self.prework_queue.is_empty() {
            self.snapshot.prework_cache_state = if cache_hit {
                RuntimePreworkCacheState::Consumed
            } else {
                RuntimePreworkCacheState::Admitted
            };
        } else if !matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
        }
        self.snapshot.last_prework_cache_hit = cache_hit;
        self.snapshot.prework_cache_block_freshness_window = PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        if !cache_hit && self.prework_queue.is_empty() {
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
        }
        if !cache_hit && prepared_was_used {
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            if let Some(cache) = self
                .prework_queue
                .iter_mut()
                .rev()
                .find(|cache| cache.source_block_sequence == context.block_sequence)
            {
                self.snapshot.last_prework_source_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_source_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admission_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_admission_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admitted_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                self.snapshot.last_prework_consumed_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                cache.consumption_count = cache.consumption_count.saturating_add(1);
            }
        } else if !cache_hit {
            self.snapshot.last_prework_consumed_from_block_sequence = None;
        }
        self.snapshot.stage_count = stage_count;
        self.snapshot.dynamic_kernel_stage_count = dynamic_kernel_stage_count;
        self.snapshot.dynamic_stage_state_model = dynamic_stage_state_model;
        self.snapshot.total_latency_samples = total_latency_samples;
        self.snapshot.max_node_latency_samples = max_node_latency_samples;
        self.snapshot.total_tail_samples = total_tail_samples;
        self.snapshot.max_node_tail_samples = max_node_tail_samples;
        self.snapshot.output_tail_samples = output_tail_samples;
        self.snapshot.max_bus_tail_samples = max_bus_tail_samples;
        self.snapshot.processed_blocks = self.snapshot.processed_blocks.saturating_add(1);
        self.snapshot.last_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_block_sequence = Some(context.block_sequence);
        self.snapshot.last_frame_count = frame_count;
        self.snapshot.last_channel_count = channel_count;
        self.snapshot.last_input_peak = Some(input_peak);
        self.snapshot.last_prework_output_peak = prework_output_peak;
        self.snapshot.last_realtime_input_peak = realtime_input_peak;
        self.snapshot.last_output_peak = Some(output_peak);
        self.snapshot.last_output_rms = Some(output_rms);
        self.snapshot.last_first_output_sample = first_output_sample;
        self.snapshot.last_execution_context = Some(context);

        Ok(RuntimeEngineBlockResult {
            snapshot: self.snapshot.clone(),
            output,
        })
    }

    fn admit_prework_for_block(
        &mut self,
        context: GraphExecutionContext,
        transport: Option<TransportProjection>,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        let (graph_id, planning) = {
            let graph = self.graph.as_ref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?;
            (
                graph.graph_id().to_string(),
                graph.planning_summary(context.anticipative_enabled),
            )
        };
        if planning
            .dispatches
            .iter()
            .all(|dispatch| dispatch.lane != signal_graph::GraphExecutionLane::Anticipative)
        {
            self.snapshot.prework_cache_state = if context.anticipative_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        }

        let input_signature = hash_audio_buffer(&buffer);
        let already_matching = self.prework_queue.iter().any(|cache| {
            self.prework_cache_matches(cache, graph_id.as_str(), &context, &buffer, input_signature)
        });
        if already_matching {
            return Ok(true);
        }
        self.retire_prework_entries_matching(
            |cache| cache.source_block_sequence == context.block_sequence,
            RuntimePreworkInvalidationReason::SupersededByAdmission,
        );

        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let Some(prepared) = graph.prepare_anticipative(&buffer, &context, None) else {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        };

        self.snapshot.prework_cache_admissions =
            self.snapshot.prework_cache_admissions.saturating_add(1);
        if admitted_from_block_sequence < context.block_sequence {
            self.snapshot.prework_cache_queued_admissions = self
                .snapshot
                .prework_cache_queued_admissions
                .saturating_add(1);
        }
        self.snapshot.last_prework_admission_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_admission_block_sequence = Some(context.block_sequence);
        self.snapshot.last_prework_admitted_from_block_sequence =
            Some(admitted_from_block_sequence);
        self.prework_queue.push_back(RuntimeEnginePreworkCache {
            graph_id,
            projection_epoch: context.projection_epoch,
            parameter_epoch: context.parameter_epoch,
            transport: transport.unwrap_or_else(|| transport_projection_from_context(&context)),
            block_size: context.configured_block_size,
            frame_count: buffer.frames().0,
            channel_count: buffer.channel_count().0,
            input_signature,
            prepared,
            valid_until_processing_epoch: context.processing_epoch.saturating_add(1),
            valid_until_block_sequence: context
                .block_sequence
                .saturating_add(PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW),
            source_processing_epoch: context.processing_epoch,
            source_block_sequence: context.block_sequence,
            admitted_from_block_sequence,
            consumption_count: 0,
        });
        self.prework_queue
            .make_contiguous()
            .sort_by_key(|cache| cache.source_block_sequence);
        while self.prework_queue.len() > PREWORK_QUEUE_CAPACITY {
            let cache = self.prework_queue.pop_front().expect("queue not empty");
            self.retire_prework_entry(
                cache,
                RuntimePreworkInvalidationReason::QueueCapacityExceeded,
            );
        }
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Admitted;
        self.snapshot.last_prework_source_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_source_block_sequence = Some(context.block_sequence);
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(true)
    }

    fn snapshot(&self) -> RuntimeEngineBlockSnapshot {
        self.snapshot.clone()
    }

    fn prework_cache_matches(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> bool {
        context.anticipative_enabled
            && cache.graph_id == graph_id
            && cache.projection_epoch == context.projection_epoch
            && cache.parameter_epoch == context.parameter_epoch
            && cache.transport.playing == context.transport_playing
            && cache.transport.tempo_bpm == context.transport_tempo_bpm
            && cache.transport.timeline_position_samples == context.timeline_position_samples
            && cache.block_size == context.configured_block_size
            && cache.frame_count == buffer.frames().0
            && cache.channel_count == buffer.channel_count().0
            && cache.input_signature == input_signature
            && context.processing_epoch <= cache.valid_until_processing_epoch
            && context.block_sequence <= cache.valid_until_block_sequence
    }

    fn prework_cache_mismatch_reason(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<RuntimePreworkInvalidationReason> {
        if !context.anticipative_enabled {
            return Some(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if cache.graph_id != graph_id || cache.projection_epoch != context.projection_epoch {
            return Some(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        }
        if cache.parameter_epoch != context.parameter_epoch {
            return Some(RuntimePreworkInvalidationReason::ParameterBatchApplied);
        }
        if cache.transport.playing != context.transport_playing
            || cache.transport.tempo_bpm != context.transport_tempo_bpm
            || cache.transport.timeline_position_samples != context.timeline_position_samples
        {
            return Some(classify_transport_invalidation_reason(
                Some(cache.transport),
                transport_projection_from_context(context),
            ));
        }
        if context.processing_epoch > cache.valid_until_processing_epoch {
            return Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired);
        }
        if context.block_sequence > cache.valid_until_block_sequence {
            return Some(RuntimePreworkInvalidationReason::BlockSequenceExpired);
        }
        if cache.block_size != context.configured_block_size
            || cache.frame_count != buffer.frames().0
            || cache.channel_count != buffer.channel_count().0
            || cache.input_signature != input_signature
        {
            return Some(RuntimePreworkInvalidationReason::InputSignatureChanged);
        }
        None
    }

    fn invalidate_prework_cache(&mut self, reason: RuntimePreworkInvalidationReason) {
        if !self.prework_queue.is_empty() {
            let drained = self.prework_queue.drain(..).collect::<Vec<_>>();
            for cache in drained {
                self.retire_prework_entry(cache, reason);
            }
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = 0;
        } else if self.snapshot.prework_cache_enabled {
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = Some(reason);
        }
    }

    fn retirement_reason_from_invalidation(
        &self,
        reason: RuntimePreworkInvalidationReason,
    ) -> RuntimePreworkRetirementReason {
        match reason {
            RuntimePreworkInvalidationReason::RuntimeReconfigured => {
                RuntimePreworkRetirementReason::RuntimeReconfigured
            }
            RuntimePreworkInvalidationReason::RuntimeStopped => {
                RuntimePreworkRetirementReason::RuntimeStopped
            }
            RuntimePreworkInvalidationReason::ForecastPlanChanged => {
                RuntimePreworkRetirementReason::ForecastPlanChanged
            }
            RuntimePreworkInvalidationReason::PlanningDisabled => {
                RuntimePreworkRetirementReason::PlanningDisabled
            }
            RuntimePreworkInvalidationReason::GraphProjectionChanged => {
                RuntimePreworkRetirementReason::GraphProjectionChanged
            }
            RuntimePreworkInvalidationReason::TransportStarted => {
                RuntimePreworkRetirementReason::TransportStarted
            }
            RuntimePreworkInvalidationReason::TransportStopped => {
                RuntimePreworkRetirementReason::TransportStopped
            }
            RuntimePreworkInvalidationReason::TransportSeeked => {
                RuntimePreworkRetirementReason::TransportSeeked
            }
            RuntimePreworkInvalidationReason::TransportTempoChanged => {
                RuntimePreworkRetirementReason::TransportTempoChanged
            }
            RuntimePreworkInvalidationReason::TransportLoopStateChanged => {
                RuntimePreworkRetirementReason::TransportLoopStateChanged
            }
            RuntimePreworkInvalidationReason::TransportLoopWrapped => {
                RuntimePreworkRetirementReason::TransportLoopWrapped
            }
            RuntimePreworkInvalidationReason::ParameterBatchApplied => {
                RuntimePreworkRetirementReason::ParameterBatchApplied
            }
            RuntimePreworkInvalidationReason::InputSignatureChanged => {
                RuntimePreworkRetirementReason::InputSignatureChanged
            }
            RuntimePreworkInvalidationReason::ProcessingEpochExpired => {
                RuntimePreworkRetirementReason::ProcessingEpochExpired
            }
            RuntimePreworkInvalidationReason::BlockSequenceExpired => {
                RuntimePreworkRetirementReason::BlockSequenceExpired
            }
            RuntimePreworkInvalidationReason::SupersededByAdmission => {
                RuntimePreworkRetirementReason::SupersededByAdmission
            }
            RuntimePreworkInvalidationReason::PlanningWindowRevised => {
                RuntimePreworkRetirementReason::PlanningWindowRevised
            }
            RuntimePreworkInvalidationReason::QueueCapacityExceeded => {
                RuntimePreworkRetirementReason::QueueCapacityExceeded
            }
        }
    }

    fn prework_freshness_state(
        &self,
        cache: Option<&RuntimeEnginePreworkCache>,
        current_block_sequence: Option<u64>,
    ) -> RuntimePreworkFreshnessState {
        if !self.snapshot.prework_cache_enabled {
            return RuntimePreworkFreshnessState::Disabled;
        }
        if matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            return RuntimePreworkFreshnessState::Invalidated;
        }
        let Some(cache) = cache else {
            return RuntimePreworkFreshnessState::Empty;
        };
        let Some(current_block_sequence) = current_block_sequence else {
            return RuntimePreworkFreshnessState::Fresh;
        };
        let remaining = cache
            .valid_until_block_sequence
            .saturating_sub(current_block_sequence);
        match remaining {
            0 => RuntimePreworkFreshnessState::Exhausted,
            1 => RuntimePreworkFreshnessState::Expiring,
            _ => RuntimePreworkFreshnessState::Fresh,
        }
    }
}

impl core::fmt::Debug for SignalRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalRuntime")
            .field("config", &self.config)
            .field("readiness", &self.readiness)
            .field("safe_mode_enabled", &self.safe_mode_enabled)
            .field("anticipative_enabled", &self.anticipative_enabled)
            .field("active_output_device", &self.active_output_device)
            .field("applied_graph", &self.applied_graph)
            .field("applied_schedule", &self.applied_schedule)
            .field("applied_transport", &self.applied_transport)
            .field("latest_parameter_epoch", &self.latest_parameter_epoch)
            .field("projection_epoch", &self.projection_epoch)
            .field("control", &self.control)
            .field("timeline", &self.timeline)
            .field("automation", &self.automation)
            .field("engine", &self.engine)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}

impl SignalRuntime {
    fn summarize_plugin_backed_bindings(&self) -> RuntimePluginBackedBindingSummary {
        let bound_sandbox_ids = self
            .engine
            .snapshot
            .planned_nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .filter_map(|node| node.plugin_sandbox_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let mut summary = RuntimePluginBackedBindingSummary {
            bound_sandbox_ids,
            ..RuntimePluginBackedBindingSummary::default()
        };
        for sandbox_id in &summary.bound_sandbox_ids {
            let matching_states = self
                .transport_concurrency
                .active_sessions
                .values()
                .filter(|session| session.sandbox_id == *sandbox_id)
                .map(|session| session.state)
                .collect::<Vec<_>>();
            if matching_states
                .iter()
                .any(|state| matches!(state, TransportSessionState::AttachActive))
            {
                summary.active_bound_sandboxes += 1;
            } else if matching_states.iter().any(|state| {
                matches!(
                    state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            }) {
                summary.degraded_bound_sandboxes += 1;
            } else {
                summary.missing_bound_sandboxes += 1;
            }
        }
        summary
    }

    fn recompute_prework_service_policy_snapshot(&mut self) {
        let binding_summary = self.summarize_plugin_backed_bindings();
        let transport_condition = self.current_prework_transport_condition();
        let semantic_policy = self
            .engine
            .graph
            .as_ref()
            .map(|graph| {
                RuntimeEngineState::classify_prework_service_semantic_policy(
                    graph,
                    self.anticipative_enabled,
                    if !binding_summary.bound_sandbox_ids.is_empty() {
                        binding_summary.active_bound_sandboxes as u32
                            + binding_summary.degraded_bound_sandboxes as u32
                            + binding_summary.missing_bound_sandboxes as u32
                    } else {
                        self.diagnostics.active_plugin_sandboxes
                    },
                )
            })
            .unwrap_or(RuntimePreworkServiceSemanticPolicy::Balanced);
        let plugin_gate_active = matches!(
            semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        ) && self.engine.snapshot.prework_service_pressure
            != RuntimePreworkServicePressure::Normal
            && if !binding_summary.bound_sandbox_ids.is_empty() {
                binding_summary.degraded_bound_sandboxes > 0
                    || binding_summary.missing_bound_sandboxes > 0
                    || binding_summary.active_bound_sandboxes > 1
            } else {
                self.diagnostics.active_plugin_sandboxes > 1
            };
        let transport_gate_active =
            transport_condition.gate_active(self.engine.snapshot.prework_service_pressure);
        self.engine.snapshot.prework_service_semantic_policy = semantic_policy;
        self.engine.set_prework_service_plugin_state(
            self.diagnostics.active_plugin_sandboxes,
            binding_summary.bound_sandbox_ids.len(),
            binding_summary.active_bound_sandboxes,
            binding_summary.degraded_bound_sandboxes,
            binding_summary.missing_bound_sandboxes,
            plugin_gate_active,
        );
        self.engine.set_prework_service_transport_state(
            transport_condition.recovery_overlap_sessions,
            transport_condition.lingering_sessions,
            transport_condition.detach_faulted_sessions,
            transport_gate_active,
        );
    }

    fn current_prework_transport_condition(&self) -> RuntimePreworkTransportCondition {
        RuntimePreworkTransportCondition {
            recovery_overlap_sessions: self.transport_concurrency.recovery_overlap_session_count(),
            lingering_sessions: self.transport_concurrency.lingering_session_count(),
            detach_faulted_sessions: self.transport_concurrency.detach_faulted_session_count(),
        }
    }

    fn refresh_prework_service_policy_and_state(&mut self, processing_epoch: Option<u64>) {
        self.recompute_prework_service_policy_snapshot();
        self.reconcile_prework_service_state(processing_epoch);
    }

    fn refresh_scheduler_topology_summary(&mut self) {
        let Some(graph) = self.engine.graph.as_ref() else {
            self.engine.snapshot.scheduler_topology = RuntimeSchedulerTopologySummary::default();
            return;
        };

        let contract = graph.contract_summary();
        let mut track_lane_groups = BTreeSet::new();
        let mut bus_groups = BTreeSet::new();
        let mut send_return_groups = BTreeSet::new();
        let mut console_groups = BTreeSet::new();
        let mut missing_track_lane_ids = 0usize;
        let mut missing_bus_group_ids = 0usize;
        let mut missing_console_group_ids = 0usize;

        for node in &contract.node_contracts {
            match node.topology_role {
                GraphNodeTopologyRole::Utility => {}
                GraphNodeTopologyRole::TrackLane => {
                    if let Some(lane_id) = &node.lane_id {
                        track_lane_groups.insert(lane_id.clone());
                    } else {
                        missing_track_lane_ids = missing_track_lane_ids.saturating_add(1);
                    }
                }
                GraphNodeTopologyRole::Bus => {
                    if let Some(bus_group_id) = &node.bus_group_id {
                        bus_groups.insert(bus_group_id.clone());
                    } else {
                        missing_bus_group_ids = missing_bus_group_ids.saturating_add(1);
                    }
                }
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                    if let Some(bus_group_id) = &node.bus_group_id {
                        send_return_groups.insert(bus_group_id.clone());
                    } else {
                        missing_bus_group_ids = missing_bus_group_ids.saturating_add(1);
                    }
                }
                GraphNodeTopologyRole::ConsoleNode => {
                    if let Some(bus_group_id) = &node.bus_group_id {
                        console_groups.insert(bus_group_id.clone());
                    } else {
                        missing_console_group_ids = missing_console_group_ids.saturating_add(1);
                    }
                }
            }
        }

        let schedule_stream_count = self
            .applied_schedule
            .as_ref()
            .map(|schedule| schedule.stream_count);
        let has_topology_groups = contract.track_lane_node_count > 0
            || contract.bus_node_count > 0
            || contract.send_return_node_count > 0
            || contract.console_node_count > 0;
        let realtime_lane_index = self
            .engine
            .snapshot
            .lane_order
            .iter()
            .position(|lane| *lane == signal_graph::GraphExecutionLane::Realtime);
        let anticipative_lane_index = self
            .engine
            .snapshot
            .lane_order
            .iter()
            .position(|lane| *lane == signal_graph::GraphExecutionLane::Anticipative);

        let mut issues = Vec::new();
        if missing_track_lane_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingTrackLaneIds {
                node_count: missing_track_lane_ids,
            });
        }
        if missing_bus_group_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingBusGroupIds {
                node_count: missing_bus_group_ids,
            });
        }
        if missing_console_group_ids > 0 {
            issues.push(RuntimeSchedulerTopologyIssue::MissingConsoleGroupIds {
                node_count: missing_console_group_ids,
            });
        }
        if has_topology_groups && realtime_lane_index.is_none() {
            issues.push(RuntimeSchedulerTopologyIssue::MissingRealtimeLaneForTopology);
        }
        if let (Some(anticipative_index), Some(realtime_index)) =
            (anticipative_lane_index, realtime_lane_index)
        {
            if anticipative_index > realtime_index {
                issues.push(RuntimeSchedulerTopologyIssue::AnticipativeLaneMustPrecedeRealtime);
            }
        }
        if contract.console_node_count > 0
            && self.engine.snapshot.dispatch_order.last().copied()
                != Some(signal_graph::GraphExecutionLane::Realtime)
        {
            issues.push(RuntimeSchedulerTopologyIssue::RealtimeDispatchMustTerminateTopology);
        }
        if !track_lane_groups.is_empty() {
            match schedule_stream_count {
                Some(actual_streams) if actual_streams < track_lane_groups.len() => {
                    issues.push(RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
                        required_streams: track_lane_groups.len(),
                        actual_streams,
                    });
                }
                None => issues.push(
                    RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
                        required_streams: track_lane_groups.len(),
                    },
                ),
                _ => {}
            }
        }

        self.engine.snapshot.scheduler_topology = RuntimeSchedulerTopologySummary {
            track_lane_node_count: contract.track_lane_node_count,
            track_lane_group_count: track_lane_groups.len(),
            bus_node_count: contract.bus_node_count,
            bus_group_count: bus_groups.len(),
            send_return_node_count: contract.send_return_node_count,
            send_return_group_count: send_return_groups.len(),
            console_node_count: contract.console_node_count,
            console_group_count: console_groups.len(),
            schedule_stream_count,
            compatible: issues.is_empty(),
            requires_host_reinterpretation: !issues.is_empty(),
            issues,
        };
    }

    pub fn new(config: RuntimeConfig) -> Self {
        let mut runtime = Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            applied_graph: None,
            applied_schedule: None,
            applied_transport: None,
            prework_forecast_requested_mode: RuntimePreworkForecastMode::RuntimeRoleDefault,
            prework_forecast_mode: RuntimePreworkForecastMode::Disabled,
            prework_forecast_policy: None,
            prework_forecast_profile: None,
            prework_forecast_profile_source: None,
            latest_parameter_epoch: 0,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            timeline: RuntimeTimelineState::default(),
            automation: RuntimeAutomationState::default(),
            engine: RuntimeEngineState::default(),
            transport_concurrency: RuntimeTransportConcurrencyState::default(),
            recording_capture: RuntimeRecordingCaptureStateModel::default(),
            media_pipeline: RuntimeMediaPipelineStateModel::default(),
            warp_pipeline: RuntimeWarpPipelineStateModel::default(),
            diagnostics: RuntimeDiagnosticsSnapshot {
                cpu_load_percent: 0.0,
                xruns: 0,
                graph_latency_ms: 0.0,
                active_plugin_sandboxes: 0,
                backend_policy_tier: BackendPolicyTier::Tier0InHost,
            },
            supervision: RuntimeSupervisionState::default(),
            next_subscription: 1,
            sinks: Vec::new(),
        };
        runtime.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RuntimeRoleDefault,
        );
        runtime.set_prework_forecast_mode_internal(RuntimePreworkForecastMode::Disabled);
        runtime.recompute_prework_service_policy_snapshot();
        runtime
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    fn set_prework_forecast_policy_internal(
        &mut self,
        policy: Option<RuntimePreworkForecastPolicy>,
    ) {
        self.prework_forecast_policy = policy.clone();
        self.engine.snapshot.prework_forecast_policy_configured = policy.is_some();
        self.engine
            .snapshot
            .prework_forecast_policy_target_window_blocks =
            policy.as_ref().map(|policy| policy.target_window_blocks);
    }

    fn set_prework_forecast_requested_mode_internal(&mut self, mode: RuntimePreworkForecastMode) {
        self.prework_forecast_requested_mode = mode;
        self.engine.snapshot.prework_forecast_requested_mode = mode;
    }

    fn set_prework_forecast_mode_internal(&mut self, mode: RuntimePreworkForecastMode) {
        self.prework_forecast_mode = mode;
        self.engine.snapshot.prework_forecast_mode = mode;
    }

    fn reconcile_prework_service_state(&mut self, processing_epoch: Option<u64>) {
        let state = if !self.engine.snapshot.prework_cache_enabled
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            RuntimePreworkServiceState::Disabled
        } else if !self.control.running {
            RuntimePreworkServiceState::Paused
        } else if !self.engine.pending_prework_targets.is_empty()
            && (self.engine.snapshot.prework_service_plugin_gate_active
                || self.engine.snapshot.prework_service_transport_gate_active)
        {
            RuntimePreworkServiceState::Yielding
        } else if !self.engine.pending_prework_targets.is_empty() {
            RuntimePreworkServiceState::Pending
        } else {
            RuntimePreworkServiceState::Idle
        };
        self.engine
            .transition_prework_service_state(state, processing_epoch);
    }

    fn set_prework_forecast_profile_internal(
        &mut self,
        selection: Option<RuntimePreworkForecastProfileSelection>,
        source: Option<RuntimePreworkForecastProfileSource>,
    ) {
        self.prework_forecast_profile = selection;
        self.prework_forecast_profile_source = source;
        self.engine.snapshot.prework_forecast_profile =
            selection.map(|selection| selection.profile);
        self.engine.snapshot.prework_forecast_profile_source = source;
        self.engine
            .snapshot
            .prework_forecast_profile_target_window_override =
            selection.and_then(|selection| selection.target_window_blocks_override);
    }

    fn invalidate_prework_for_forecast_plan_change_if_needed(
        &mut self,
        previous_requested_mode: RuntimePreworkForecastMode,
        previous_effective_mode: RuntimePreworkForecastMode,
        previous_profile: Option<RuntimePreworkForecastProfileSelection>,
        previous_profile_source: Option<RuntimePreworkForecastProfileSource>,
        previous_policy: Option<RuntimePreworkForecastPolicy>,
    ) -> Result<(), RuntimeError> {
        let changed = previous_requested_mode != self.prework_forecast_requested_mode
            || previous_effective_mode != self.prework_forecast_mode
            || previous_profile != self.prework_forecast_profile
            || previous_profile_source != self.prework_forecast_profile_source
            || previous_policy != self.prework_forecast_policy;
        if !changed {
            return Ok(());
        }

        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            if !matches!(
                self.engine.snapshot.last_prework_invalidation_reason,
                Some(
                    RuntimePreworkInvalidationReason::PlanningDisabled
                        | RuntimePreworkInvalidationReason::RuntimeReconfigured
                )
            ) {
                self.engine.invalidate_prework_cache(
                    RuntimePreworkInvalidationReason::ForecastPlanChanged,
                );
            }
            return Ok(());
        }

        let Some(policy) = self.prework_forecast_policy.clone() else {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::ForecastPlanChanged);
            return Ok(());
        };
        if self.engine.prework_queue.is_empty() {
            let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        } else {
            self.reconcile_prework_queue_with_current_forecast_plan(&policy);
        }
        self.reconcile_prework_service_state(None);
        Ok(())
    }

    fn reconcile_prework_queue_with_current_forecast_plan(
        &mut self,
        policy: &RuntimePreworkForecastPolicy,
    ) {
        let current_block_sequence = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.admitted_from_block_sequence)
            .max()
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.source_processing_epoch)
            .max()
            .unwrap_or_else(|| {
                self.engine
                    .snapshot
                    .last_processing_epoch
                    .unwrap_or(self.projection_epoch)
            });
        let desired_sequences = (1..=policy.target_window_blocks)
            .map(|offset| current_block_sequence.saturating_add(offset as u64))
            .collect::<Vec<_>>();
        let projection_epoch = self.projection_epoch;
        let sample_rate = self.config.sample_rate;
        let block_size = self.config.graph.block_size;
        let retire_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let expected_loop_length_blocks = policy.transport_loop_length_blocks.max(1);
                let loop_end_samples =
                    (block_size.saturating_mul(expected_loop_length_blocks)) as i64;
                let expected_timeline_position_samples = ((cache.source_block_sequence as i64)
                    .saturating_mul(block_size as i64))
                .rem_euclid(loop_end_samples);
                let expected_parameter_epoch = projection_epoch
                    .saturating_add(cache.source_block_sequence)
                    .saturating_add(1);
                let expected_input_signature = hash_audio_buffer(&synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
                    cache
                        .source_block_sequence
                        .saturating_add(policy.buffer_seed_offset),
                ));
                let compatible = cache.projection_epoch == projection_epoch
                    && cache.parameter_epoch == expected_parameter_epoch
                    && cache.transport.playing == policy.transport_playing
                    && cache.transport.tempo_bpm == policy.transport_tempo_bpm
                    && cache.transport.timeline_position_samples
                        == expected_timeline_position_samples
                    && cache.block_size == block_size
                    && cache.frame_count == block_size
                    && cache.channel_count == 2
                    && cache.input_signature == expected_input_signature
                    && cache.source_block_sequence > cache.admitted_from_block_sequence
                    && cache.source_block_sequence
                        <= cache
                            .admitted_from_block_sequence
                            .saturating_add(policy.target_window_blocks as u64);
                (!desired_sequences.contains(&cache.source_block_sequence) || !compatible)
                    .then_some(cache.source_block_sequence)
            })
            .collect::<Vec<_>>();
        self.engine.retire_prework_entries_matching(
            |cache| retire_sequences.contains(&cache.source_block_sequence),
            RuntimePreworkInvalidationReason::ForecastPlanChanged,
        );

        let targets = desired_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
                    target_block_sequence.saturating_add(policy.buffer_seed_offset),
                ),
                parameter_epoch_override: Some(
                    self.forecast_parameter_batch_for_block(target_block_sequence, policy)
                        .epoch,
                ),
                transport_override: Some(
                    self.forecast_transport_projection_for_block(target_block_sequence, policy),
                ),
            })
            .collect::<Vec<_>>();
        let graph_id = self
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        self.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            self.projection_epoch,
            self.latest_parameter_epoch,
            self.applied_transport,
            block_size,
        );
        let _ = self.service_pending_prework_cycle(
            processing_epoch,
            policy.prepare_budget_per_cycle,
            RuntimePreworkBacklogClass::Deferred,
        );
    }

    fn maybe_rebuild_prework_window_from_current_forecast_plan(
        &mut self,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            return Ok(0);
        }
        if self.engine.graph.is_none() || !self.engine.snapshot.prework_cache_enabled {
            return Ok(0);
        }
        let Some(policy) = self.prework_forecast_policy.clone() else {
            return Ok(0);
        };
        let current_block_sequence = self
            .engine
            .snapshot
            .last_block_sequence
            .or_else(|| self.timeline.next_block_sequence.checked_sub(1))
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .snapshot
            .last_processing_epoch
            .unwrap_or(self.projection_epoch);
        let rebuilt = self.prime_engine_prework_window_with_forecast(
            processing_epoch,
            current_block_sequence,
            &policy,
        )?;
        self.reconcile_prework_service_state(Some(processing_epoch));
        Ok(rebuilt)
    }

    fn default_prework_forecast_profile_selection_for_runtime_profile(
        profile: RuntimeProfile,
    ) -> RuntimePreworkForecastProfileSelection {
        RuntimePreworkForecastProfileSelection {
            profile: match profile {
                RuntimeProfile::Local => RuntimePreworkForecastProfile::Local,
                RuntimeProfile::Server => RuntimePreworkForecastProfile::Server,
            },
            target_window_blocks_override: None,
        }
    }

    fn prework_forecast_policy_for_profile(
        selection: RuntimePreworkForecastProfileSelection,
    ) -> RuntimePreworkForecastPolicy {
        let mut policy = match selection.profile {
            RuntimePreworkForecastProfile::Local => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            },
            RuntimePreworkForecastProfile::Server => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 17,
                transport_playing: true,
                transport_tempo_bpm: 122.0,
                transport_loop_length_blocks: 24,
                parameter_target: "engine.server.balance".into(),
                parameter_cycle_length: 6,
            },
        };
        if let Some(target_window_blocks) = selection.target_window_blocks_override {
            policy.target_window_blocks = target_window_blocks;
        }
        policy
    }

    fn reconcile_prework_forecast_mode_state(&mut self) -> Result<(), RuntimeError> {
        match self.prework_forecast_requested_mode {
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                let selection =
                    Self::default_prework_forecast_profile_selection_for_runtime_profile(
                        self.config.profile,
                    );
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::ExplicitProfile => {
                let selection = self.prework_forecast_profile.ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "explicit forecast mode requires a stored forecast profile selection",
                    )
                })?;
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::RawPolicyOverride => {
                let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "raw forecast override mode requires a stored forecast policy",
                    )
                })?;
                self.set_prework_forecast_profile_internal(
                    None,
                    Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::Disabled => {}
        }

        let effective_mode = if self.anticipative_enabled {
            self.prework_forecast_requested_mode
        } else {
            RuntimePreworkForecastMode::Disabled
        };
        self.set_prework_forecast_mode_internal(effective_mode);
        Ok(())
    }

    fn set_prework_forecast_mode_state(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        if mode == RuntimePreworkForecastMode::Disabled
            && self.prework_forecast_mode != RuntimePreworkForecastMode::Disabled
        {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::PlanningDisabled);
        }
        self.set_prework_forecast_requested_mode_internal(mode);
        match mode {
            RuntimePreworkForecastMode::Disabled => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                self.refresh_prework_service_policy_and_state(None);
                Ok(())
            }
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                self.refresh_prework_service_policy_and_state(None);
                Ok(())
            }
            RuntimePreworkForecastMode::ExplicitProfile => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "explicit-profile forecast mode requires a profile selection",
            )),
            RuntimePreworkForecastMode::RawPolicyOverride => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "raw-policy forecast mode requires an explicit forecast policy",
            )),
        }
    }

    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::PluginSandboxChanged {
            active_sandboxes: self.diagnostics.active_plugin_sandboxes,
        });
    }

    pub fn apply_plugin_node_render_batch(
        &mut self,
        batch: PluginNodeRenderBatch,
    ) -> Result<(), RuntimeError> {
        self.engine.apply_plugin_node_render_batch(batch)
    }

    pub fn set_backend_policy_tier(&mut self, tier: BackendPolicyTier) {
        self.diagnostics.backend_policy_tier = tier;
    }

    pub fn set_cpu_load_percent(&mut self, cpu_load_percent: f32) {
        self.diagnostics.cpu_load_percent = cpu_load_percent.max(0.0);
    }

    pub fn set_graph_latency_ms(&mut self, graph_latency_ms: f32) {
        self.diagnostics.graph_latency_ms = graph_latency_ms.max(0.0);
    }

    pub fn increment_xruns(&mut self) {
        self.diagnostics.xruns = self.diagnostics.xruns.saturating_add(1);
    }

    pub fn record_plugin_sandbox_fault(
        &mut self,
        sandbox_id: impl Into<String>,
        kind: PluginFaultKind,
        detail: impl Into<String>,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::PluginSandboxFault {
            sandbox_id: sandbox_id.into(),
            kind,
            detail: detail.into(),
            processing_epoch,
        });
    }

    pub fn record_watchdog_restart(
        &mut self,
        record: WatchdogRestartRecord,
    ) -> RuntimeSupervisionSnapshot {
        if self.supervision.record_watchdog_restart(record) {
            self.safe_mode_enabled = true;
        }
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        self.emit(RuntimeEvent::SupervisionChanged(
            self.get_supervision_snapshot(),
        ));
        self.get_supervision_snapshot()
    }

    pub fn record_recovery_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::RecoveryCycle {
            sandbox_id: sandbox_id.into(),
            intent,
            stop_reason,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_lifecycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::PluginSandboxLifecycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_instance_state(
        &mut self,
        state: PluginSandboxInstanceStateRecord,
    ) {
        self.emit(RuntimeEvent::PluginSandboxInstanceState { state });
    }

    pub fn record_plugin_sandbox_transport(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        region_id: impl Into<String>,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    ) {
        let sandbox_id = sandbox_id.into();
        let lease_id = lease_id.into();
        let region_id = region_id.into();
        match stage {
            PluginSandboxTransportStage::Attached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::AttachActive,
                );
            }
            PluginSandboxTransportStage::DetachRequested => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachRequested,
                );
            }
            PluginSandboxTransportStage::DetachFault => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachFaulted,
                );
            }
            PluginSandboxTransportStage::Detached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::Detached,
                );
            }
        }
        self.refresh_prework_service_policy_and_state(processing_epoch);
        self.emit(RuntimeEvent::PluginSandboxTransport {
            sandbox_id,
            lease_id,
            region_id,
            stage,
            processing_epoch,
            detail,
        });
    }

    pub fn record_heartbeat_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    ) {
        self.emit(RuntimeEvent::HeartbeatCycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
            block_sequence,
        });
    }

    pub fn record_block_dispatch(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    ) {
        self.emit(RuntimeEvent::BlockDispatch {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            frame_count,
            stage,
            completion_state,
        });
    }

    pub fn record_broker_invalidation(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerInvalidation {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
            reason: reason.into(),
        });
    }

    pub fn record_completion_slot_transition(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    ) {
        self.emit(RuntimeEvent::CompletionSlotTransition {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
        });
    }

    pub fn record_broker_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            block_sequence,
            stage,
            detail: detail.into(),
        });
    }

    pub fn record_sandbox_operation_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: impl Into<String>,
        error_kind: impl Into<String>,
        stage: SandboxOperationFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::SandboxOperationFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            operation: operation.into(),
            error_kind: error_kind.into(),
            stage,
            detail: detail.into(),
        });
    }

    pub fn projection_epoch(&self) -> u64 {
        self.projection_epoch
    }

    pub fn reset_block_timeline(&mut self) {
        self.timeline.reset();
    }

    pub fn reset_automation_tracking(&mut self) {
        self.automation.reset();
    }

    pub fn process_engine_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before processing engine blocks",
            ));
        }
        let transport = self.applied_transport;
        let context = self.build_engine_execution_context(processing_epoch, block_sequence);
        let pending_transition = self
            .timeline
            .consume_pending_transport_transition(block_sequence);
        let mut result = self.engine.process_block(context, transport, buffer)?;
        let transport_advance = self.advance_engine_transport(result.output.frames().0 as i64);
        self.timeline.record_engine_block_window(
            transport_advance.start_samples,
            transport_advance.end_samples,
        );
        if let Some(transport) = self.applied_transport {
            self.timeline.update_transport_state(transport);
            if transport_advance.loop_wrapped {
                self.timeline
                    .record_loop_wrap(processing_epoch, block_sequence, transport);
            }
        }
        result.snapshot.transport_epoch = self.timeline.transport_epoch;
        result.snapshot.transport_transition = pending_transition
            .map(|transition| transition.kind)
            .or(transport_advance
                .loop_wrapped
                .then_some(RuntimeTransportTransitionKind::LoopWrapped));
        result.snapshot.transport_block_start_samples = transport_advance.start_samples;
        result.snapshot.transport_block_end_samples = transport_advance.end_samples;
        result.snapshot.transport_loop_wrapped = transport_advance.loop_wrapped;
        self.engine.snapshot.transport_epoch = result.snapshot.transport_epoch;
        self.engine.snapshot.transport_transition = result.snapshot.transport_transition;
        self.engine.snapshot.transport_block_start_samples =
            result.snapshot.transport_block_start_samples;
        self.engine.snapshot.transport_block_end_samples =
            result.snapshot.transport_block_end_samples;
        self.engine.snapshot.transport_loop_wrapped = result.snapshot.transport_loop_wrapped;
        self.recording_capture.record_output_block(&result.output);
        let _ = self.enforce_scheduler_after_engine_block(processing_epoch, block_sequence)?;
        self.refresh_scheduler_topology_summary();
        result.snapshot = self.engine.snapshot.clone();
        Ok(result)
    }

    pub fn prepare_engine_prework_for_block(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        self.prepare_engine_prework_for_block_with_future_state(
            processing_epoch,
            target_block_sequence,
            admitted_from_block_sequence,
            buffer,
            None,
            None,
        )
    }

    pub fn prepare_engine_prework_for_block_with_future_state(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> Result<bool, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        let context = self.build_engine_execution_context_with_overrides(
            processing_epoch,
            target_block_sequence,
            parameter_epoch_override,
            transport_override,
        );
        let transport = self.resolve_transport(transport_override);
        self.engine.admit_prework_for_block(
            context,
            transport,
            admitted_from_block_sequence,
            buffer,
        )
    }

    pub fn prepare_engine_prework_window(
        &mut self,
        processing_epoch: u64,
        targets: Vec<RuntimePreworkWindowTarget>,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        if targets.is_empty() {
            return Ok(0);
        }

        let mut sorted_targets = targets;
        sorted_targets.sort_by_key(|target| target.target_block_sequence);
        let current_block_sequence = sorted_targets
            .iter()
            .map(|target| target.admitted_from_block_sequence)
            .min()
            .unwrap_or(0);

        let planned_sequences: Vec<u64> = sorted_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .collect();
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence >= current_block_sequence
                    && !planned_sequences.contains(&cache.source_block_sequence)
            },
            RuntimePreworkInvalidationReason::PlanningWindowRevised,
        );
        self.engine.pending_prework_targets.clear();

        let mut admitted = 0usize;
        for target in sorted_targets {
            let context = self.build_engine_execution_context_with_overrides(
                processing_epoch,
                target.target_block_sequence,
                target.parameter_epoch_override,
                target.transport_override,
            );
            let transport = self.resolve_transport(target.transport_override);
            if self.engine.admit_prework_for_block(
                context,
                transport,
                target.admitted_from_block_sequence,
                target.buffer,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        self.engine.update_prework_queue_snapshot(
            Some(current_block_sequence),
            self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(admitted)
    }

    fn service_pending_prework_cycle(
        &mut self,
        processing_epoch: u64,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Result<usize, RuntimeError> {
        if budget == 0 || !self.control.configured {
            return Ok(0);
        }
        let targets = self
            .engine
            .take_pending_prework_targets(budget, max_backlog_class);
        if targets.is_empty() {
            return Ok(0);
        }

        let mut admitted = 0usize;
        for target in targets {
            self.engine.record_last_serviced_pending_target(&target);
            if self.prepare_engine_prework_for_block_with_future_state(
                processing_epoch,
                target.target_block_sequence,
                target.admitted_from_block_sequence,
                target.buffer,
                target.parameter_epoch_override,
                target.transport_override,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        Ok(admitted)
    }

    pub fn prime_engine_prework_window_with_forecast(
        &mut self,
        processing_epoch: u64,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> Result<usize, RuntimeError> {
        self.reconcile_prework_window_with_forecast(current_block_sequence, policy);
        let admitted = if self.control.running {
            self.service_prework_lane_with_policy(
                processing_epoch,
                1,
                policy.prepare_budget_per_cycle,
            )?
        } else {
            self.prime_pending_prework_targets(
                processing_epoch,
                policy.prepare_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            )?
        };
        self.reconcile_prework_service_state(Some(processing_epoch));
        Ok(admitted)
    }

    fn enforce_scheduler_after_engine_block(
        &mut self,
        processing_epoch: u64,
        current_block_sequence: u64,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Ok(0);
        }
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
            || !self.engine.snapshot.prework_cache_enabled
        {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        let Some(policy) = self.prework_forecast_policy.clone() else {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        };

        self.reconcile_prework_window_with_forecast(current_block_sequence, &policy);
        if self.control.running {
            self.service_prework_lane_with_policy(
                processing_epoch,
                1,
                policy.prepare_budget_per_cycle,
            )
        } else {
            let prepared = self.prime_pending_prework_targets(
                processing_epoch,
                policy.prepare_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            )?;
            self.reconcile_prework_service_state(Some(processing_epoch));
            Ok(prepared)
        }
    }

    fn reconcile_prework_window_with_forecast(
        &mut self,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> usize {
        let desired_count = policy.target_window_blocks;
        let target_block_sequences =
            self.plan_prework_window_block_sequences(current_block_sequence, desired_count);
        if target_block_sequences.is_empty() {
            self.engine.update_prework_queue_snapshot(
                Some(current_block_sequence),
                self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
            );
            return 0;
        }

        let targets = target_block_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    self.config.sample_rate,
                    FrameCount(self.config.graph.block_size),
                    target_block_sequence.saturating_add(policy.buffer_seed_offset),
                ),
                parameter_epoch_override: Some(
                    self.forecast_parameter_batch_for_block(target_block_sequence, policy)
                        .epoch,
                ),
                transport_override: Some(
                    self.forecast_transport_projection_for_block(target_block_sequence, policy),
                ),
            })
            .collect::<Vec<_>>();
        let graph_id = self
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        self.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            self.projection_epoch,
            self.latest_parameter_epoch,
            self.applied_transport,
            self.config.graph.block_size,
        );
        targets.len()
    }

    fn service_prework_lane_with_policy(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
        budget_per_cycle: usize,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Ok(0);
        }
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
            || !self.engine.snapshot.prework_cache_enabled
        {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if !self.control.running {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if self.engine.pending_prework_targets.is_empty() {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        self.recompute_prework_service_policy_snapshot();
        let pressure = self.engine.snapshot.prework_service_pressure;
        let semantic_policy = self.engine.snapshot.prework_service_semantic_policy;
        let transport_condition = self.current_prework_transport_condition();
        if self.engine.snapshot.prework_service_plugin_gate_active
            || self.engine.snapshot.prework_service_transport_gate_active
        {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        let (effective_cycles, effective_budget_per_cycle, max_backlog_class) = match pressure {
            RuntimePreworkServicePressure::Normal => (
                cycles,
                budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            ),
            RuntimePreworkServicePressure::Elevated => match semantic_policy {
                RuntimePreworkServiceSemanticPolicy::Balanced => (
                    cycles.min(1),
                    budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::NearTerm,
                ),
                RuntimePreworkServiceSemanticPolicy::PluginConstrained => (
                    cycles.min(1),
                    budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::Immediate,
                ),
                RuntimePreworkServiceSemanticPolicy::LatencyFocused => (
                    cycles.min(1),
                    budget_per_cycle.min(2),
                    RuntimePreworkBacklogClass::Deferred,
                ),
            },
            RuntimePreworkServicePressure::Critical => {
                (0, 0, RuntimePreworkBacklogClass::Immediate)
            }
        };
        let (effective_cycles, effective_budget_per_cycle, max_backlog_class) = transport_condition
            .reduce_service_scope(
                effective_cycles,
                effective_budget_per_cycle,
                max_backlog_class,
            );
        if pressure == RuntimePreworkServicePressure::Critical {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        self.engine.record_prework_service_request(
            cycles,
            effective_cycles,
            budget_per_cycle,
            effective_budget_per_cycle,
        );
        if effective_cycles == 0 || effective_budget_per_cycle == 0 {
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Starved,
                Some(processing_epoch),
            );
            return Ok(0);
        }

        self.engine.transition_prework_service_state(
            RuntimePreworkServiceState::Servicing,
            Some(processing_epoch),
        );
        let mut total_prepared = 0usize;
        let mut executed_cycles = 0usize;
        for _ in 0..effective_cycles {
            executed_cycles = executed_cycles.saturating_add(1);
            total_prepared = total_prepared.saturating_add(self.service_pending_prework_cycle(
                processing_epoch,
                effective_budget_per_cycle,
                max_backlog_class,
            )?);
            if self.engine.pending_prework_targets.is_empty() {
                break;
            }
        }
        self.engine.record_prework_service_cycle(
            processing_epoch,
            executed_cycles,
            budget_per_cycle,
            total_prepared,
        );
        if !self.engine.pending_prework_targets.is_empty() && total_prepared == 0 {
            if pressure == RuntimePreworkServicePressure::Elevated {
                self.engine.record_prework_service_yield(
                    processing_epoch,
                    cycles,
                    budget_per_cycle,
                );
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Yielding,
                    Some(processing_epoch),
                );
            } else {
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Starved,
                    Some(processing_epoch),
                );
            }
        } else {
            self.reconcile_prework_service_state(Some(processing_epoch));
        }
        Ok(total_prepared)
    }

    pub fn forecast_transport_projection_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> TransportProjection {
        let loop_length_blocks = policy.transport_loop_length_blocks.max(1);
        let loop_end_samples = (self
            .config
            .graph
            .block_size
            .saturating_mul(loop_length_blocks)) as i64;
        let timeline_position_samples = ((block_sequence as i64)
            .saturating_mul(self.config.graph.block_size as i64))
        .rem_euclid(loop_end_samples);
        TransportProjection {
            playing: policy.transport_playing,
            timeline_position_samples,
            tempo_bpm: policy.transport_tempo_bpm,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 0,
                end_samples: loop_end_samples,
            }),
        }
    }

    pub fn forecast_parameter_batch_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> ParameterBatch {
        let cycle_length = policy.parameter_cycle_length.max(1);
        let denominator = cycle_length.saturating_sub(1).max(1) as f32;
        ParameterBatch {
            epoch: self
                .projection_epoch
                .saturating_add(block_sequence)
                .saturating_add(1),
            events: vec![crate::interfaces::ParameterEvent {
                target: policy.parameter_target.clone(),
                normalized_value: ((block_sequence % cycle_length) as f32) / denominator,
            }],
        }
    }

    pub fn apply_forecast_state_for_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<usize, RuntimeError> {
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before applying forecast state",
            )
        })?;
        self.apply_forecast_transport_projection(
            self.forecast_transport_projection_for_block(block_sequence, &policy),
        )?;
        self.apply_parameter_batch(
            self.forecast_parameter_batch_for_block(block_sequence, &policy),
        )?;
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let _ = processing_epoch;
        Ok(self.reconcile_prework_window_with_forecast(block_sequence, &policy))
    }

    pub fn prepare_plugin_dispatch_state_for_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<RuntimePluginDispatchState, RuntimeError> {
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(RuntimePluginDispatchState {
                transport: self.applied_transport,
                parameter_batch: None,
            });
        }

        let Some(policy) = self.prework_forecast_policy.clone() else {
            return Ok(RuntimePluginDispatchState {
                transport: self.applied_transport,
                parameter_batch: None,
            });
        };

        let transport = self.forecast_transport_projection_for_block(block_sequence, &policy);
        let parameter_batch = self.forecast_parameter_batch_for_block(block_sequence, &policy);
        self.apply_forecast_transport_projection(transport)?;
        self.apply_parameter_batch(parameter_batch.clone())?;
        let _ = processing_epoch;
        let _ = self.reconcile_prework_window_with_forecast(block_sequence, &policy);

        Ok(RuntimePluginDispatchState {
            transport: Some(transport),
            parameter_batch: Some(parameter_batch),
        })
    }

    fn apply_forecast_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        if projection.tempo_bpm <= 0.0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "tempo_bpm must be positive",
            ));
        }
        self.applied_transport = Some(projection);
        self.timeline.update_transport_state(projection);
        self.engine.snapshot.transport_epoch = self.timeline.transport_epoch;
        self.engine.snapshot.transport_transition = None;
        self.engine.snapshot.transport_loop_wrapped = false;
        Ok(())
    }

    pub fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before servicing the prework lane",
            )
        })?;
        self.service_prework_lane_with_policy(
            processing_epoch,
            cycles,
            policy.prepare_budget_per_cycle,
        )
    }

    pub fn next_planned_prework_block_sequence(
        &self,
        after_block_sequence: Option<u64>,
    ) -> Option<u64> {
        self.engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                match after_block_sequence {
                    Some(after) if target <= after => None,
                    _ => Some(target),
                }
            })
            .min()
    }

    pub fn plan_prework_window_block_sequences(
        &mut self,
        current_block_sequence: u64,
        desired_count: usize,
    ) -> Vec<u64> {
        if !self.engine.snapshot.prework_cache_enabled {
            return Vec::new();
        }

        let mut retained_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                (target > current_block_sequence).then_some(target)
            })
            .collect::<Vec<_>>();
        retained_sequences.sort_unstable();
        retained_sequences.dedup();

        if retained_sequences.len() > desired_count {
            retained_sequences.truncate(desired_count);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence > current_block_sequence
                        && !retained_sequences.contains(&cache.source_block_sequence)
                },
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
        } else if desired_count == 0 {
            self.engine.retire_prework_entries_matching(
                |cache| cache.source_block_sequence > current_block_sequence,
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
            retained_sequences.clear();
        }

        let mut next_sequence = current_block_sequence.saturating_add(1);
        while retained_sequences.len() < desired_count {
            if !retained_sequences.contains(&next_sequence) {
                retained_sequences.push(next_sequence);
            }
            next_sequence = next_sequence.saturating_add(1);
        }
        retained_sequences
    }

    fn prime_pending_prework_targets(
        &mut self,
        processing_epoch: u64,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Result<usize, RuntimeError> {
        if budget == 0 || !self.control.configured {
            return Ok(0);
        }
        let prepared =
            self.service_pending_prework_cycle(processing_epoch, budget, max_backlog_class)?;
        if prepared > 0 {
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Pending,
                Some(processing_epoch),
            );
        }
        Ok(prepared)
    }

    fn build_engine_execution_context(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> GraphExecutionContext {
        self.build_engine_execution_context_with_overrides(
            processing_epoch,
            block_sequence,
            None,
            None,
        )
    }

    fn build_engine_execution_context_with_overrides(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> GraphExecutionContext {
        let transport = self.resolve_transport(transport_override);
        GraphExecutionContext {
            processing_epoch,
            block_sequence,
            projection_epoch: self.projection_epoch,
            parameter_epoch: parameter_epoch_override.unwrap_or(self.latest_parameter_epoch),
            configured_block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            transport_playing: transport.map(|t| t.playing).unwrap_or(false),
            transport_tempo_bpm: transport.map(|t| t.tempo_bpm).unwrap_or(0.0),
            timeline_position_samples: transport.map(|t| t.timeline_position_samples).unwrap_or(0),
        }
    }

    fn resolve_transport(
        &self,
        transport_override: Option<TransportProjection>,
    ) -> Option<TransportProjection> {
        transport_override.or(self.applied_transport)
    }

    fn advance_engine_transport(&mut self, frame_count: i64) -> RuntimeEngineTransportAdvance {
        let Some(mut transport) = self.applied_transport else {
            return RuntimeEngineTransportAdvance::default();
        };
        let start_samples = Some(transport.timeline_position_samples);
        if !transport.playing || frame_count <= 0 {
            return RuntimeEngineTransportAdvance {
                start_samples,
                end_samples: start_samples,
                loop_wrapped: false,
            };
        }

        let advanced = transport
            .timeline_position_samples
            .saturating_add(frame_count);
        let mut loop_wrapped = false;
        transport.timeline_position_samples = if let Some(loop_region) = transport.loop_state {
            let loop_start = loop_region.start_samples;
            let loop_end = loop_region.end_samples;
            if loop_end > loop_start && advanced >= loop_end {
                loop_wrapped = true;
                let loop_len = loop_end.saturating_sub(loop_start);
                loop_start.saturating_add((advanced - loop_start).rem_euclid(loop_len))
            } else {
                advanced
            }
        } else {
            advanced
        };
        self.applied_transport = Some(transport);
        RuntimeEngineTransportAdvance {
            start_samples,
            end_samples: Some(transport.timeline_position_samples),
            loop_wrapped,
        }
    }

    pub fn allocate_block_sequence(&mut self) -> u64 {
        self.timeline.allocate_block_sequence()
    }

    pub fn record_block_sequence(
        &mut self,
        sandbox_id: impl Into<String>,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        let sandbox_id = sandbox_id.into();
        if let Some(rollover) = self.timeline.record_block_sequence(
            &sandbox_id,
            processing_epoch,
            lease_id,
            block_sequence,
        ) {
            self.emit(RuntimeEvent::LeaseRollover {
                sandbox_id: rollover.sandbox_id,
                previous_lease_id: rollover.previous_lease_id,
                lease_id: rollover.lease_id,
                processing_epoch: rollover.processing_epoch,
                first_block_sequence: rollover.first_block_sequence,
            });
        }
    }

    pub fn record_automation_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        self.automation
            .record_summary(processing_epoch, lease_id, summary);
    }

    pub fn begin_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            transport_session_provenance(intent),
            None,
            None,
            None,
        )?;
        self.refresh_prework_service_policy_and_state(None);
        Ok(snapshot)
    }

    pub fn begin_transport_session_with_metadata(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        self.begin_transport_session_with_metadata_for_epoch(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            None,
            transport_session_provenance(intent),
            backing_path,
            total_bytes,
        )
    }

    pub fn begin_transport_session_with_metadata_for_epoch(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        attach_processing_epoch: Option<u64>,
        provenance: TransportSessionProvenance,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            provenance,
            attach_processing_epoch,
            backing_path,
            total_bytes,
        )?;
        self.refresh_prework_service_policy_and_state(attach_processing_epoch);
        Ok(snapshot)
    }

    pub fn enqueue_lingering_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        self.transport_concurrency.enqueue_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            0,
            processing_epoch,
            None,
            exclude_lease_id,
            exclude_region_id,
        )
    }

    pub fn dequeue_lingering_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<crate::interfaces::LingeringCleanupPlan> {
        self.transport_concurrency
            .dequeue_cleanup_work_for_sandbox(sandbox_id, current_processing_epoch)
    }

    pub fn record_lingering_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency.record_cleanup_failure(
            sandbox_id,
            lease_id,
            region_id,
            mode,
            processing_epoch,
            error,
        );
        if matches!(mode, LingeringCleanupMode::BestEffortPostStart) {
            let retry_count = self
                .transport_concurrency
                .cleanup_attempt_count(sandbox_id, lease_id, region_id);
            let cleanup_wave = self
                .transport_concurrency
                .cleanup_wave_for_session(sandbox_id, lease_id, region_id);
            let _ = self.transport_concurrency.enqueue_cleanup_work(
                sandbox_id,
                mode,
                LingeringCleanupTrigger::DeferredRetry,
                retry_count,
                processing_epoch,
                cleanup_wave,
                Some(lease_id),
                Some(region_id),
            );
        }
        self.transport_concurrency.snapshot()
    }

    pub fn clear_lingering_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id)
    }

    pub fn complete_lingering_cleanup_success(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id);
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }

    pub fn end_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }

    pub fn promote_transport_session_to_steady_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        let snapshot = self
            .transport_concurrency
            .promote_session_to_steady_state(sandbox_id, lease_id, region_id);
        self.refresh_prework_service_policy_and_state(None);
        snapshot
    }

    fn require_handshake(&self) -> Result<(), RuntimeError> {
        if self.control.handshaken {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be handshaken before control requests",
            ))
        }
    }

    fn require_configured(&self) -> Result<(), RuntimeError> {
        if self.control.configured {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before this request",
            ))
        }
    }

    fn scheduler_state(&self) -> RuntimeSchedulerState {
        match self.readiness {
            RuntimeReadiness::Failed { .. } | RuntimeReadiness::Stopped => {
                RuntimeSchedulerState::Stopped
            }
            RuntimeReadiness::Degraded { .. } => RuntimeSchedulerState::Degraded,
            RuntimeReadiness::Starting => RuntimeSchedulerState::Configured,
            RuntimeReadiness::Ready => {
                if !self.control.running {
                    RuntimeSchedulerState::Configured
                } else if self.engine.snapshot.graph_id.is_none()
                    || self.engine.snapshot.node_count == 0
                {
                    RuntimeSchedulerState::ReadyIdle
                } else if !self.anticipative_enabled
                    || !self.engine.snapshot.prework_cache_enabled
                    || self.engine.snapshot.anticipative_phase_count == 0
                {
                    RuntimeSchedulerState::RealtimeOnly
                } else {
                    RuntimeSchedulerState::Anticipative
                }
            }
        }
    }

    fn scheduler_phase(&self, state: RuntimeSchedulerState) -> RuntimeExecutionPhase {
        if matches!(
            state,
            RuntimeSchedulerState::Stopped | RuntimeSchedulerState::Configured
        ) {
            return RuntimeExecutionPhase::Idle;
        }
        if matches!(state, RuntimeSchedulerState::Degraded) {
            return RuntimeExecutionPhase::Degraded;
        }
        let last_prework_epoch = self
            .engine
            .snapshot
            .last_prework_service_processing_epoch
            .unwrap_or(0);
        let last_realtime_epoch = self.engine.snapshot.last_processing_epoch.unwrap_or(0);
        if last_prework_epoch > 0 && last_prework_epoch > last_realtime_epoch {
            return RuntimeExecutionPhase::Prework;
        }
        if self.engine.snapshot.processed_blocks == 0 {
            return if self.engine.snapshot.graph_id.is_some()
                || self.engine.snapshot.prework_pending_target_count > 0
            {
                RuntimeExecutionPhase::Priming
            } else {
                RuntimeExecutionPhase::Idle
            };
        }
        RuntimeExecutionPhase::Realtime
    }

    fn scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot {
        let state = self.scheduler_state();
        RuntimeSchedulerSnapshot {
            state,
            phase: self.scheduler_phase(state),
            graph_applied: self.applied_graph.is_some(),
            schedule_applied: self.applied_schedule.is_some(),
            transport_projected: self.applied_transport.is_some(),
            anticipative_enabled: self.anticipative_enabled,
            active_graph_id: self.engine.snapshot.graph_id.clone(),
            phase_count: self.engine.snapshot.phase_count,
            lane_count: self.engine.snapshot.lane_count,
            dispatch_count: self.engine.snapshot.dispatch_count,
            pending_prework_target_count: self.engine.snapshot.prework_pending_target_count,
            processed_block_count: self.engine.snapshot.processed_blocks,
        }
    }

    fn transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot {
        let timeline = self.timeline.snapshot();
        RuntimeTransportObservationSnapshot {
            transport_epoch: timeline.transport_epoch,
            projected_playing: self.applied_transport.map(|transport| transport.playing),
            projected_tempo_bpm: self.applied_transport.map(|transport| transport.tempo_bpm),
            projected_timeline_position_samples: self
                .applied_transport
                .map(|transport| transport.timeline_position_samples),
            projected_loop_start_samples: self.applied_transport.and_then(|transport| {
                transport
                    .loop_state
                    .map(|loop_state| loop_state.start_samples)
            }),
            projected_loop_end_samples: self.applied_transport.and_then(|transport| {
                transport
                    .loop_state
                    .map(|loop_state| loop_state.end_samples)
            }),
            observed_playing: timeline.last_transport_playing,
            observed_tempo_bpm: timeline.last_transport_tempo_bpm,
            observed_timeline_position_samples: timeline.last_transport_timeline_position_samples,
            observed_loop_start_samples: timeline.last_transport_loop_start_samples,
            observed_loop_end_samples: timeline.last_transport_loop_end_samples,
            last_transition: timeline.last_transport_transition,
            last_transition_processing_epoch: timeline.last_transport_transition_processing_epoch,
            last_transition_block_sequence: timeline.last_transport_transition_block_sequence,
            last_engine_block_start_samples: timeline.last_engine_block_start_samples,
            last_engine_block_end_samples: timeline.last_engine_block_end_samples,
            loop_wrap_count: timeline.loop_wrap_count,
        }
    }

    fn recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot {
        self.recording_capture
            .snapshot(self.control.configured, &self.readiness)
    }

    fn media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        self.media_pipeline.snapshot()
    }

    fn current_project_tempo_bpm(&self) -> f64 {
        self.applied_transport
            .map(|transport| transport.tempo_bpm)
            .or(self.timeline.last_transport_tempo_bpm)
            .filter(|tempo| tempo.is_finite() && *tempo > 0.0)
            .unwrap_or(120.0)
    }

    fn warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot {
        self.warp_pipeline
            .snapshot(self.current_project_tempo_bpm(), &self.media_pipeline)
    }

    pub fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError> {
        self.recording_capture.start_capture(
            request,
            self.config.sample_rate.0,
            self.control.configured,
            &self.readiness,
        )
    }

    pub fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        self.recording_capture.finish_capture()
    }

    pub fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError> {
        self.recording_capture.cancel_capture()
    }

    pub fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        self.media_pipeline.reconcile_assets(assets)
    }

    pub fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError> {
        self.warp_pipeline.reconcile_clips(clips);
        Ok(())
    }

    fn refresh_runtime_state(&mut self) {
        match self.readiness {
            RuntimeReadiness::Failed { .. } | RuntimeReadiness::Stopped => {}
            RuntimeReadiness::Starting => {}
            RuntimeReadiness::Ready | RuntimeReadiness::Degraded { .. } => {
                self.readiness = if self.safe_mode_enabled {
                    RuntimeReadiness::Degraded {
                        reasons: vec![
                            DegradedReason("safe-mode-enabled"),
                            DegradedReason("watchdog-restart-threshold-exceeded"),
                        ],
                    }
                } else {
                    RuntimeReadiness::Ready
                };
            }
        }
    }

    fn emit(&mut self, event: RuntimeEvent) {
        for sink in &mut self.sinks {
            sink.push(event.clone());
        }
    }
}

impl RuntimeLifecycleApi for SignalRuntime {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError> {
        if request.client_version.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "client_version must not be empty",
            ));
        }
        if matches!(request.max_sample_rate_hint, Some(0)) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "max_sample_rate_hint must be positive when provided",
            ));
        }

        self.control.handshaken = true;
        self.control.handshake_count = self.control.handshake_count.saturating_add(1);
        self.control.last_client_version = Some(request.client_version.clone());

        Ok(HandshakeResponse {
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            supports_anticipative: true,
            supports_dynamic_reconfigure: true,
            max_channels: 2048,
            max_sample_rate: request.max_sample_rate_hint.unwrap_or(192_000),
        })
    }

    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.block_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "sample_rate and block_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.block_size;
        self.anticipative_enabled = request.anticipative_enabled;
        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        self.reconcile_prework_forecast_mode_state()?;
        self.engine.refresh_planning(self.anticipative_enabled);
        self.safe_mode_enabled = request.realtime_safe_mode;
        self.control.configured = true;
        self.control.running = false;
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.configure_count = self.control.configure_count.saturating_add(1);
        self.control.last_reconfigure = Some(request);
        self.timeline.reset();
        self.automation.reset();
        self.transport_concurrency.reset();
        self.recording_capture = RuntimeRecordingCaptureStateModel::default();
        self.media_pipeline = RuntimeMediaPipelineStateModel::default();
        self.warp_pipeline = RuntimeWarpPipelineStateModel::default();
        self.readiness = RuntimeReadiness::Starting;
        self.refresh_runtime_state();
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }

    fn start(&mut self) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is already running",
            ));
        }

        self.readiness = RuntimeReadiness::Ready;
        self.control.running = true;
        self.control.start_count = self.control.start_count.saturating_add(1);
        self.refresh_runtime_state();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        if !self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is not running",
            ));
        }

        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeStopped);
        self.readiness = RuntimeReadiness::Stopped;
        self.control.running = false;
        self.recording_capture.active = None;
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.stop_count = self.control.stop_count.saturating_add(1);
        self.control.last_stop_reason = Some(reason);
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.reconfigure.is_none() {
            self.require_configured()?;
        }
        if self.control.running {
            self.stop(StopReason::DeviceReconfigure)?;
        }
        if let Some(config) = request.reconfigure {
            self.configure(config)?;
        }
        self.control.restart_count = self.control.restart_count.saturating_add(1);
        self.start()
    }

    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError> {
        self.safe_mode_enabled = request.enabled;
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}

impl RuntimeProjectionApi for SignalRuntime {
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.engine.set_prework_service_pressure(pressure);
        self.refresh_prework_service_policy_and_state(None);
        Ok(())
    }

    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.set_prework_forecast_mode_state(mode)
    }

    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            Some(selection),
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
        );
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::ExplicitProfile,
        );
        self.reconcile_prework_forecast_mode_state()?;
        self.invalidate_prework_for_forecast_plan_change_if_needed(
            previous_requested_mode,
            previous_effective_mode,
            previous_profile,
            previous_profile_source,
            previous_policy,
        )?;
        self.refresh_prework_service_policy_and_state(None);
        Ok(())
    }

    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            None,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
        );
        self.set_prework_forecast_policy_internal(Some(policy));
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RawPolicyOverride,
        );
        self.reconcile_prework_forecast_mode_state()?;
        self.invalidate_prework_for_forecast_plan_change_if_needed(
            previous_requested_mode,
            previous_effective_mode,
            previous_profile,
            previous_profile_source,
            previous_policy,
        )?;
        self.refresh_prework_service_policy_and_state(None);
        Ok(())
    }

    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        SignalRuntime::service_prework_lane(self, processing_epoch, cycles)
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.require_configured()?;
        self.engine
            .apply_plugin_backed_node_bindings(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.graph_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph_id must not be empty",
            ));
        }

        self.require_configured()?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.engine
            .apply_graph_contract_projection(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.graph_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.engine
            .apply_graph_projection(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        self.applied_graph = Some(projection);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.schedule_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "schedule_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.applied_schedule = Some(projection);
        self.refresh_scheduler_topology_summary();
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        if projection.tempo_bpm <= 0.0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "tempo_bpm must be positive",
            ));
        }

        let transition = classify_transport_transition(self.applied_transport, projection);
        let Some(transition) = transition else {
            self.applied_transport = Some(projection);
            self.timeline.update_transport_state(projection);
            return Ok(());
        };

        let current_ready_block = self.engine.snapshot.last_block_sequence.unwrap_or(0);
        let reason = classify_transport_invalidation_reason(self.applied_transport, projection);
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence <= current_ready_block
                    && (cache.transport.playing != projection.playing
                        || cache.transport.tempo_bpm != projection.tempo_bpm
                        || cache.transport.timeline_position_samples
                            != projection.timeline_position_samples
                        || cache.transport.loop_state != projection.loop_state)
            },
            reason,
        );
        self.applied_transport = Some(projection);
        self.timeline.record_transport_projection(
            transition,
            self.engine
                .snapshot
                .last_block_sequence
                .map(|block_sequence| block_sequence.saturating_add(1)),
            self.engine.snapshot.last_processing_epoch,
            projection,
        );
        self.engine.snapshot.transport_epoch = self.timeline.transport_epoch;
        self.engine.snapshot.transport_transition = Some(transition);
        self.engine.snapshot.transport_loop_wrapped = false;
        Ok(())
    }

    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError> {
        if batch.epoch < self.projection_epoch {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "parameter batch epoch is stale",
            ));
        }
        if !batch.events.is_empty() {
            let current_ready_block = self.timeline.next_block_sequence.saturating_sub(1);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence <= current_ready_block
                        && cache.parameter_epoch != batch.epoch
                },
                RuntimePreworkInvalidationReason::ParameterBatchApplied,
            );
        }
        self.latest_parameter_epoch = batch.epoch;
        Ok(())
    }

    fn apply_hardware_config(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "hardware config sample_rate and buffer_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.buffer_size;
        self.diagnostics.backend_policy_tier = request.backend_policy;
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}

impl RuntimeObservationApi for SignalRuntime {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle {
        let handle = SubscriptionHandle(self.next_subscription);
        self.next_subscription = self.next_subscription.saturating_add(1);
        self.sinks.push(sink);
        handle
    }

    fn get_readiness(&self) -> RuntimeReadiness {
        self.readiness.clone()
    }

    fn get_effective_config(&self) -> EffectiveRuntimeConfig {
        EffectiveRuntimeConfig {
            sample_rate: self.config.sample_rate,
            block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            safe_mode_enabled: self.safe_mode_enabled,
            active_output_device: self.active_output_device.clone(),
        }
    }

    fn get_control_snapshot(&self) -> RuntimeControlSnapshot {
        self.control.clone()
    }

    fn get_scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot {
        self.scheduler_snapshot()
    }

    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        self.diagnostics
    }

    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot {
        self.supervision.snapshot(self.safe_mode_enabled)
    }

    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot {
        self.timeline.snapshot()
    }

    fn get_transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot {
        self.transport_observation_snapshot()
    }

    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot {
        self.recording_capture_snapshot()
    }

    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        self.media_pipeline_snapshot()
    }

    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot {
        self.warp_pipeline_snapshot()
    }

    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot {
        self.automation.snapshot()
    }

    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot {
        self.engine.snapshot()
    }

    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{RuntimeConfig, RuntimeProfile, SignalRuntime};
    use crate::interfaces::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
        GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
        GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        LingeringCleanupTrigger, ParameterBatch, ParameterEvent, PluginBackedNodeBinding,
        PluginBackedNodeBindingProjection, PluginNodeRender, PluginNodeRenderBatch,
        PluginSandboxLifecycleStage, PluginSandboxTransportStage, RecoveryRestartIntent,
        RestartRequest, RuntimeConfigRequest, RuntimeErrorKind, RuntimeEvent, RuntimeEventRecorder,
        RuntimeEventSink, RuntimeExecutionPhase, RuntimeLifecycleApi,
        RuntimeMediaAssetRegistration, RuntimeMediaAssetState, RuntimeObservationApi,
        RuntimeObservationReport, RuntimePreworkBacklogClass, RuntimePreworkCacheState,
        RuntimePreworkForecastMode, RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
        RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
        RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason,
        RuntimePreworkRetirementReason, RuntimePreworkServicePressure,
        RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
        RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
        RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState, RuntimeSchedulerState,
        RuntimeSchedulerTopologyIssue, RuntimeSupervisorReport, RuntimeWarpClipRegistration,
        RuntimeWarpMode, RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest,
        SandboxOperationFailureStage, ScheduleProjection, StopReason, TransportAttachIntent,
        TransportProjection, TransportSessionProvenance, WatchdogRestartRecord,
    };
    use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};
    use signal_graph::{
        synthetic_stereo_block, ExecutableGraph, GraphNodeBufferContract, GraphNodeBusEndpoint,
        GraphNodeExecutionClass, GraphNodeSpec, GraphNodeTopologyMetadata, GraphNodeTopologyRole,
        GraphStageSpec,
    };
    use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
    use signal_plugin::{CompletionState, ParameterAutomationSummary};
    use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};

    #[derive(Default)]
    struct TestSink {
        events: Vec<RuntimeEvent>,
    }

    impl RuntimeEventSink for TestSink {
        fn push(&mut self, event: RuntimeEvent) {
            self.events.push(event);
        }
    }

    fn handshake_and_configure(runtime: &mut SignalRuntime) {
        handshake_and_configure_with_anticipative(runtime, true);
    }

    fn handshake_and_configure_with_anticipative(
        runtime: &mut SignalRuntime,
        anticipative_enabled: bool,
    ) {
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let mut request = RuntimeConfigRequest::new(48_000, 256);
        request.anticipative_enabled = anticipative_enabled;
        runtime.configure(request).unwrap();
    }

    fn temp_capture_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be monotonic enough for temp files")
            .as_nanos();
        env::temp_dir().join(format!("signal-runtime-{label}-{nonce}.wav"))
    }

    fn write_test_wav(path: &Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: HoundSampleFormat::Float,
        };
        let mut writer = WavWriter::create(path, spec).expect("test wav should be created");
        for frame in 0..128 {
            let sample = ((frame as f32 / 128.0) * 2.0) - 1.0;
            writer
                .write_sample(sample)
                .expect("test wav sample should be written");
        }
        writer.finalize().expect("test wav should finalize");
    }

    fn handshake_and_configure_with_disabled_forecast(
        runtime: &mut SignalRuntime,
        anticipative_enabled: bool,
    ) {
        handshake_and_configure_with_anticipative(runtime, anticipative_enabled);
        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
            .unwrap();
    }

    fn seed_pending_prework_targets(
        runtime: &mut SignalRuntime,
        admitted_from_block_sequence: u64,
        target_block_sequences: &[u64],
    ) {
        runtime.engine.pending_prework_targets.clear();
        let targets = target_block_sequences
            .iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence: *target_block_sequence,
                admitted_from_block_sequence,
                buffer: synthetic_stereo_block(
                    runtime.config.sample_rate,
                    FrameCount(runtime.config.graph.block_size),
                    *target_block_sequence,
                ),
                parameter_epoch_override: None,
                transport_override: None,
            })
            .collect::<Vec<_>>();
        let graph_id = runtime
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        runtime.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            runtime.projection_epoch,
            runtime.latest_parameter_epoch,
            runtime.applied_transport,
            runtime.config.graph.block_size,
        );
    }

    fn apply_current_forecast_block_state(runtime: &mut SignalRuntime, block_sequence: u64) {
        let policy = runtime
            .prework_forecast_policy
            .clone()
            .expect("forecast policy configured");
        runtime
            .apply_forecast_transport_projection(
                runtime.forecast_transport_projection_for_block(block_sequence, &policy),
            )
            .expect("apply forecast transport projection");
        runtime
            .apply_parameter_batch(
                runtime.forecast_parameter_batch_for_block(block_sequence, &policy),
            )
            .expect("apply forecast parameter batch");
    }

    fn apply_latency_runtime_graph(runtime: &mut SignalRuntime, graph_id: &str) {
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: graph_id.into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
    }

    fn install_scheduler_topology_runtime_graph(
        runtime: &mut SignalRuntime,
        graph_id: &str,
        track_lane_ids: &[&str],
        include_missing_track_lane_id: bool,
    ) {
        let mut nodes = vec![GraphNodeSpec {
            node_id: "lookahead".into(),
            execution_class: GraphNodeExecutionClass::LatencyBearing,
            latency_samples: 32,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:lookahead", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::Utility),
                lane_id: None,
                bus_group_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
        }];

        for (index, lane_id) in track_lane_ids.iter().enumerate() {
            nodes.push(GraphNodeSpec {
                node_id: format!("track-{index}"),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                tail_samples: 0,
                buffer_contract: GraphNodeBufferContract {
                    input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                    output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                    ..GraphNodeBufferContract::default()
                },
                topology: GraphNodeTopologyMetadata {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    lane_id: Some((*lane_id).into()),
                    bus_group_id: Some("mix:tracks".into()),
                },
                stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
            });
        }

        if include_missing_track_lane_id {
            nodes.push(GraphNodeSpec {
                node_id: "track-missing".into(),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                tail_samples: 0,
                buffer_contract: GraphNodeBufferContract {
                    input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                    output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                    ..GraphNodeBufferContract::default()
                },
                topology: GraphNodeTopologyMetadata {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    lane_id: None,
                    bus_group_id: Some("mix:tracks".into()),
                },
                stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
            });
        }

        nodes.push(GraphNodeSpec {
            node_id: "bus-main".into(),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::Bus),
                lane_id: None,
                bus_group_id: Some("mix:master".into()),
            },
            stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
        });

        nodes.push(GraphNodeSpec {
            node_id: "console-main".into(),
            execution_class: GraphNodeExecutionClass::PureTransform,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("main:out", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::ConsoleNode),
                lane_id: None,
                bus_group_id: Some("console:main".into()),
            },
            stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
        });

        runtime.engine.graph = Some(ExecutableGraph::new(graph_id, nodes));
        runtime
            .engine
            .refresh_planning(runtime.anticipative_enabled);
        runtime.refresh_scheduler_topology_summary();
    }

    #[test]
    fn runtime_starts_and_reports_ready() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
        assert_eq!(runtime.config().profile, RuntimeProfile::Local);
    }

    #[test]
    fn configure_updates_effective_config() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(96_000, 256))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
    }

    #[test]
    fn configure_resets_runtime_block_timeline() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 0);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 0);
    }

    #[test]
    fn runtime_timeline_tracks_sequences_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let first = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first);
        let second = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second);
        let third = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 2, "lease-b", third);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 3);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 2);
        assert_eq!(timeline.block_sequence_continuity.lease_rollovers, 1);
        assert_eq!(
            timeline.block_sequence_continuity.first_block_sequence(),
            Some(0)
        );
        assert_eq!(
            timeline.block_sequence_continuity.last_block_sequence(),
            Some(2)
        );
    }

    #[test]
    fn configure_resets_runtime_automation_tracking() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 0);
        assert_eq!(automation.segment_count, 0);
        assert_eq!(automation.first_epoch, None);
    }

    #[test]
    fn runtime_automation_tracking_rolls_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 4096);
        assert_eq!(automation.value_events, 4);
        assert_eq!(automation.segment_count, 2);
        assert_eq!(automation.segment_epochs, vec![1, 2]);
        assert_eq!(automation.lease_rollovers, 1);
        assert_eq!(automation.first_epoch, Some(1));
        assert_eq!(automation.last_epoch, Some(2));
    }

    #[test]
    fn handshake_requires_client_version() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .handshake(HandshakeRequest {
                client_version: String::new(),
                anticipative_preferred: true,
                max_sample_rate_hint: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn schedule_projection_advances_epoch() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let receipt = runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-1".into(),
                stream_count: 2,
            })
            .unwrap();

        assert_eq!(receipt.accepted_epoch, 1);
        assert!(receipt.applied_at_block_boundary);
    }

    #[test]
    fn runtime_scheduler_topology_summary_validates_track_bus_console_groups() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology",
            &["track:drums", "track:bass"],
            false,
        );

        let missing_schedule = runtime.get_engine_block_snapshot();
        assert_eq!(missing_schedule.scheduler_topology.track_lane_node_count, 2);
        assert_eq!(
            missing_schedule.scheduler_topology.track_lane_group_count,
            2
        );
        assert_eq!(missing_schedule.scheduler_topology.bus_node_count, 1);
        assert_eq!(missing_schedule.scheduler_topology.bus_group_count, 1);
        assert_eq!(missing_schedule.scheduler_topology.console_node_count, 1);
        assert_eq!(missing_schedule.scheduler_topology.console_group_count, 1);
        assert_eq!(
            missing_schedule.scheduler_topology.schedule_stream_count,
            None
        );
        assert!(!missing_schedule.scheduler_topology.compatible);
        assert!(
            missing_schedule
                .scheduler_topology
                .requires_host_reinterpretation
        );
        assert!(matches!(
            missing_schedule.scheduler_topology.issues.as_slice(),
            [
                RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
                    required_streams: 2
                }
            ]
        ));

        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-topology".into(),
                stream_count: 2,
            })
            .expect("apply matching schedule projection");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        let result = runtime
            .process_engine_block(1, 1, block)
            .expect("process topology-aware block");

        assert_eq!(result.snapshot.lane_order.len(), 2);
        assert_eq!(
            result.snapshot.lane_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(
            result.snapshot.dispatch_order.last().copied(),
            Some(signal_graph::GraphExecutionLane::Realtime)
        );
        assert!(result.snapshot.scheduler_topology.compatible);
        assert!(
            !result
                .snapshot
                .scheduler_topology
                .requires_host_reinterpretation
        );
        assert!(result.snapshot.scheduler_topology.issues.is_empty());
        assert_eq!(
            result.snapshot.scheduler_topology.schedule_stream_count,
            Some(2)
        );
    }

    #[test]
    fn runtime_scheduler_topology_summary_flags_insufficient_schedule_streams() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-insufficient",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-too-small".into(),
                stream_count: 1,
            })
            .expect("apply undersized schedule projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert!(!snapshot.scheduler_topology.compatible);
        assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
        assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
            matches!(
                issue,
                RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
                    required_streams: 2,
                    actual_streams: 1
                }
            )
        }));
    }

    #[test]
    fn runtime_scheduler_topology_summary_flags_missing_track_lane_metadata() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-missing-metadata",
            &["track:drums"],
            true,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-metadata".into(),
                stream_count: 2,
            })
            .expect("apply schedule projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert!(!snapshot.scheduler_topology.compatible);
        assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
        assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
            matches!(
                issue,
                RuntimeSchedulerTopologyIssue::MissingTrackLaneIds { node_count: 1 }
            )
        }));
    }

    #[test]
    fn runtime_scheduler_topology_projects_into_runtime_reports() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        install_scheduler_topology_runtime_graph(
            &mut runtime,
            "graph:runtime:scheduler-topology-report",
            &["track:drums", "track:bass"],
            false,
        );
        runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-topology-report".into(),
                stream_count: 2,
            })
            .expect("apply matching schedule projection");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        runtime
            .process_engine_block(1, 1, block)
            .expect("process topology report block");

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.execution_topology_summary.node_count, 5);
        assert_eq!(
            observation.execution_topology_summary.track_lane_node_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
        assert_eq!(observation.execution_topology_summary.console_node_count, 1);
        assert_eq!(
            observation
                .execution_topology_summary
                .track_lane_group_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_group_count, 1);
        assert_eq!(
            observation.execution_topology_summary.console_group_count,
            1
        );
        assert_eq!(observation.execution_topology_summary.lanes.len(), 2);
        assert!(observation
            .render_compact()
            .contains("engine_scheduler_topology_compatible=true"));
        assert!(observation
            .render_compact()
            .contains("engine_scheduler_topology_track_lanes=2/2"));
        assert!(observation
            .render_compact()
            .contains("execution_topology_summary_roles=1/2/1/0/1"));
        assert!(observation
            .render_compact()
            .contains("execution_topology_summary_lane_shapes=Anticipative:1|Realtime:4"));

        let supervisor =
            RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_bus_groups=1"));
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_console_groups=1"));
        assert!(supervisor
            .render_multiline()
            .contains("engine_scheduler_topology_issue_count=0"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_lane_0=Anticipative"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_lane_1=Realtime"));
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_node_2=track-1/Realtime/StatefulRealtime/TrackLane/lane_id=Some(\"track:bass\")"));
        assert!(supervisor.render_multiline().contains(
            "execution_topology_summary_node_4=console-main/Realtime/InlineRealtime/ConsoleNode"
        ));

        let json = supervisor.render_json();
        assert!(json.contains("\"scheduler_topology\":{\"track_lane_node_count\":2"));
        assert!(json.contains("\"track_lane_group_count\":2"));
        assert!(json.contains("\"schedule_stream_count\":2"));
        assert!(json.contains("\"compatible\":true"));
        assert!(json.contains("\"execution_topology_summary\":{\"node_count\":5"));
        assert!(json.contains("\"track_lane_node_count\":2"));
        assert!(json.contains("\"lane\":\"Anticipative\""));
        assert!(json.contains("\"lane\":\"Realtime\""));
        assert!(json.contains("\"node_id\":\"track-0\""));
        assert!(json.contains("\"lane_id\":\"track:drums\""));
        assert!(json.contains("\"bus_group_id\":\"mix:master\""));
        assert!(json.contains("\"node_id\":\"console-main\""));
        assert!(json.contains("\"output_bus_id\":\"main:out\""));
    }

    #[test]
    fn runtime_graph_contract_projection_updates_execution_topology_for_projected_graphs() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:projected-topology".into(),
                node_count: 4,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "track-input".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin-insert".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                    },
                    GraphNodeProjection {
                        node_id: "bus-main".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                    },
                    GraphNodeProjection {
                        node_id: "output-main".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.15 }],
                    },
                ],
            })
            .expect("apply projected graph");
        runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:runtime:projected-topology".into(),
                contract_count: 4,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "track-input".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "main:in".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:track:lead".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "plugin-insert".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:track:lead".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:mix:tracks".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            lane_id: Some("track:lead".into()),
                            bus_group_id: Some("mix:tracks".into()),
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "bus-main".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:mix:tracks".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:console:main".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::Bus),
                            lane_id: None,
                            bus_group_id: Some("mix:master".into()),
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "output-main".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "bus:console:main".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "main:out".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::ConsoleNode),
                            lane_id: None,
                            bus_group_id: Some("console:main".into()),
                        },
                    },
                ],
            })
            .expect("apply projected graph contracts");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:projected-topology".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin-insert".into(),
                    sandbox_id: "sandbox:lead".into(),
                }],
            })
            .expect("apply plugin bindings");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        runtime
            .process_engine_block(1, 1, block)
            .expect("process projected topology block");

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(observation.execution_topology_summary.node_count, 4);
        assert_eq!(
            observation.execution_topology_summary.track_lane_node_count,
            2
        );
        assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
        assert_eq!(observation.execution_topology_summary.console_node_count, 1);
        assert_eq!(
            observation
                .execution_topology_summary
                .track_lane_group_count,
            1
        );
        assert_eq!(observation.execution_topology_summary.bus_group_count, 1);
        assert_eq!(
            observation.execution_topology_summary.console_group_count,
            1
        );
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "track-input"
                    && node.topology_role == GraphNodeTopologyRole::TrackLane
                    && node.lane_id.as_deref() == Some("track:lead")
                    && node.output_bus_id == "bus:track:lead"
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "plugin-insert"
                    && node.plugin_sandbox_id.as_deref() == Some("sandbox:lead")
                    && node.input_bus_id == "bus:track:lead"
                    && node.output_bus_id == "bus:mix:tracks"
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "bus-main"
                    && node.topology_role == GraphNodeTopologyRole::Bus
                    && node.bus_group_id.as_deref() == Some("mix:master")
            }));
        assert!(observation
            .execution_topology_summary
            .nodes
            .iter()
            .any(|node| {
                node.node_id == "output-main"
                    && node.topology_role == GraphNodeTopologyRole::ConsoleNode
                    && node.input_bus_id == "bus:console:main"
                    && node.output_bus_id == "main:out"
            }));
    }

    #[test]
    fn hardware_config_updates_runtime_and_backend_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime
            .apply_hardware_config(HardwareConfigRequest::new(
                96_000,
                256,
                BackendPolicyTier::Tier1Brokered,
            ))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
        assert_eq!(
            runtime.get_diagnostics_snapshot().backend_policy_tier,
            BackendPolicyTier::Tier1Brokered
        );
    }

    #[test]
    fn runtime_executes_applied_graph_block_and_updates_snapshot() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:test".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![
                            GraphStageSpec::Gain { linear: 0.5 },
                            GraphStageSpec::Bias { amount: 0.2 },
                        ],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 96,
                tempo_bpm: 120.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 64,
                    end_samples: 128,
                }),
            })
            .unwrap();
        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch(),
                events: vec![ParameterEvent {
                    target: "engine.runtime.test".into(),
                    normalized_value: 0.5,
                }],
            })
            .unwrap();

        let result = runtime
            .process_engine_block(
                1,
                42,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 3),
            )
            .unwrap();

        assert_eq!(
            result.snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(result.snapshot.node_count, 2);
        assert_eq!(result.snapshot.stateful_node_count, 1);
        assert_eq!(result.snapshot.latency_node_count, 1);
        assert!(result.snapshot.anticipative_planning_enabled);
        assert_eq!(result.snapshot.inline_realtime_node_count, 1);
        assert_eq!(result.snapshot.stateful_realtime_node_count, 0);
        assert_eq!(result.snapshot.anticipative_eligible_node_count, 1);
        assert_eq!(result.snapshot.phase_count, 2);
        assert_eq!(result.snapshot.anticipative_phase_count, 1);
        assert_eq!(result.snapshot.lane_count, 2);
        assert_eq!(result.snapshot.anticipative_lane_count, 1);
        assert_eq!(
            result.snapshot.lane_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.dispatch_count, 2);
        assert_eq!(result.snapshot.dispatch_boundary_count, 1);
        assert_eq!(
            result.snapshot.dispatch_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.prepared_dispatch_count, 1);
        assert_eq!(result.snapshot.realtime_dispatch_count, 1);
        assert_eq!(result.snapshot.dispatch_handoff_count, 1);
        assert!(result.snapshot.prework_cache_enabled);
        assert_eq!(
            result.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(result.snapshot.prework_cache_admissions, 1);
        assert_eq!(result.snapshot.prework_cache_consumptions, 1);
        assert_eq!(result.snapshot.prework_cache_hits, 0);
        assert_eq!(result.snapshot.prework_cache_misses, 1);
        assert_eq!(result.snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(result.snapshot.prework_cache_retirement_count, 0);
        assert_eq!(
            result.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(result.snapshot.prework_cache_block_freshness_window, 2);
        assert_eq!(
            result.snapshot.prework_cache_remaining_valid_blocks,
            Some(2)
        );
        assert!(!result.snapshot.last_prework_cache_hit);
        assert_eq!(result.snapshot.last_prework_invalidation_reason, None);
        assert_eq!(
            result.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            result.snapshot.prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            result.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(result.snapshot.last_prework_source_block_sequence, Some(42));
        assert_eq!(
            result.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            result.snapshot.last_prework_admission_block_sequence,
            Some(42)
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_processing_epoch,
            Some(1)
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_block_sequence,
            Some(42)
        );
        assert_eq!(
            result.snapshot.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::AnticipativeEligible,
            ]
        );
        assert_eq!(result.snapshot.planned_nodes.len(), 2);
        assert_eq!(result.snapshot.stage_count, 3);
        assert_eq!(result.snapshot.total_latency_samples, 16);
        assert_eq!(result.snapshot.max_node_latency_samples, 16);
        assert_eq!(result.snapshot.processed_blocks, 1);
        assert_eq!(result.snapshot.last_processing_epoch, Some(1));
        assert_eq!(result.snapshot.last_block_sequence, Some(42));
        assert_eq!(result.snapshot.last_frame_count, 8);
        assert_eq!(result.snapshot.last_channel_count, 2);
        assert!(result.snapshot.last_prework_output_peak.is_some());
        assert_eq!(
            result.snapshot.last_prework_output_peak,
            result.snapshot.last_realtime_input_peak
        );
        assert!(result.snapshot.last_output_peak.unwrap_or_default() <= 0.7);
        assert!(result.snapshot.last_output_rms.unwrap_or_default() > 0.0);
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(96)
        );
        assert!(result.output.samples().first().is_some());
        assert_eq!(
            runtime
                .applied_transport
                .map(|transport| transport.timeline_position_samples),
            Some(104)
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.engine_block_snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(observation.engine_block_snapshot.node_count, 2);
        assert_eq!(observation.engine_block_snapshot.stateful_node_count, 1);
        assert!(
            observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            observation.engine_block_snapshot.inline_realtime_node_count,
            1
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            0
        );
        assert_eq!(observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            observation.engine_block_snapshot.anticipative_phase_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.lane_count, 2);
        assert_eq!(observation.engine_block_snapshot.anticipative_lane_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_count, 2);
        assert_eq!(observation.engine_block_snapshot.dispatch_boundary_count, 1);
        assert_eq!(observation.engine_block_snapshot.prepared_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.realtime_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_handoff_count, 1);
        assert_eq!(observation.scheduler_summary.phase_count, 2);
        assert_eq!(observation.scheduler_summary.lane_count, 2);
        assert_eq!(observation.scheduler_summary.dispatch_count, 2);
        assert_eq!(
            observation.scheduler_snapshot.state,
            RuntimeSchedulerState::Anticipative
        );
        assert_eq!(
            observation.scheduler_snapshot.phase,
            RuntimeExecutionPhase::Realtime
        );
        assert!(observation.scheduler_snapshot.graph_applied);
        assert!(!observation.scheduler_snapshot.schedule_applied);
        assert!(observation.scheduler_snapshot.transport_projected);
        assert_eq!(
            observation.scheduler_summary.prework_service_state,
            RuntimePreworkServiceState::Disabled
        );
        assert_eq!(observation.block_summary.processed_blocks, 1);
        assert_eq!(observation.block_summary.transport_epoch, 1);
        assert_eq!(
            observation.block_summary.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert!(!observation.degradation_summary.readiness_degraded);
        assert_eq!(observation.degradation_summary.xrun_count, 0);
        assert!(observation.engine_block_snapshot.prework_cache_enabled);
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_admissions,
            1
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_consumptions,
            1
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(observation.engine_block_snapshot.prework_cache_hits, 0);
        assert_eq!(observation.engine_block_snapshot.prework_cache_misses, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_retirement_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_invalidation_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.processed_blocks, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn scheduler_snapshot_tracks_state_and_phase_transitions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let configured = runtime.get_scheduler_snapshot();
        assert_eq!(configured.state, RuntimeSchedulerState::Configured);
        assert_eq!(configured.phase, RuntimeExecutionPhase::Idle);
        assert!(!configured.graph_applied);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:scheduler".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "track".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                    },
                    GraphNodeProjection {
                        node_id: "master".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().unwrap();

        let primed = runtime.get_scheduler_snapshot();
        assert_eq!(primed.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(primed.phase, RuntimeExecutionPhase::Prework);
        assert!(primed.graph_applied);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(2),
            })
            .unwrap();
        seed_pending_prework_targets(&mut runtime, 1, &[2, 3]);

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime.service_prework_lane(1, 1).unwrap();

        let prework = runtime.get_scheduler_snapshot();
        assert_eq!(prework.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(prework.phase, RuntimeExecutionPhase::Prework);
        assert!(prework.transport_projected);

        runtime
            .process_engine_block(
                2,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(256)),
            )
            .unwrap();

        let realtime = runtime.get_scheduler_snapshot();
        assert_eq!(realtime.state, RuntimeSchedulerState::Anticipative);
        assert_eq!(realtime.phase, RuntimeExecutionPhase::Realtime);
        assert_eq!(realtime.processed_block_count, 1);
    }

    #[test]
    fn scheduler_snapshot_surfaces_realtime_only_and_degraded_runtime_states() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, false);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:realtime-only".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "track".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                }],
            })
            .unwrap();
        runtime.start().unwrap();

        let realtime_only = runtime.get_scheduler_snapshot();
        assert_eq!(realtime_only.state, RuntimeSchedulerState::RealtimeOnly);
        assert_eq!(realtime_only.phase, RuntimeExecutionPhase::Priming);

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();

        let degraded = runtime.get_scheduler_snapshot();
        assert_eq!(degraded.state, RuntimeSchedulerState::Degraded);
        assert_eq!(degraded.phase, RuntimeExecutionPhase::Degraded);
    }

    #[test]
    fn runtime_replans_graph_when_anticipative_mode_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:planning".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "drive".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::TanhDrive { drive: 1.4 }],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 32,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                    },
                ],
            })
            .unwrap();

        let initial = runtime.get_engine_block_snapshot();
        assert!(initial.anticipative_planning_enabled);
        assert_eq!(initial.inline_realtime_node_count, 1);
        assert_eq!(initial.stateful_realtime_node_count, 1);
        assert_eq!(initial.anticipative_eligible_node_count, 1);
        assert_eq!(initial.prepared_dispatch_count, 1);
        assert_eq!(initial.realtime_dispatch_count, 1);
        assert_eq!(initial.dispatch_handoff_count, 1);
        assert!(initial.prework_cache_enabled);
        assert_eq!(initial.prework_cache_state, RuntimePreworkCacheState::Empty);
        assert_eq!(
            initial.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Empty
        );
        assert_eq!(initial.prework_cache_admissions, 0);
        assert_eq!(initial.prework_cache_consumptions, 0);
        assert_eq!(initial.prework_cache_hits, 0);
        assert_eq!(initial.prework_cache_misses, 0);
        assert_eq!(initial.prework_cache_invalidation_count, 0);
        assert_eq!(initial.prework_cache_retirement_count, 0);

        let mut request = RuntimeConfigRequest::new(48_000, 256);
        request.anticipative_enabled = false;
        runtime.configure(request).unwrap();

        let replanned = runtime.get_engine_block_snapshot();
        assert!(!replanned.anticipative_planning_enabled);
        assert_eq!(replanned.inline_realtime_node_count, 1);
        assert_eq!(replanned.stateful_realtime_node_count, 2);
        assert_eq!(replanned.anticipative_eligible_node_count, 0);
        assert_eq!(replanned.phase_count, 2);
        assert_eq!(replanned.anticipative_phase_count, 0);
        assert_eq!(replanned.lane_count, 1);
        assert_eq!(replanned.anticipative_lane_count, 0);
        assert_eq!(
            replanned.lane_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(replanned.dispatch_count, 1);
        assert_eq!(replanned.dispatch_boundary_count, 0);
        assert_eq!(replanned.prepared_dispatch_count, 0);
        assert_eq!(replanned.realtime_dispatch_count, 1);
        assert_eq!(replanned.dispatch_handoff_count, 0);
        assert!(!replanned.prework_cache_enabled);
        assert_eq!(
            replanned.prework_cache_state,
            RuntimePreworkCacheState::Disabled
        );
        assert_eq!(
            replanned.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Disabled
        );
        assert_eq!(replanned.prework_cache_admissions, 0);
        assert_eq!(replanned.prework_cache_consumptions, 0);
        assert_eq!(replanned.prework_cache_valid_until_processing_epoch, None);
        assert_eq!(
            replanned.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(replanned.prework_cache_invalidation_count, 0);
        assert_eq!(replanned.prework_cache_retirement_count, 0);
        assert_eq!(
            replanned.dispatch_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(
            replanned.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::StatefulRealtime,
            ]
        );
        assert_eq!(replanned.planned_nodes.len(), 3);
        assert_eq!(
            replanned
                .planned_nodes
                .iter()
                .map(|node| (node.node_id.as_str(), format!("{:?}", node.group)))
                .collect::<Vec<_>>(),
            vec![
                ("input", "InlineRealtime".into()),
                ("drive", "StatefulRealtime".into()),
                ("output", "StatefulRealtime".into()),
            ]
        );
    }

    #[test]
    fn safe_mode_sets_degraded_readiness() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();

        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn runtime_reuses_prework_cache_for_matching_adjacent_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:cache".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 11);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(2, 2, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_hits, 0);
        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(first.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(first.snapshot.prework_cache_queued_consumptions, 0);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(first.snapshot.prework_cache_remaining_valid_blocks, Some(2));
        assert!(!first.snapshot.last_prework_cache_hit);
        assert_eq!(
            first.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_misses, 1);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 1);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(second.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Expiring
        );
        assert_eq!(
            second.snapshot.prework_cache_remaining_valid_blocks,
            Some(1)
        );
        assert!(second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(second.snapshot.last_prework_source_block_sequence, Some(1));
        assert_eq!(
            second.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_admission_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prepared_dispatch_count, 1);
        assert_eq!(second.snapshot.realtime_dispatch_count, 1);
    }

    #[test]
    fn runtime_consumes_primed_prework_for_the_next_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let next_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let next_batch = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.5,
            }],
        };
        let next_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                next_block.clone(),
                Some(next_batch.epoch),
                Some(next_transport),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_admissions, 1);
        assert_eq!(primed.prework_cache_queued_admissions, 1);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(2));
        assert_eq!(primed.last_prework_admitted_from_block_sequence, Some(1));

        runtime.apply_parameter_batch(next_batch).unwrap();
        runtime.apply_transport_projection(next_transport).unwrap();
        let consumed = runtime.process_engine_block(1, 2, next_block).unwrap();
        assert_eq!(consumed.snapshot.prework_cache_hits, 1);
        assert_eq!(consumed.snapshot.prework_cache_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_consumptions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_consumptions, 1);
        assert!(consumed.snapshot.last_prework_cache_hit);
        assert_eq!(consumed.snapshot.last_prework_invalidation_reason, None);
        assert_eq!(
            consumed.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(72)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn runtime_prework_queue_consumes_multiple_future_blocks_in_order() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-pipeline".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let block2 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let block3 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 13);
        let batch2 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.5,
            }],
        };
        let batch3 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(4),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.65,
            }],
        };
        let transport2 = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let transport3 = TransportProjection {
            playing: true,
            timeline_position_samples: 80,
            tempo_bpm: 120.0,
            loop_state: None,
        };

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block2.clone(),
                Some(batch2.epoch),
                Some(transport2),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                3,
                1,
                block3.clone(),
                Some(batch3.epoch),
                Some(transport3),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 2);
        assert_eq!(primed.prework_cache_peak_queue_depth, 2);
        assert_eq!(primed.prework_cache_queued_admissions, 2);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(3));

        runtime.apply_parameter_batch(batch2).unwrap();
        runtime.apply_transport_projection(transport2).unwrap();
        let second = runtime.process_engine_block(1, 2, block2).unwrap();
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );

        runtime.apply_parameter_batch(batch3).unwrap();
        runtime.apply_transport_projection(transport3).unwrap();
        let third = runtime.process_engine_block(1, 3, block3).unwrap();
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_queued_consumptions, 2);
        assert_eq!(third.snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            third.snapshot.last_prework_consumption_block_sequence,
            Some(3)
        );
        assert_eq!(
            third.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
    }

    #[test]
    fn runtime_prework_queue_evicts_oldest_future_entry_when_capacity_is_exceeded() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-eviction".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        for offset in 0..4 {
            let target_block_sequence = 2 + offset;
            let block = synthetic_stereo_block(
                SampleRate(48_000),
                FrameCount(8),
                12 + target_block_sequence,
            );
            let batch_epoch = runtime
                .projection_epoch()
                .saturating_add(3)
                .saturating_add(offset);
            let transport = TransportProjection {
                playing: true,
                timeline_position_samples: 72 + (offset as i64 * 8),
                tempo_bpm: 120.0,
                loop_state: None,
            };
            assert!(runtime
                .prepare_engine_prework_for_block_with_future_state(
                    1,
                    target_block_sequence,
                    1,
                    block,
                    Some(batch_epoch),
                    Some(transport),
                )
                .unwrap());
        }

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 3);
        assert_eq!(primed.prework_cache_peak_queue_depth, 3);
        assert_eq!(primed.prework_cache_queued_admissions, 4);
        assert_eq!(primed.prework_cache_invalidation_count, 1);
        assert_eq!(
            primed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::QueueCapacityExceeded)
        );
        assert_eq!(
            primed.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::QueueCapacityExceeded)
        );
        assert_eq!(primed.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_reuses_existing_future_queue_entry_when_target_state_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-reuse".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block.clone(),
                Some(9),
                Some(transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                block,
                Some(9),
                Some(transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 1);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(1));
    }

    #[test]
    fn runtime_replaces_future_queue_entry_when_target_state_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-replace".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let first_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let replacement_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 104,
            tempo_bpm: 121.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 42),
                Some(9),
                Some(first_transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43),
                Some(10),
                Some(replacement_transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 2);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::SupersededByAdmission)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::SupersededByAdmission)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(2));
    }

    #[test]
    fn runtime_planning_window_retires_future_entries_not_in_revised_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-revision".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: 2,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 52),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: 3,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(1, targets)
                .expect("initial planning window"),
            2
        );

        let revised_targets = vec![RuntimePreworkWindowTarget {
            target_block_sequence: 3,
            admitted_from_block_sequence: 2,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
            parameter_epoch_override: Some(10),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 104,
                tempo_bpm: 121.0,
                loop_state: None,
            }),
        }];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(2, revised_targets)
                .expect("revised planning window"),
            1
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningWindowRevised)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::PlanningWindowRevised)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_planning_window_reuses_existing_future_sequences_and_allocates_missing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-sequences".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let first_future_sequence = runtime.allocate_block_sequence();
        let second_future_sequence = runtime.allocate_block_sequence();

        let initial_targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: first_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: second_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        runtime
            .prepare_engine_prework_window(1, initial_targets)
            .expect("initial planning window");

        let revised_sequences =
            runtime.plan_prework_window_block_sequences(first_future_sequence, 2);
        assert_eq!(
            revised_sequences,
            vec![
                second_future_sequence,
                second_future_sequence.saturating_add(1)
            ]
        );
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_window_target_count, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![first_future_sequence, second_future_sequence]
        );
    }

    #[test]
    fn runtime_primes_prework_window_from_forecast_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-prework".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 2,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 17,
            transport_playing: true,
            transport_tempo_bpm: 122.0,
            transport_loop_length_blocks: 24,
            parameter_target: "engine.server.balance".into(),
            parameter_cycle_length: 6,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime forecast window");
        assert_eq!(admitted, 2);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));

        let transport = runtime.forecast_transport_projection_for_block(2, &policy);
        assert_eq!(transport.tempo_bpm, 122.0);
        assert_eq!(transport.timeline_position_samples, 512);

        let batch = runtime.forecast_parameter_batch_for_block(2, &policy);
        assert_eq!(batch.epoch, 4);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].target, "engine.server.balance");
        assert!((batch.events[0].normalized_value - 0.4).abs() < 1.0e-6);
    }

    #[test]
    fn runtime_forecast_policy_limits_prework_window_depth() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-limit".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 1,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime limited forecast window");
        assert_eq!(admitted, 1);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
    }

    #[test]
    fn runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded raw forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-budget".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        assert_eq!(runtime.engine.prework_queue.len(), 1);
        assert!(runtime.engine.pending_prework_targets.len() > 1);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert!(snapshot.prework_pending_target_count > 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 8);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );

        runtime.start().expect("start runtime");
        let started = runtime.get_engine_block_snapshot();
        assert_eq!(started.prework_cache_queue_depth, 2);
        assert!(started.prework_pending_target_count > 0);

        let serviced_once = runtime
            .service_prework_lane(1, 1)
            .expect("service pending prework once");
        assert_eq!(serviced_once, 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_cycle_count >= 1);
        assert!(snapshot.prework_service_prepared_targets >= 1);
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(1));
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
        assert!(snapshot.last_prework_service_prepared_targets >= 1);

        let serviced_again = runtime
            .service_prework_lane(1, 2)
            .expect("service pending prework until idle");
        assert!(serviced_again >= 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert!(snapshot.prework_cache_queue_depth >= 3);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_cycle_count >= 2);
        assert!(snapshot.prework_service_prepared_targets >= 3);
        assert_eq!(snapshot.last_prework_service_cycle_count, 2);
        assert_eq!(snapshot.last_prework_service_prepared_targets, 2);
    }

    #[test]
    fn runtime_prework_service_lane_enters_starved_state_when_budget_is_zero() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 0,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set zero-budget forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-starved".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert!(paused.prework_pending_target_count > 0);

        runtime.start().expect("start runtime");
        runtime
            .service_prework_lane(1, 1)
            .expect("service prework lane with zero effective budget");
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Starved
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_starvation_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_resumes_after_start() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-resume".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert!(paused.prework_pending_target_count > 0);

        runtime.start().expect("start runtime");

        let resumed = runtime.get_engine_block_snapshot();
        assert!(matches!(
            resumed.prework_service_state,
            RuntimePreworkServiceState::Pending | RuntimePreworkServiceState::Idle
        ));
        assert!(resumed.prework_service_pause_count >= 1);
        assert!(resumed.prework_service_resume_count >= 1);
        assert!(resumed.prework_service_prepared_targets >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_yields_under_critical_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-critical".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Critical)
            .expect("set critical prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under critical pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Critical
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0)
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_throttles_under_elevated_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-elevated".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under elevated pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert!(snapshot.last_prework_service_effective_cycles <= 1);
        assert!(matches!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0 | 1)
        ));
        assert!(snapshot.prework_service_throttle_count >= 1);
        assert!(
            snapshot.prework_service_prepared_targets >= 1
                || snapshot.prework_service_yield_count >= 1
        );
    }

    #[test]
    fn runtime_elevated_pressure_preserves_deferred_prework_targets() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set elevated forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-backlog-classes".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");
        runtime
            .service_prework_lane(2, 3)
            .expect("service elevated lane second cycle");
        runtime
            .service_prework_lane(3, 3)
            .expect("service elevated lane third cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
        assert_eq!(snapshot.prework_pending_near_term_target_count, 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert_eq!(
            snapshot.prework_pending_target_count,
            snapshot.prework_pending_deferred_target_count
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_latency_focused_graph_expands_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set latency-focused forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:latency-focused-prework-priority".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(2)
        );
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle
        );
        assert_eq!(
            snapshot.last_prework_serviced_backlog_class,
            Some(RuntimePreworkBacklogClass::Deferred)
        );
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 8,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-constrained-prework-priority".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(1);
        runtime.start().expect("start runtime");
        seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.plugin_backed_node_count, 1);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_policy_tracks_active_plugin_sandbox_count() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-policy-tracking".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        runtime.set_active_plugin_sandboxes(1);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        runtime.set_active_plugin_sandboxes(0);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
    }

    #[test]
    fn runtime_plugin_constrained_lane_yields_when_multiple_plugin_sandboxes_are_active() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(2);
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_active_plugin_sandboxes, 2);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-bound".into(),
                }],
            })
            .expect("apply plugin-backed bindings");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 1);
        assert!(snapshot.planned_nodes.iter().any(|node| {
            node.node_id == "plugin" && node.plugin_sandbox_id.as_deref() == Some("sandbox-bound")
        }));

        runtime
            .begin_transport_session(
                "sandbox-bound",
                "lease-bound",
                "region-bound",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin bound transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-bound",
            "lease-bound",
            "region-bound",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
    }

    #[test]
    fn runtime_consumes_plugin_node_render_batch_on_matching_engine_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-render".into(),
                node_count: 1,
                nodes: vec![GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.2 }],
                }],
            })
            .expect("apply graph");
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-render".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox:render".into(),
                }],
            })
            .expect("apply bindings");
        runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: "graph:runtime:plugin-render".into(),
                processing_epoch: 1,
                block_sequence: 1,
                renders: vec![PluginNodeRender {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox:render".into(),
                    output: AudioBuffer::from_interleaved(
                        SampleRate(48_000),
                        ChannelLayout::Stereo,
                        vec![0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625],
                    ),
                    latency_samples: 24,
                    tail_samples: 40,
                    bypassed: false,
                }],
            })
            .expect("apply plugin node render batch");

        let first = runtime
            .process_engine_block(
                1,
                1,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process plugin render block");
        let second = runtime
            .process_engine_block(
                1,
                2,
                AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
            )
            .expect("process fallback block");

        assert_eq!(
            first.output.samples(),
            &[0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625]
        );
        assert_eq!(first.snapshot.output_tail_samples, 40);
        assert_eq!(second.output.samples(), &[0.0; 8]);
        assert_eq!(second.snapshot.output_tail_samples, 0);
    }

    #[test]
    fn runtime_degraded_bound_plugin_session_gates_prework_lane() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-bound forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply plugin-backed bindings");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late detach fault".into()),
        );

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated prework lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_realtime_block_services_prework_window_under_normal_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set realtime-driven forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-normal");
        runtime.start().expect("start runtime");

        let before = runtime.get_engine_block_snapshot();
        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert_eq!(
            first.snapshot.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert!(snapshot.prework_service_cycle_count > before.prework_service_cycle_count);
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(4)
        );
        assert!(snapshot.last_prework_service_prepared_targets >= 1);
        assert!(snapshot
            .last_prework_serviced_target_block_sequence
            .is_some_and(|block_sequence| block_sequence >= 5));
        assert_eq!(
            snapshot.last_prework_serviced_backlog_class,
            Some(RuntimePreworkBacklogClass::Deferred)
        );
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle
        );
        assert!(snapshot
            .prework_cache_window_target_block_sequences
            .contains(&5));
    }

    #[test]
    fn runtime_realtime_block_respects_elevated_pressure_backlog_limits() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set elevated realtime-driven forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-elevated");
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert_eq!(
            first.snapshot.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
        assert!(snapshot
            .prework_next_pending_target_block_sequence
            .is_some());
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_recovery_overlap_throttles_realtime_scheduler_under_normal_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set recovery-overlap realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-overlap");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin steady session");
        runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .expect("begin overlap session");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block")
            .snapshot;

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Normal
        );
        assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 1);
        assert_eq!(snapshot.prework_service_lingering_sessions, 0);
        assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
        assert!(!snapshot.prework_service_transport_gate_active);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(1)
        );
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert!(snapshot.prework_service_throttle_count >= 1);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(report.degradation_summary.recovery_overlap_sessions, 1);
        assert_eq!(report.degradation_summary.lingering_sessions, 0);
        assert!(!report.degradation_summary.transport_gate_active);
        assert_eq!(
            report.scheduler_summary.prework_pending_target_count,
            snapshot.prework_pending_target_count
        );
        assert!(report
            .render_compact()
            .contains("degradation_summary_sessions=1/0/0/0"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("degradation_summary_recovery_overlap_sessions=1"));
        let json = supervisor.render_json();
        assert!(json.contains("\"degradation_summary\":{"));
        assert!(json.contains("\"recovery_overlap_sessions\":1"));
        assert!(json.contains("\"lingering_sessions\":0"));
    }

    #[test]
    fn runtime_lingering_transport_enters_yielding_scheduler_state_under_elevated_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 4,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set lingering realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-lingering");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin steady session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated pressure");
        seed_pending_prework_targets(&mut runtime, 1, &[2, 3, 4]);
        runtime.refresh_prework_service_policy_and_state(None);
        let snapshot = runtime.get_engine_block_snapshot();

        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 0);
        assert_eq!(snapshot.prework_service_lingering_sessions, 1);
        assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
        assert!(snapshot.prework_service_transport_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
    }

    #[test]
    fn runtime_restart_and_reconfigure_keep_realtime_scheduler_window_coherent() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set restart forecast policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-restart");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        let first = runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");
        assert!(first
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&4));

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart runtime");
        let restarted = runtime.get_engine_block_snapshot();
        assert_eq!(
            restarted.prework_cache_window_target_block_sequences,
            vec![2, 3, 4]
        );

        let restart_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let after_restart = runtime
            .process_engine_block(2, 2, restart_block)
            .expect("process realtime block after restart");
        assert!(after_restart
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&5));
        assert_eq!(
            after_restart.snapshot.last_prework_service_processing_epoch,
            Some(2)
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure runtime");
        let reconfigured = runtime.get_engine_block_snapshot();
        assert_eq!(
            reconfigured.prework_cache_window_target_block_sequences,
            vec![3, 4, 5]
        );
        assert_eq!(
            reconfigured.prework_service_state,
            RuntimePreworkServiceState::Paused
        );

        let reconfigured_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3);
        runtime.start().expect("restart after reconfigure");
        apply_current_forecast_block_state(&mut runtime, 3);
        let after_reconfigure = runtime
            .process_engine_block(3, 3, reconfigured_block)
            .expect("process realtime block after reconfigure");
        assert!(after_reconfigure
            .snapshot
            .prework_cache_window_target_block_sequences
            .contains(&6));
        assert_eq!(
            after_reconfigure
                .snapshot
                .last_prework_service_processing_epoch,
            Some(3)
        );

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(report.control_snapshot.restart_count, 1);
        assert!(report.scheduler_summary.prework_pending_target_count > 0);
        assert!(report.render_compact().contains("restarts=1"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor.render_multiline().contains("restart_count=1"));
        assert!(supervisor
            .render_multiline()
            .contains("scheduler_summary_pending_targets="));
        let json = supervisor.render_json();
        assert!(json.contains("\"restart_count\":1"));
        assert!(json.contains("\"scheduler_summary\":{"));
    }

    #[test]
    fn runtime_forecast_profile_change_keeps_realtime_scheduler_coherent() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set initial realtime policy");
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-profile-change");
        runtime.start().expect("start runtime");

        let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
        apply_current_forecast_block_state(&mut runtime, 1);
        runtime
            .process_engine_block(1, 1, first_block)
            .expect("process first realtime block");

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("switch forecast profile while running");

        let reprofiled = runtime.get_engine_block_snapshot();
        assert_eq!(
            reprofiled.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            reprofiled.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            reprofiled.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            reprofiled.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
        assert_eq!(
            reprofiled.prework_service_state,
            RuntimePreworkServiceState::Pending
        );
        assert!(reprofiled.prework_pending_target_count > 0);

        let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
        apply_current_forecast_block_state(&mut runtime, 2);
        let snapshot = runtime
            .process_engine_block(2, 2, second_block)
            .expect("process second realtime block after profile change")
            .snapshot;

        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
        assert!(snapshot
            .prework_cache_window_target_block_sequences
            .contains(&6));
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
        assert!(matches!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Idle | RuntimePreworkServiceState::Pending
        ));
    }

    #[test]
    fn runtime_apply_forecast_state_primes_window_and_applies_current_block_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-advance".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: None,
            })
            .expect("set prework forecast profile");

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 2);

        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.tempo_bpm),
            Some(122.0)
        );
        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.timeline_position_samples),
            Some(0)
        );
        assert_eq!(
            runtime.latest_parameter_epoch,
            runtime
                .forecast_parameter_batch_for_block(
                    current_sequence,
                    &SignalRuntime::prework_forecast_policy_for_profile(
                        RuntimePreworkForecastProfileSelection {
                            profile: RuntimePreworkForecastProfile::Server,
                            target_window_blocks_override: None,
                        },
                    ),
                )
                .epoch
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));
    }

    #[test]
    fn runtime_reconfigure_uses_role_default_after_requested_mode_is_reset() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set prework forecast profile");
        assert_eq!(
            runtime.get_engine_block_snapshot().prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("reset requested mode to runtime role default");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-role-default-after-reconfigure".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("forecast apply should use runtime-role default");
        assert_eq!(admitted, 2);
    }

    #[test]
    fn runtime_selects_forecast_profile_with_target_window_override() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-profile-override".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set prework forecast profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 4);
    }

    #[test]
    fn runtime_configure_seeds_default_forecast_profile_from_runtime_role() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
    }

    #[test]
    fn runtime_can_disable_and_restore_role_default_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-mode-toggle".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
            .expect("disable prework forecast mode");
        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(disabled_snapshot.prework_forecast_policy_configured);
        assert_eq!(
            disabled_snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningDisabled)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state while disabled");
        assert_eq!(admitted, 0);

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("restore role-default forecast mode");
        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        let next_block_sequence = runtime.allocate_block_sequence();
        let restored_admitted = runtime
            .apply_forecast_state_for_block(2, next_block_sequence)
            .expect("apply forecast state after restore");
        assert_eq!(restored_admitted, 2);
    }

    #[test]
    fn runtime_retires_queued_prework_when_forecast_profile_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-change".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("switch explicit profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_pending_target_count, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
        assert_eq!(snapshot.prework_cache_queued_admissions, 4);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_rebuilds_missing_queued_prework_when_forecast_window_expands() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-expand".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("expand local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.prework_cache_retirement_count, 0);
    }

    #[test]
    fn runtime_preserves_compatible_queued_prework_when_forecast_mode_changes_but_plan_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-compatible".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: None,
            })
            .expect("switch to explicit profile with matching plan");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_selectively_trims_queued_prework_when_forecast_window_shrinks() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-shrink".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(1),
            })
            .expect("shrink local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
    }

    #[test]
    fn runtime_configure_with_anticipative_disabled_enters_disabled_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, false);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("store explicit profile while anticipative planning is off");
        let explicit_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            explicit_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_retires_queued_prework_when_effective_mode_drops_to_disabled() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:disable-retire".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::RuntimeReconfigured)
        );
    }

    #[test]
    fn runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:auto-prime-on-graph-apply".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .expect("apply graph projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_start_rebuilds_prework_window_after_runtime_stop() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:restart-rebuild".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .expect("apply graph projection");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime.start().expect("start runtime");
        runtime
            .stop(StopReason::UserRequested)
            .expect("stop runtime");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            0
        );

        runtime.start().expect("restart runtime");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_reconfigure_preserves_explicit_forecast_profile_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set explicit forecast profile");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
    }

    #[test]
    fn runtime_restores_requested_explicit_forecast_mode_after_anticipative_reenable() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set explicit forecast profile");

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reenable anticipative");

        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_restart_preserves_raw_forecast_override_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime.start().expect("start runtime");

        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 5,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 11,
                transport_playing: true,
                transport_tempo_bpm: 130.0,
                transport_loop_length_blocks: 12,
                parameter_target: "engine.test.raw".into(),
                parameter_cycle_length: 9,
            })
            .expect("set raw forecast policy");

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart without reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(5)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride)
        );
    }

    #[test]
    fn runtime_prework_cache_expires_by_block_sequence_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:block-expiry".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(1, 2, block.clone()).unwrap();
        let third = runtime.process_engine_block(1, 3, block.clone()).unwrap();
        let fourth = runtime.process_engine_block(1, 4, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_consumptions, 3);
        assert_eq!(
            third.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Exhausted
        );
        assert_eq!(third.snapshot.prework_cache_remaining_valid_blocks, Some(0));
        assert_eq!(
            fourth.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::BlockSequenceExpired)
        );
        assert_eq!(
            fourth.snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::BlockSequenceExpired)
        );
        assert_eq!(fourth.snapshot.last_prework_retired_unconsumed, Some(false));
        assert_eq!(fourth.snapshot.prework_cache_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_consumed_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(fourth.snapshot.prework_cache_misses, 2);
        assert_eq!(
            fourth.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(fourth.snapshot.prework_cache_consumptions, 4);
        assert_eq!(
            fourth.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(
            fourth.snapshot.prework_cache_valid_until_block_sequence,
            Some(6)
        );
        assert_eq!(fourth.snapshot.last_prework_source_block_sequence, Some(4));
    }

    #[test]
    fn runtime_invalidates_prework_cache_on_parameter_and_transport_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_disabled_forecast(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:invalidate");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 21);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(1, 2, 1, block.clone(), None, None)
            .unwrap());

        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch().saturating_add(1),
                events: vec![ParameterEvent {
                    target: "invalidate.param".into(),
                    normalized_value: 0.25,
                }],
            })
            .unwrap();
        let after_parameter = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_parameter.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(after_parameter.last_prework_invalidation_reason, None);

        let second = runtime.process_engine_block(2, 2, block.clone()).unwrap();
        assert_eq!(second.snapshot.prework_cache_misses, 2);
        assert!(!second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 2);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(2, 3, 2, block.clone(), None, None)
            .unwrap());

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 130.0,
                loop_state: None,
            })
            .unwrap();
        let after_transport = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_transport.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        );
        assert_eq!(
            after_transport.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStarted)
        );
        assert_eq!(after_transport.prework_cache_invalidation_count, 2);
        assert_eq!(after_transport.prework_cache_retirement_count, 2);
        assert_eq!(
            after_transport.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::TransportStarted)
        );
        assert_eq!(after_transport.last_prework_retired_unconsumed, Some(false));
        assert_eq!(after_transport.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(after_transport.prework_cache_consumed_retirement_count, 2);
        assert_eq!(
            after_transport.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Invalidated
        );
        assert_eq!(
            after_transport.prework_cache_valid_until_processing_epoch,
            None
        );
    }

    #[test]
    fn restart_reconfigures_runtime() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .restart(RestartRequest {
                reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
            })
            .unwrap();

        assert_eq!(runtime.get_effective_config().sample_rate.0, 44_100);
        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
    }

    #[test]
    fn transport_projection_rejects_non_positive_tempo() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 0.0,
                loop_state: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn runtime_classifies_transport_invalidation_boundaries() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-boundaries");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let started = runtime.get_engine_block_snapshot();
        assert_eq!(
            started.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStarted)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );

        runtime.process_engine_block(2, 2, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let seeked = runtime.get_engine_block_snapshot();
        assert_eq!(
            seeked.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportSeeked)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );

        runtime.process_engine_block(3, 3, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 520,
                tempo_bpm: 130.0,
                loop_state: None,
            })
            .unwrap();
        let tempo_changed = runtime.get_engine_block_snapshot();
        assert_eq!(
            tempo_changed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportTempoChanged)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::TempoChanged)
        );

        runtime.process_engine_block(4, 4, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 528,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            })
            .unwrap();
        let loop_state_changed = runtime.get_engine_block_snapshot();
        assert_eq!(
            loop_state_changed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportLoopStateChanged)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopStateChanged)
        );

        runtime.process_engine_block(5, 5, block).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 536,
                tempo_bpm: 130.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 256,
                    end_samples: 1024,
                }),
            })
            .unwrap();
        let stopped = runtime.get_engine_block_snapshot();
        assert_eq!(
            stopped.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportStopped)
        );
        assert_eq!(
            runtime.get_timeline_snapshot().last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Stopped)
        );
    }

    #[test]
    fn runtime_records_transport_progression_in_timeline_and_engine_snapshot() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-progression");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let result = runtime.process_engine_block(2, 2, block).unwrap();
        assert_eq!(result.snapshot.transport_epoch, 1);
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(result.snapshot.transport_block_start_samples, Some(64));
        assert_eq!(result.snapshot.transport_block_end_samples, Some(72));
        assert!(!result.snapshot.transport_loop_wrapped);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.transport_epoch, 1);
        assert_eq!(
            timeline.last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(timeline.last_transport_transition_block_sequence, Some(2));
        assert_eq!(timeline.last_transport_playing, Some(true));
        assert_eq!(timeline.last_transport_tempo_bpm, Some(120.0));
        assert_eq!(timeline.last_transport_timeline_position_samples, Some(72));
        assert_eq!(timeline.last_engine_block_start_samples, Some(64));
        assert_eq!(timeline.last_engine_block_end_samples, Some(72));
        assert_eq!(timeline.loop_wrap_count, 0);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        let compact = report.render_compact();
        assert!(compact.contains("transport_epoch=1"));
        assert!(compact.contains("engine_transport_transition=Some(Started)"));
        let json = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        )
        .render_json();
        assert!(json.contains("\"transport_epoch\":1"));
        assert!(json.contains("\"transport_transition\":\"Started\""));

        let transport = runtime.get_transport_observation_snapshot();
        assert_eq!(transport.transport_epoch, 1);
        assert_eq!(transport.projected_playing, Some(true));
        assert_eq!(transport.projected_tempo_bpm, Some(120.0));
        assert_eq!(transport.projected_timeline_position_samples, Some(72));
        assert_eq!(transport.observed_playing, Some(true));
        assert_eq!(transport.observed_tempo_bpm, Some(120.0));
        assert_eq!(transport.observed_timeline_position_samples, Some(72));
        assert_eq!(
            transport.last_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(transport.last_transition_block_sequence, Some(2));
        assert_eq!(transport.last_engine_block_start_samples, Some(64));
        assert_eq!(transport.last_engine_block_end_samples, Some(72));
        assert_eq!(transport.loop_wrap_count, 0);
    }

    #[test]
    fn runtime_seek_invalidation_projects_into_export_summaries_on_real_engine_path() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:seek-export");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime.process_engine_block(2, 2, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        let boundary_report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(
            boundary_report
                .block_summary
                .last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportSeeked)
        );

        let result = runtime.process_engine_block(3, 3, block).unwrap();
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );
        assert_eq!(
            result.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
        );

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert_eq!(
            report.block_summary.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
        );
        assert_eq!(
            report.block_summary.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
        );

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_transition=Some(Seeked)"));
        let json = supervisor.render_json();
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"transport_transition\":\"Seeked\""));
        assert!(json.contains("\"last_prework_invalidation_reason\":\"ProcessingEpochExpired\""));
    }

    #[test]
    fn runtime_records_loop_wrap_as_transport_boundary() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:loop-wrap");

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 51);
        runtime.process_engine_block(1, 1, block.clone()).unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 60,
                tempo_bpm: 120.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 32,
                    end_samples: 68,
                }),
            })
            .unwrap();

        let result = runtime.process_engine_block(2, 2, block).unwrap();
        assert_eq!(result.snapshot.transport_epoch, 2);
        assert_eq!(
            result.snapshot.transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
        );
        assert_eq!(result.snapshot.transport_block_start_samples, Some(60));
        assert_eq!(result.snapshot.transport_block_end_samples, Some(32));
        assert!(result.snapshot.transport_loop_wrapped);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.transport_epoch, 2);
        assert_eq!(
            timeline.last_transport_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopWrapped)
        );
        assert_eq!(timeline.last_transport_transition_processing_epoch, Some(2));
        assert_eq!(timeline.last_transport_transition_block_sequence, Some(2));
        assert_eq!(timeline.last_transport_timeline_position_samples, Some(32));
        assert_eq!(timeline.last_engine_block_start_samples, Some(60));
        assert_eq!(timeline.last_engine_block_end_samples, Some(32));
        assert_eq!(timeline.loop_wrap_count, 1);

        let transport = runtime.get_transport_observation_snapshot();
        assert_eq!(transport.transport_epoch, 2);
        assert_eq!(transport.projected_timeline_position_samples, Some(32));
        assert_eq!(transport.observed_timeline_position_samples, Some(32));
        assert_eq!(
            transport.last_transition,
            Some(crate::interfaces::RuntimeTransportTransitionKind::LoopWrapped)
        );
        assert_eq!(transport.last_transition_processing_epoch, Some(2));
        assert_eq!(transport.last_transition_block_sequence, Some(2));
        assert_eq!(transport.last_engine_block_start_samples, Some(60));
        assert_eq!(transport.last_engine_block_end_samples, Some(32));
        assert_eq!(transport.loop_wrap_count, 1);

        let report = crate::interfaces::RuntimeObservationReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(report.block_summary.transport_loop_wrapped);
        assert_eq!(report.block_summary.transport_epoch, 2);
        assert!(report
            .render_compact()
            .contains("block_summary_transport=2/Some(Started)/true"));

        let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
            &runtime,
            &RuntimeEventRecorder::default(),
        );
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_loop_wrapped=true"));
        let json = supervisor.render_json();
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"transport_loop_wrapped\":true"));
    }

    #[test]
    fn runtime_recording_capture_buffers_output_and_commits_wav() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-capture");

        let capture_path = temp_capture_path("recording-capture");
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 2_048,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                take_id: "take:test:0001".to_string(),
                track_id: "track:test:0001".to_string(),
                start_samples: 2_048,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();

        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(16), 77),
            )
            .unwrap();

        let recording = runtime.get_recording_capture_snapshot();
        assert!(recording.capture_ready);
        assert_eq!(
            recording.state,
            Some(RuntimeRecordingCaptureState::Capturing)
        );
        assert_eq!(recording.active_take_id.as_deref(), Some("take:test:0001"));
        assert_eq!(recording.buffered_block_count, 1);
        assert_eq!(recording.buffered_frame_count, 16);
        assert_eq!(recording.captured_channel_count, 2);

        let receipt = runtime.finish_recording_capture().unwrap();
        assert_eq!(receipt.take_id, "take:test:0001");
        assert_eq!(receipt.duration_samples, 16);
        assert_eq!(receipt.channel_count, 2);
        assert!(capture_path.exists());

        let committed = runtime.get_recording_capture_snapshot();
        assert_eq!(committed.state, Some(RuntimeRecordingCaptureState::Idle));
        assert_eq!(
            committed.last_committed_path.as_deref(),
            Some(capture_path.to_string_lossy().as_ref())
        );
        assert_eq!(committed.last_committed_duration_samples, Some(16));

        let _ = fs::remove_file(capture_path);
    }

    #[test]
    fn runtime_recording_capture_cancels_without_committing_file() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        apply_latency_runtime_graph(&mut runtime, "graph:runtime:recording-cancel");

        let capture_path = temp_capture_path("recording-cancel");
        runtime
            .start_recording_capture(RuntimeRecordingCaptureStartRequest {
                take_id: "take:test:cancel".to_string(),
                track_id: "track:test:cancel".to_string(),
                start_samples: 512,
                capture_path: capture_path.display().to_string(),
            })
            .unwrap();
        runtime
            .process_engine_block(
                1,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 33),
            )
            .unwrap();
        runtime.cancel_recording_capture().unwrap();

        let recording = runtime.get_recording_capture_snapshot();
        assert_eq!(recording.state, Some(RuntimeRecordingCaptureState::Idle));
        assert_eq!(recording.active_take_id, None);
        assert_eq!(recording.last_committed_path, None);
        assert!(!capture_path.exists());
    }

    #[test]
    fn runtime_reconciles_media_assets_into_shared_ready_cache_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("media-imported");
        let recorded_path = temp_capture_path("media-recorded");
        write_test_wav(&imported_path);
        write_test_wav(&recorded_path);

        runtime
            .reconcile_media_assets(vec![
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:imported".to_string(),
                    content_hash: "imported".to_string(),
                    source_path: imported_path.display().to_string(),
                    file_name: "imported.wav".to_string(),
                    byte_size: fs::metadata(&imported_path).unwrap().len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:recorded".to_string(),
                    content_hash: "recorded".to_string(),
                    source_path: recorded_path.display().to_string(),
                    file_name: "recorded.wav".to_string(),
                    byte_size: fs::metadata(&recorded_path).unwrap().len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 8,
                },
            ])
            .unwrap();

        let snapshot = runtime.get_media_pipeline_snapshot();
        assert_eq!(snapshot.asset_count, 2);
        assert_eq!(snapshot.ready_asset_count, 2);
        assert_eq!(snapshot.invalid_asset_count, 0);
        assert!(snapshot.assets.iter().all(|asset| {
            asset.state == Some(RuntimeMediaAssetState::Ready)
                && asset.cache_path.as_deref().is_some()
        }));

        let cached_path = PathBuf::from(
            snapshot.assets[0]
                .cache_path
                .as_deref()
                .expect("cached media should exist"),
        );
        fs::remove_file(&cached_path).unwrap();

        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:imported".to_string(),
                content_hash: "imported".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "imported.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();

        let rebuilt = runtime.get_media_pipeline_snapshot();
        assert_eq!(rebuilt.asset_count, 1);
        assert_eq!(rebuilt.ready_asset_count, 1);
        assert_eq!(rebuilt.assets[0].state, Some(RuntimeMediaAssetState::Ready));
        assert!(rebuilt.assets[0].rebuild_count >= 1);

        let _ = fs::remove_file(imported_path);
        let _ = fs::remove_file(recorded_path);
        if let Some(path) = rebuilt.assets[0].cache_path.as_deref() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn runtime_reconciles_warp_clips_against_media_readiness_and_project_tempo() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let imported_path = temp_capture_path("warp-ready");
        write_test_wav(&imported_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:warp-ready".to_string(),
                content_hash: "warp-ready".to_string(),
                source_path: imported_path.display().to_string(),
                file_name: "warp-ready.wav".to_string(),
                byte_size: fs::metadata(&imported_path).unwrap().len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 8,
            }])
            .unwrap();
        runtime
            .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
                clip_id: "clip:warp-ready".to_string(),
                media_asset_id: Some("asset:sha256:warp-ready".to_string()),
                mode: RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .unwrap();

        let ready = runtime.get_warp_pipeline_snapshot();
        assert_eq!(ready.clip_count, 1);
        assert_eq!(ready.ready_clip_count, 1);
        assert_eq!(ready.degraded_clip_count, 0);
        assert_eq!(ready.clips[0].readiness, RuntimeWarpReadiness::Ready);
        assert!((ready.clips[0].realized_ratio - 1.5).abs() < 0.000_1);

        runtime
            .apply_transport_projection(TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 300.0,
                loop_state: None,
            })
            .unwrap();
        let degraded = runtime.get_warp_pipeline_snapshot();
        assert_eq!(degraded.ready_clip_count, 0);
        assert_eq!(degraded.degraded_clip_count, 1);
        assert_eq!(degraded.clips[0].readiness, RuntimeWarpReadiness::Degraded);
        assert!(
            degraded.clips[0]
                .last_error
                .as_deref()
                .unwrap_or_default()
                .contains("outside baseline support")
        );

        let _ = fs::remove_file(imported_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn runtime_emits_events_to_subscribers() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let sink = Box::new(TestSink::default());
        runtime.subscribe(sink);

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
        runtime.set_active_output_device("coreaudio:default");
        runtime.set_active_plugin_sandboxes(2);

        let readiness = runtime.get_readiness();
        assert_eq!(readiness, RuntimeReadiness::Ready);
        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            2
        );
    }

    #[test]
    fn runtime_records_plugin_fault_events() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::interfaces::PluginFaultKind::ProtocolViolation,
            "epoch mismatch",
            Some(3),
        );

        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            0
        );
    }

    #[test]
    fn runtime_owns_watchdog_restart_escalation() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        let first = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        assert_eq!(first.watchdog_restart_count, 1);
        assert!(!first.safe_mode_enabled);

        let second = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });
        assert_eq!(second.watchdog_restart_count, 2);
        assert!(second.safe_mode_enabled);
        assert_eq!(
            second.last_watchdog_trigger,
            Some(RuntimeWatchdogTrigger::DeadlineMisses)
        );
        assert_eq!(second.last_processing_epoch, Some(2));
        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn runtime_event_recorder_builds_reusable_observation_diagnostics() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SupervisionChanged(crate::interfaces::RuntimeSupervisionSnapshot {
                watchdog_restart_count: 2,
                safe_mode_enabled: true,
                last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
                last_sandbox_id: Some("sandbox-a".into()),
                last_processing_epoch: Some(4),
            }),
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "heartbeat watchdog missed twice".into(),
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "block deadline missed twice".into(),
                processing_epoch: Some(3),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxInstanceState {
                state: crate::interfaces::PluginSandboxInstanceStateRecord {
                    sandbox_id: "sandbox-a".into(),
                    plugin_type_id: "plugin:clap:default".into(),
                    instance_id: "instance:runtime:default".into(),
                    lifecycle_state: "Active".into(),
                    readiness_state: "Ready".into(),
                    degraded_reasons: Vec::new(),
                    active: true,
                    processing_epoch: Some(4),
                    processing_sample_rate_hz: Some(48_000),
                    processing_max_block_frames: Some(512),
                    audio_inputs: Some(2),
                    audio_outputs: Some(2),
                    midi_inputs: Some(1),
                    midi_outputs: Some(0),
                    last_fault: None,
                },
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-a".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-a".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-a".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 12,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(12),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        assert_eq!(diagnostics.total_events, 18);
        assert_eq!(diagnostics.supervision_update_count(), 1);
        assert_eq!(diagnostics.plugin_fault_count(), 2);
        assert_eq!(diagnostics.plugin_instance_state_event_count(), 1);
        assert_eq!(diagnostics.recovery_event_count(), 1);
        assert_eq!(diagnostics.lifecycle_event_count(), 1);
        assert_eq!(diagnostics.transport_event_count(), 4);
        assert_eq!(diagnostics.heartbeat_event_count(), 1);
        assert_eq!(diagnostics.block_dispatch_event_count(), 1);
        assert_eq!(diagnostics.lease_rollover_event_count(), 1);
        assert_eq!(diagnostics.invalidation_event_count(), 1);
        assert_eq!(diagnostics.completion_slot_event_count(), 2);
        assert_eq!(diagnostics.transport_fault_event_count(), 8);
        assert_eq!(diagnostics.broker_failure_event_count(), 1);
        assert_eq!(diagnostics.sandbox_operation_failure_event_count(), 1);
        assert_eq!(diagnostics.fault_detail_count_containing("watchdog"), 1);
        assert_eq!(
            diagnostics.fault_detail_count_containing("block deadline"),
            1
        );
        assert_eq!(
            diagnostics.last_plugin_instance_state().map(|state| (
                state.instance_id.as_str(),
                state.lifecycle_state.as_str(),
                state.readiness_state.as_str(),
                state.processing_sample_rate_hz,
            )),
            Some(("instance:runtime:default", "Active", "Ready", Some(48_000)))
        );
        assert_eq!(
            diagnostics
                .last_supervision_update()
                .and_then(|snapshot| snapshot.last_processing_epoch),
            Some(4)
        );
        assert_eq!(
            diagnostics
                .last_recovery_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics
                .last_lifecycle_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics.last_transport_event().map(|event| event.stage),
            Some(PluginSandboxTransportStage::DetachFault)
        );
        assert_eq!(
            diagnostics
                .transport_events
                .first()
                .map(|event| event.region_id.as_str()),
            Some("region-4")
        );
        assert_eq!(
            diagnostics
                .last_heartbeat_event()
                .map(|event| event.block_sequence),
            Some(Some(12))
        );
        assert_eq!(
            diagnostics
                .last_block_dispatch_event()
                .map(|event| event.completion_state),
            Some(Some(CompletionState::Completed))
        );
        assert_eq!(
            diagnostics
                .last_lease_rollover_event()
                .map(|event| event.previous_lease_id.as_str()),
            Some("lease-3")
        );
        assert_eq!(
            diagnostics
                .last_invalidation_event()
                .map(|event| event.reason.as_str()),
            Some("watchdog recovery teardown")
        );
        assert_eq!(
            diagnostics
                .last_completion_slot_event()
                .map(|event| event.stage),
            Some(CompletionSlotStage::FallbackApplied)
        );
        assert_eq!(
            diagnostics.last_transport_fault_event().map(|event| (
                event.source,
                event.stage,
                event.phase,
                event.resource
            )),
            Some((
                crate::interfaces::TransportFaultSource::SandboxOperation,
                crate::interfaces::TransportFaultStage::ProcessAttach,
                crate::interfaces::TransportFaultPhase::Dispatch,
                crate::interfaces::TransportFaultResource::SharedMemoryLease,
            ))
        );
        assert_eq!(
            diagnostics
                .last_broker_failure_event()
                .map(|event| event.stage),
            Some(BrokerFailureStage::PayloadRead)
        );
        assert_eq!(
            diagnostics
                .last_sandbox_operation_failure_event()
                .map(|event| event.stage),
            Some(SandboxOperationFailureStage::ProcessAttach)
        );
        assert!(diagnostics.render_compact().contains("plugin_faults=2"));
        assert!(diagnostics
            .render_compact()
            .contains("plugin_instance_states=1"));
        assert!(diagnostics.render_compact().contains("recovery_events=1"));
        assert!(diagnostics.render_compact().contains("lifecycle_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("block_dispatch_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("lease_rollover_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("invalidation_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("completion_slot_events=2"));
        assert!(diagnostics
            .render_compact()
            .contains("transport_fault_events=8"));
        assert!(diagnostics
            .render_compact()
            .contains("broker_failure_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("sandbox_operation_failure_events=1"));

        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);
        let second_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second_sequence);
        let report = RuntimeObservationReport::capture(&runtime, &recorder);
        assert!(report.render_compact().contains("readiness=Ready"));
        assert!(report.render_compact().contains("handshaken=true"));
        assert!(report.render_compact().contains("configures=1"));
        assert!(report.render_compact().contains("plugin_faults=2"));
        assert!(report.render_compact().contains("plugin_instance_states=1"));
        assert!(report.render_compact().contains("next_block_sequence=2"));
        assert!(report
            .render_compact()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert!(report
            .render_compact()
            .contains("degradation_summary_faults=2/8/1/1"));
        assert_eq!(report.scheduler_summary.topology_issue_count, 0);
        assert_eq!(report.scheduler_summary.dispatch_count, 0);
        assert!(
            !report
                .scheduler_summary
                .topology_requires_host_reinterpretation
        );
        assert_eq!(report.degradation_summary.plugin_fault_count, 2);
        assert_eq!(report.degradation_summary.transport_fault_event_count, 8);
        assert_eq!(
            report.degradation_summary.last_watchdog_trigger,
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert_eq!(
            report.transport_fault_summary.boundary_mode,
            crate::interfaces::TransportFaultBoundaryMode::FaultAdjacentOnly
        );
        assert_eq!(report.transport_fault_summary.total_events, 8);
        assert_eq!(report.transport_fault_summary.host_broker_events, 4);
        assert_eq!(report.transport_fault_summary.sandbox_operation_events, 1);
        assert_eq!(report.transport_fault_summary.runtime_dispatch_events, 3);
        assert_eq!(report.transport_fault_summary.prepare_events, 0);
        assert_eq!(report.transport_fault_summary.dispatch_events, 4);
        assert_eq!(report.transport_fault_summary.teardown_events, 4);
        assert_eq!(report.transport_fault_summary.control_events, 0);
        assert_eq!(
            report.transport_fault_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(report.transport_fault_summary.last_block_sequence, Some(12));
        assert_eq!(
            report.transport_session_summary.boundary_mode,
            crate::interfaces::TransportSessionBoundaryMode::HealthyPathVisible
        );
        assert_eq!(
            report.transport_session_summary.current_state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert!(!report.transport_session_summary.currently_attached);
        assert_eq!(
            report.transport_session_summary.heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            report.transport_session_summary.dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(report.transport_session_summary.attach_events, 1);
        assert_eq!(report.transport_session_summary.detach_requested_events, 1);
        assert_eq!(report.transport_session_summary.detached_events, 1);
        assert_eq!(report.transport_session_summary.detach_fault_events, 1);
        assert_eq!(
            report.transport_session_summary.heartbeat_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.heartbeat_responded_events,
            1
        );
        assert_eq!(report.transport_session_summary.heartbeat_missed_events, 0);
        assert_eq!(
            report.transport_session_summary.dispatch_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.dispatch_completed_events,
            1
        );
        assert_eq!(
            report.transport_session_summary.dispatch_timed_out_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(
            report.transport_session_summary.last_block_sequence,
            Some(12)
        );
        assert_eq!(
            report
                .transport_session_summary
                .active_sandbox_id
                .as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_lease_id.as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_region_id.as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_block_sequence,
            Some(12)
        );
        assert_eq!(
            report
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(
            report
                .transport_session_summary
                .max_concurrent_attached_sessions,
            1
        );
        assert!(report.transport_session_summary.active_sessions.is_empty());
        assert_eq!(
            report.transport_session_summary.last_sandbox_id.as_deref(),
            Some("sandbox-a")
        );
        assert_eq!(
            report.transport_session_summary.last_lease_id.as_deref(),
            Some("lease-4")
        );
        assert_eq!(
            report.transport_session_summary.last_region_id.as_deref(),
            Some("region-4")
        );
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(supervisor.event_count(), 18);
        assert_eq!(supervisor.supervision_update_count(), 1);
        assert_eq!(supervisor.plugin_fault_count(), 2);
        assert_eq!(supervisor.plugin_instance_state_event_count(), 1);
        assert_eq!(supervisor.recovery_event_count(), 1);
        assert_eq!(supervisor.lifecycle_event_count(), 1);
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(supervisor.render_compact().contains("event_stream=18"));
        assert!(supervisor
            .render_compact()
            .contains("plugin_instance_states=1"));
        assert!(supervisor.render_compact().contains("recovery_events=1"));
        assert!(supervisor.render_compact().contains("lifecycle_events=1"));
        assert!(supervisor.render_multiline().contains("plugin_faults=2"));
        assert!(supervisor
            .render_multiline()
            .contains("recovery_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("lifecycle_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("sequence_segments=1"));
        assert!(supervisor
            .render_multiline()
            .contains("automation_param=4096"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_host_broker_events=4"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_boundary=HealthyPathVisible"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_attach_events=1"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_state=DetachFaulted"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_heartbeat_state=Fresh"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_dispatch_state=Completed"));
        assert!(supervisor
            .render_multiline()
            .contains("scheduler_summary_topology_issue_count=0"));
        assert!(supervisor
            .render_multiline()
            .contains("block_summary_transport_transition=None"));
        assert!(supervisor
            .render_multiline()
            .contains("degradation_summary_transport_fault_events=8"));
        let json = supervisor.render_json();
        assert!(json.contains("\"readiness\":\"Ready\""));
        assert!(json.contains("\"control\":{\"handshaken\":true"));
        assert!(json.contains("\"next_block_sequence\":2"));
        assert!(json.contains("\"sequence_segments\":1"));
        assert!(json.contains("\"plugin_faults\":2"));
        assert!(json.contains("\"recovery_events\":1"));
        assert!(json.contains("\"recovery_sequence\":[{"));
        assert!(json.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(json.contains("\"lifecycle_events\":1"));
        assert!(json.contains("\"lifecycle_sequence\":[{"));
        assert!(json.contains("\"stage\":\"TransportAttached\""));
        assert!(json.contains("\"transport_fault_events\":8"));
        assert!(json.contains("\"last_transport_fault\":{"));
        assert!(json.contains("\"transport_fault_sequence\":[{"));
        assert!(json.contains("\"source\":\"HostBroker\""));
        assert!(json.contains("\"source\":\"SandboxOperation\""));
        assert!(json.contains("\"source\":\"RuntimeDispatch\""));
        assert!(json.contains("\"phase\":\"Dispatch\""));
        assert!(json.contains("\"phase\":\"Teardown\""));
        assert!(json.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(json.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(json.contains("\"resource\":\"CompletionSlot\""));
        assert!(json.contains("\"operation\":\"block_payload.read\""));
        assert!(json.contains("\"operation\":\"transport.detach_request\""));
        assert!(json.contains("\"operation\":\"transport.detached\""));
        assert!(json.contains("\"operation\":\"transport.detach_fault\""));
        assert!(json.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(json.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(json.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(json.contains("\"operation\":\"processBlock\""));
        assert!(json.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(json.contains("\"stage\":\"TransportDetached\""));
        assert!(json.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(json.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(json.contains("\"stage\":\"FallbackApplied\""));
        assert!(json.contains("\"stage\":\"PayloadRead\""));
        assert!(json.contains("\"stage\":\"ProcessAttach\""));
        assert!(json.contains("\"transport_fault_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(json.contains("\"host_broker_events\":4"));
        assert!(json.contains("\"sandbox_operation_events\":1"));
        assert!(json.contains("\"runtime_dispatch_events\":3"));
        assert!(json.contains("\"scheduler_summary\":{"));
        assert!(json.contains("\"topology_issue_count\":0"));
        assert!(json.contains("\"block_summary\":{"));
        assert!(json.contains("\"degradation_summary\":{"));
        assert!(json.contains("\"plugin_fault_count\":2"));
        assert!(json.contains("\"transport_fault_event_count\":8"));
        assert!(json.contains("\"dispatch_events\":4"));
        assert!(json.contains("\"teardown_events\":4"));
        assert!(json.contains("\"transport_session_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(json.contains("\"current_state\":\"DetachFaulted\""));
        assert!(json.contains("\"currently_attached\":false"));
        assert!(json.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(json.contains("\"dispatch_state\":\"Completed\""));
        assert!(json.contains("\"current_attached_session_count\":0"));
        assert!(json.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(json.contains("\"attach_events\":1"));
        assert!(json.contains("\"detach_requested_events\":1"));
        assert!(json.contains("\"detached_events\":1"));
        assert!(json.contains("\"detach_fault_events\":1"));
        assert!(json.contains("\"heartbeat_responded_events\":1"));
        assert!(json.contains("\"dispatch_completed_events\":1"));
        assert!(json.contains("\"active_sandbox_id\":null"));
        assert!(json.contains("\"active_lease_id\":null"));
        assert!(json.contains("\"active_region_id\":null"));
        assert!(json.contains("\"active_block_sequence\":12"));
        assert!(json.contains("\"active_sessions\":[]"));
        assert!(json.contains("\"last_region_id\":\"region-4\""));
        assert!(json.contains("\"automation\":{\"parameter_id\":4096"));
    }

    #[test]
    fn transport_session_summary_tracks_concurrent_active_sessions() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        let summary = crate::interfaces::TransportSessionSummary::from_diagnostics(&diagnostics);
        assert_eq!(summary.current_attached_session_count, 2);
        assert_eq!(summary.max_concurrent_attached_sessions, 2);
        assert_eq!(
            summary.current_state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.currently_attached);
        assert_eq!(summary.active_sessions.len(), 2);
        assert_eq!(summary.active_sandbox_id.as_deref(), Some("sandbox-a"));
        assert_eq!(summary.active_lease_id.as_deref(), Some("lease-a"));
        assert_eq!(summary.active_region_id.as_deref(), Some("region-a"));
        assert_eq!(summary.active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[0].sandbox_id.as_str(), "sandbox-a");
        assert_eq!(
            summary.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.active_sessions[0].currently_attached);
        assert_eq!(
            summary.active_sessions[0].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            summary.active_sessions[0].dispatch_state,
            crate::interfaces::TransportDispatchState::TimedOut
        );
        assert_eq!(summary.active_sessions[0].processing_epoch, Some(4));
        assert_eq!(summary.active_sessions[0].active_block_sequence, Some(11));
        assert_eq!(summary.active_sessions[0].transport_fault_count, 2);
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::RuntimeDispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::CompletionSlotTimedOut)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_processing_epoch,
            Some(4)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_block_sequence,
            Some(11)
        );
        assert_eq!(summary.active_sessions[1].sandbox_id.as_str(), "sandbox-b");
        assert_eq!(
            summary.active_sessions[1].state,
            crate::interfaces::TransportSessionState::AttachActive
        );
        assert!(summary.active_sessions[1].currently_attached);
        assert_eq!(
            summary.active_sessions[1].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            summary.active_sessions[1].dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(summary.active_sessions[1].processing_epoch, Some(5));
        assert_eq!(summary.active_sessions[1].active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[1].transport_fault_count, 1);
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::HostBroker)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::PayloadRead)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_processing_epoch,
            Some(5)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_block_sequence,
            Some(12)
        );
    }

    #[test]
    fn runtime_owns_transport_session_admission_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        let first = runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        assert_eq!(first.current_attached_sessions, 1);
        assert_eq!(first.peak_attached_sessions, 1);
        assert_eq!(first.current_recovery_overlap_sessions, 0);
        assert_eq!(first.current_lingering_sessions, 0);
        assert_eq!(
            first.active_sessions[0].state,
            crate::interfaces::TransportSessionState::AttachActive
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.peak_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.peak_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 0);

        let overlap_reject = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap_err();
        assert_eq!(overlap_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(overlap_reject
            .message
            .contains("recovery overlap session limit 1"));

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.current_attached_sessions, 2);
        assert_eq!(snapshot.peak_attached_sessions, 2);
        assert_eq!(snapshot.current_lingering_sessions, 0);
        assert_eq!(
            snapshot.last_admitted_sandbox_id.as_deref(),
            Some("sandbox-b")
        );
        assert_eq!(
            snapshot.last_rejected_sandbox_id.as_deref(),
            Some("sandbox-c")
        );
        assert!(snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));

        let after_end = runtime.end_transport_session("sandbox-a", "lease-a", "region-a");
        assert_eq!(after_end.current_attached_sessions, 1);
        assert_eq!(after_end.current_recovery_overlap_sessions, 1);
        assert_eq!(after_end.current_lingering_sessions, 0);

        let promoted =
            runtime.promote_transport_session_to_steady_state("sandbox-b", "lease-b", "region-b");
        assert_eq!(promoted.current_attached_sessions, 1);
        assert_eq!(promoted.current_recovery_overlap_sessions, 0);
        assert_eq!(promoted.current_lingering_sessions, 0);
        assert_eq!(
            promoted.active_sessions[0].provenance,
            TransportSessionProvenance::RecoveryReplacement
        );

        let re_admit = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(re_admit.current_attached_sessions, 2);
        assert_eq!(re_admit.current_recovery_overlap_sessions, 1);

        let after_overlap_end = runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        assert_eq!(after_overlap_end.current_attached_sessions, 1);
        assert_eq!(after_overlap_end.current_recovery_overlap_sessions, 1);
        assert_eq!(after_overlap_end.current_lingering_sessions, 0);

        let after_final_end = runtime.end_transport_session("sandbox-c", "lease-c", "region-c");
        assert_eq!(after_final_end.current_attached_sessions, 0);
        assert_eq!(after_final_end.current_recovery_overlap_sessions, 0);
        assert_eq!(after_final_end.current_lingering_sessions, 0);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .unwrap();
        let reset = runtime.get_transport_concurrency_snapshot();
        assert_eq!(reset.current_attached_sessions, 0);
        assert_eq!(reset.current_lingering_sessions, 0);
        assert!(reset.active_sessions.is_empty());
        assert_eq!(reset.peak_attached_sessions, 0);
        assert_eq!(reset.peak_lingering_sessions, 0);
    }

    #[test]
    fn runtime_tracks_lingering_transport_sessions_as_first_class_admission_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(2),
            None,
        );

        let requested = runtime.get_transport_concurrency_snapshot();
        assert_eq!(requested.current_attached_sessions, 1);
        assert_eq!(requested.current_lingering_sessions, 1);
        assert_eq!(requested.peak_lingering_sessions, 1);
        assert_eq!(requested.current_detach_requested_sessions, 1);
        assert_eq!(requested.current_detach_faulted_sessions, 0);
        assert_eq!(
            requested.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(steady_reject.message.contains("lingering session"));

        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("teardown fault".into()),
        );

        let faulted = runtime.get_transport_concurrency_snapshot();
        assert_eq!(faulted.current_attached_sessions, 1);
        assert_eq!(faulted.current_lingering_sessions, 1);
        assert_eq!(faulted.current_detach_requested_sessions, 0);
        assert_eq!(faulted.current_detach_faulted_sessions, 1);
        assert_eq!(
            faulted.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 1);
        assert_eq!(overlap.current_detach_faulted_sessions, 1);
        assert_eq!(overlap.peak_lingering_sessions, 1);

        runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        runtime.end_transport_session("sandbox-a", "lease-a", "region-a");

        let cleared = runtime.get_transport_concurrency_snapshot();
        assert_eq!(cleared.current_attached_sessions, 0);
        assert_eq!(cleared.current_lingering_sessions, 0);
        assert_eq!(cleared.current_detach_requested_sessions, 0);
        assert_eq!(cleared.current_detach_faulted_sessions, 0);
    }

    #[test]
    fn runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-origin",
                "region-origin",
                TransportAttachIntent::SteadyState,
                Some(2),
                TransportSessionProvenance::SteadyOrigin,
                Some("/tmp/signal-origin".into()),
                Some(4096),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("origin detach fault".into()),
        );

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-replacement",
                "region-replacement",
                TransportAttachIntent::RecoveryOverlap,
                Some(3),
                TransportSessionProvenance::RecoveryReplacement,
                Some("/tmp/signal-replacement".into()),
                Some(8192),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            PluginSandboxTransportStage::DetachRequested,
            Some(3),
            None,
        );

        let cleanup_receipt = runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::StrictPreAttach,
                LingeringCleanupTrigger::RecoveryPreAttach,
                4,
                None,
                None,
            )
            .expect("cleanup work should be queued");
        let queued = runtime.get_transport_concurrency_snapshot();
        assert_eq!(queued.pending_cleanup_work_items, 1);
        assert_eq!(queued.pending_deferred_retry_work_items, 0);
        assert_eq!(queued.next_cleanup_epoch, 2);
        assert_eq!(queued.oldest_pending_cleanup_ready_epoch, Some(4));
        assert_eq!(queued.pending_cleanup_waves.len(), 1);
        assert_eq!(queued.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            queued.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(
            queued.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );

        let cleanup_plan = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 4)
            .expect("cleanup plan should dequeue");
        assert_eq!(cleanup_plan.work_id, cleanup_receipt.work_id);
        assert_eq!(cleanup_plan.cleanup_epoch, cleanup_receipt.cleanup_epoch);
        assert_eq!(cleanup_plan.cleanup_wave, cleanup_receipt.cleanup_wave);
        assert_eq!(cleanup_plan.sandbox_id, "sandbox-a");
        assert_eq!(cleanup_plan.mode, LingeringCleanupMode::StrictPreAttach);
        assert_eq!(
            cleanup_plan.trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(cleanup_plan.retry_count, 0);
        assert_eq!(cleanup_plan.processing_epoch, 4);
        assert_eq!(cleanup_plan.ready_at_processing_epoch, 4);
        assert_eq!(cleanup_plan.exclude_lease_id, None);
        assert_eq!(cleanup_plan.exclude_region_id, None);
        let cleanup_candidates = cleanup_plan.candidates;
        assert_eq!(cleanup_candidates.len(), 2);
        assert!(cleanup_candidates[0].attach_sequence < cleanup_candidates[1].attach_sequence);

        assert_eq!(
            cleanup_candidates[0].provenance,
            TransportSessionProvenance::SteadyOrigin
        );
        assert_eq!(cleanup_candidates[0].attach_processing_epoch, Some(2));
        assert_eq!(
            cleanup_candidates[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert_eq!(cleanup_candidates[0].lease_id, "lease-origin");
        assert_eq!(cleanup_candidates[0].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[0].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[0].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[0].cleanup_in_progress);
        assert_eq!(cleanup_candidates[0].last_cleanup_epoch, Some(4));
        assert_eq!(cleanup_candidates[0].last_cleanup_error, None);

        assert_eq!(
            cleanup_candidates[1].provenance,
            TransportSessionProvenance::RecoveryReplacement
        );
        assert_eq!(cleanup_candidates[1].attach_processing_epoch, Some(3));
        assert_eq!(
            cleanup_candidates[1].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert_eq!(cleanup_candidates[1].lease_id, "lease-replacement");
        assert_eq!(cleanup_candidates[1].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[1].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[1].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[1].cleanup_in_progress);

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.active_sessions.len(), 2);
        assert!(snapshot
            .active_sessions
            .iter()
            .all(|session| session.cleanup_in_progress));

        let failed = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            LingeringCleanupMode::StrictPreAttach,
            4,
            "cleanup failed",
        );
        let origin = failed
            .active_sessions
            .iter()
            .find(|session| session.lease_id == "lease-origin")
            .unwrap();
        assert!(!origin.cleanup_in_progress);
        assert_eq!(origin.cleanup_attempt_count, 1);
        assert_eq!(
            origin.last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(origin.last_cleanup_epoch, Some(4));
        assert_eq!(origin.last_cleanup_error.as_deref(), Some("cleanup failed"));

        let retried = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            LingeringCleanupMode::BestEffortPostStart,
            5,
            "late cleanup failed",
        );
        assert_eq!(retried.pending_cleanup_work_items, 1);
        assert_eq!(retried.pending_deferred_retry_work_items, 1);
        assert_eq!(retried.next_cleanup_epoch, 3);
        assert_eq!(retried.oldest_pending_cleanup_ready_epoch, Some(6));
        assert_eq!(retried.pending_cleanup_waves.len(), 1);
        assert_eq!(retried.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            retried.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert!(runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 5)
            .is_none());
        let deferred_retry = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 6)
            .expect("deferred retry should dequeue");
        assert_eq!(deferred_retry.cleanup_epoch, 2);
        assert_eq!(deferred_retry.cleanup_wave, 1);
        assert_eq!(
            deferred_retry.trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert_eq!(deferred_retry.retry_count, 1);
        assert_eq!(
            deferred_retry.mode,
            LingeringCleanupMode::BestEffortPostStart
        );
        assert_eq!(deferred_retry.ready_at_processing_epoch, 6);
        assert_eq!(
            deferred_retry.exclude_lease_id.as_deref(),
            Some("lease-replacement")
        );
        assert_eq!(
            deferred_retry.exclude_region_id.as_deref(),
            Some("region-replacement")
        );

        runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::BestEffortPostStart,
                LingeringCleanupTrigger::PostStartReconciliation,
                7,
                None,
                None,
            )
            .expect("second cleanup wave should queue");
        let next_wave = runtime.get_transport_concurrency_snapshot();
        assert_eq!(next_wave.pending_cleanup_waves.len(), 1);
        assert_eq!(next_wave.pending_cleanup_waves[0].cleanup_wave, 2);
        assert_eq!(
            next_wave.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::PostStartReconciliation
        );
    }

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
}
