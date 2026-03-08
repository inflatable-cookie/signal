//! Rhythm analysis surfaces for Signal.

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_dsp_spectral::{Spectrogram, Stft, StftConfig};
use signal_primitives::{AudioBuffer, Sample, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatTrackerConfig {
    pub stft: StftConfig,
    pub min_bpm: f32,
    pub max_bpm: f32,
    pub beat_tolerance: f32,
}

impl Default for BeatTrackerConfig {
    fn default() -> Self {
        Self {
            stft: StftConfig::new(2048, 512),
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempoCandidate {
    pub bpm: f32,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalTempoPoint {
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub bpm: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempoDiagnostics {
    pub interval_tempi: Vec<LocalTempoPoint>,
    pub windowed_tempi: Vec<LocalTempoPoint>,
    pub median_bpm: f32,
    pub drift_span_bpm: f32,
    pub mean_abs_deviation_bpm: f32,
    pub windowed_median_bpm: f32,
    pub windowed_drift_span_bpm: f32,
    pub windowed_mean_abs_deviation_bpm: f32,
    pub core_windowed_median_bpm: f32,
    pub core_windowed_drift_span_bpm: f32,
    pub core_windowed_mean_abs_deviation_bpm: f32,
    pub boundary_bias_bpm: f32,
    pub trend: TempoTrendDiagnostics,
    pub beat_grid_error: BeatGridErrorDiagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrustLevel {
    Stable,
    Guarded,
    Tentative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoRecommendation {
    UseRefined,
    UseCoreWindow,
    SnapInteger,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoInterpretationReason {
    StableRefinedPulse,
    StableCoreWindow,
    NearIntegerPulse,
    UnstableTempo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationSupport {
    pub core_consensus: Confidence,
    pub drift_stability: Confidence,
    pub grid_stability: Confidence,
    pub integer_closeness: Confidence,
    pub boundary_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretationProfile {
    pub refined_bpm: f32,
    pub core_window_bpm: f32,
    pub nearest_integer_bpm: f32,
    pub snap_error_bpm: f32,
    pub stability_score: Confidence,
    pub boundary_edge_gap_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoInterpretation {
    pub trust: TempoTrustLevel,
    pub recommendation: TempoRecommendation,
    pub reason: TempoInterpretationReason,
    pub recommended_bpm: f32,
    pub snapped_bpm: Option<f32>,
    pub support: TempoInterpretationSupport,
    pub profile: TempoInterpretationProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStateAction {
    Lock,
    Monitor,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoStateReason {
    StableIntegerTempo,
    StableRefinedTempo,
    CoreWindowFallback,
    TempoDeferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityAction {
    Lock,
    Retain,
    Reacquire,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySource {
    CurrentTempo,
    PriorTempo,
    CoreWindow,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityReason {
    StableTempo,
    IntegerTempoSnap,
    CoreWindowCarry,
    RevalidationDecay,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuitySeverity {
    Confirmed,
    Guarded,
    Fragile,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityHistory {
    Reinforcing,
    Preserving,
    Degrading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoContinuityProvenance {
    IntegerSnap,
    StableRefinedEstimate,
    GuardedRefinedEstimate,
    CoreWindowEstimate,
    PriorTempoCarry,
    NoTempo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TempoContinuityExpiry {
    pub guaranteed_until_beats: usize,
    pub downgrade_after_beats: usize,
    pub clear_after_beats: usize,
    pub max_failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityTransition {
    pub after_beats: usize,
    pub action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub reason: TempoContinuityReason,
    pub provenance: TempoContinuityProvenance,
    pub confidence: Confidence,
    pub refresh_strength: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityLifecycle {
    pub refresh: TempoContinuityTransition,
    pub decay: [TempoContinuityTransition; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoContinuityPlan {
    pub action: TempoContinuityAction,
    pub source: TempoContinuitySource,
    pub severity: TempoContinuitySeverity,
    pub history: TempoContinuityHistory,
    pub reason: TempoContinuityReason,
    pub provenance: TempoContinuityProvenance,
    pub confidence: Confidence,
    pub refresh_strength: Confidence,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub expiry: TempoContinuityExpiry,
    pub lifecycle: TempoContinuityLifecycle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoStateRecommendation {
    pub action: TempoStateAction,
    pub reason: TempoStateReason,
    pub confidence: Confidence,
    pub continuity: TempoContinuityPlan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempoTrendDirection {
    Stable,
    Accelerating,
    Decelerating,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TempoTrendDiagnostics {
    pub direction: TempoTrendDirection,
    pub start_bpm: f32,
    pub end_bpm: f32,
    pub total_drift_bpm: f32,
    pub slope_bpm_per_beat: f32,
    pub fit_mean_abs_deviation_bpm: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGridResidualPoint {
    pub beat_index: usize,
    pub seconds: f32,
    pub fitted_residual_ms: f32,
    pub anchored_drift_ms: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeatGridErrorDiagnostics {
    pub residuals: Vec<BeatGridResidualPoint>,
    pub mean_abs_residual_ms: f32,
    pub max_abs_residual_ms: f32,
    pub edge_mean_abs_residual_ms: f32,
    pub core_mean_abs_residual_ms: f32,
    pub end_anchored_drift_ms: f32,
    pub mean_abs_anchored_drift_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterSupportProfile {
    pub whole_track_strength: Confidence,
    pub segment_recovery_strength: Confidence,
    pub recovery_duration_strength: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterConfidenceBreakdown {
    pub phase_margin: f32,
    pub support: f32,
    pub meter_support: f32,
    pub regularity: f32,
    pub recent_stability: f32,
    pub salience: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterDetectionKind {
    WholeTrack,
    SegmentRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterTrustLevel {
    Stable,
    Recovering,
    Tentative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterRecommendation {
    Lock,
    Monitor,
    Defer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateAction {
    Lock,
    Hold,
    Watch,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterStateReason {
    StableMeter,
    RecoveringMeter,
    TentativeMeter,
    DestabilizedHold,
    RecoveryEmerging,
    MeterCleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityAction {
    Lock,
    Retain,
    Reacquire,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySource {
    CurrentMeter,
    PriorMeter,
    RecoveryWindow,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuitySeverity {
    Confirmed,
    Guarded,
    Fragile,
    Cleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityReason {
    StableEvidence,
    TentativeEvidence,
    PriorStateCarry,
    RecoveryWindowSupport,
    PhaseDisplacement,
    RevalidationDecay,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeterContinuityTrigger {
    StableRevalidation,
    TentativeCarry,
    PhaseRecovery,
    PriorStateDrift,
    RecoveryWindowDrift,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityUnresolvedSpan {
    pub beats: usize,
    pub bars: usize,
    pub failed_revalidations: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityCause {
    StableMeterEvidence,
    TempoAmbiguity,
    PhaseDisplacement,
    SparseMeterSupport,
    IrregularBarStructure,
    PriorContinuityCarry,
    RecoveryWindowInstability,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterContinuityCauseStack {
    pub primary: MeterContinuityCause,
    pub secondary: [Option<MeterContinuityCause>; 2],
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityHistory {
    Reinforcing,
    Preserving,
    Degrading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArc {
    Recovering,
    Stalling,
    Collapsing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterContinuityArcRationale {
    RefreshStrength,
    StableCarry,
    UnresolvedDrift,
    StructuralInstability,
    EvidenceLoss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityArcSupport {
    pub refresh_strength: Confidence,
    pub drift_pressure: Confidence,
    pub structural_pressure: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityTransition {
    pub after_beats: usize,
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub severity: MeterContinuitySeverity,
    pub history: MeterContinuityHistory,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityLifecycle {
    pub refresh: MeterContinuityTransition,
    pub decay: [MeterContinuityTransition; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityPlan {
    pub action: MeterContinuityAction,
    pub source: MeterContinuitySource,
    pub severity: MeterContinuitySeverity,
    pub history: MeterContinuityHistory,
    pub arc: MeterContinuityArc,
    pub arc_rationale: MeterContinuityArcRationale,
    pub arc_support: MeterContinuityArcSupport,
    pub reason: MeterContinuityReason,
    pub confidence: Confidence,
    pub trigger: MeterContinuityTrigger,
    pub unresolved: MeterContinuityUnresolvedSpan,
    pub causes: MeterContinuityCauseStack,
    pub trusted_beats: usize,
    pub revalidate_after_beats: usize,
    pub lifecycle: MeterContinuityLifecycle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterContinuityRecommendation {
    pub bar_length: MeterContinuityPlan,
    pub downbeat_phase: MeterContinuityPlan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeterStateRecommendation {
    pub action: MeterStateAction,
    pub reason: MeterStateReason,
    pub confidence: Confidence,
    pub continuity: MeterContinuityRecommendation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeterRecoveryContext {
    pub start_beat_index: usize,
    pub end_beat_index: usize,
    pub recovered_beats: usize,
    pub recovered_bars: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub supporting_windows: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeterEstimate {
    pub beats_per_bar: usize,
    pub confidence: Confidence,
    pub detection_kind: MeterDetectionKind,
    pub trust: MeterTrustLevel,
    pub recommendation: MeterRecommendation,
    pub support_profile: MeterSupportProfile,
    pub confidence_breakdown: MeterConfidenceBreakdown,
    pub recovery: Option<MeterRecoveryContext>,
    pub downbeat_positions_seconds: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeatAnalysisResult {
    pub bpm: f32,
    pub confidence: Confidence,
    pub beat_positions_seconds: Vec<f32>,
    pub onset_envelope: Vec<f32>,
    pub tempo_candidates: Vec<TempoCandidate>,
    pub tempo_diagnostics: TempoDiagnostics,
    pub tempo_interpretation: TempoInterpretation,
    pub tempo_state: TempoStateRecommendation,
    pub tempo_ambiguity: Confidence,
    pub meter_state: MeterStateRecommendation,
    pub meter: Option<MeterEstimate>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TempoHypothesis {
    bpm: f32,
    lag_frames: usize,
    refined_lag_frames: f32,
    phase_offset_frames: usize,
    phase_score: f32,
    score: f32,
    confidence: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TempoEstimate {
    bpm: f32,
    confidence: Confidence,
    lag_frames: usize,
    phase_offset_frames: usize,
    candidates: [Option<TempoHypothesis>; 3],
    ambiguity: Confidence,
}

#[derive(Clone, Copy, Debug)]
struct MeterSuppressionProfile {
    best_confidence: Confidence,
    best_support: f32,
    best_regularity: f32,
    trailing_confidence: Confidence,
    trailing_recent_stability: f32,
}

struct MeterDecision {
    estimate: Option<MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
}

#[derive(Debug, Default)]
pub struct BeatTracker {
    config: BeatTrackerConfig,
}

impl BeatTracker {
    pub fn new(config: BeatTrackerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> BeatTrackerConfig {
        self.config
    }

    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> BeatAnalysisResult {
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let onset_envelope = multifeature_onset_envelope(
            &spectrogram,
            mono_samples,
            sample_rate,
            self.config.stft.hop_size.0.max(1),
        );
        let tempo = estimate_tempo(
            &onset_envelope,
            sample_rate,
            self.config.stft.hop_size.0.max(1),
            self.config.min_bpm,
            self.config.max_bpm,
        );

        let beat_frames = track_beats(
            &onset_envelope,
            tempo.lag_frames,
            tempo.phase_offset_frames,
            self.config.beat_tolerance,
        );
        let refined_beat_frames = refine_beat_frames(&onset_envelope, &beat_frames);
        let refined_bpm = refine_bpm_from_beats(
            tempo.bpm,
            &refined_beat_frames,
            sample_rate,
            self.config.stft.hop_size.0.max(1),
        );
        let beat_positions_seconds = beat_frames_to_seconds_refined(
            &refined_beat_frames,
            sample_rate,
            self.config.stft.hop_size.0.max(1),
        );
        let low_band_cue = low_band_flux(&spectrogram, 180.0);
        let profile_change_cue = band_profile_change(&spectrogram, 5);
        let meter_cue = combine_meter_cues(&low_band_cue, &profile_change_cue);
        let meter_decision = infer_meter(
            &onset_envelope,
            &meter_cue,
            &beat_frames,
            sample_rate,
            self.config.stft.hop_size.0.max(1),
        );
        let confidence = combined_confidence(
            &onset_envelope,
            tempo.confidence,
            &beat_positions_seconds,
            refined_bpm,
        );
        let tempo_diagnostics = analyze_local_tempo(&beat_positions_seconds);
        let tempo_interpretation =
            interpret_tempo(refined_bpm, confidence, tempo.ambiguity, &tempo_diagnostics);
        let tempo_state =
            tempo_state_recommendation(tempo_interpretation, confidence, tempo.ambiguity);
        let meter_state = meter_state_recommendation(
            meter_decision.estimate.as_ref(),
            meter_decision.suppression_profile,
            confidence,
            tempo.ambiguity,
            refined_bpm,
            &beat_positions_seconds,
        );
        let mut tempo_candidates: Vec<TempoCandidate> = tempo
            .candidates
            .into_iter()
            .flatten()
            .map(|candidate| TempoCandidate {
                bpm: candidate.bpm,
                confidence: candidate.confidence,
            })
            .collect();
        if let Some(primary_candidate) = tempo_candidates.first_mut() {
            primary_candidate.bpm = refined_bpm;
        }

        BeatAnalysisResult {
            bpm: refined_bpm,
            confidence,
            beat_positions_seconds,
            onset_envelope,
            tempo_candidates,
            tempo_diagnostics,
            tempo_interpretation,
            tempo_state,
            tempo_ambiguity: tempo.ambiguity,
            meter_state,
            meter: meter_decision.estimate,
        }
    }
}

impl AnalysisStage<BeatAnalysisResult> for BeatTracker {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> BeatAnalysisResult {
        self.analyze_mono(audio.sample_rate(), &audio.to_mono())
    }
}

fn spectral_flux(spectrogram: &Spectrogram) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(spectrogram.frames.len());
    let mut previous: Option<&[f32]> = None;

    for frame in &spectrogram.frames {
        let current = frame.magnitudes.as_slice();
        let flux = if let Some(last) = previous {
            current
                .iter()
                .zip(last.iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum()
        } else {
            0.0
        };
        envelope.push(flux);
        previous = Some(current);
    }

    normalize(&mut envelope);
    envelope
}

fn high_frequency_content(spectrogram: &Spectrogram) -> Vec<f32> {
    let mut envelope = Vec::with_capacity(spectrogram.frames.len());
    for frame in &spectrogram.frames {
        let hfc = frame
            .magnitudes
            .iter()
            .enumerate()
            .map(|(index, magnitude)| index as f32 * magnitude)
            .sum();
        envelope.push(hfc);
    }
    normalize(&mut envelope);
    envelope
}

fn bandwise_spectral_flux(spectrogram: &Spectrogram, bands: usize) -> Vec<f32> {
    if spectrogram.frames.is_empty() || bands == 0 {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let band_width = ((bin_count - 1) + bands - 1) / bands;
    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for frame_index in 1..spectrogram.frames.len() {
        let current = spectrogram.frames[frame_index].magnitudes.as_slice();
        let previous = spectrogram.frames[frame_index - 1].magnitudes.as_slice();

        let mut score = 0.0;
        let mut active_bands = 0usize;
        let mut band_start = 1usize;

        while band_start < bin_count {
            let band_end = (band_start + band_width).min(bin_count);
            let band_flux: f32 = current[band_start..band_end]
                .iter()
                .zip(previous[band_start..band_end].iter())
                .map(|(now, then)| (now - then).max(0.0))
                .sum();
            if band_flux > 0.0 {
                score += band_flux / (band_end - band_start) as f32;
                active_bands += 1;
            }
            band_start = band_end;
        }

        envelope[frame_index] = if active_bands > 0 {
            score / active_bands as f32
        } else {
            0.0
        };
    }

    normalize(&mut envelope);
    envelope
}

fn complex_domain_difference(spectrogram: &Spectrogram) -> Vec<f32> {
    if spectrogram.frames.is_empty() {
        return Vec::new();
    }

    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for frame_index in 1..spectrogram.frames.len() {
        let current = &spectrogram.frames[frame_index];
        let previous = &spectrogram.frames[frame_index - 1];
        let older = frame_index
            .checked_sub(2)
            .map(|index| &spectrogram.frames[index]);

        let bin_count = current
            .magnitudes
            .len()
            .min(previous.magnitudes.len())
            .min(current.phases.len())
            .min(previous.phases.len());

        let mut score = 0.0;
        for bin_index in 1..bin_count {
            let current_magnitude = current.magnitudes[bin_index];
            let previous_magnitude = previous.magnitudes[bin_index];
            let predicted_phase = older
                .and_then(|frame| frame.phases.get(bin_index).copied())
                .map(|older_phase| 2.0 * previous.phases[bin_index] - older_phase)
                .unwrap_or(previous.phases[bin_index]);
            let phase_delta = current.phases[bin_index] - predicted_phase;
            let distance = (current_magnitude * current_magnitude
                + previous_magnitude * previous_magnitude
                - 2.0 * current_magnitude * previous_magnitude * phase_delta.cos())
            .max(0.0)
            .sqrt();
            score += distance;
        }

        envelope[frame_index] = score;
    }

    normalize(&mut envelope);
    envelope
}

fn energy_flux(samples: &[f32], sample_rate: SampleRate, hop_size: usize) -> Vec<f32> {
    if samples.is_empty() || sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    let window_size = hop_size * 2;
    let mut energies = Vec::new();
    let mut start = 0usize;

    while start < samples.len() {
        let end = (start + window_size).min(samples.len());
        let window = &samples[start..end];
        if window.is_empty() {
            break;
        }
        let rms =
            (window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32).sqrt();
        energies.push(rms);
        if end == samples.len() {
            break;
        }
        start = start.saturating_add(hop_size);
    }

    let mut flux = Vec::with_capacity(energies.len());
    let mut previous: Option<f32> = None;
    for energy in energies {
        let delta = previous.map(|last| (energy - last).max(0.0)).unwrap_or(0.0);
        flux.push(delta);
        previous = Some(energy);
    }

    normalize(&mut flux);
    flux
}

fn low_band_flux(spectrogram: &Spectrogram, max_frequency_hz: f32) -> Vec<f32> {
    if spectrogram.frames.is_empty()
        || spectrogram.sample_rate.0 == 0
        || spectrogram.config.window_size.0 == 0
    {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let max_bin = (((max_frequency_hz.max(0.0) * spectrogram.config.window_size.0 as f32)
        / spectrogram.sample_rate.0 as f32)
        .ceil() as usize)
        .clamp(1, bin_count - 1);
    let mut envelope = vec![0.0; spectrogram.frames.len()];

    for frame_index in 1..spectrogram.frames.len() {
        let current = &spectrogram.frames[frame_index].magnitudes[..=max_bin];
        let previous = &spectrogram.frames[frame_index - 1].magnitudes[..=max_bin];
        envelope[frame_index] = current
            .iter()
            .zip(previous.iter())
            .map(|(now, then)| (now - then).max(0.0))
            .sum();
    }

    normalize(&mut envelope);
    envelope
}

fn band_profile_change(spectrogram: &Spectrogram, bands: usize) -> Vec<f32> {
    if spectrogram.frames.is_empty()
        || spectrogram.sample_rate.0 == 0
        || spectrogram.config.window_size.0 == 0
        || bands == 0
    {
        return Vec::new();
    }

    let bin_count = spectrogram.bins();
    if bin_count <= 1 {
        return vec![0.0; spectrogram.frames.len()];
    }

    let band_width = ((bin_count - 1) + bands - 1) / bands;
    let mut profiles = Vec::with_capacity(spectrogram.frames.len());

    for frame in &spectrogram.frames {
        let mut profile = vec![0.0; bands];
        let mut band_start = 1usize;
        let mut band_index = 0usize;
        while band_start < bin_count && band_index < bands {
            let band_end = (band_start + band_width).min(bin_count);
            profile[band_index] = frame.magnitudes[band_start..band_end].iter().copied().sum();
            band_start = band_end;
            band_index += 1;
        }

        let total = profile.iter().copied().sum::<f32>();
        if total > 0.0 {
            for value in &mut profile {
                *value /= total;
            }
        }
        profiles.push(profile);
    }

    let mut envelope = vec![0.0; spectrogram.frames.len()];
    for frame_index in 1..profiles.len() {
        envelope[frame_index] = profiles[frame_index]
            .iter()
            .zip(profiles[frame_index - 1].iter())
            .map(|(now, then)| (now - then).abs())
            .sum();
    }

    normalize(&mut envelope);
    envelope
}

fn multifeature_onset_envelope(
    spectrogram: &Spectrogram,
    mono_samples: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
) -> Vec<f32> {
    let flux = spectral_flux(spectrogram);
    let band_flux = bandwise_spectral_flux(spectrogram, 6);
    let complex = complex_domain_difference(spectrogram);
    let hfc = high_frequency_content(spectrogram);
    let energy = energy_flux(mono_samples, sample_rate, hop_size);

    let len = flux
        .len()
        .max(band_flux.len())
        .max(complex.len())
        .max(hfc.len())
        .max(energy.len());

    let mut combined = vec![0.0; len];
    for index in 0..len {
        let flux_value = flux.get(index).copied().unwrap_or(0.0);
        let band_flux_value = band_flux.get(index).copied().unwrap_or(0.0);
        let complex_value = complex.get(index).copied().unwrap_or(0.0);
        let hfc_value = hfc.get(index).copied().unwrap_or(0.0);
        let energy_value = energy.get(index).copied().unwrap_or(0.0);
        combined[index] = 0.28 * flux_value
            + 0.22 * band_flux_value
            + 0.30 * complex_value
            + 0.12 * hfc_value
            + 0.08 * energy_value;
    }

    sharpen_onset_envelope(&mut combined);
    normalize(&mut combined);
    combined
}

fn sharpen_onset_envelope(values: &mut [f32]) {
    if values.is_empty() {
        return;
    }

    let source = values.to_vec();
    let radius = 8usize.min(source.len().saturating_sub(1)).max(1);
    let mut prefix = vec![0.0; source.len() + 1];

    for (index, value) in source.iter().copied().enumerate() {
        prefix[index + 1] = prefix[index] + value;
    }

    for index in 0..source.len() {
        let start = index.saturating_sub(radius);
        let end = (index + radius + 1).min(source.len());
        let local_mean = (prefix[end] - prefix[start]) / (end - start) as f32;
        let previous = index.checked_sub(1).map(|i| source[i]).unwrap_or(0.0);
        let rising_edge = (source[index] - previous).max(0.0);
        values[index] = (source[index] - 0.65 * local_mean).max(0.0) + 0.2 * rising_edge;
    }
}

fn estimate_tempo(
    onset_envelope: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
    min_bpm: f32,
    max_bpm: f32,
) -> TempoEstimate {
    if onset_envelope.len() < 2 || sample_rate.0 == 0 || hop_size == 0 {
        return TempoEstimate {
            bpm: 0.0,
            confidence: Confidence::new(0.0),
            lag_frames: 0,
            phase_offset_frames: 0,
            candidates: [None, None, None],
            ambiguity: Confidence::new(0.0),
        };
    }

    let onset_rate = sample_rate.0 as f32 / hop_size as f32;
    let min_lag = ((60.0 * onset_rate) / max_bpm).round().max(1.0) as usize;
    let max_lag = ((60.0 * onset_rate) / min_bpm).round().max(min_lag as f32) as usize;

    let max_lag = max_lag.min(onset_envelope.len().saturating_sub(1));
    let mut lag_scores = vec![0.0; max_lag + 1];

    for lag in min_lag..=max_lag {
        lag_scores[lag] = tempo_score(onset_envelope, lag);
    }

    let candidates = tempo_candidates(&lag_scores, min_lag, max_lag);
    let mut hypotheses = Vec::new();

    for lag in candidates.into_iter().take(6) {
        let raw_score = lag_scores[lag];
        if raw_score <= 0.0 {
            continue;
        }

        let refined_lag = refine_tempo_lag(&lag_scores, lag, min_lag, max_lag);
        let phase_offset = select_beat_phase(onset_envelope, lag);
        let phase_score = beat_phase_score(onset_envelope, lag, phase_offset);
        let hypothesis_score = raw_score * (0.7 + 0.3 * phase_score.clamp(0.0, 1.0));
        hypotheses.push(TempoHypothesis {
            bpm: 60.0 * onset_rate / refined_lag.max(1.0),
            lag_frames: lag,
            refined_lag_frames: refined_lag,
            phase_offset_frames: phase_offset,
            phase_score,
            score: hypothesis_score,
            confidence: Confidence::new(0.0),
        });
    }

    hypotheses.sort_by(|lhs, rhs| {
        rhs.score
            .partial_cmp(&lhs.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    if hypotheses.is_empty() {
        return TempoEstimate {
            bpm: 0.0,
            confidence: Confidence::new(0.0),
            lag_frames: 0,
            phase_offset_frames: 0,
            candidates: [None, None, None],
            ambiguity: Confidence::new(0.0),
        };
    }

    let best_score = hypotheses[0].score;
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);

    for hypothesis in &mut hypotheses {
        let score_ratio = if best_score > 0.0 {
            hypothesis.score / best_score
        } else {
            0.0
        };
        hypothesis.confidence = Confidence::new(0.7 * score_ratio + 0.3 * hypothesis.phase_score);
    }

    let best_candidate = hypotheses[0];
    let ambiguity = if best_score > 0.0 {
        let runner_ratio = runner_up / best_score;
        let relation_bonus = hypotheses
            .get(1)
            .map(|candidate| {
                let ratio = best_candidate.bpm / candidate.bpm.max(1.0);
                if (ratio - 2.0).abs() < 0.18
                    || (ratio - 0.5).abs() < 0.09
                    || (ratio - 1.5).abs() < 0.12
                {
                    0.2
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        Confidence::new((runner_ratio + relation_bonus).min(1.0))
    } else {
        Confidence::new(0.0)
    };

    TempoEstimate {
        bpm: best_candidate.bpm,
        confidence: if best_score > 0.0 {
            let margin = (best_score - runner_up).max(0.0) / best_score;
            Confidence::new(0.65 * margin + 0.35 * best_candidate.phase_score)
        } else {
            Confidence::new(0.0)
        },
        lag_frames: best_candidate.lag_frames,
        phase_offset_frames: best_candidate.phase_offset_frames,
        candidates: [
            hypotheses.first().copied(),
            hypotheses.get(1).copied(),
            hypotheses.get(2).copied(),
        ],
        ambiguity,
    }
}

fn tempo_candidates(lag_scores: &[f32], min_lag: usize, max_lag: usize) -> Vec<usize> {
    let mut candidates = Vec::new();

    for lag in min_lag..=max_lag {
        let score = lag_scores[lag];
        if score <= 0.0 {
            continue;
        }

        let previous = if lag > min_lag {
            lag_scores[lag - 1]
        } else {
            0.0
        };
        let next = if lag < max_lag {
            lag_scores[lag + 1]
        } else {
            0.0
        };
        if score >= previous && score >= next {
            candidates.push(lag);
        }
    }

    candidates.sort_by(|lhs, rhs| {
        lag_scores[*rhs]
            .partial_cmp(&lag_scores[*lhs])
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    let mut filtered = Vec::new();
    for lag in candidates {
        if filtered
            .iter()
            .all(|existing: &usize| existing.abs_diff(lag) > 2)
        {
            filtered.push(lag);
        }
    }

    filtered
}

fn refine_tempo_lag(lag_scores: &[f32], lag: usize, min_lag: usize, max_lag: usize) -> f32 {
    if lag <= min_lag || lag >= max_lag {
        return lag as f32;
    }

    let left = lag_scores[lag - 1];
    let center = lag_scores[lag];
    let right = lag_scores[lag + 1];
    let denominator = left - 2.0 * center + right;

    if denominator.abs() <= f32::EPSILON {
        return lag as f32;
    }

    let delta = (0.5 * (left - right) / denominator).clamp(-0.5, 0.5);
    lag as f32 + delta
}

fn tempo_score(onset_envelope: &[f32], lag: usize) -> f32 {
    if lag == 0 || lag >= onset_envelope.len() {
        return 0.0;
    }

    let base = autocorrelation(onset_envelope, lag);
    let second = autocorrelation(onset_envelope, lag * 2) * 0.5;
    let third = autocorrelation(onset_envelope, lag * 3) * 0.25;
    base + second + third
}

fn autocorrelation(onset_envelope: &[f32], lag: usize) -> f32 {
    if lag == 0 || lag >= onset_envelope.len() {
        return 0.0;
    }

    let mut score = 0.0;
    for index in lag..onset_envelope.len() {
        score += onset_envelope[index] * onset_envelope[index - lag];
    }
    score
}

fn combined_confidence(
    onset_envelope: &[f32],
    tempo_confidence: Confidence,
    beat_positions_seconds: &[f32],
    bpm: f32,
) -> Confidence {
    if onset_envelope.is_empty() || bpm <= 0.0 {
        return Confidence::new(0.0);
    }

    let peak = onset_envelope
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value));
    let mean = onset_envelope.iter().copied().sum::<f32>() / onset_envelope.len() as f32;
    let onset_strength = (peak - mean).max(0.0);
    let beat_density = (beat_positions_seconds.len() as f32 / 16.0).clamp(0.0, 1.0);
    Confidence::new(0.5 * onset_strength + 0.35 * tempo_confidence.0 + 0.15 * beat_density)
}

fn track_beats(
    onset_envelope: &[f32],
    lag_frames: usize,
    phase_offset_frames: usize,
    beat_tolerance: f32,
) -> Vec<usize> {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return Vec::new();
    }

    let tolerance_frames = (lag_frames as f32 * beat_tolerance).round().max(1.0) as isize;
    let phase_offset_frames = phase_offset_frames.min(onset_envelope.len().saturating_sub(1));

    let mut beats = vec![refine_beat(
        onset_envelope,
        phase_offset_frames as isize,
        tolerance_frames,
    )];

    let mut next = phase_offset_frames as isize + lag_frames as isize;
    while next < onset_envelope.len() as isize {
        beats.push(refine_beat(onset_envelope, next, tolerance_frames));
        next += lag_frames as isize;
    }

    let mut previous = phase_offset_frames as isize - lag_frames as isize;
    while previous >= 0 {
        beats.push(refine_beat(onset_envelope, previous, tolerance_frames));
        previous -= lag_frames as isize;
    }

    beats.sort_unstable();
    beats.dedup();

    beats
        .into_iter()
        .filter(|frame| *frame >= 0)
        .map(|frame| frame as usize)
        .collect()
}

fn beat_frames_to_seconds(
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
) -> Vec<f32> {
    if sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    beat_frames
        .iter()
        .map(|frame| *frame as f32 * hop_size as f32 / sample_rate.0 as f32)
        .collect()
}

fn beat_frames_to_seconds_refined(
    beat_frames: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
) -> Vec<f32> {
    if sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    beat_frames
        .iter()
        .map(|frame| *frame * hop_size as f32 / sample_rate.0 as f32)
        .collect()
}

fn refine_peak_frame(onset_envelope: &[f32], frame: usize) -> f32 {
    if onset_envelope.is_empty() {
        return 0.0;
    }
    if frame == 0 || frame + 1 >= onset_envelope.len() {
        return frame as f32;
    }

    let left = onset_envelope[frame - 1];
    let center = onset_envelope[frame];
    let right = onset_envelope[frame + 1];
    let denominator = left - 2.0 * center + right;
    if denominator.abs() <= f32::EPSILON {
        return frame as f32;
    }

    let delta = (0.5 * (left - right) / denominator).clamp(-0.5, 0.5);
    frame as f32 + delta
}

fn refine_beat_frames(onset_envelope: &[f32], beat_frames: &[usize]) -> Vec<f32> {
    beat_frames
        .iter()
        .map(|frame| refine_peak_frame(onset_envelope, *frame))
        .collect()
}

fn refine_bpm_from_beats(
    coarse_bpm: f32,
    beat_frames: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
) -> f32 {
    if coarse_bpm <= 0.0 || sample_rate.0 == 0 || hop_size == 0 || beat_frames.len() < 2 {
        return coarse_bpm.max(0.0);
    }

    let mut intervals: Vec<f32> = beat_frames
        .windows(2)
        .filter_map(|pair| {
            let interval = pair[1] - pair[0];
            (interval > 0.0).then_some(interval)
        })
        .collect();
    if intervals.is_empty() {
        return coarse_bpm;
    }

    intervals.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let median = intervals[intervals.len() / 2];
    let min_interval = median * 0.85;
    let max_interval = median * 1.15;
    let filtered: Vec<f32> = intervals
        .into_iter()
        .filter(|interval| *interval >= min_interval && *interval <= max_interval)
        .collect();
    let intervals = if filtered.is_empty() {
        vec![median]
    } else {
        filtered
    };
    let average_interval = intervals.iter().copied().sum::<f32>() / intervals.len() as f32;
    if average_interval <= 0.0 {
        return coarse_bpm;
    }

    let onset_rate = sample_rate.0 as f32 / hop_size as f32;
    let beat_grid_bpm = 60.0 * onset_rate / average_interval;
    let mean_abs_deviation = intervals
        .iter()
        .map(|interval| (interval - median).abs())
        .sum::<f32>()
        / intervals.len() as f32;
    let consistency = (1.0 - (mean_abs_deviation / median.max(1.0)) / 0.02).clamp(0.0, 1.0);
    let mismatch = (beat_grid_bpm - coarse_bpm).abs();
    let agreement = (1.0 - mismatch / 0.6).clamp(0.0, 1.0);
    let correction_strength = (consistency * agreement).clamp(0.0, 1.0);
    coarse_bpm + (beat_grid_bpm - coarse_bpm) * correction_strength
}

fn tempo_points(beat_positions_seconds: &[f32], beat_span: usize) -> Vec<LocalTempoPoint> {
    if beat_span == 0 || beat_positions_seconds.len() <= beat_span {
        return Vec::new();
    }

    let mut points = Vec::with_capacity(beat_positions_seconds.len() - beat_span);
    for start_beat in 0..(beat_positions_seconds.len() - beat_span) {
        let end_beat = start_beat + beat_span;
        let start_seconds = beat_positions_seconds[start_beat];
        let end_seconds = beat_positions_seconds[end_beat];
        let duration = end_seconds - start_seconds;
        if duration <= 0.0 {
            continue;
        }

        points.push(LocalTempoPoint {
            start_beat_index: start_beat,
            end_beat_index: end_beat,
            start_seconds,
            end_seconds,
            bpm: 60.0 * beat_span as f32 / duration,
        });
    }

    points
}

fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        0.5 * (values[mid - 1] + values[mid])
    } else {
        values[mid]
    }
}

fn tempo_summary(points: &[LocalTempoPoint]) -> (f32, f32, f32) {
    let mut bpms: Vec<f32> = points.iter().map(|point| point.bpm).collect();
    let median_bpm = median(&mut bpms);
    let (min_bpm, max_bpm) = bpms.iter().copied().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(min_bpm, max_bpm), bpm| (min_bpm.min(bpm), max_bpm.max(bpm)),
    );
    let drift_span_bpm = if bpms.is_empty() {
        0.0
    } else {
        max_bpm - min_bpm
    };
    let mean_abs_deviation_bpm = if bpms.is_empty() {
        0.0
    } else {
        bpms.iter().map(|bpm| (bpm - median_bpm).abs()).sum::<f32>() / bpms.len() as f32
    };

    (median_bpm, drift_span_bpm, mean_abs_deviation_bpm)
}

fn linear_fit(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }

    let count = points.len() as f32;
    let mean_x = points.iter().map(|(x, _)| *x).sum::<f32>() / count;
    let mean_y = points.iter().map(|(_, y)| *y).sum::<f32>() / count;
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (x, y) in points {
        let dx = *x - mean_x;
        numerator += dx * (*y - mean_y);
        denominator += dx * dx;
    }

    if denominator <= f32::EPSILON {
        return None;
    }

    let slope = numerator / denominator;
    let intercept = mean_y - slope * mean_x;
    Some((intercept, slope))
}

fn mean_abs(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().map(|value| value.abs()).sum::<f32>() / values.len() as f32
    }
}

fn core_tempo_points(points: &[LocalTempoPoint]) -> &[LocalTempoPoint] {
    if points.len() <= 4 {
        points
    } else {
        &points[1..points.len() - 1]
    }
}

fn analyze_tempo_trend(points: &[LocalTempoPoint]) -> TempoTrendDiagnostics {
    let fit_points: Vec<(f32, f32)> = points
        .iter()
        .map(|point| (point.start_beat_index as f32, point.bpm))
        .collect();
    let Some((intercept, slope_bpm_per_beat)) = linear_fit(&fit_points) else {
        return TempoTrendDiagnostics {
            direction: TempoTrendDirection::Stable,
            start_bpm: points.first().map(|point| point.bpm).unwrap_or(0.0),
            end_bpm: points.last().map(|point| point.bpm).unwrap_or(0.0),
            total_drift_bpm: 0.0,
            slope_bpm_per_beat: 0.0,
            fit_mean_abs_deviation_bpm: 0.0,
        };
    };

    let start_x = fit_points.first().map(|(x, _)| *x).unwrap_or(0.0);
    let end_x = fit_points.last().map(|(x, _)| *x).unwrap_or(start_x);
    let start_bpm = intercept + slope_bpm_per_beat * start_x;
    let end_bpm = intercept + slope_bpm_per_beat * end_x;
    let total_drift_bpm = end_bpm - start_bpm;
    let fit_mean_abs_deviation_bpm = if fit_points.is_empty() {
        0.0
    } else {
        fit_points
            .iter()
            .map(|(x, bpm)| {
                let fitted = intercept + slope_bpm_per_beat * *x;
                (bpm - fitted).abs()
            })
            .sum::<f32>()
            / fit_points.len() as f32
    };
    let direction = if total_drift_bpm.abs() < 0.15 {
        TempoTrendDirection::Stable
    } else if total_drift_bpm > 0.0 {
        TempoTrendDirection::Accelerating
    } else {
        TempoTrendDirection::Decelerating
    };

    TempoTrendDiagnostics {
        direction,
        start_bpm,
        end_bpm,
        total_drift_bpm,
        slope_bpm_per_beat,
        fit_mean_abs_deviation_bpm,
    }
}

fn analyze_beat_grid_error(beat_positions_seconds: &[f32]) -> BeatGridErrorDiagnostics {
    if beat_positions_seconds.is_empty() {
        return BeatGridErrorDiagnostics {
            residuals: Vec::new(),
            mean_abs_residual_ms: 0.0,
            max_abs_residual_ms: 0.0,
            edge_mean_abs_residual_ms: 0.0,
            core_mean_abs_residual_ms: 0.0,
            end_anchored_drift_ms: 0.0,
            mean_abs_anchored_drift_ms: 0.0,
        };
    }

    let fit_points: Vec<(f32, f32)> = beat_positions_seconds
        .iter()
        .enumerate()
        .map(|(beat_index, seconds)| (beat_index as f32, *seconds))
        .collect();
    let median_interval_seconds = {
        let mut intervals: Vec<f32> = beat_positions_seconds
            .windows(2)
            .map(|window| window[1] - window[0])
            .collect();
        median(&mut intervals)
    };
    let (intercept, slope_seconds_per_beat) =
        linear_fit(&fit_points).unwrap_or((beat_positions_seconds[0], median_interval_seconds));
    let anchor = beat_positions_seconds[0];
    let residuals: Vec<BeatGridResidualPoint> = beat_positions_seconds
        .iter()
        .enumerate()
        .map(|(beat_index, seconds)| {
            let beat = beat_index as f32;
            let fitted = intercept + slope_seconds_per_beat * beat;
            let anchored = anchor + median_interval_seconds * beat;
            BeatGridResidualPoint {
                beat_index,
                seconds: *seconds,
                fitted_residual_ms: 1_000.0 * (*seconds - fitted),
                anchored_drift_ms: 1_000.0 * (*seconds - anchored),
            }
        })
        .collect();
    let fitted_residuals: Vec<f32> = residuals
        .iter()
        .map(|point| point.fitted_residual_ms)
        .collect();
    let anchored_drifts: Vec<f32> = residuals
        .iter()
        .map(|point| point.anchored_drift_ms)
        .collect();
    let edge_count = residuals.len().min(2);
    let edge_values: Vec<f32> = residuals
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < edge_count || *index + edge_count >= residuals.len())
        .map(|(_, point)| point.fitted_residual_ms)
        .collect();
    let core_values: Vec<f32> = if residuals.len() > edge_count * 2 {
        residuals[edge_count..residuals.len() - edge_count]
            .iter()
            .map(|point| point.fitted_residual_ms)
            .collect()
    } else {
        fitted_residuals.clone()
    };

    BeatGridErrorDiagnostics {
        residuals,
        mean_abs_residual_ms: mean_abs(&fitted_residuals),
        max_abs_residual_ms: fitted_residuals
            .iter()
            .map(|value| value.abs())
            .fold(0.0, f32::max),
        edge_mean_abs_residual_ms: mean_abs(&edge_values),
        core_mean_abs_residual_ms: mean_abs(&core_values),
        end_anchored_drift_ms: anchored_drifts.last().copied().unwrap_or(0.0),
        mean_abs_anchored_drift_ms: mean_abs(&anchored_drifts),
    }
}

fn analyze_local_tempo(beat_positions_seconds: &[f32]) -> TempoDiagnostics {
    let interval_tempi = tempo_points(beat_positions_seconds, 1);
    let windowed_tempi = tempo_points(beat_positions_seconds, 4);
    let (median_bpm, drift_span_bpm, mean_abs_deviation_bpm) = tempo_summary(&interval_tempi);
    let (windowed_median_bpm, windowed_drift_span_bpm, windowed_mean_abs_deviation_bpm) =
        tempo_summary(&windowed_tempi);
    let core_windowed_tempi = core_tempo_points(&windowed_tempi);
    let (
        core_windowed_median_bpm,
        core_windowed_drift_span_bpm,
        core_windowed_mean_abs_deviation_bpm,
    ) = tempo_summary(core_windowed_tempi);
    let boundary_bias_bpm = if windowed_tempi.len() <= core_windowed_tempi.len() {
        0.0
    } else {
        windowed_tempi
            .iter()
            .enumerate()
            .filter(|(index, _)| *index == 0 || *index + 1 == windowed_tempi.len())
            .map(|(_, point)| (point.bpm - core_windowed_median_bpm).abs())
            .fold(0.0, f32::max)
    };
    let trend_points = if core_windowed_tempi.is_empty() {
        &windowed_tempi
    } else {
        core_windowed_tempi
    };
    let trend = analyze_tempo_trend(trend_points);
    let beat_grid_error = analyze_beat_grid_error(beat_positions_seconds);

    TempoDiagnostics {
        interval_tempi,
        windowed_tempi,
        median_bpm,
        drift_span_bpm,
        mean_abs_deviation_bpm,
        windowed_median_bpm,
        windowed_drift_span_bpm,
        windowed_mean_abs_deviation_bpm,
        core_windowed_median_bpm,
        core_windowed_drift_span_bpm,
        core_windowed_mean_abs_deviation_bpm,
        boundary_bias_bpm,
        trend,
        beat_grid_error,
    }
}

fn interpret_tempo(
    refined_bpm: f32,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
    diagnostics: &TempoDiagnostics,
) -> TempoInterpretation {
    let drift_stability = Confidence::new(
        (1.0 - (0.6 * (diagnostics.trend.total_drift_bpm.abs() / 0.25)
            + 0.4 * (diagnostics.trend.fit_mean_abs_deviation_bpm / 0.18)))
            .clamp(0.0, 1.0),
    );
    let grid_stability = Confidence::new(
        (1.0 - (0.45 * (diagnostics.beat_grid_error.mean_abs_residual_ms / 6.0)
            + 0.35 * (diagnostics.beat_grid_error.core_mean_abs_residual_ms / 4.0)
            + 0.20 * (diagnostics.beat_grid_error.mean_abs_anchored_drift_ms / 10.0)))
            .clamp(0.0, 1.0),
    );
    let core_consensus = Confidence::new(
        (1.0 - ((refined_bpm - diagnostics.core_windowed_median_bpm).abs() / 0.35)).clamp(0.0, 1.0),
    );
    let integer_closeness =
        Confidence::new((1.0 - ((refined_bpm - refined_bpm.round()).abs() / 0.12)).clamp(0.0, 1.0));
    let edge_core_gap_ms = (diagnostics.beat_grid_error.edge_mean_abs_residual_ms
        - diagnostics.beat_grid_error.core_mean_abs_residual_ms)
        .max(0.0);
    let boundary_pressure = Confidence::new(
        (0.55 * (diagnostics.boundary_bias_bpm / 0.8) + 0.45 * (edge_core_gap_ms / 6.0))
            .clamp(0.0, 1.0),
    );
    let support = TempoInterpretationSupport {
        core_consensus,
        drift_stability,
        grid_stability,
        integer_closeness,
        boundary_pressure,
    };
    let stability_score = (0.35 * confidence.0
        + 0.20 * core_consensus.0
        + 0.20 * drift_stability.0
        + 0.15 * grid_stability.0
        + 0.10 * (1.0 - tempo_ambiguity.0))
        .clamp(0.0, 1.0);
    let nearest_integer_bpm = refined_bpm.round();
    let snap_error_bpm = (refined_bpm - nearest_integer_bpm).abs();
    let profile = TempoInterpretationProfile {
        refined_bpm,
        core_window_bpm: diagnostics.core_windowed_median_bpm,
        nearest_integer_bpm,
        snap_error_bpm,
        stability_score: Confidence::new(stability_score),
        boundary_edge_gap_ms: edge_core_gap_ms,
    };
    let destabilized_edge_pressure = boundary_pressure.0 > 0.72
        && edge_core_gap_ms > 2.5
        && (stability_score < 0.62 || drift_stability.0 < 0.48 || grid_stability.0 < 0.48);

    if confidence.0 < 0.4
        || stability_score < 0.45
        || destabilized_edge_pressure
        || (tempo_ambiguity.0 > 0.6 && integer_closeness.0 < 0.8)
    {
        return TempoInterpretation {
            trust: TempoTrustLevel::Tentative,
            recommendation: TempoRecommendation::Defer,
            reason: TempoInterpretationReason::UnstableTempo,
            recommended_bpm: refined_bpm,
            snapped_bpm: None,
            support,
            profile,
        };
    }

    if boundary_pressure.0 > 0.55
        && core_consensus.0 > 0.8
        && drift_stability.0 > 0.55
        && diagnostics.core_windowed_mean_abs_deviation_bpm
            <= diagnostics.windowed_mean_abs_deviation_bpm + 0.02
    {
        return TempoInterpretation {
            trust: if stability_score >= 0.8 {
                TempoTrustLevel::Stable
            } else {
                TempoTrustLevel::Guarded
            },
            recommendation: TempoRecommendation::UseCoreWindow,
            reason: TempoInterpretationReason::StableCoreWindow,
            recommended_bpm: diagnostics.core_windowed_median_bpm,
            snapped_bpm: None,
            support,
            profile,
        };
    }

    if integer_closeness.0 > 0.8
        && snap_error_bpm >= 0.04
        && boundary_pressure.0 < 0.45
        && drift_stability.0 > 0.58
        && grid_stability.0 > 0.53
        && tempo_ambiguity.0 < 0.5
    {
        let snapped_bpm = nearest_integer_bpm;
        return TempoInterpretation {
            trust: if stability_score >= 0.8 {
                TempoTrustLevel::Stable
            } else {
                TempoTrustLevel::Guarded
            },
            recommendation: TempoRecommendation::SnapInteger,
            reason: TempoInterpretationReason::NearIntegerPulse,
            recommended_bpm: snapped_bpm,
            snapped_bpm: Some(snapped_bpm),
            support,
            profile,
        };
    }

    TempoInterpretation {
        trust: if stability_score >= 0.7 {
            TempoTrustLevel::Stable
        } else {
            TempoTrustLevel::Guarded
        },
        recommendation: TempoRecommendation::UseRefined,
        reason: TempoInterpretationReason::StableRefinedPulse,
        recommended_bpm: refined_bpm,
        snapped_bpm: None,
        support,
        profile,
    }
}

fn tempo_state_recommendation(
    interpretation: TempoInterpretation,
    confidence: Confidence,
    tempo_ambiguity: Confidence,
) -> TempoStateRecommendation {
    fn continuity_severity(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
    ) -> TempoContinuitySeverity {
        match action {
            TempoContinuityAction::Lock => TempoContinuitySeverity::Confirmed,
            TempoContinuityAction::Retain => match source {
                TempoContinuitySource::CurrentTempo | TempoContinuitySource::CoreWindow => {
                    TempoContinuitySeverity::Guarded
                }
                TempoContinuitySource::PriorTempo => TempoContinuitySeverity::Fragile,
                TempoContinuitySource::Cleared => TempoContinuitySeverity::Cleared,
            },
            TempoContinuityAction::Reacquire => TempoContinuitySeverity::Fragile,
            TempoContinuityAction::Clear => TempoContinuitySeverity::Cleared,
        }
    }

    fn continuity_history(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
    ) -> TempoContinuityHistory {
        match action {
            TempoContinuityAction::Clear => TempoContinuityHistory::Degrading,
            TempoContinuityAction::Lock => TempoContinuityHistory::Reinforcing,
            TempoContinuityAction::Retain => match source {
                TempoContinuitySource::PriorTempo => TempoContinuityHistory::Degrading,
                TempoContinuitySource::CurrentTempo | TempoContinuitySource::CoreWindow => {
                    TempoContinuityHistory::Preserving
                }
                TempoContinuitySource::Cleared => TempoContinuityHistory::Degrading,
            },
            TempoContinuityAction::Reacquire => match (source, reason) {
                (TempoContinuitySource::CurrentTempo, TempoContinuityReason::StableTempo) => {
                    TempoContinuityHistory::Reinforcing
                }
                (TempoContinuitySource::CurrentTempo, _) => TempoContinuityHistory::Preserving,
                (TempoContinuitySource::PriorTempo, _) => TempoContinuityHistory::Degrading,
                (TempoContinuitySource::CoreWindow, _) => TempoContinuityHistory::Preserving,
                (TempoContinuitySource::Cleared, _) => TempoContinuityHistory::Degrading,
            },
        }
    }

    fn continuity_provenance(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
    ) -> TempoContinuityProvenance {
        match reason {
            TempoContinuityReason::IntegerTempoSnap => TempoContinuityProvenance::IntegerSnap,
            TempoContinuityReason::StableTempo => match action {
                TempoContinuityAction::Lock => TempoContinuityProvenance::StableRefinedEstimate,
                TempoContinuityAction::Reacquire => {
                    TempoContinuityProvenance::GuardedRefinedEstimate
                }
                TempoContinuityAction::Retain => match source {
                    TempoContinuitySource::CurrentTempo => {
                        TempoContinuityProvenance::StableRefinedEstimate
                    }
                    TempoContinuitySource::PriorTempo => TempoContinuityProvenance::PriorTempoCarry,
                    TempoContinuitySource::CoreWindow => {
                        TempoContinuityProvenance::CoreWindowEstimate
                    }
                    TempoContinuitySource::Cleared => TempoContinuityProvenance::NoTempo,
                },
                TempoContinuityAction::Clear => TempoContinuityProvenance::NoTempo,
            },
            TempoContinuityReason::CoreWindowCarry => TempoContinuityProvenance::CoreWindowEstimate,
            TempoContinuityReason::RevalidationDecay => match source {
                TempoContinuitySource::CurrentTempo => {
                    TempoContinuityProvenance::GuardedRefinedEstimate
                }
                TempoContinuitySource::PriorTempo => TempoContinuityProvenance::PriorTempoCarry,
                TempoContinuitySource::CoreWindow => TempoContinuityProvenance::CoreWindowEstimate,
                TempoContinuitySource::Cleared => TempoContinuityProvenance::NoTempo,
            },
            TempoContinuityReason::InsufficientEvidence => TempoContinuityProvenance::NoTempo,
        }
    }

    fn continuity_refresh_strength(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        confidence: Confidence,
        history: TempoContinuityHistory,
        beat_span: usize,
    ) -> Confidence {
        if matches!(action, TempoContinuityAction::Clear)
            || matches!(source, TempoContinuitySource::Cleared)
        {
            return Confidence::new(0.0);
        }

        let action_scale = match action {
            TempoContinuityAction::Lock => 0.96,
            TempoContinuityAction::Retain => 0.76,
            TempoContinuityAction::Reacquire => 0.64,
            TempoContinuityAction::Clear => 0.0,
        };
        let source_bias = match source {
            TempoContinuitySource::CurrentTempo => 0.10,
            TempoContinuitySource::CoreWindow => 0.04,
            TempoContinuitySource::PriorTempo => -0.06,
            TempoContinuitySource::Cleared => -0.30,
        };
        let history_bias = match history {
            TempoContinuityHistory::Reinforcing => 0.16,
            TempoContinuityHistory::Preserving => 0.06,
            TempoContinuityHistory::Degrading => -0.12,
        };
        let span_bias = (beat_span as f32 / 16.0).min(1.0) * 0.10;

        Confidence::new(
            (confidence.0 * action_scale + source_bias + history_bias + span_bias).clamp(0.0, 1.0),
        )
    }

    fn continuity_expiry(
        trusted_beats: usize,
        revalidate_after_beats: usize,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> TempoContinuityExpiry {
        let max_failed_revalidations =
            if revalidate_after_beats == 0 || final_decay.after_beats == 0 {
                0
            } else {
                final_decay.after_beats.div_ceil(revalidate_after_beats)
            };
        TempoContinuityExpiry {
            guaranteed_until_beats: trusted_beats,
            downgrade_after_beats: first_decay.after_beats,
            clear_after_beats: final_decay.after_beats,
            max_failed_revalidations,
        }
    }

    fn continuity_transition(
        after_beats: usize,
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
        confidence: Confidence,
    ) -> TempoContinuityTransition {
        let severity = continuity_severity(action, source);
        let history = continuity_history(action, source, reason);
        TempoContinuityTransition {
            after_beats,
            action,
            source,
            severity,
            history,
            reason,
            provenance: continuity_provenance(action, source, reason),
            confidence,
            refresh_strength: continuity_refresh_strength(
                action,
                source,
                confidence,
                history,
                after_beats,
            ),
        }
    }

    fn continuity_plan(
        action: TempoContinuityAction,
        source: TempoContinuitySource,
        reason: TempoContinuityReason,
        confidence: Confidence,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        refresh: TempoContinuityTransition,
        first_decay: TempoContinuityTransition,
        final_decay: TempoContinuityTransition,
    ) -> TempoContinuityPlan {
        let severity = continuity_severity(action, source);
        let history = continuity_history(action, source, reason);
        let provenance = continuity_provenance(action, source, reason);
        let expiry = continuity_expiry(
            trusted_beats,
            revalidate_after_beats,
            first_decay,
            final_decay,
        );
        TempoContinuityPlan {
            action,
            source,
            severity,
            history,
            reason,
            provenance,
            confidence,
            refresh_strength: continuity_refresh_strength(
                action,
                source,
                confidence,
                history,
                trusted_beats.max(revalidate_after_beats),
            ),
            trusted_beats,
            revalidate_after_beats,
            expiry,
            lifecycle: TempoContinuityLifecycle {
                refresh,
                decay: [first_decay, final_decay],
            },
        }
    }

    let base_confidence = (0.45 * interpretation.profile.stability_score.0
        + 0.25 * confidence.0
        + 0.15 * (1.0 - tempo_ambiguity.0)
        + 0.15 * interpretation.support.grid_stability.0)
        .clamp(0.0, 1.0);

    match interpretation.recommendation {
        TempoRecommendation::SnapInteger
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.78
                && interpretation.profile.snap_error_bpm >= 0.04
                && tempo_ambiguity.0 < 0.28 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.82));
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: TempoStateReason::StableIntegerTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Lock,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::IntegerTempoSnap,
                    state_confidence,
                    16,
                    12,
                    continuity_transition(
                        12,
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::IntegerTempoSnap,
                        state_confidence,
                    ),
                    continuity_transition(
                        20,
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        Confidence::new((state_confidence.0 * 0.72).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        28,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Stable
                && interpretation.profile.stability_score.0 >= 0.72
                && interpretation.support.boundary_pressure.0 < 0.55
                && tempo_ambiguity.0 < 0.35 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.76));
            TempoStateRecommendation {
                action: TempoStateAction::Lock,
                reason: TempoStateReason::StableRefinedTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Lock,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::StableTempo,
                    state_confidence,
                    16,
                    12,
                    continuity_transition(
                        12,
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::StableTempo,
                        state_confidence,
                    ),
                    continuity_transition(
                        20,
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        Confidence::new((state_confidence.0 * 0.72).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        28,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseCoreWindow
            if interpretation.profile.stability_score.0 >= 0.55
                && interpretation.support.boundary_pressure.0 >= 0.45 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.58));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::CoreWindowFallback,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Retain,
                    TempoContinuitySource::CoreWindow,
                    TempoContinuityReason::CoreWindowCarry,
                    state_confidence,
                    8,
                    4,
                    continuity_transition(
                        4,
                        TempoContinuityAction::Retain,
                        TempoContinuitySource::CoreWindow,
                        TempoContinuityReason::CoreWindowCarry,
                        state_confidence,
                    ),
                    continuity_transition(
                        8,
                        TempoContinuityAction::Reacquire,
                        TempoContinuitySource::PriorTempo,
                        TempoContinuityReason::RevalidationDecay,
                        Confidence::new((state_confidence.0 * 0.68).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        12,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        TempoRecommendation::UseRefined
            if interpretation.trust == TempoTrustLevel::Guarded
                && interpretation.profile.stability_score.0 >= 0.58 =>
        {
            let state_confidence = Confidence::new(base_confidence.max(0.56));
            TempoStateRecommendation {
                action: TempoStateAction::Monitor,
                reason: TempoStateReason::StableRefinedTempo,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Reacquire,
                    TempoContinuitySource::CurrentTempo,
                    TempoContinuityReason::RevalidationDecay,
                    state_confidence,
                    4,
                    4,
                    continuity_transition(
                        4,
                        TempoContinuityAction::Lock,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::StableTempo,
                        Confidence::new((state_confidence.0 * 0.96).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        8,
                        TempoContinuityAction::Reacquire,
                        TempoContinuitySource::CurrentTempo,
                        TempoContinuityReason::RevalidationDecay,
                        Confidence::new((state_confidence.0 * 0.66).clamp(0.0, 1.0)),
                    ),
                    continuity_transition(
                        12,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
        _ => {
            let state_confidence = Confidence::new(
                (0.55 * (1.0 - interpretation.profile.stability_score.0)
                    + 0.45 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            );
            TempoStateRecommendation {
                action: TempoStateAction::Defer,
                reason: TempoStateReason::TempoDeferred,
                confidence: state_confidence,
                continuity: continuity_plan(
                    TempoContinuityAction::Clear,
                    TempoContinuitySource::Cleared,
                    TempoContinuityReason::InsufficientEvidence,
                    state_confidence,
                    0,
                    0,
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                    continuity_transition(
                        0,
                        TempoContinuityAction::Clear,
                        TempoContinuitySource::Cleared,
                        TempoContinuityReason::InsufficientEvidence,
                        Confidence::new(0.0),
                    ),
                ),
            }
        }
    }
}

fn combine_meter_cues(low_band_cue: &[f32], profile_change_cue: &[f32]) -> Vec<f32> {
    let len = low_band_cue.len().max(profile_change_cue.len());
    let mut combined = vec![0.0; len];

    for index in 0..len {
        let low = low_band_cue.get(index).copied().unwrap_or(0.0);
        let profile = profile_change_cue.get(index).copied().unwrap_or(0.0);
        combined[index] = 0.55 * low + 0.45 * profile;
    }

    normalize(&mut combined);
    combined
}

#[derive(Clone, Copy)]
struct MeterHypothesis {
    beats_per_bar: usize,
    phase_offset_beats: usize,
    score: f32,
    support_ratio: f32,
    meter_support_ratio: f32,
    meter_contrast_mean: f32,
    regularity: f32,
    recent_strength: f32,
}

#[derive(Clone, Copy)]
struct MeterWindowCandidate {
    start_beat: usize,
    end_beat: usize,
    hypothesis: MeterHypothesis,
    confidence: Confidence,
    confidence_breakdown: MeterConfidenceBreakdown,
    supporting_windows: usize,
}

fn meter_hypotheses(beat_strengths: &[f32], meter_strengths: &[f32]) -> Vec<MeterHypothesis> {
    let mut hypotheses = Vec::new();

    for beats_per_bar in [3usize, 4usize] {
        for phase_offset_beats in 0..beats_per_bar {
            let mut bars = 0usize;
            let total_bars =
                (beat_strengths.len().saturating_sub(phase_offset_beats)) / beats_per_bar;
            let mut supported_weight = 0.0f32;
            let mut meter_supported_weight = 0.0f32;
            let mut downbeat_sum = 0.0f32;
            let mut weakbeat_sum = 0.0f32;
            let mut contrast_sum = 0.0f32;
            let mut meter_contrast_sum = 0.0f32;
            let mut weight_sum = 0.0f32;
            let mut bar_strengths = Vec::new();

            let mut index = phase_offset_beats;
            while index + beats_per_bar <= beat_strengths.len() {
                let bar = &beat_strengths[index..index + beats_per_bar];
                let meter_bar = &meter_strengths[index..index + beats_per_bar];
                let downbeat = 0.55 * bar[0] + 0.45 * meter_bar[0];
                let weakbeat_mean = if beats_per_bar > 1 {
                    let onset_mean =
                        bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32;
                    let meter_mean =
                        meter_bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32;
                    0.6 * onset_mean + 0.4 * meter_mean
                } else {
                    0.0
                };
                let contrast = (downbeat - weakbeat_mean).max(0.0);
                let meter_contrast = (meter_bar[0]
                    - if beats_per_bar > 1 {
                        meter_bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32
                    } else {
                        0.0
                    })
                .max(0.0);
                let progress = if total_bars > 1 {
                    bars as f32 / (total_bars - 1) as f32
                } else {
                    1.0
                };
                let weight = 0.65 + 0.35 * progress;

                bars += 1;
                if contrast > 0.06 || meter_contrast > 0.08 {
                    supported_weight += weight;
                }
                if meter_contrast > 0.08 {
                    meter_supported_weight += weight;
                }
                downbeat_sum += downbeat * weight;
                weakbeat_sum += weakbeat_mean * weight;
                contrast_sum += contrast * weight;
                meter_contrast_sum += meter_contrast * weight;
                weight_sum += weight;
                bar_strengths.push(0.7 * contrast + 0.3 * meter_contrast);
                index += beats_per_bar;
            }

            if bars < 2 || weight_sum <= 0.0 {
                continue;
            }

            let downbeat_mean = downbeat_sum / weight_sum;
            let weakbeat_mean = weakbeat_sum / weight_sum;
            let contrast_mean = contrast_sum / weight_sum;
            let meter_contrast_mean = meter_contrast_sum / weight_sum;
            let support_ratio = supported_weight / weight_sum;
            let meter_support_ratio = meter_supported_weight / weight_sum;
            let coverage = (bars as f32 / 4.0).clamp(0.0, 1.0);
            let mean_bar_strength =
                bar_strengths.iter().copied().sum::<f32>() / bar_strengths.len() as f32;
            let regularity = if mean_bar_strength > 0.0 {
                let deviation = bar_strengths
                    .iter()
                    .copied()
                    .map(|strength| (strength - mean_bar_strength).abs())
                    .sum::<f32>()
                    / bar_strengths.len() as f32;
                (1.0 - deviation / mean_bar_strength).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let recent_strength = if bar_strengths.is_empty() {
                0.0
            } else {
                let tail = bar_strengths.len().min(2);
                bar_strengths[bar_strengths.len() - tail..]
                    .iter()
                    .copied()
                    .sum::<f32>()
                    / tail as f32
            };
            let score = (0.38 * contrast_mean
                + 0.22 * meter_contrast_mean
                + 0.15 * (downbeat_mean - weakbeat_mean).max(0.0)
                + 0.06 * support_ratio
                + 0.04 * meter_support_ratio
                + 0.07 * regularity
                + 0.08 * recent_strength)
                * coverage;

            hypotheses.push(MeterHypothesis {
                beats_per_bar,
                phase_offset_beats,
                score,
                support_ratio,
                meter_support_ratio,
                meter_contrast_mean,
                regularity,
                recent_strength,
            });
        }
    }

    hypotheses.sort_by(|lhs, rhs| {
        rhs.score
            .partial_cmp(&lhs.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    hypotheses
}

fn meter_confidence_breakdown(
    best: MeterHypothesis,
    runner_up_score: f32,
) -> MeterConfidenceBreakdown {
    let margin = if best.score > 0.0 {
        (best.score - runner_up_score).max(0.0) / best.score
    } else {
        0.0
    };
    let salience = (best.score / 0.35).clamp(0.0, 1.0);
    MeterConfidenceBreakdown {
        phase_margin: margin,
        support: best.support_ratio,
        meter_support: best.meter_support_ratio,
        regularity: best.regularity,
        recent_stability: best.recent_strength,
        salience,
    }
}

fn meter_hypothesis_confidence(best: MeterHypothesis, runner_up_score: f32) -> Confidence {
    if best.score <= 0.0 {
        return Confidence::new(0.0);
    }

    let breakdown = meter_confidence_breakdown(best, runner_up_score);
    Confidence::new(
        (0.38 * breakdown.phase_margin
            + 0.18 * breakdown.support
            + 0.09 * breakdown.meter_support
            + 0.07 * best.meter_contrast_mean.clamp(0.0, 1.0)
            + 0.15 * breakdown.regularity
            + 0.20 * breakdown.recent_stability)
            * breakdown.salience,
    )
}

fn meter_window_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
    start_beat: usize,
    end_beat: usize,
) -> Option<MeterWindowCandidate> {
    if end_beat <= start_beat || end_beat > beat_strengths.len() || end_beat > meter_strengths.len()
    {
        return None;
    }

    let hypotheses = meter_hypotheses(
        &beat_strengths[start_beat..end_beat],
        &meter_strengths[start_beat..end_beat],
    );
    let hypothesis = hypotheses.first().copied()?;
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let confidence_breakdown = meter_confidence_breakdown(hypothesis, runner_up);

    Some(MeterWindowCandidate {
        start_beat,
        end_beat,
        hypothesis,
        confidence: meter_hypothesis_confidence(hypothesis, runner_up),
        confidence_breakdown,
        supporting_windows: 1,
    })
}

fn window_absolute_phase(window: MeterWindowCandidate) -> usize {
    (window.start_beat + window.hypothesis.phase_offset_beats) % window.hypothesis.beats_per_bar
}

fn window_phase_offset(beats_per_bar: usize, absolute_phase: usize, start_beat: usize) -> usize {
    (absolute_phase + beats_per_bar - (start_beat % beats_per_bar)) % beats_per_bar
}

fn window_is_recoverable(window: MeterWindowCandidate) -> bool {
    window.hypothesis.score >= 0.17
        && window.confidence.0 >= 0.24
        && window.hypothesis.support_ratio >= 0.72
        && window.hypothesis.regularity >= 0.56
        && window.hypothesis.recent_strength >= 0.12
}

fn select_segment_meter_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
) -> Option<MeterWindowCandidate> {
    if beat_strengths.len() < 12 || meter_strengths.len() < 12 {
        return None;
    }

    let total_beats = beat_strengths.len().min(meter_strengths.len());
    let mut windows = Vec::new();

    for beat_count in [8usize, 12, 16] {
        if beat_count > total_beats {
            continue;
        }

        for trailing_offset in [0usize, 4, 8] {
            if total_beats < beat_count + trailing_offset {
                continue;
            }

            let end_beat = total_beats - trailing_offset;
            let start_beat = end_beat.saturating_sub(beat_count);
            if let Some(window) =
                meter_window_candidate(beat_strengths, meter_strengths, start_beat, end_beat)
            {
                if window_is_recoverable(window) {
                    windows.push(window);
                }
            }
        }
    }

    if windows.is_empty() {
        return None;
    }

    let mut best_cluster = None;

    for candidate in windows.iter().copied() {
        let absolute_phase = window_absolute_phase(candidate);
        let mut supporters = Vec::new();

        for window in windows.iter().copied() {
            if window.hypothesis.beats_per_bar == candidate.hypothesis.beats_per_bar
                && window_absolute_phase(window) == absolute_phase
            {
                supporters.push(window);
            }
        }

        let cluster_start = supporters
            .iter()
            .map(|window| window.start_beat)
            .min()
            .unwrap_or(candidate.start_beat);
        let min_end = supporters
            .iter()
            .map(|window| window.end_beat)
            .min()
            .unwrap_or(candidate.end_beat);
        let max_end = supporters
            .iter()
            .map(|window| window.end_beat)
            .max()
            .unwrap_or(candidate.end_beat);
        let mean_confidence = supporters
            .iter()
            .map(|window| window.confidence.0)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_support = supporters
            .iter()
            .map(|window| window.hypothesis.support_ratio)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_regularity = supporters
            .iter()
            .map(|window| window.hypothesis.regularity)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_recent = supporters
            .iter()
            .map(|window| window.hypothesis.recent_strength)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_margin = supporters
            .iter()
            .map(|window| window.confidence_breakdown.phase_margin)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_meter_support = supporters
            .iter()
            .map(|window| window.confidence_breakdown.meter_support)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_salience = supporters
            .iter()
            .map(|window| window.confidence_breakdown.salience)
            .sum::<f32>()
            / supporters.len() as f32;

        if max_end != total_beats
            || max_end.saturating_sub(min_end) < 8
            || mean_confidence < 0.28
            || mean_support < 0.78
            || mean_regularity < 0.62
            || mean_recent < 0.14
        {
            continue;
        }

        let lead_end = cluster_start.min(total_beats);
        let lead_window = if lead_end >= 8 {
            meter_window_candidate(
                beat_strengths,
                meter_strengths,
                lead_end.saturating_sub(lead_end.min(16)),
                lead_end,
            )
        } else {
            None
        };
        let lead_improvement = if let Some(lead_window) = lead_window {
            let lead_unstable = lead_window.hypothesis.support_ratio < 0.58
                || lead_window.hypothesis.regularity < 0.48
                || lead_window.hypothesis.meter_support_ratio < 0.46
                || lead_window.hypothesis.meter_contrast_mean < 0.045;
            lead_unstable
                && (mean_confidence >= lead_window.confidence.0 + 0.05
                    || candidate.hypothesis.score >= lead_window.hypothesis.score + 0.03)
        } else {
            false
        };

        if !lead_improvement {
            continue;
        }

        let cluster_score = 0.45 * mean_confidence
            + 0.20 * mean_regularity
            + 0.15 * mean_support
            + 0.10 * mean_recent
            + 0.10 * ((max_end.saturating_sub(min_end)) as f32 / 8.0).clamp(0.0, 1.0);
        let adjusted_phase_offset = window_phase_offset(
            candidate.hypothesis.beats_per_bar,
            absolute_phase,
            cluster_start,
        );
        let adjusted_candidate = MeterWindowCandidate {
            start_beat: cluster_start,
            end_beat: total_beats,
            hypothesis: MeterHypothesis {
                phase_offset_beats: adjusted_phase_offset,
                ..candidate.hypothesis
            },
            confidence: Confidence::new(cluster_score.clamp(0.0, 1.0)),
            confidence_breakdown: MeterConfidenceBreakdown {
                phase_margin: mean_margin,
                support: mean_support,
                meter_support: mean_meter_support,
                regularity: mean_regularity,
                recent_stability: mean_recent,
                salience: mean_salience,
            },
            supporting_windows: supporters.len(),
        };

        match best_cluster {
            Some((best_score, _)) if best_score >= cluster_score => {}
            _ => best_cluster = Some((cluster_score, adjusted_candidate)),
        }
    }

    best_cluster.map(|(_, candidate)| candidate)
}

fn downbeat_frames_for_hypothesis(
    beat_frames: &[usize],
    beat_offset: usize,
    hypothesis: MeterHypothesis,
) -> Vec<usize> {
    beat_frames
        .iter()
        .skip(beat_offset + hypothesis.phase_offset_beats)
        .step_by(hypothesis.beats_per_bar)
        .copied()
        .collect()
}

fn beat_index_to_seconds(
    beat_frames: &[usize],
    beat_index: usize,
    sample_rate: SampleRate,
    hop_size: usize,
) -> f32 {
    if sample_rate.0 == 0 || hop_size == 0 {
        return 0.0;
    }

    beat_frames
        .get(beat_index)
        .copied()
        .unwrap_or_else(|| *beat_frames.last().unwrap_or(&0)) as f32
        * hop_size as f32
        / sample_rate.0 as f32
}

fn meter_recovery_context(
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
    candidate: MeterWindowCandidate,
) -> MeterRecoveryContext {
    let recovered_beats = candidate.end_beat.saturating_sub(candidate.start_beat);
    MeterRecoveryContext {
        start_beat_index: candidate.start_beat,
        end_beat_index: candidate.end_beat,
        recovered_beats,
        recovered_bars: recovered_beats / candidate.hypothesis.beats_per_bar,
        start_seconds: beat_index_to_seconds(
            beat_frames,
            candidate.start_beat,
            sample_rate,
            hop_size,
        ),
        end_seconds: beat_index_to_seconds(
            beat_frames,
            candidate.end_beat.saturating_sub(1),
            sample_rate,
            hop_size,
        ),
        supporting_windows: candidate.supporting_windows,
    }
}

fn meter_recovery_duration_strength(candidate: MeterWindowCandidate) -> Confidence {
    let recovered_beats = candidate.end_beat.saturating_sub(candidate.start_beat) as f32;
    let beat_span_strength = (recovered_beats / 16.0).clamp(0.0, 1.0);
    let window_strength = (candidate.supporting_windows as f32 / 3.0).clamp(0.0, 1.0);
    Confidence::new(0.7 * beat_span_strength + 0.3 * window_strength)
}

fn meter_support_profile(
    whole_track_strength: Option<Confidence>,
    segment_candidate: Option<MeterWindowCandidate>,
) -> MeterSupportProfile {
    MeterSupportProfile {
        whole_track_strength: whole_track_strength.unwrap_or(Confidence::new(0.0)),
        segment_recovery_strength: segment_candidate
            .map(|candidate| candidate.confidence)
            .unwrap_or(Confidence::new(0.0)),
        recovery_duration_strength: segment_candidate
            .map(meter_recovery_duration_strength)
            .unwrap_or(Confidence::new(0.0)),
    }
}

fn meter_trust_level(
    detection_kind: MeterDetectionKind,
    confidence: Confidence,
    support_profile: MeterSupportProfile,
    confidence_breakdown: MeterConfidenceBreakdown,
) -> MeterTrustLevel {
    match detection_kind {
        MeterDetectionKind::WholeTrack
            if confidence.0 >= 0.30
                && support_profile.whole_track_strength.0 >= 0.30
                && confidence_breakdown.support >= 0.80
                && confidence_breakdown.regularity >= 0.45
                && confidence_breakdown.phase_margin >= 0.25 =>
        {
            MeterTrustLevel::Stable
        }
        MeterDetectionKind::SegmentRecovery
            if confidence.0 >= 0.24
                && support_profile.segment_recovery_strength.0 >= 0.24
                && support_profile.recovery_duration_strength.0 >= 0.55
                && confidence_breakdown.recent_stability >= 0.14
                && confidence_breakdown.regularity >= 0.62 =>
        {
            MeterTrustLevel::Recovering
        }
        _ => MeterTrustLevel::Tentative,
    }
}

fn meter_recommendation(
    trust: MeterTrustLevel,
    detection_kind: MeterDetectionKind,
    confidence: Confidence,
    support_profile: MeterSupportProfile,
    confidence_breakdown: MeterConfidenceBreakdown,
) -> MeterRecommendation {
    match trust {
        MeterTrustLevel::Stable
            if detection_kind == MeterDetectionKind::WholeTrack
                && confidence.0 >= 0.38
                && support_profile.whole_track_strength.0 >= 0.38
                && confidence_breakdown.support >= 0.82
                && confidence_breakdown.phase_margin >= 0.30 =>
        {
            MeterRecommendation::Lock
        }
        MeterTrustLevel::Recovering => MeterRecommendation::Monitor,
        MeterTrustLevel::Tentative
            if confidence.0 >= 0.24
                && confidence_breakdown.support >= 0.72
                && confidence_breakdown.phase_margin >= 0.18 =>
        {
            MeterRecommendation::Monitor
        }
        _ => MeterRecommendation::Defer,
    }
}

fn trailing_meter_window_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
) -> Option<MeterWindowCandidate> {
    let total_beats = beat_strengths.len().min(meter_strengths.len());
    let mut best_candidate: Option<MeterWindowCandidate> = None;

    for beat_count in [8usize, 12, 16] {
        if beat_count > total_beats {
            continue;
        }

        for trailing_offset in [0usize, 4, 8] {
            if total_beats < beat_count + trailing_offset {
                continue;
            }

            let end_beat = total_beats - trailing_offset;
            let start_beat = end_beat.saturating_sub(beat_count);
            let Some(candidate) =
                meter_window_candidate(beat_strengths, meter_strengths, start_beat, end_beat)
            else {
                continue;
            };

            match best_candidate {
                Some(best) if best.confidence.0 >= candidate.confidence.0 => {}
                _ => best_candidate = Some(candidate),
            }
        }
    }

    best_candidate
}

fn meter_state_recommendation(
    estimate: Option<&MeterEstimate>,
    suppression_profile: MeterSuppressionProfile,
    rhythm_confidence: Confidence,
    tempo_ambiguity: Confidence,
    bpm: f32,
    beat_positions_seconds: &[f32],
) -> MeterStateRecommendation {
    fn push_cause(
        slots: &mut [Option<MeterContinuityCause>; 3],
        count: &mut usize,
        cause: MeterContinuityCause,
    ) {
        if slots.iter().flatten().any(|existing| *existing == cause) {
            return;
        }
        if *count < slots.len() {
            slots[*count] = Some(cause);
            *count += 1;
        }
    }

    fn cause_stack(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        trigger: MeterContinuityTrigger,
        suppression_profile: MeterSuppressionProfile,
        tempo_ambiguity: Confidence,
        phase_displaced: bool,
        stage_index: usize,
    ) -> MeterContinuityCauseStack {
        let mut causes = [None; 3];
        let mut count = 0usize;

        match reason {
            MeterContinuityReason::StableEvidence => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityReason::PriorStateCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityReason::RecoveryWindowSupport => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityReason::PhaseDisplacement => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityReason::InsufficientEvidence => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
            MeterContinuityReason::TentativeEvidence | MeterContinuityReason::RevalidationDecay => {
            }
        }

        match trigger {
            MeterContinuityTrigger::StableRevalidation => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::StableMeterEvidence,
                );
            }
            MeterContinuityTrigger::TentativeCarry => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::SparseMeterSupport,
                );
            }
            MeterContinuityTrigger::PhaseRecovery => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PhaseDisplacement,
                );
            }
            MeterContinuityTrigger::PriorStateDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::PriorContinuityCarry,
                );
            }
            MeterContinuityTrigger::RecoveryWindowDrift => {
                push_cause(
                    &mut causes,
                    &mut count,
                    MeterContinuityCause::RecoveryWindowInstability,
                );
            }
            MeterContinuityTrigger::EvidenceLoss => {
                push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
            }
        }

        if phase_displaced {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::PhaseDisplacement,
            );
        }

        if tempo_ambiguity.0 >= 0.28 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::TempoAmbiguity,
            );
        }

        if suppression_profile.best_support < 0.58 || suppression_profile.best_confidence.0 < 0.24 {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::SparseMeterSupport,
            );
        }

        if suppression_profile.best_regularity < 0.32
            || (stage_index > 0 && suppression_profile.trailing_recent_stability < 0.30)
        {
            push_cause(
                &mut causes,
                &mut count,
                MeterContinuityCause::IrregularBarStructure,
            );
        }

        if matches!(source, MeterContinuitySource::Cleared)
            || matches!(action, MeterContinuityAction::Clear)
        {
            push_cause(&mut causes, &mut count, MeterContinuityCause::EvidenceLoss);
        }

        let primary = causes[0].unwrap_or_else(|| match action {
            MeterContinuityAction::Lock => MeterContinuityCause::StableMeterEvidence,
            MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                MeterContinuityCause::SparseMeterSupport
            }
            MeterContinuityAction::Clear => MeterContinuityCause::EvidenceLoss,
        });

        MeterContinuityCauseStack {
            primary,
            secondary: [causes[1], causes[2]],
            count: count.max(1),
        }
    }

    fn has_cause(stack: MeterContinuityCauseStack, cause: MeterContinuityCause) -> bool {
        stack.primary == cause
            || stack
                .secondary
                .into_iter()
                .flatten()
                .any(|entry| entry == cause)
    }

    fn continuity_history(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        stage_index: usize,
    ) -> MeterContinuityHistory {
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let has_phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);

        match action {
            MeterContinuityAction::Clear => MeterContinuityHistory::Degrading,
            MeterContinuityAction::Lock
                if matches!(source, MeterContinuitySource::CurrentMeter)
                    && matches!(reason, MeterContinuityReason::StableEvidence)
                    && matches!(trigger, MeterContinuityTrigger::StableRevalidation)
                    && confidence.0 >= 0.28
                    && unresolved.failed_revalidations == 0
                    && !has_evidence_loss =>
            {
                MeterContinuityHistory::Reinforcing
            }
            MeterContinuityAction::Lock => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Retain
                if stage_index > 0
                    || has_evidence_loss
                    || (matches!(source, MeterContinuitySource::PriorMeter)
                        && unresolved.failed_revalidations >= 2)
                    || (matches!(source, MeterContinuitySource::RecoveryWindow)
                        && has_irregularity
                        && confidence.0 < 0.30) =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Retain => MeterContinuityHistory::Preserving,
            MeterContinuityAction::Reacquire
                if matches!(reason, MeterContinuityReason::PhaseDisplacement)
                    || stage_index > 0
                    || unresolved.failed_revalidations > 0
                    || has_evidence_loss
                    || has_phase_displacement =>
            {
                MeterContinuityHistory::Degrading
            }
            MeterContinuityAction::Reacquire => MeterContinuityHistory::Preserving,
        }
    }

    fn continuity_arc_support(
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityArcSupport {
        let refresh_bonus = match refresh.history {
            MeterContinuityHistory::Reinforcing => 0.28,
            MeterContinuityHistory::Preserving => 0.12,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let current_bonus = match current {
            MeterContinuityHistory::Reinforcing => 0.22,
            MeterContinuityHistory::Preserving => 0.08,
            MeterContinuityHistory::Degrading => 0.0,
        };
        let decay_penalty = match first_decay.history {
            MeterContinuityHistory::Degrading => 0.08,
            _ => 0.0,
        } + match final_decay.history {
            MeterContinuityHistory::Degrading => 0.12,
            _ => 0.0,
        };
        let refresh_strength = Confidence::new(
            (refresh.confidence.0 + refresh_bonus + current_bonus - decay_penalty).clamp(0.0, 1.0),
        );

        let drift_pressure = Confidence::new(
            ((unresolved.failed_revalidations as f32 * 0.18)
                + (unresolved.bars as f32 * 0.08)
                + match current {
                    MeterContinuityHistory::Degrading => 0.18,
                    MeterContinuityHistory::Preserving => 0.08,
                    MeterContinuityHistory::Reinforcing => 0.0,
                }
                + match first_decay.history {
                    MeterContinuityHistory::Degrading => 0.16,
                    _ => 0.0,
                }
                + match final_decay.history {
                    MeterContinuityHistory::Degrading => 0.20,
                    _ => 0.0,
                })
            .clamp(0.0, 1.0),
        );

        let evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let phase_displacement = has_cause(causes, MeterContinuityCause::PhaseDisplacement);
        let tempo_ambiguity = has_cause(causes, MeterContinuityCause::TempoAmbiguity);
        let structural_pressure = Confidence::new(
            ((if evidence_loss { 0.42f32 } else { 0.0f32 })
                + (if irregularity { 0.28f32 } else { 0.0f32 })
                + (if phase_displacement { 0.18f32 } else { 0.0f32 })
                + (if tempo_ambiguity { 0.12f32 } else { 0.0f32 }))
            .clamp(0.0, 1.0),
        );

        MeterContinuityArcSupport {
            refresh_strength,
            drift_pressure,
            structural_pressure,
        }
    }

    fn continuity_arc_assessment(
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        current: MeterContinuityHistory,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> (
        MeterContinuityArc,
        MeterContinuityArcRationale,
        MeterContinuityArcSupport,
    ) {
        let has_evidence_loss = has_cause(causes, MeterContinuityCause::EvidenceLoss);
        let has_irregularity = has_cause(causes, MeterContinuityCause::IrregularBarStructure);
        let persistent_decay = matches!(first_decay.history, MeterContinuityHistory::Degrading)
            && matches!(final_decay.history, MeterContinuityHistory::Degrading);
        let support = continuity_arc_support(
            unresolved,
            causes,
            current,
            refresh,
            first_decay,
            final_decay,
        );

        if matches!(current, MeterContinuityHistory::Degrading)
            && (persistent_decay || has_evidence_loss)
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        if matches!(refresh.history, MeterContinuityHistory::Reinforcing) && !has_evidence_loss {
            if matches!(current, MeterContinuityHistory::Reinforcing) {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }

            if matches!(current, MeterContinuityHistory::Preserving)
                && matches!(source, MeterContinuitySource::RecoveryWindow)
                && matches!(reason, MeterContinuityReason::RecoveryWindowSupport)
                && confidence.0 >= 0.80
                && unresolved.failed_revalidations <= 2
                && !has_irregularity
            {
                return (
                    MeterContinuityArc::Recovering,
                    MeterContinuityArcRationale::RefreshStrength,
                    support,
                );
            }
        }

        if has_evidence_loss
            || (persistent_decay && confidence.0 < 0.24)
            || (matches!(current, MeterContinuityHistory::Degrading)
                && !matches!(refresh.history, MeterContinuityHistory::Reinforcing))
        {
            return (
                MeterContinuityArc::Collapsing,
                if has_evidence_loss {
                    MeterContinuityArcRationale::EvidenceLoss
                } else if has_irregularity {
                    MeterContinuityArcRationale::StructuralInstability
                } else {
                    MeterContinuityArcRationale::UnresolvedDrift
                },
                support,
            );
        }

        (
            MeterContinuityArc::Stalling,
            if has_irregularity {
                MeterContinuityArcRationale::StructuralInstability
            } else if unresolved.failed_revalidations >= 2 {
                MeterContinuityArcRationale::UnresolvedDrift
            } else {
                MeterContinuityArcRationale::StableCarry
            },
            support,
        )
    }

    fn continuity_trigger(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
    ) -> MeterContinuityTrigger {
        match reason {
            MeterContinuityReason::StableEvidence => MeterContinuityTrigger::StableRevalidation,
            MeterContinuityReason::TentativeEvidence => MeterContinuityTrigger::TentativeCarry,
            MeterContinuityReason::PriorStateCarry => MeterContinuityTrigger::PriorStateDrift,
            MeterContinuityReason::RecoveryWindowSupport => {
                MeterContinuityTrigger::RecoveryWindowDrift
            }
            MeterContinuityReason::PhaseDisplacement => MeterContinuityTrigger::PhaseRecovery,
            MeterContinuityReason::RevalidationDecay => match source {
                MeterContinuitySource::PriorMeter => MeterContinuityTrigger::PriorStateDrift,
                MeterContinuitySource::RecoveryWindow => {
                    MeterContinuityTrigger::RecoveryWindowDrift
                }
                MeterContinuitySource::CurrentMeter => match action {
                    MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                        MeterContinuityTrigger::TentativeCarry
                    }
                    MeterContinuityAction::Lock => MeterContinuityTrigger::StableRevalidation,
                    MeterContinuityAction::Clear => MeterContinuityTrigger::EvidenceLoss,
                },
                MeterContinuitySource::Cleared => MeterContinuityTrigger::EvidenceLoss,
            },
            MeterContinuityReason::InsufficientEvidence => MeterContinuityTrigger::EvidenceLoss,
        }
    }

    fn unresolved_span(
        trigger: MeterContinuityTrigger,
        beat_span: usize,
        revalidate_after_beats: usize,
        beats_per_bar: usize,
        phase_displacement_beats: usize,
        stage_index: usize,
    ) -> MeterContinuityUnresolvedSpan {
        let beats = match trigger {
            MeterContinuityTrigger::StableRevalidation => 0,
            MeterContinuityTrigger::PhaseRecovery => beat_span
                .max(revalidate_after_beats)
                .max(phase_displacement_beats.max(1)),
            MeterContinuityTrigger::TentativeCarry
            | MeterContinuityTrigger::PriorStateDrift
            | MeterContinuityTrigger::RecoveryWindowDrift => {
                beat_span.max(revalidate_after_beats.max(1))
            }
            MeterContinuityTrigger::EvidenceLoss => beat_span,
        };
        let bars = if beats == 0 {
            0
        } else {
            (beats + beats_per_bar.saturating_sub(1)) / beats_per_bar.max(1)
        };
        let failed_revalidations = if beats == 0 || revalidate_after_beats == 0 {
            0
        } else {
            ((beats + revalidate_after_beats - 1) / revalidate_after_beats).max(stage_index)
        };
        MeterContinuityUnresolvedSpan {
            beats,
            bars,
            failed_revalidations,
        }
    }

    fn continuity_reason(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        phase_displaced: bool,
        is_decay: bool,
    ) -> MeterContinuityReason {
        if matches!(action, MeterContinuityAction::Clear)
            || matches!(source, MeterContinuitySource::Cleared)
        {
            return MeterContinuityReason::InsufficientEvidence;
        }

        if phase_displaced && matches!(action, MeterContinuityAction::Reacquire) {
            return MeterContinuityReason::PhaseDisplacement;
        }

        if is_decay {
            return MeterContinuityReason::RevalidationDecay;
        }

        match source {
            MeterContinuitySource::CurrentMeter => match action {
                MeterContinuityAction::Lock => MeterContinuityReason::StableEvidence,
                MeterContinuityAction::Retain | MeterContinuityAction::Reacquire => {
                    MeterContinuityReason::TentativeEvidence
                }
                MeterContinuityAction::Clear => MeterContinuityReason::InsufficientEvidence,
            },
            MeterContinuitySource::PriorMeter => MeterContinuityReason::PriorStateCarry,
            MeterContinuitySource::RecoveryWindow => MeterContinuityReason::RecoveryWindowSupport,
            MeterContinuitySource::Cleared => MeterContinuityReason::InsufficientEvidence,
        }
    }

    fn continuity_severity(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
    ) -> MeterContinuitySeverity {
        match action {
            MeterContinuityAction::Lock => MeterContinuitySeverity::Confirmed,
            MeterContinuityAction::Retain => match source {
                MeterContinuitySource::CurrentMeter | MeterContinuitySource::RecoveryWindow => {
                    MeterContinuitySeverity::Guarded
                }
                MeterContinuitySource::PriorMeter => MeterContinuitySeverity::Fragile,
                MeterContinuitySource::Cleared => MeterContinuitySeverity::Cleared,
            },
            MeterContinuityAction::Reacquire => MeterContinuitySeverity::Fragile,
            MeterContinuityAction::Clear => MeterContinuitySeverity::Cleared,
        }
    }

    fn continuity_confidence(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        state_confidence: Confidence,
        beat_span: usize,
        stage_index: usize,
    ) -> Confidence {
        let action_scale = match action {
            MeterContinuityAction::Lock => 1.0,
            MeterContinuityAction::Retain => 0.72,
            MeterContinuityAction::Reacquire => 0.45,
            MeterContinuityAction::Clear => 0.0,
        };
        let source_bias = match source {
            MeterContinuitySource::CurrentMeter => 0.12,
            MeterContinuitySource::RecoveryWindow => 0.06,
            MeterContinuitySource::PriorMeter => -0.02,
            MeterContinuitySource::Cleared => -0.30,
        };
        let span_bias = (beat_span as f32 / 24.0).clamp(0.0, 0.25);
        let decay_penalty = stage_index as f32 * 0.12;
        Confidence::new(
            (state_confidence.0 * action_scale + source_bias + span_bias - decay_penalty)
                .clamp(0.0, 1.0),
        )
    }

    fn transition(
        after_beats: usize,
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        stage_index: usize,
    ) -> MeterContinuityTransition {
        MeterContinuityTransition {
            after_beats,
            action,
            source,
            severity: continuity_severity(action, source),
            history: continuity_history(
                action,
                source,
                reason,
                confidence,
                trigger,
                unresolved,
                causes,
                stage_index,
            ),
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
        }
    }

    fn continuity_plan(
        action: MeterContinuityAction,
        source: MeterContinuitySource,
        reason: MeterContinuityReason,
        confidence: Confidence,
        trigger: MeterContinuityTrigger,
        unresolved: MeterContinuityUnresolvedSpan,
        causes: MeterContinuityCauseStack,
        trusted_beats: usize,
        revalidate_after_beats: usize,
        refresh: MeterContinuityTransition,
        first_decay: MeterContinuityTransition,
        final_decay: MeterContinuityTransition,
    ) -> MeterContinuityPlan {
        let history = continuity_history(
            action, source, reason, confidence, trigger, unresolved, causes, 0,
        );
        let (arc, arc_rationale, arc_support) = continuity_arc_assessment(
            source,
            reason,
            confidence,
            unresolved,
            causes,
            history,
            refresh,
            first_decay,
            final_decay,
        );
        MeterContinuityPlan {
            action,
            source,
            severity: continuity_severity(action, source),
            history,
            arc,
            arc_rationale,
            arc_support,
            reason,
            confidence,
            trigger,
            unresolved,
            causes,
            trusted_beats,
            revalidate_after_beats,
            lifecycle: MeterContinuityLifecycle {
                refresh,
                decay: [first_decay, final_decay],
            },
        }
    }

    fn continuity_for(
        action: MeterStateAction,
        reason: MeterStateReason,
        estimate: Option<&MeterEstimate>,
        suppression_profile: MeterSuppressionProfile,
        confidence: Confidence,
        tempo_ambiguity: Confidence,
        bpm: f32,
        beat_positions_seconds: &[f32],
    ) -> MeterContinuityRecommendation {
        let beat_duration = if bpm > 0.0 { 60.0 / bpm } else { 0.0 };
        let pickup_like_phase = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| beat_duration > 0.0 && first_downbeat >= beat_duration * 1.5)
            .unwrap_or(false);
        let phase_displacement_beats = estimate
            .and_then(|estimate| estimate.downbeat_positions_seconds.first().copied())
            .map(|first_downbeat| {
                if beat_duration > 0.0 {
                    let downbeat_guard = first_downbeat - beat_duration * 0.25;
                    beat_positions_seconds
                        .iter()
                        .copied()
                        .take_while(|&beat| beat < downbeat_guard)
                        .count()
                } else {
                    0
                }
            })
            .unwrap_or(0);
        let recovery_beats = estimate
            .and_then(|estimate| estimate.recovery.as_ref())
            .map(|recovery| recovery.recovered_beats)
            .unwrap_or(0);
        let beats_per_bar = estimate
            .map(|estimate| estimate.beats_per_bar)
            .unwrap_or(4)
            .max(1);
        let support_beats = ((confidence.0 * 12.0).round() as usize).clamp(2, 12);
        let retained_beats = if estimate.is_some() {
            recovery_beats.clamp(6, 24).max(support_beats)
        } else {
            support_beats
        };
        let stage = |after_beats: usize,
                     stage_action: MeterContinuityAction,
                     stage_source: MeterContinuitySource,
                     stage_reason: MeterContinuityReason,
                     stage_index: usize| {
            let stage_trigger = continuity_trigger(stage_action, stage_source, stage_reason);
            let stage_unresolved = unresolved_span(
                stage_trigger,
                after_beats,
                after_beats,
                beats_per_bar,
                phase_displacement_beats,
                stage_index,
            );
            let stage_causes = cause_stack(
                stage_action,
                stage_source,
                stage_reason,
                stage_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displacement_beats > 0,
                stage_index,
            );
            transition(
                after_beats,
                stage_action,
                stage_source,
                stage_reason,
                continuity_confidence(
                    stage_action,
                    stage_source,
                    confidence,
                    after_beats,
                    stage_index,
                ),
                stage_trigger,
                stage_unresolved,
                stage_causes,
                stage_index,
            )
        };
        let plan = |plan_action: MeterContinuityAction,
                    plan_source: MeterContinuitySource,
                    plan_reason: MeterContinuityReason,
                    trusted_beats: usize,
                    revalidate_after_beats: usize,
                    refresh: MeterContinuityTransition,
                    first_decay: MeterContinuityTransition,
                    final_decay: MeterContinuityTransition| {
            let plan_trigger = continuity_trigger(plan_action, plan_source, plan_reason);
            let plan_unresolved = unresolved_span(
                plan_trigger,
                trusted_beats,
                revalidate_after_beats,
                beats_per_bar,
                phase_displacement_beats,
                0,
            );
            let plan_causes = cause_stack(
                plan_action,
                plan_source,
                plan_reason,
                plan_trigger,
                suppression_profile,
                tempo_ambiguity,
                phase_displacement_beats > 0,
                0,
            );
            continuity_plan(
                plan_action,
                plan_source,
                plan_reason,
                continuity_confidence(plan_action, plan_source, confidence, trusted_beats, 0),
                plan_trigger,
                plan_unresolved,
                plan_causes,
                trusted_beats,
                revalidate_after_beats,
                refresh,
                first_decay,
                final_decay,
            )
        };

        match (action, reason) {
            (MeterStateAction::Lock, _) if pickup_like_phase => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::PhaseDisplacement,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::PhaseDisplacement,
                        1,
                    ),
                    stage(
                        8,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Lock, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Lock,
                    MeterContinuitySource::CurrentMeter,
                    MeterContinuityReason::StableEvidence,
                    16,
                    16,
                    stage(
                        16,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        24,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        32,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, MeterStateReason::TentativeMeter) => {
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        retained_beats.min(8),
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            retained_beats.min(8).saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::TentativeEvidence,
                        0,
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Lock,
                            MeterContinuitySource::CurrentMeter,
                            MeterContinuityReason::StableEvidence,
                            0,
                        ),
                        stage(
                            4,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::CurrentMeter,
                            continuity_reason(
                                MeterContinuityAction::Reacquire,
                                MeterContinuitySource::CurrentMeter,
                                false,
                                true,
                            ),
                            1,
                        ),
                        stage(
                            6,
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Hold, MeterStateReason::DestabilizedHold) => {
                let trailing_beats = if suppression_profile.trailing_confidence.0 > 0.0 {
                    (((suppression_profile.trailing_confidence.0
                        + suppression_profile.trailing_recent_stability)
                        * 8.0)
                        .round() as usize)
                        .clamp(4, 8)
                } else {
                    4
                };
                MeterContinuityRecommendation {
                    bar_length: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats,
                        4,
                        stage(
                            4,
                            MeterContinuityAction::Retain,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::PriorStateCarry,
                            0,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(4),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                    downbeat_phase: plan(
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        trailing_beats.saturating_sub(2).max(2),
                        2,
                        stage(
                            2,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::PriorMeter,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats,
                            MeterContinuityAction::Reacquire,
                            MeterContinuitySource::RecoveryWindow,
                            MeterContinuityReason::RevalidationDecay,
                            1,
                        ),
                        stage(
                            trailing_beats.saturating_add(2),
                            MeterContinuityAction::Clear,
                            MeterContinuitySource::Cleared,
                            MeterContinuityReason::InsufficientEvidence,
                            2,
                        ),
                    ),
                }
            }
            (MeterStateAction::Watch, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    retained_beats,
                    retained_beats.saturating_div(2).max(4),
                    stage(
                        retained_beats.saturating_div(2).max(4),
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        retained_beats.saturating_add(4),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.saturating_add(8),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Reacquire,
                    MeterContinuitySource::RecoveryWindow,
                    MeterContinuityReason::RecoveryWindowSupport,
                    0,
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Lock,
                        MeterContinuitySource::CurrentMeter,
                        MeterContinuityReason::StableEvidence,
                        0,
                    ),
                    stage(
                        4,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        6,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Clear, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Clear,
                    MeterContinuitySource::Cleared,
                    MeterContinuityReason::InsufficientEvidence,
                    0,
                    0,
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                    stage(
                        0,
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
            (MeterStateAction::Hold, _) => MeterContinuityRecommendation {
                bar_length: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(6),
                    4,
                    stage(
                        4,
                        MeterContinuityAction::Retain,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::PriorStateCarry,
                        0,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(2),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(6).saturating_add(4),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
                downbeat_phase: plan(
                    MeterContinuityAction::Retain,
                    MeterContinuitySource::PriorMeter,
                    MeterContinuityReason::PriorStateCarry,
                    retained_beats.min(4),
                    2,
                    stage(
                        2,
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::PriorMeter,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(1),
                        MeterContinuityAction::Reacquire,
                        MeterContinuitySource::RecoveryWindow,
                        MeterContinuityReason::RevalidationDecay,
                        1,
                    ),
                    stage(
                        retained_beats.min(4).saturating_add(2),
                        MeterContinuityAction::Clear,
                        MeterContinuitySource::Cleared,
                        MeterContinuityReason::InsufficientEvidence,
                        2,
                    ),
                ),
            },
        }
    }

    fn build_meter_state(
        action: MeterStateAction,
        reason: MeterStateReason,
        confidence: Confidence,
        estimate: Option<&MeterEstimate>,
        suppression_profile: MeterSuppressionProfile,
        tempo_ambiguity: Confidence,
        bpm: f32,
        beat_positions_seconds: &[f32],
    ) -> MeterStateRecommendation {
        MeterStateRecommendation {
            action,
            reason,
            confidence,
            continuity: continuity_for(
                action,
                reason,
                estimate,
                suppression_profile,
                confidence,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
        }
    }

    if let Some(estimate) = estimate {
        return match estimate.recommendation {
            MeterRecommendation::Lock => build_meter_state(
                MeterStateAction::Lock,
                MeterStateReason::StableMeter,
                estimate.confidence,
                Some(estimate),
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
            MeterRecommendation::Monitor if estimate.trust == MeterTrustLevel::Recovering => {
                build_meter_state(
                    MeterStateAction::Watch,
                    MeterStateReason::RecoveringMeter,
                    Confidence::new(
                        0.5 * estimate.support_profile.segment_recovery_strength.0
                            + 0.3 * estimate.support_profile.recovery_duration_strength.0
                            + 0.2 * estimate.confidence.0,
                    ),
                    Some(estimate),
                    suppression_profile,
                    tempo_ambiguity,
                    bpm,
                    beat_positions_seconds,
                )
            }
            MeterRecommendation::Monitor | MeterRecommendation::Defer => build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::TentativeMeter,
                Confidence::new(
                    0.6 * estimate.confidence.0
                        + 0.4 * estimate.support_profile.whole_track_strength.0,
                ),
                Some(estimate),
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            ),
        };
    }

    let pulse_stability =
        (0.65 * rhythm_confidence.0 + 0.35 * (1.0 - tempo_ambiguity.0)).clamp(0.0, 1.0);
    let trailing_recovery_strength = (0.6 * suppression_profile.trailing_confidence.0
        + 0.4 * suppression_profile.trailing_recent_stability)
        .clamp(0.0, 1.0);

    if trailing_recovery_strength >= 0.24 && pulse_stability >= 0.58 {
        if tempo_ambiguity.0 >= 0.43 {
            return build_meter_state(
                MeterStateAction::Clear,
                MeterStateReason::MeterCleared,
                Confidence::new(
                    (0.5 * tempo_ambiguity.0
                        + 0.3 * trailing_recovery_strength
                        + 0.2 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0)))
                    .clamp(0.0, 1.0),
                ),
                None,
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            );
        }

        if tempo_ambiguity.0 <= 0.33 {
            return build_meter_state(
                MeterStateAction::Hold,
                MeterStateReason::DestabilizedHold,
                Confidence::new(
                    (0.55 * pulse_stability
                        + 0.25 * suppression_profile.best_confidence.0
                        + 0.20 * suppression_profile.best_support)
                        .clamp(0.0, 1.0),
                ),
                None,
                suppression_profile,
                tempo_ambiguity,
                bpm,
                beat_positions_seconds,
            );
        }

        build_meter_state(
            MeterStateAction::Watch,
            MeterStateReason::RecoveryEmerging,
            Confidence::new(
                (0.55 * trailing_recovery_strength + 0.45 * pulse_stability).clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    } else if pulse_stability >= 0.55
        && suppression_profile.best_confidence.0 >= 0.12
        && suppression_profile.best_support >= 0.48
        && suppression_profile.best_regularity >= 0.20
    {
        build_meter_state(
            MeterStateAction::Hold,
            MeterStateReason::DestabilizedHold,
            Confidence::new(
                (0.5 * pulse_stability
                    + 0.3 * suppression_profile.best_confidence.0
                    + 0.2 * suppression_profile.best_support)
                    .clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    } else {
        build_meter_state(
            MeterStateAction::Clear,
            MeterStateReason::MeterCleared,
            Confidence::new(
                (0.45 * (1.0 - suppression_profile.best_confidence.0)
                    + 0.35 * (1.0 - suppression_profile.best_support.clamp(0.0, 1.0))
                    + 0.20 * tempo_ambiguity.0)
                    .clamp(0.0, 1.0),
            ),
            None,
            suppression_profile,
            tempo_ambiguity,
            bpm,
            beat_positions_seconds,
        )
    }
}

fn infer_meter(
    onset_envelope: &[f32],
    meter_cue: &[f32],
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
) -> MeterDecision {
    if onset_envelope.is_empty() || beat_frames.len() < 6 {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
        };
    }

    let beat_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(onset_envelope, *frame, 3))
        .collect();
    let meter_strengths: Vec<f32> = beat_frames
        .iter()
        .map(|frame| neighborhood_peak(meter_cue, *frame, 3))
        .collect();
    let hypotheses = meter_hypotheses(&beat_strengths, &meter_strengths);

    let Some(best) = hypotheses.first().copied() else {
        return MeterDecision {
            estimate: None,
            suppression_profile: MeterSuppressionProfile {
                best_confidence: Confidence::new(0.0),
                best_support: 0.0,
                best_regularity: 0.0,
                trailing_confidence: Confidence::new(0.0),
                trailing_recent_stability: 0.0,
            },
        };
    };
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let confidence = meter_hypothesis_confidence(best, runner_up);
    let global_estimate = if best.score >= 0.12
        && confidence.0 >= 0.18
        && best.regularity >= 0.35
        && best.support_ratio >= 0.60
    {
        let confidence_breakdown = meter_confidence_breakdown(best, runner_up);
        let downbeat_frames = downbeat_frames_for_hypothesis(beat_frames, 0, best);
        Some((
            MeterEstimate {
                beats_per_bar: best.beats_per_bar,
                confidence,
                detection_kind: MeterDetectionKind::WholeTrack,
                trust: MeterTrustLevel::Tentative,
                recommendation: MeterRecommendation::Defer,
                support_profile: MeterSupportProfile {
                    whole_track_strength: confidence,
                    segment_recovery_strength: Confidence::new(0.0),
                    recovery_duration_strength: Confidence::new(0.0),
                },
                confidence_breakdown,
                recovery: None,
                downbeat_positions_seconds: beat_frames_to_seconds(
                    &downbeat_frames,
                    sample_rate,
                    hop_size,
                ),
            },
            best.regularity,
        ))
    } else {
        None
    };

    let segment_candidate = select_segment_meter_candidate(&beat_strengths, &meter_strengths);
    let trailing_candidate = trailing_meter_window_candidate(&beat_strengths, &meter_strengths);
    let support_profile = meter_support_profile(
        global_estimate
            .as_ref()
            .map(|(estimate, _)| estimate.confidence),
        segment_candidate,
    );

    let global_estimate = global_estimate.map(|(mut estimate, regularity)| {
        estimate.support_profile = support_profile;
        estimate.trust = meter_trust_level(
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        estimate.recommendation = meter_recommendation(
            estimate.trust,
            estimate.detection_kind,
            estimate.confidence,
            estimate.support_profile,
            estimate.confidence_breakdown,
        );
        (estimate, regularity)
    });

    let segment_estimate = if let Some(segment_best) = segment_candidate {
        let downbeat_frames = downbeat_frames_for_hypothesis(
            beat_frames,
            segment_best.start_beat,
            segment_best.hypothesis,
        );
        Some(MeterEstimate {
            beats_per_bar: segment_best.hypothesis.beats_per_bar,
            confidence: segment_best.confidence,
            detection_kind: MeterDetectionKind::SegmentRecovery,
            trust: meter_trust_level(
                MeterDetectionKind::SegmentRecovery,
                segment_best.confidence,
                support_profile,
                segment_best.confidence_breakdown,
            ),
            recommendation: MeterRecommendation::Monitor,
            support_profile,
            confidence_breakdown: segment_best.confidence_breakdown,
            recovery: Some(meter_recovery_context(
                beat_frames,
                sample_rate,
                hop_size,
                segment_best,
            )),
            downbeat_positions_seconds: beat_frames_to_seconds(
                &downbeat_frames,
                sample_rate,
                hop_size,
            ),
        })
        .map(|mut estimate| {
            estimate.recommendation = meter_recommendation(
                estimate.trust,
                estimate.detection_kind,
                estimate.confidence,
                estimate.support_profile,
                estimate.confidence_breakdown,
            );
            estimate
        })
    } else {
        None
    };

    let estimate = match (global_estimate, segment_estimate) {
        (Some((global, global_regularity)), Some(segment))
            if segment.confidence.0 >= global.confidence.0 + 0.04 && global_regularity < 0.58 =>
        {
            Some(segment)
        }
        (Some((global, _)), _) => Some(global),
        (None, Some(segment)) => Some(segment),
        (None, None) => None,
    };

    MeterDecision {
        estimate,
        suppression_profile: MeterSuppressionProfile {
            best_confidence: confidence,
            best_support: best.support_ratio,
            best_regularity: best.regularity,
            trailing_confidence: trailing_candidate
                .map(|candidate| candidate.confidence)
                .unwrap_or(Confidence::new(0.0)),
            trailing_recent_stability: trailing_candidate
                .map(|candidate| candidate.hypothesis.recent_strength)
                .unwrap_or(0.0),
        },
    }
}

fn select_beat_phase(onset_envelope: &[f32], lag_frames: usize) -> usize {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0;
    }

    let search_len = lag_frames.min(onset_envelope.len());
    let mut best_phase = 0usize;
    let mut best_score = 0.0f32;

    for phase in 0..search_len {
        let score = beat_phase_score(onset_envelope, lag_frames, phase);
        if score > best_score {
            best_score = score;
            best_phase = phase;
        }
    }

    best_phase
}

fn beat_phase_score(onset_envelope: &[f32], lag_frames: usize, phase_offset_frames: usize) -> f32 {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0.0;
    }

    let radius = ((lag_frames as f32) * 0.15).round().max(1.0) as usize;
    let half_lag = lag_frames / 2;
    let mut beat_sum = 0.0;
    let mut beat_count = 0usize;
    let mut supported_beats = 0usize;
    let mut offbeat_sum = 0.0;
    let mut offbeat_count = 0usize;

    let mut index = phase_offset_frames.min(onset_envelope.len().saturating_sub(1));
    while index < onset_envelope.len() {
        let beat_peak = neighborhood_peak(onset_envelope, index, radius);
        beat_sum += beat_peak;
        beat_count += 1;
        if beat_peak > 0.35 {
            supported_beats += 1;
        }

        if half_lag > 1 {
            let midpoint = index + half_lag;
            if midpoint < onset_envelope.len() {
                offbeat_sum += neighborhood_peak(onset_envelope, midpoint, radius);
                offbeat_count += 1;
            }
        }

        index += lag_frames;
    }

    if beat_count == 0 {
        return 0.0;
    }

    let beat_average = beat_sum / beat_count as f32;
    let support_ratio = supported_beats as f32 / beat_count as f32;
    let offbeat_average = if offbeat_count > 0 {
        offbeat_sum / offbeat_count as f32
    } else {
        0.0
    };

    (0.55 * beat_average + 0.45 * support_ratio - 0.35 * offbeat_average)
        .max(0.0)
        .clamp(0.0, 1.0)
}

fn neighborhood_peak(onset_envelope: &[f32], center: usize, radius: usize) -> f32 {
    if onset_envelope.is_empty() {
        return 0.0;
    }

    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(onset_envelope.len());
    onset_envelope[start..end]
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value))
}

fn refine_beat(onset_envelope: &[f32], center: isize, tolerance_frames: isize) -> isize {
    let start = (center - tolerance_frames).max(0) as usize;
    let end =
        (center + tolerance_frames).min(onset_envelope.len().saturating_sub(1) as isize) as usize;

    let mut best_index = center.clamp(0, onset_envelope.len().saturating_sub(1) as isize) as usize;
    let mut best_value = onset_envelope[best_index];

    for index in start..=end {
        let value = onset_envelope[index];
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }

    best_index as isize
}

fn normalize(values: &mut [f32]) {
    let max_value = values
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value));

    if max_value > 0.0 {
        for value in values {
            *value /= max_value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BeatTracker, BeatTrackerConfig};
    use signal_analysis::AnalysisStage;
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    const CLICK_LENGTH: usize = 64;
    const TONE_BURST_LENGTH: usize = 2_048;
    const KICK_TONES: &[f32] = &[60.0, 95.0];
    const SNARE_TONES: &[f32] = &[220.0, 330.0, 1800.0];
    const HAT_TONES: &[f32] = &[4000.0, 6200.0, 8400.0];
    const CHORD_A: &[f32] = &[220.0, 277.18, 329.63];
    const CHORD_B: &[f32] = &[261.63, 329.63, 392.0];
    const CHORD_C: &[f32] = &[196.0, 246.94, 293.66];
    const CHORD_D: &[f32] = &[246.94, 311.13, 369.99];
    const CHORD_CYCLE_A: &[&[f32]] = &[CHORD_A];
    const CHORD_CYCLE_AB: &[&[f32]] = &[CHORD_A, CHORD_B];
    const CHORD_CYCLE_ABCD: &[&[f32]] = &[CHORD_A, CHORD_B, CHORD_C, CHORD_D];
    const CHORD_CYCLE_CD: &[&[f32]] = &[CHORD_C, CHORD_D];
    const FILL_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.46, 0.24, 0.36, 0.24],
        [0.44, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.38, 0.24],
        [0.36, 0.32, 0.44, 0.62],
        [0.48, 0.24, 0.36, 0.26],
        [0.46, 0.24, 0.36, 0.24],
        [0.5, 0.26, 0.38, 0.24],
        [0.46, 0.24, 0.34, 0.24],
    ];
    const FILL_BAR_CHORDS: &[&[f32]] = &[
        CHORD_A, CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D,
    ];
    const DENSE_FILL_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.26, 0.38, 0.26],
        [0.42, 0.34, 0.38, 0.28],
        [0.5, 0.28, 0.4, 0.28],
        [0.34, 0.4, 0.48, 0.7],
        [0.5, 0.3, 0.4, 0.3],
        [0.38, 0.34, 0.42, 0.56],
        [0.52, 0.3, 0.4, 0.32],
        [0.36, 0.36, 0.42, 0.66],
    ];
    const DENSE_FILL_BAR_CHORDS: &[&[f32]] = &[
        CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D, CHORD_A,
    ];
    const REENTRY_HARMONIC_SHIFT_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C, CHORD_B, CHORD_D];
    const REENTRY_ACCELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C];
    const REENTRY_DECELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_D];
    const REENTRY_ACCELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.54, 0.28, 0.4, 0.3], [0.5, 0.32, 0.42, 0.32]];
    const REENTRY_DECELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.58, 0.3, 0.44, 0.32], [0.54, 0.34, 0.46, 0.36]];
    const REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.28, 0.66, 0.3, 0.58], [0.26, 0.62, 0.3, 0.6]];
    const REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.3, 0.6, 0.28, 0.62], [0.28, 0.64, 0.3, 0.58]];
    const REENTRY_HARMONIC_RESET_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.56, 0.24, 0.4, 0.24], [0.58, 0.24, 0.42, 0.24]];
    const REENTRY_SUSTAINED_RESET_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.6, 0.24, 0.42, 0.24],
        [0.62, 0.24, 0.44, 0.24],
        [0.64, 0.22, 0.44, 0.22],
        [0.62, 0.24, 0.42, 0.24],
        [0.66, 0.22, 0.46, 0.22],
        [0.64, 0.24, 0.44, 0.24],
    ];
    const REENTRY_CADENTIAL_REANCHOR_BAR_PATTERNS: &[[f32; 4]] =
        &[[0.72, 0.22, 0.44, 0.24], [0.64, 0.24, 0.42, 0.24]];
    const REENTRY_CADENTIAL_REANCHOR_BAR_CHORDS: &[&[f32]] = &[CHORD_D, CHORD_A];
    const LATE_SHIFT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.5, 0.26, 0.38, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.28, 0.72, 0.34, 0.22],
        [0.52, 0.28, 0.38, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.5, 0.26, 0.38, 0.24],
    ];
    const LATE_SHIFT_BAR_CHORDS: &[&[f32]] =
        &[CHORD_A, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_A];
    const LIGHT_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.48, 0.24, 0.36, 0.24],
        [0.3, 0.12, 0.24, 0.12],
        [0.5, 0.24, 0.38, 0.24],
        [0.46, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.36, 0.24],
    ];
    const MEDIUM_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.24, 0.1, 0.18, 0.08],
        [0.5, 0.24, 0.38, 0.24],
        [0.22, 0.08, 0.16, 0.08],
        [0.46, 0.22, 0.34, 0.22],
        [0.48, 0.24, 0.36, 0.24],
    ];
    const DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
        [0.48, 0.24, 0.36, 0.24],
        [0.04, 0.0, 0.03, 0.0],
        [0.5, 0.24, 0.38, 0.24],
        [0.03, 0.0, 0.02, 0.0],
        [0.46, 0.22, 0.34, 0.22],
        [0.02, 0.0, 0.02, 0.0],
    ];

    #[derive(Clone, Copy, Debug)]
    enum HarmonicRhythmVariant {
        Sparse,
        Active,
    }

    #[derive(Clone, Copy, Debug)]
    enum FillDensityVariant {
        Medium,
        Dense,
    }

    #[derive(Clone, Copy, Debug)]
    enum DropoutVariant {
        Light,
        Medium,
        Heavy,
        ExtendedHeavy,
    }

    #[derive(Clone, Copy, Debug)]
    enum BarTransitionVariant {
        Pickup,
        PickupExtended,
        LateShift,
        MixedLength,
        Modulation,
        Reentry,
        CadentialElongation,
        ReentryHarmonicShift,
        ReentryDenseFill,
        ReentryAcceleratingHarmony,
        ReentryDeceleratingHarmony,
        ReentryAcceleratingHarmonyDenseFill,
        ReentryDeceleratingHarmonyDenseFill,
        ReentryAcceleratingHarmonyAccentShift,
        ReentryDeceleratingHarmonyAccentShift,
        ReentryAcceleratingHarmonyReset,
        ReentryDeceleratingHarmonyReset,
        ReentryAcceleratingHarmonySustainedReset,
        ReentryAcceleratingHarmonyLongSustainedReset,
        ReentryDeceleratingHarmonySustainedReset,
        ReentryAcceleratingHarmonyCadentialReanchor,
        ReentryDeceleratingHarmonyCadentialReanchor,
        ModulationDenseFill,
        ModulationDenseFillExtended,
    }

    #[derive(Clone, Copy, Debug)]
    enum RhythmPreset {
        NeutralClick120,
        StructuredHarmony120(HarmonicRhythmVariant),
        AmbiguousSubdivision90,
        WeakBackbeat118,
        SectionTransition122,
        FillTransition124(FillDensityVariant),
        Dropout120(DropoutVariant),
        BarTransition120(BarTransitionVariant),
    }

    #[derive(Clone, Copy)]
    struct GrooveSection {
        bars: usize,
        beat_pattern: [f32; 4],
        chord_cycle: &'static [&'static [f32]],
        chord_every_bars: usize,
        section_marker: Option<(usize, &'static [f32], f32)>,
        bar_patterns: Option<&'static [[f32; 4]]>,
        bar_chords: Option<&'static [&'static [f32]]>,
        dropout_bars: &'static [usize],
    }

    #[derive(Default)]
    struct FixtureBuilder {
        beats: Vec<f32>,
        tone_events: Vec<(usize, &'static [f32], f32)>,
    }

    impl FixtureBuilder {
        fn new() -> Self {
            Self::default()
        }

        fn beat_len(&self) -> usize {
            self.beats.len()
        }

        fn push_four_four_section(&mut self, section: GrooveSection) {
            let start_beat = self.beat_len();
            push_four_four_groove(&mut self.beats, &mut self.tone_events, start_beat, section);
        }

        fn build(self, sample_rate: u32, bpm: f32) -> AudioBuffer {
            beat_sequence_track(sample_rate, bpm, &self.beats, &self.tone_events)
        }
    }

    fn analyze_fixture(audio: &AudioBuffer) -> super::BeatAnalysisResult {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        tracker.analyze(audio)
    }

    fn build_structured_harmony_preset(
        sample_rate: u32,
        bpm: f32,
        harmonic_rhythm: HarmonicRhythmVariant,
    ) -> AudioBuffer {
        let (chord_every_bars, section_marker) = match harmonic_rhythm {
            HarmonicRhythmVariant::Sparse => (2, Some((12, CHORD_B, 0.68))),
            HarmonicRhythmVariant::Active => (1, Some((12, CHORD_C, 0.8))),
        };
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 6,
            beat_pattern: [0.5, 0.26, 0.38, 0.24],
            chord_cycle: CHORD_CYCLE_ABCD,
            chord_every_bars,
            section_marker,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        fixture.build(sample_rate, bpm)
    }

    fn build_fill_transition_preset(
        sample_rate: u32,
        bpm: f32,
        density: FillDensityVariant,
    ) -> AudioBuffer {
        let (bar_patterns, bar_chords, section_marker) = match density {
            FillDensityVariant::Medium => (FILL_BAR_PATTERNS, FILL_BAR_CHORDS, (16, CHORD_C, 0.85)),
            FillDensityVariant::Dense => (
                DENSE_FILL_BAR_PATTERNS,
                DENSE_FILL_BAR_CHORDS,
                (16, CHORD_D, 0.95),
            ),
        };
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 8,
            beat_pattern: [0.46, 0.24, 0.36, 0.24],
            chord_cycle: CHORD_CYCLE_ABCD,
            chord_every_bars: 2,
            section_marker: Some(section_marker),
            bar_patterns: Some(bar_patterns),
            bar_chords: Some(bar_chords),
            dropout_bars: &[],
        });
        fixture.build(sample_rate, bpm)
    }

    fn build_dropout_preset(sample_rate: u32, bpm: f32, variant: DropoutVariant) -> AudioBuffer {
        let (bar_patterns, dropout_bars, chord_cycle, chord_every_bars, section_marker) =
            match variant {
                DropoutVariant::Light => (
                    Some(LIGHT_DROPOUT_BAR_PATTERNS),
                    &[][..],
                    CHORD_CYCLE_ABCD,
                    2,
                    Some((8, CHORD_C, 0.82)),
                ),
                DropoutVariant::Medium => (
                    Some(MEDIUM_DROPOUT_BAR_PATTERNS),
                    &[3][..],
                    CHORD_CYCLE_ABCD,
                    2,
                    Some((8, CHORD_D, 0.84)),
                ),
                DropoutVariant::Heavy => (
                    Some(DROPOUT_BAR_PATTERNS),
                    &[1, 3, 5][..],
                    &[CHORD_A][..],
                    16,
                    None,
                ),
                DropoutVariant::ExtendedHeavy => {
                    (None, &[1, 3, 5, 7, 9][..], &[CHORD_A][..], 24, None)
                }
            };
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: if matches!(variant, DropoutVariant::ExtendedHeavy) {
                10
            } else {
                6
            },
            beat_pattern: [0.48, 0.24, 0.36, 0.24],
            chord_cycle,
            chord_every_bars,
            section_marker,
            bar_patterns,
            bar_chords: None,
            dropout_bars,
        });
        fixture.build(sample_rate, bpm)
    }

    fn build_reentry_transition_fixture(
        sample_rate: u32,
        bpm: f32,
        recovery_sections: &[GrooveSection],
    ) -> AudioBuffer {
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 2,
            beat_pattern: [0.48, 0.24, 0.36, 0.24],
            chord_cycle: CHORD_CYCLE_AB,
            chord_every_bars: 1,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        fixture.push_four_four_section(GrooveSection {
            bars: 2,
            beat_pattern: [0.48, 0.24, 0.36, 0.24],
            chord_cycle: CHORD_CYCLE_CD,
            chord_every_bars: 1,
            section_marker: Some((4, CHORD_A, 1.0)),
            bar_patterns: Some(MEDIUM_DROPOUT_BAR_PATTERNS),
            bar_chords: None,
            dropout_bars: &[0, 1],
        });
        for &section in recovery_sections {
            fixture.push_four_four_section(section);
        }
        fixture.build(sample_rate, bpm)
    }

    fn build_bar_transition_preset(
        sample_rate: u32,
        bpm: f32,
        variant: BarTransitionVariant,
    ) -> AudioBuffer {
        match variant {
            BarTransitionVariant::Pickup => {
                let mut beats = vec![0.45, 0.7];
                for _ in 0..5 {
                    beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
                }

                let mut tone_events = Vec::new();
                for (bar_index, chord) in
                    CHORD_CYCLE_ABCD.iter().copied().cycle().take(5).enumerate()
                {
                    tone_events.push((2 + bar_index * 4, chord, 0.82));
                }

                beat_sequence_track(sample_rate, bpm, &beats, &tone_events)
            }
            BarTransitionVariant::PickupExtended => {
                let mut beats = vec![0.32, 0.58, 0.38, 0.68, 0.42, 0.72];
                for _ in 0..6 {
                    beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
                }

                let mut tone_events = Vec::new();
                for (bar_index, chord) in
                    CHORD_CYCLE_ABCD.iter().copied().cycle().take(6).enumerate()
                {
                    tone_events.push((6 + bar_index * 4, chord, 0.84));
                }

                beat_sequence_track(sample_rate, bpm, &beats, &tone_events)
            }
            BarTransitionVariant::LateShift => {
                let mut fixture = FixtureBuilder::new();
                fixture.push_four_four_section(GrooveSection {
                    bars: 6,
                    beat_pattern: [0.5, 0.26, 0.38, 0.24],
                    chord_cycle: CHORD_CYCLE_ABCD,
                    chord_every_bars: 1,
                    section_marker: Some((10, CHORD_C, 0.9)),
                    bar_patterns: Some(LATE_SHIFT_BAR_PATTERNS),
                    bar_chords: Some(LATE_SHIFT_BAR_CHORDS),
                    dropout_bars: &[],
                });
                fixture.build(sample_rate, bpm)
            }
            BarTransitionVariant::MixedLength => {
                let beats = [
                    1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 0.95, 0.38, 0.48, 1.0, 0.35, 0.55,
                    0.4, 0.92, 0.38, 0.46, 1.0, 0.35, 0.55, 0.4,
                ];
                let tone_events: &[(usize, &'static [f32], f32)] = &[
                    (0, CHORD_A, 0.8),
                    (4, CHORD_B, 0.8),
                    (8, CHORD_C, 0.82),
                    (11, CHORD_D, 0.78),
                    (15, CHORD_C, 0.78),
                    (18, CHORD_A, 0.8),
                ];
                beat_sequence_track(sample_rate, bpm, &beats, tone_events)
            }
            BarTransitionVariant::Modulation => {
                let beats = [
                    1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.42, 0.48,
                    1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4,
                ];
                let tone_events: &[(usize, &'static [f32], f32)] = &[
                    (0, CHORD_A, 0.8),
                    (4, CHORD_B, 0.8),
                    (8, CHORD_C, 0.82),
                    (11, CHORD_D, 0.84),
                    (14, CHORD_C, 0.8),
                    (18, CHORD_A, 0.82),
                ];
                beat_sequence_track(sample_rate, bpm, &beats, tone_events)
            }
            BarTransitionVariant::Reentry => build_reentry_transition_fixture(
                sample_rate,
                bpm,
                &[GrooveSection {
                    bars: 4,
                    beat_pattern: [0.62, 0.24, 0.42, 0.26],
                    chord_cycle: CHORD_CYCLE_ABCD,
                    chord_every_bars: 1,
                    section_marker: Some((0, CHORD_D, 1.1)),
                    bar_patterns: None,
                    bar_chords: None,
                    dropout_bars: &[],
                }],
            ),
            BarTransitionVariant::CadentialElongation => {
                let beats = [
                    1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 0.9, 0.32,
                    0.48, 0.38, 0.62, 1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4,
                ];
                let tone_events: &[(usize, &'static [f32], f32)] = &[
                    (0, CHORD_A, 0.8),
                    (4, CHORD_B, 0.8),
                    (8, CHORD_C, 0.82),
                    (12, CHORD_D, 0.88),
                    (17, CHORD_A, 0.86),
                    (21, CHORD_B, 0.82),
                ];
                beat_sequence_track(sample_rate, bpm, &beats, tone_events)
            }
            BarTransitionVariant::ReentryHarmonicShift => build_reentry_transition_fixture(
                sample_rate,
                bpm,
                &[GrooveSection {
                    bars: 4,
                    beat_pattern: [0.62, 0.24, 0.42, 0.26],
                    chord_cycle: CHORD_CYCLE_ABCD,
                    chord_every_bars: 1,
                    section_marker: Some((0, CHORD_D, 1.1)),
                    bar_patterns: None,
                    bar_chords: Some(REENTRY_HARMONIC_SHIFT_BAR_CHORDS),
                    dropout_bars: &[],
                }],
            ),
            BarTransitionVariant::ReentryDenseFill => build_reentry_transition_fixture(
                sample_rate,
                bpm,
                &[GrooveSection {
                    bars: 4,
                    beat_pattern: [0.58, 0.26, 0.4, 0.26],
                    chord_cycle: CHORD_CYCLE_ABCD,
                    chord_every_bars: 1,
                    section_marker: Some((0, CHORD_D, 1.12)),
                    bar_patterns: Some(DENSE_FILL_BAR_PATTERNS),
                    bar_chords: Some(DENSE_FILL_BAR_CHORDS),
                    dropout_bars: &[],
                }],
            ),
            BarTransitionVariant::ReentryAcceleratingHarmony => build_reentry_transition_fixture(
                sample_rate,
                bpm,
                &[
                    GrooveSection {
                        bars: 2,
                        beat_pattern: [0.56, 0.24, 0.38, 0.24],
                        chord_cycle: CHORD_CYCLE_AB,
                        chord_every_bars: 2,
                        section_marker: Some((0, CHORD_B, 1.02)),
                        bar_patterns: None,
                        bar_chords: None,
                        dropout_bars: &[],
                    },
                    GrooveSection {
                        bars: 2,
                        beat_pattern: [0.62, 0.24, 0.42, 0.26],
                        chord_cycle: CHORD_CYCLE_ABCD,
                        chord_every_bars: 1,
                        section_marker: Some((0, CHORD_D, 1.12)),
                        bar_patterns: None,
                        bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                        dropout_bars: &[],
                    },
                ],
            ),
            BarTransitionVariant::ReentryDeceleratingHarmony => build_reentry_transition_fixture(
                sample_rate,
                bpm,
                &[
                    GrooveSection {
                        bars: 2,
                        beat_pattern: [0.62, 0.24, 0.42, 0.26],
                        chord_cycle: CHORD_CYCLE_ABCD,
                        chord_every_bars: 1,
                        section_marker: Some((0, CHORD_D, 1.12)),
                        bar_patterns: None,
                        bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                        dropout_bars: &[],
                    },
                    GrooveSection {
                        bars: 2,
                        beat_pattern: [0.56, 0.24, 0.38, 0.24],
                        chord_cycle: CHORD_CYCLE_AB,
                        chord_every_bars: 2,
                        section_marker: Some((0, CHORD_B, 1.02)),
                        bar_patterns: None,
                        bar_chords: None,
                        dropout_bars: &[],
                    },
                ],
            ),
            BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.56, 0.24, 0.38, 0.24],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((0, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_DENSE_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.62, 0.24, 0.42, 0.26],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_D, 1.14)),
                            bar_patterns: Some(REENTRY_DECELERATING_DENSE_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.62, 0.24, 0.42, 0.26],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_D, 1.14)),
                            bar_patterns: Some(REENTRY_DECELERATING_DENSE_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.56, 0.24, 0.38, 0.24],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((0, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_DENSE_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryAcceleratingHarmonyReset => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 4,
                            beat_pattern: [0.62, 0.24, 0.42, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_A, 1.12)),
                            bar_patterns: Some(REENTRY_HARMONIC_RESET_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryDeceleratingHarmonyReset => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 4,
                            beat_pattern: [0.62, 0.24, 0.42, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_A, 1.12)),
                            bar_patterns: Some(REENTRY_HARMONIC_RESET_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 6,
                            beat_pattern: [0.64, 0.24, 0.44, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_A, 1.14)),
                            bar_patterns: Some(REENTRY_SUSTAINED_RESET_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 8,
                            beat_pattern: [0.64, 0.24, 0.44, 0.24],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_A, 1.14)),
                            bar_patterns: None,
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 6,
                            beat_pattern: [0.64, 0.24, 0.44, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_A, 1.14)),
                            bar_patterns: Some(REENTRY_SUSTAINED_RESET_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_ACCELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 4,
                            beat_pattern: [0.72, 0.22, 0.44, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_D, 1.2)),
                            bar_patterns: Some(REENTRY_CADENTIAL_REANCHOR_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_CADENTIAL_REANCHOR_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor => {
                build_reentry_transition_fixture(
                    sample_rate,
                    bpm,
                    &[
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.28, 0.66, 0.3, 0.6],
                            chord_cycle: CHORD_CYCLE_ABCD,
                            chord_every_bars: 1,
                            section_marker: Some((1, CHORD_D, 1.12)),
                            bar_patterns: Some(REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_DECELERATING_STAGE_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 2,
                            beat_pattern: [0.3, 0.64, 0.3, 0.58],
                            chord_cycle: CHORD_CYCLE_AB,
                            chord_every_bars: 2,
                            section_marker: Some((1, CHORD_B, 1.02)),
                            bar_patterns: Some(REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS),
                            bar_chords: None,
                            dropout_bars: &[],
                        },
                        GrooveSection {
                            bars: 4,
                            beat_pattern: [0.72, 0.22, 0.44, 0.24],
                            chord_cycle: CHORD_CYCLE_A,
                            chord_every_bars: 1,
                            section_marker: Some((0, CHORD_D, 1.2)),
                            bar_patterns: Some(REENTRY_CADENTIAL_REANCHOR_BAR_PATTERNS),
                            bar_chords: Some(REENTRY_CADENTIAL_REANCHOR_BAR_CHORDS),
                            dropout_bars: &[],
                        },
                    ],
                )
            }
            BarTransitionVariant::ModulationDenseFill => {
                let beats = [
                    1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.42, 0.48,
                    1.0, 0.36, 0.44, 0.92, 0.34, 0.58, 1.0, 0.36, 0.46, 0.96, 0.34, 0.56,
                ];
                let tone_events: &[(usize, &'static [f32], f32)] = &[
                    (0, CHORD_A, 0.8),
                    (4, CHORD_B, 0.8),
                    (8, CHORD_C, 0.86),
                    (11, CHORD_D, 0.88),
                    (14, CHORD_A, 0.9),
                    (18, CHORD_C, 0.9),
                    (21, CHORD_D, 0.88),
                ];
                beat_sequence_track(sample_rate, bpm, &beats, tone_events)
            }
            BarTransitionVariant::ModulationDenseFillExtended => {
                let beats = [
                    1.0, 0.35, 0.55, 0.4, 0.98, 0.36, 0.44, 0.92, 0.34, 0.58, 1.0, 0.42, 0.48,
                    0.94, 0.34, 0.56, 1.0, 0.36, 0.46, 0.96, 0.34, 0.58, 0.9, 0.32, 0.46, 0.4,
                    0.64, 1.0, 0.36, 0.46, 0.98, 0.34, 0.6,
                ];
                let tone_events: &[(usize, &'static [f32], f32)] = &[
                    (0, CHORD_A, 0.82),
                    (4, CHORD_C, 0.86),
                    (7, CHORD_B, 0.84),
                    (10, CHORD_D, 0.88),
                    (13, CHORD_C, 0.9),
                    (17, CHORD_A, 0.88),
                    (22, CHORD_D, 0.9),
                    (27, CHORD_B, 0.86),
                ];
                beat_sequence_track(sample_rate, bpm, &beats, tone_events)
            }
        }
    }

    fn render_preset(preset: RhythmPreset, sample_rate: u32) -> (f32, AudioBuffer) {
        match preset {
            RhythmPreset::NeutralClick120 => (120.0, click_track(sample_rate, 120.0, 8.0)),
            RhythmPreset::StructuredHarmony120(harmonic_rhythm) => (
                120.0,
                build_structured_harmony_preset(sample_rate, 120.0, harmonic_rhythm),
            ),
            RhythmPreset::AmbiguousSubdivision90 => (
                90.0,
                grid_click_track(sample_rate, 90.0, 2, 8.0, &[1.0, 0.3], None),
            ),
            RhythmPreset::WeakBackbeat118 => {
                let bpm = 118.0;
                let mut fixture = FixtureBuilder::new();
                fixture.push_four_four_section(GrooveSection {
                    bars: 8,
                    beat_pattern: [0.42, 0.24, 0.34, 0.22],
                    chord_cycle: CHORD_CYCLE_ABCD,
                    chord_every_bars: 2,
                    section_marker: None,
                    bar_patterns: None,
                    bar_chords: None,
                    dropout_bars: &[],
                });
                (bpm, fixture.build(sample_rate, bpm))
            }
            RhythmPreset::SectionTransition122 => {
                let bpm = 122.0;
                let mut fixture = FixtureBuilder::new();
                fixture.push_four_four_section(GrooveSection {
                    bars: 4,
                    beat_pattern: [0.48, 0.22, 0.36, 0.26],
                    chord_cycle: CHORD_CYCLE_AB,
                    chord_every_bars: 2,
                    section_marker: Some((16, CHORD_C, 0.9)),
                    bar_patterns: None,
                    bar_chords: None,
                    dropout_bars: &[],
                });
                fixture.push_four_four_section(GrooveSection {
                    bars: 4,
                    beat_pattern: [0.55, 0.26, 0.38, 0.28],
                    chord_cycle: CHORD_CYCLE_CD,
                    chord_every_bars: 2,
                    section_marker: None,
                    bar_patterns: None,
                    bar_chords: None,
                    dropout_bars: &[],
                });
                (bpm, fixture.build(sample_rate, bpm))
            }
            RhythmPreset::FillTransition124(density) => (
                124.0,
                build_fill_transition_preset(sample_rate, 124.0, density),
            ),
            RhythmPreset::Dropout120(variant) => {
                (120.0, build_dropout_preset(sample_rate, 120.0, variant))
            }
            RhythmPreset::BarTransition120(variant) => (
                120.0,
                build_bar_transition_preset(sample_rate, 120.0, variant),
            ),
        }
    }

    fn analyze_preset(preset: RhythmPreset) -> (f32, super::BeatAnalysisResult) {
        let sample_rate = 48_000;
        let (bpm, audio) = render_preset(preset, sample_rate);
        (bpm, analyze_fixture(&audio))
    }

    fn synthetic_tempo_diagnostics(
        core_window_bpm: f32,
        boundary_bias_bpm: f32,
        trend_total_drift_bpm: f32,
        trend_fit_mad_bpm: f32,
        mean_abs_residual_ms: f32,
        core_abs_residual_ms: f32,
        anchored_drift_ms: f32,
        edge_abs_residual_ms: f32,
    ) -> super::TempoDiagnostics {
        super::TempoDiagnostics {
            interval_tempi: Vec::new(),
            windowed_tempi: Vec::new(),
            median_bpm: core_window_bpm,
            drift_span_bpm: boundary_bias_bpm,
            mean_abs_deviation_bpm: trend_fit_mad_bpm,
            windowed_median_bpm: core_window_bpm,
            windowed_drift_span_bpm: boundary_bias_bpm,
            windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            core_windowed_median_bpm: core_window_bpm,
            core_windowed_drift_span_bpm: trend_total_drift_bpm.abs(),
            core_windowed_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            boundary_bias_bpm,
            trend: super::TempoTrendDiagnostics {
                direction: if trend_total_drift_bpm.abs() < 0.15 {
                    super::TempoTrendDirection::Stable
                } else if trend_total_drift_bpm > 0.0 {
                    super::TempoTrendDirection::Accelerating
                } else {
                    super::TempoTrendDirection::Decelerating
                },
                start_bpm: core_window_bpm - 0.5 * trend_total_drift_bpm,
                end_bpm: core_window_bpm + 0.5 * trend_total_drift_bpm,
                total_drift_bpm: trend_total_drift_bpm,
                slope_bpm_per_beat: trend_total_drift_bpm / 8.0,
                fit_mean_abs_deviation_bpm: trend_fit_mad_bpm,
            },
            beat_grid_error: super::BeatGridErrorDiagnostics {
                residuals: Vec::new(),
                mean_abs_residual_ms,
                max_abs_residual_ms: edge_abs_residual_ms.max(core_abs_residual_ms),
                edge_mean_abs_residual_ms: edge_abs_residual_ms,
                core_mean_abs_residual_ms: core_abs_residual_ms,
                end_anchored_drift_ms: anchored_drift_ms,
                mean_abs_anchored_drift_ms: anchored_drift_ms.abs(),
            },
        }
    }

    fn synthetic_tempo_interpretation(
        recommendation: super::TempoRecommendation,
        trust: super::TempoTrustLevel,
        reason: super::TempoInterpretationReason,
        recommended_bpm: f32,
        snapped_bpm: Option<f32>,
        stability_score: f32,
        snap_error_bpm: f32,
        boundary_pressure: f32,
        grid_stability: f32,
    ) -> super::TempoInterpretation {
        super::TempoInterpretation {
            trust,
            recommendation,
            reason,
            recommended_bpm,
            snapped_bpm,
            support: super::TempoInterpretationSupport {
                core_consensus: super::Confidence::new(0.9),
                drift_stability: super::Confidence::new(0.8),
                grid_stability: super::Confidence::new(grid_stability),
                integer_closeness: super::Confidence::new(
                    (1.0 - snap_error_bpm / 0.12).clamp(0.0, 1.0),
                ),
                boundary_pressure: super::Confidence::new(boundary_pressure),
            },
            profile: super::TempoInterpretationProfile {
                refined_bpm: recommended_bpm,
                core_window_bpm: recommended_bpm,
                nearest_integer_bpm: recommended_bpm.round(),
                snap_error_bpm,
                stability_score: super::Confidence::new(stability_score),
                boundary_edge_gap_ms: 4.0 * boundary_pressure,
            },
        }
    }

    fn assert_detected_bpm(
        preset: RhythmPreset,
        result: &super::BeatAnalysisResult,
        expected_bpm: f32,
        tolerance: f32,
    ) {
        assert!(
            (result.bpm - expected_bpm).abs() < tolerance,
            "preset {:?} detected bpm {} expected {} +/- {}",
            preset,
            result.bpm,
            expected_bpm,
            tolerance
        );
    }

    fn assert_meter(
        preset: RhythmPreset,
        result: &super::BeatAnalysisResult,
        beats_per_bar: usize,
        min_confidence: f32,
    ) -> &super::MeterEstimate {
        let meter = result
            .meter
            .as_ref()
            .unwrap_or_else(|| panic!("preset {:?} expected meter estimate", preset));
        assert_eq!(
            meter.beats_per_bar, beats_per_bar,
            "preset {:?} beats_per_bar {}",
            preset, meter.beats_per_bar
        );
        assert!(
            meter.confidence.0 > min_confidence,
            "preset {:?} meter confidence {}",
            preset,
            meter.confidence.0
        );
        meter
    }

    fn add_click(samples: &mut [f32], index: usize, amplitude: f32) {
        for offset in 0..CLICK_LENGTH {
            if let Some(sample) = samples.get_mut(index + offset) {
                *sample += amplitude * (1.0 - offset as f32 / CLICK_LENGTH as f32);
            }
        }
    }

    fn add_tone_burst(
        samples: &mut [f32],
        sample_rate: u32,
        index: usize,
        frequencies: &[f32],
        amplitude: f32,
    ) {
        for offset in 0..TONE_BURST_LENGTH {
            let Some(sample) = samples.get_mut(index + offset) else {
                break;
            };
            let t = offset as f32 / sample_rate as f32;
            let envelope = (1.0 - offset as f32 / TONE_BURST_LENGTH as f32).max(0.0);
            let tone = frequencies
                .iter()
                .copied()
                .map(|frequency| (core::f32::consts::TAU * frequency * t).sin())
                .sum::<f32>();
            *sample += amplitude * envelope * tone / frequencies.len().max(1) as f32;
        }
    }

    fn click_track(sample_rate: u32, bpm: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = vec![0.0; frames];
        let interval = (60.0 / bpm * sample_rate as f32).round() as usize;

        let mut index = 0usize;
        while index < frames {
            add_click(&mut samples, index, 1.0);
            index = index.saturating_add(interval.max(1));
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn grid_click_track(
        sample_rate: u32,
        bpm: f32,
        steps_per_beat: usize,
        seconds: f32,
        pattern: &[f32],
        swing_ratio: Option<f32>,
    ) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let beat_frames = 60.0 / bpm * sample_rate as f32;
        let subdivision_frames = beat_frames / steps_per_beat.max(1) as f32;
        let mut samples = vec![0.0; frames];
        let total_steps = ((seconds * bpm / 60.0) * steps_per_beat as f32).ceil() as usize;

        for step in 0..total_steps {
            let amplitude = pattern[step % pattern.len()];
            if amplitude <= 0.0 {
                continue;
            }

            let beat_index = step / steps_per_beat.max(1);
            let step_in_beat = step % steps_per_beat.max(1);
            let offset_frames = if steps_per_beat == 2 {
                match (step_in_beat, swing_ratio) {
                    (0, _) => 0.0,
                    (1, Some(ratio)) => beat_frames * ratio.clamp(0.5, 0.85),
                    _ => subdivision_frames,
                }
            } else {
                step_in_beat as f32 * subdivision_frames
            };
            let index = (beat_index as f32 * beat_frames + offset_frames).round() as usize;
            add_click(&mut samples, index, amplitude);
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn beat_sequence_track(
        sample_rate: u32,
        bpm: f32,
        beat_amplitudes: &[f32],
        tone_events: &[(usize, &'static [f32], f32)],
    ) -> AudioBuffer {
        let beat_frames = (60.0 / bpm * sample_rate as f32).round() as usize;
        let frames = beat_frames
            .saturating_mul(beat_amplitudes.len())
            .saturating_add(TONE_BURST_LENGTH);
        let mut samples = vec![0.0; frames];

        for (beat_index, amplitude) in beat_amplitudes.iter().copied().enumerate() {
            if amplitude > 0.0 {
                add_click(
                    &mut samples,
                    beat_index.saturating_mul(beat_frames),
                    amplitude,
                );
            }
        }

        for (beat_index, frequencies, amplitude) in tone_events {
            add_tone_burst(
                &mut samples,
                sample_rate,
                beat_index.saturating_mul(beat_frames),
                frequencies,
                *amplitude,
            );
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn push_four_four_groove(
        beats: &mut Vec<f32>,
        tone_events: &mut Vec<(usize, &'static [f32], f32)>,
        start_beat: usize,
        section: GrooveSection,
    ) {
        for bar in 0..section.bars {
            let beat_pattern = section
                .bar_patterns
                .and_then(|patterns| patterns.get(bar).copied())
                .unwrap_or(section.beat_pattern);
            let is_dropout_bar = section.dropout_bars.contains(&bar);

            for beat_in_bar in 0..4usize {
                let beat_index = start_beat + bar * 4 + beat_in_bar;
                let beat_amplitude = if is_dropout_bar {
                    0.35 * beat_pattern[beat_in_bar]
                } else {
                    beat_pattern[beat_in_bar]
                };
                beats.push(beat_amplitude);

                if !is_dropout_bar {
                    tone_events.push((beat_index, KICK_TONES, 0.18 * beat_amplitude));
                    if beat_in_bar == 1 || beat_in_bar == 3 {
                        tone_events.push((beat_index, SNARE_TONES, 0.28));
                    } else {
                        tone_events.push((beat_index, HAT_TONES, 0.12));
                    }
                } else if beat_in_bar == 3 {
                    tone_events.push((beat_index, HAT_TONES, 0.08));
                }
            }

            let bar_chord = section
                .bar_chords
                .and_then(|plan| plan.get(bar).copied())
                .or_else(|| {
                    if bar % section.chord_every_bars == 0 {
                        Some(
                            section.chord_cycle
                                [(bar / section.chord_every_bars) % section.chord_cycle.len()],
                        )
                    } else {
                        None
                    }
                });
            if let Some(chord) = bar_chord {
                tone_events.push((
                    start_beat + bar * 4,
                    chord,
                    if is_dropout_bar { 0.65 } else { 0.55 },
                ));
            }
        }

        if let Some((offset_beats, chord, amplitude)) = section.section_marker {
            tone_events.push((start_beat + offset_beats, chord, amplitude));
        }
    }

    #[test]
    fn beat_tracker_detects_click_track_tempo() {
        let audio = click_track(48_000, 120.0, 8.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.2,
            "confidence {}",
            result.confidence.0
        );
        assert!(result.beat_positions_seconds.len() >= 6);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_detects_slower_click_track_tempo() {
        let audio = click_track(48_000, 90.0, 8.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 90.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.15,
            "confidence {}",
            result.confidence.0
        );
    }

    #[test]
    fn beat_tracker_refines_integer_click_track_tempo_to_sub_tenth_bpm() {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());

        let fast = tracker.analyze(&click_track(48_000, 120.0, 8.0));
        assert!(
            (fast.bpm - 120.0).abs() < 0.1,
            "refined detected bpm {}",
            fast.bpm
        );
        assert!(
            fast.tempo_candidates
                .first()
                .map(|candidate| (candidate.bpm - 120.0).abs() < 0.1)
                .unwrap_or(false),
            "top tempo candidate {:?}",
            fast.tempo_candidates.first()
        );

        let slow = tracker.analyze(&click_track(48_000, 90.0, 8.0));
        assert!(
            (slow.bpm - 90.0).abs() < 0.1,
            "refined detected bpm {}",
            slow.bpm
        );
        assert!(
            slow.tempo_candidates
                .first()
                .map(|candidate| (candidate.bpm - 90.0).abs() < 0.1)
                .unwrap_or(false),
            "top tempo candidate {:?}",
            slow.tempo_candidates.first()
        );
    }

    #[test]
    fn beat_tracker_exposes_stable_local_tempo_for_integer_click_track() {
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&click_track(48_000, 120.0, 8.0));

        assert!(result.tempo_diagnostics.interval_tempi.len() >= 10);
        assert!(result.tempo_diagnostics.windowed_tempi.len() >= 6);
        assert!(
            (result.tempo_diagnostics.median_bpm - 120.0).abs() < 0.15,
            "median local tempo {}",
            result.tempo_diagnostics.median_bpm
        );
        assert!(
            result.tempo_diagnostics.mean_abs_deviation_bpm < 0.15,
            "local tempo MAD {}",
            result.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            result.tempo_diagnostics.windowed_mean_abs_deviation_bpm
                < result.tempo_diagnostics.mean_abs_deviation_bpm,
            "windowed MAD {} raw MAD {}",
            result.tempo_diagnostics.windowed_mean_abs_deviation_bpm,
            result.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            (result.tempo_diagnostics.core_windowed_median_bpm - 120.0).abs() < 0.15,
            "core windowed median {}",
            result.tempo_diagnostics.core_windowed_median_bpm
        );
        assert!(
            result
                .tempo_diagnostics
                .core_windowed_mean_abs_deviation_bpm
                < 0.15,
            "core windowed MAD {}",
            result
                .tempo_diagnostics
                .core_windowed_mean_abs_deviation_bpm
        );
        assert!(
            result.tempo_diagnostics.boundary_bias_bpm > 0.05,
            "boundary bias {}",
            result.tempo_diagnostics.boundary_bias_bpm
        );
        assert!(
            result.tempo_diagnostics.boundary_bias_bpm
                < result.tempo_diagnostics.windowed_drift_span_bpm,
            "boundary bias {} full windowed span {}",
            result.tempo_diagnostics.boundary_bias_bpm,
            result.tempo_diagnostics.windowed_drift_span_bpm
        );
        assert_eq!(
            result.tempo_diagnostics.trend.direction,
            super::TempoTrendDirection::Stable
        );
        assert!(
            result.tempo_diagnostics.trend.total_drift_bpm.abs() < 0.15,
            "tempo drift {}",
            result.tempo_diagnostics.trend.total_drift_bpm
        );
        assert_eq!(
            result.tempo_diagnostics.beat_grid_error.residuals.len(),
            result.beat_positions_seconds.len()
        );
        assert!(
            result
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_residual_ms
                < 6.0,
            "mean abs residual ms {}",
            result
                .tempo_diagnostics
                .beat_grid_error
                .mean_abs_residual_ms
        );
        assert_eq!(
            result.tempo_interpretation.recommendation,
            super::TempoRecommendation::SnapInteger
        );
        assert_eq!(
            result.tempo_interpretation.reason,
            super::TempoInterpretationReason::NearIntegerPulse
        );
        assert_eq!(result.tempo_interpretation.snapped_bpm, Some(120.0));
        assert!(result.tempo_interpretation.profile.snap_error_bpm < 0.12);
        assert!(result.tempo_interpretation.profile.stability_score.0 > 0.75);
        assert_eq!(result.tempo_state.action, super::TempoStateAction::Lock);
        assert_eq!(
            result.tempo_state.reason,
            super::TempoStateReason::StableIntegerTempo
        );
        assert_eq!(
            result.tempo_state.continuity.action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            result.tempo_state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            result.tempo_state.continuity.reason,
            super::TempoContinuityReason::IntegerTempoSnap
        );
        assert_eq!(
            result.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            result.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            result.tempo_state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            result.tempo_state.continuity.expiry.guaranteed_until_beats,
            16
        );
        assert_eq!(
            result.tempo_state.continuity.expiry.downgrade_after_beats,
            20
        );
        assert_eq!(result.tempo_state.continuity.expiry.clear_after_beats, 28);
        assert!(result.tempo_state.continuity.refresh_strength.0 > 0.9);
    }

    #[test]
    fn beat_tracker_exposes_non_empty_onset_envelope_for_click_track() {
        let audio = click_track(48_000, 120.0, 4.0);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(!result.onset_envelope.is_empty());
        assert!(result.onset_envelope.iter().any(|value| *value > 0.5));
    }

    #[test]
    fn beat_tracker_returns_zero_for_silence() {
        let audio = AudioBuffer::new(
            SampleRate(48_000),
            ChannelLayout::Mono,
            signal_primitives::FrameCount(48_000),
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert_eq!(result.bpm, 0.0);
        assert_eq!(result.confidence.0, 0.0);
        assert!(result.beat_positions_seconds.is_empty());
        assert!(result.tempo_candidates.is_empty());
        assert_eq!(result.tempo_ambiguity.0, 0.0);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_detects_swung_click_track_tempo() {
        let audio = grid_click_track(
            48_000,
            120.0,
            2,
            8.0,
            &[1.0, 0.45, 0.85, 0.35],
            Some(2.0 / 3.0),
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.15,
            "confidence {}",
            result.confidence.0
        );
    }

    #[test]
    fn beat_tracker_handles_syncopated_pattern_without_halving_tempo() {
        let audio = grid_click_track(
            48_000,
            120.0,
            2,
            8.0,
            &[1.0, 0.0, 0.35, 0.8, 0.95, 0.0, 0.3, 0.75],
            None,
        );
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.5,
            "detected bpm {}",
            result.bpm
        );
    }

    #[test]
    fn beat_tracker_prefers_base_tempo_over_double_time_subdivisions() {
        let audio = grid_click_track(48_000, 90.0, 2, 8.0, &[1.0, 0.3], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 90.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert!(
            result.confidence.0 > 0.1,
            "confidence {}",
            result.confidence.0
        );
        assert!(result.tempo_candidates.len() >= 2);
        assert!(result
            .tempo_candidates
            .iter()
            .skip(1)
            .any(|candidate| (candidate.bpm - 180.0).abs() < 4.0));
        assert!(result.tempo_ambiguity.0 > 0.2);
    }

    #[test]
    fn beat_tracker_selects_consistent_phase_over_single_loud_offbeat() {
        let sample_rate = 48_000;
        let bpm = 120.0;
        let mut audio = click_track(sample_rate, bpm, 8.0);
        let offbeat_index = (60.0 / bpm * sample_rate as f32 / 2.0).round() as usize;
        add_click(audio.samples_mut(), offbeat_index, 1.25);

        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(
            (result.bpm - 120.0).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        let quarter_note_seconds = 60.0 / bpm;
        assert!(result.beat_positions_seconds.iter().take(6).all(|beat| {
            let nearest_grid = (*beat / quarter_note_seconds).round() * quarter_note_seconds;
            (nearest_grid - *beat).abs() < 0.08
        }));
    }

    #[test]
    fn beat_tracker_infers_four_four_bar_phase_from_accent_pattern() {
        let bpm = 120.0;
        let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.35, 0.55, 0.4], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 4.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(4)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.08
            }));
    }

    #[test]
    fn beat_tracker_infers_three_four_bar_phase_from_waltz_pattern() {
        let bpm = 120.0;
        let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.4, 0.45], None);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 3);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 3.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(4)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.08
            }));
    }

    #[test]
    fn beat_tracker_infers_four_four_after_two_beat_pickup() {
        let bpm = 120.0;
        let mut beats = vec![0.45, 0.7];
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
        beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);

        let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let beat_seconds = 60.0 / bpm;
        assert!((meter.downbeat_positions_seconds[0] - 2.0 * beat_seconds).abs() < 0.08);
    }

    #[test]
    fn beat_tracker_uses_spectral_change_to_support_weak_four_four_meter() {
        let bpm = 120.0;
        let beats = [
            0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4,
            0.35,
        ];
        let tone_events: &[(usize, &'static [f32], f32)] = &[
            (0, &[220.0, 277.18, 329.63], 0.85),
            (4, &[261.63, 329.63, 392.0], 0.85),
            (8, &[196.0, 246.94, 293.66], 0.85),
            (12, &[246.94, 311.13, 369.99], 0.85),
        ];
        let audio = beat_sequence_track(48_000, bpm, &beats, tone_events);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.18,
            "confidence {}",
            meter.confidence.0
        );
    }

    #[test]
    fn beat_tracker_suppresses_meter_on_mixed_bar_lengths() {
        let bpm = 120.0;
        let beats = [
            1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45,
        ];
        let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);

        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_handles_realistic_weak_backbeat_fixture() {
        let bpm = 118.0;
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 8,
            beat_pattern: [0.42, 0.24, 0.34, 0.22],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C, CHORD_D],
            chord_every_bars: 2,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });

        let audio = fixture.build(48_000, bpm);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert!(
            (result.bpm - bpm).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
    }

    #[test]
    fn beat_tracker_preserves_four_four_across_section_transition_fixture() {
        let bpm = 122.0;
        let mut fixture = FixtureBuilder::new();
        fixture.push_four_four_section(GrooveSection {
            bars: 4,
            beat_pattern: [0.48, 0.22, 0.36, 0.26],
            chord_cycle: &[CHORD_A, CHORD_B],
            chord_every_bars: 2,
            section_marker: Some((16, CHORD_C, 0.9)),
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        fixture.push_four_four_section(GrooveSection {
            bars: 4,
            beat_pattern: [0.55, 0.26, 0.38, 0.28],
            chord_cycle: &[CHORD_C, CHORD_D],
            chord_every_bars: 2,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });

        let audio = fixture.build(48_000, bpm);
        let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
        let result = tracker.analyze(&audio);
        let meter = result.meter.as_ref().expect("meter estimate");

        assert!(
            (result.bpm - bpm).abs() < 3.0,
            "detected bpm {}",
            result.bpm
        );
        assert_eq!(meter.beats_per_bar, 4);
        assert!(
            meter.confidence.0 > 0.2,
            "confidence {}",
            meter.confidence.0
        );
        let bar_seconds = 60.0 / bpm * 4.0;
        assert!(meter
            .downbeat_positions_seconds
            .iter()
            .take(6)
            .all(|downbeat| {
                let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
                (nearest_bar - *downbeat).abs() < 0.09
            }));
    }

    #[test]
    fn beat_tracker_calibrates_meter_confidence_between_neutral_and_structured_fixtures() {
        let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let structured_meter = structured.meter.as_ref().expect("structured meter");

        assert!(neutral.meter.is_none());
        assert_eq!(structured_meter.beats_per_bar, 4);
        assert!(structured_meter.confidence.0 > 0.2);
        assert_eq!(
            structured_meter.detection_kind,
            super::MeterDetectionKind::WholeTrack
        );
        assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
        assert!(structured_meter.recovery.is_none());
        assert!(structured_meter.confidence_breakdown.support > 0.6);
        assert!(
            structured_meter.support_profile.whole_track_strength.0
                > structured_meter.support_profile.segment_recovery_strength.0
        );
        assert_eq!(
            structured_meter
                .support_profile
                .recovery_duration_strength
                .0,
            0.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_tempo_ambiguity_between_stable_and_subdivided_fixtures() {
        let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

        assert!(ambiguous.tempo_ambiguity.0 > stable.tempo_ambiguity.0);
        assert!(ambiguous.tempo_candidates.len() >= 2);
        assert!(stable.confidence.0 >= ambiguous.confidence.0);
    }

    #[test]
    fn beat_tracker_calibrates_local_tempo_drift_between_stable_and_irregular_fixtures() {
        let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
        let slow = analyze_fixture(&click_track(48_000, 90.0, 8.0));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

        assert!(
            weak_backbeat.tempo_diagnostics.mean_abs_deviation_bpm
                > stable.tempo_diagnostics.mean_abs_deviation_bpm
        );
        assert!(
            section.tempo_diagnostics.drift_span_bpm >= stable.tempo_diagnostics.drift_span_bpm
        );
        assert!(!weak_backbeat.tempo_diagnostics.windowed_tempi.is_empty());
        assert!(!section.tempo_diagnostics.windowed_tempi.is_empty());
        assert!(stable.tempo_diagnostics.boundary_bias_bpm > 0.0);
        assert!(
            section.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
                >= stable.tempo_diagnostics.trend.fit_mean_abs_deviation_bpm
        );
        assert!(
            slow.tempo_diagnostics
                .beat_grid_error
                .edge_mean_abs_residual_ms
                > slow
                    .tempo_diagnostics
                    .beat_grid_error
                    .core_mean_abs_residual_ms
        );
        assert!(
            slow.tempo_diagnostics
                .beat_grid_error
                .mean_abs_anchored_drift_ms
                > stable
                    .tempo_diagnostics
                    .beat_grid_error
                    .mean_abs_anchored_drift_ms
        );
        assert_eq!(
            slow.tempo_interpretation.recommendation,
            super::TempoRecommendation::UseCoreWindow
        );
        assert_eq!(
            slow.tempo_interpretation.reason,
            super::TempoInterpretationReason::StableCoreWindow
        );
        assert!(
            (slow.tempo_interpretation.recommended_bpm - 90.0).abs() < 0.1,
            "slow recommended bpm {}",
            slow.tempo_interpretation.recommended_bpm
        );
        assert!(
            slow.tempo_interpretation.profile.boundary_edge_gap_ms > 0.0,
            "slow boundary edge gap {}",
            slow.tempo_interpretation.profile.boundary_edge_gap_ms
        );
        assert_eq!(slow.tempo_state.action, super::TempoStateAction::Monitor);
        assert_eq!(
            slow.tempo_state.reason,
            super::TempoStateReason::CoreWindowFallback
        );
        assert_eq!(
            slow.tempo_state.continuity.action,
            super::TempoContinuityAction::Retain
        );
        assert_eq!(
            slow.tempo_state.continuity.source,
            super::TempoContinuitySource::CoreWindow
        );
        assert_eq!(
            slow.tempo_state.continuity.reason,
            super::TempoContinuityReason::CoreWindowCarry
        );
        assert_eq!(
            slow.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
        );
        assert_eq!(
            slow.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            slow.tempo_state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(slow.tempo_state.continuity.expiry.guaranteed_until_beats, 8);
        assert_eq!(slow.tempo_state.continuity.expiry.downgrade_after_beats, 8);
        assert_eq!(slow.tempo_state.continuity.expiry.clear_after_beats, 12);
        assert_eq!(
            weak_backbeat.tempo_interpretation.recommendation,
            super::TempoRecommendation::UseRefined
        );
        assert_eq!(
            weak_backbeat.tempo_interpretation.reason,
            super::TempoInterpretationReason::StableRefinedPulse
        );
        assert_eq!(
            weak_backbeat.tempo_state.action,
            super::TempoStateAction::Lock
        );
        assert_eq!(
            weak_backbeat.tempo_state.reason,
            super::TempoStateReason::StableRefinedTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.action,
            super::TempoContinuityAction::Lock
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.reason,
            super::TempoContinuityReason::StableTempo
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::StableRefinedEstimate
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            weak_backbeat.tempo_state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            weak_backbeat
                .tempo_state
                .continuity
                .expiry
                .max_failed_revalidations,
            3
        );
        assert_eq!(
            ambiguous.tempo_interpretation.recommendation,
            super::TempoRecommendation::Defer
        );
        assert_eq!(
            ambiguous.tempo_interpretation.trust,
            super::TempoTrustLevel::Tentative
        );
        assert!(ambiguous.tempo_interpretation.profile.stability_score.0 < 0.6);
        assert_eq!(ambiguous.tempo_state.action, super::TempoStateAction::Defer);
        assert_eq!(
            ambiguous.tempo_state.reason,
            super::TempoStateReason::TempoDeferred
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.source,
            super::TempoContinuitySource::Cleared
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.reason,
            super::TempoContinuityReason::InsufficientEvidence
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            ambiguous.tempo_state.continuity.history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(
            ambiguous
                .tempo_state
                .continuity
                .expiry
                .max_failed_revalidations,
            0
        );
    }

    #[test]
    fn tempo_interpretation_prefers_refined_when_snap_benefit_is_too_small() {
        let diagnostics = synthetic_tempo_diagnostics(120.0, 0.02, 0.04, 0.03, 1.0, 0.8, 1.4, 1.1);
        let interpretation = super::interpret_tempo(
            120.01,
            super::Confidence::new(0.92),
            super::Confidence::new(0.08),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::UseRefined
        );
        assert_eq!(
            interpretation.reason,
            super::TempoInterpretationReason::StableRefinedPulse
        );
        assert!(interpretation.profile.snap_error_bpm < 0.04);
        assert!(interpretation.profile.stability_score.0 > 0.8);
    }

    #[test]
    fn tempo_interpretation_defers_when_edge_pressure_overwhelms_stability() {
        let diagnostics = synthetic_tempo_diagnostics(90.0, 0.95, 0.22, 0.16, 5.8, 2.1, 8.4, 5.2);
        let interpretation = super::interpret_tempo(
            89.93,
            super::Confidence::new(0.63),
            super::Confidence::new(0.18),
            &diagnostics,
        );

        assert_eq!(
            interpretation.recommendation,
            super::TempoRecommendation::Defer
        );
        assert_eq!(
            interpretation.reason,
            super::TempoInterpretationReason::UnstableTempo
        );
        assert_eq!(interpretation.trust, super::TempoTrustLevel::Tentative);
        assert!(interpretation.profile.boundary_edge_gap_ms > 2.5);
        assert!(interpretation.profile.stability_score.0 < 0.62);
    }

    #[test]
    fn tempo_state_locks_stable_integer_interpretation() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::SnapInteger,
            super::TempoTrustLevel::Stable,
            super::TempoInterpretationReason::NearIntegerPulse,
            120.0,
            Some(120.0),
            0.86,
            0.08,
            0.22,
            0.82,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );

        assert_eq!(state.action, super::TempoStateAction::Lock);
        assert_eq!(state.reason, super::TempoStateReason::StableIntegerTempo);
        assert!(state.confidence.0 >= 0.82);
        assert_eq!(state.continuity.action, super::TempoContinuityAction::Lock);
        assert_eq!(
            state.continuity.reason,
            super::TempoContinuityReason::IntegerTempoSnap
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::IntegerSnap
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 16);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert!(state.continuity.refresh_strength.0 > 0.9);
        assert_eq!(
            state.continuity.lifecycle.decay[1].action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.lifecycle.decay[1].history,
            super::TempoContinuityHistory::Degrading
        );
    }

    #[test]
    fn tempo_state_monitors_core_window_fallback() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::UseCoreWindow,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableCoreWindow,
            90.0,
            None,
            0.64,
            0.07,
            0.72,
            0.64,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );

        assert_eq!(state.action, super::TempoStateAction::Monitor);
        assert_eq!(state.reason, super::TempoStateReason::CoreWindowFallback);
        assert!(state.confidence.0 >= 0.58);
        assert_eq!(
            state.continuity.action,
            super::TempoContinuityAction::Retain
        );
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CoreWindow
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::CoreWindowEstimate
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 8);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert_eq!(
            state.continuity.lifecycle.decay[0].action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].provenance,
            super::TempoContinuityProvenance::PriorTempoCarry
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].history,
            super::TempoContinuityHistory::Degrading
        );
    }

    #[test]
    fn tempo_state_reacquires_guarded_refined_estimate_before_clearing() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::UseRefined,
            super::TempoTrustLevel::Guarded,
            super::TempoInterpretationReason::StableRefinedPulse,
            117.8,
            None,
            0.61,
            0.09,
            0.32,
            0.62,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );

        assert_eq!(state.action, super::TempoStateAction::Monitor);
        assert_eq!(state.reason, super::TempoStateReason::StableRefinedTempo);
        assert_eq!(
            state.continuity.action,
            super::TempoContinuityAction::Reacquire
        );
        assert_eq!(
            state.continuity.source,
            super::TempoContinuitySource::CurrentTempo
        );
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(state.continuity.expiry.guaranteed_until_beats, 4);
        assert_eq!(state.continuity.expiry.downgrade_after_beats, 8);
        assert_eq!(state.continuity.expiry.clear_after_beats, 12);
        assert_eq!(state.continuity.expiry.max_failed_revalidations, 3);
        assert_eq!(
            state.continuity.lifecycle.refresh.provenance,
            super::TempoContinuityProvenance::StableRefinedEstimate
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            state.continuity.lifecycle.decay[0].provenance,
            super::TempoContinuityProvenance::GuardedRefinedEstimate
        );
        assert!(
            state.continuity.lifecycle.refresh.refresh_strength.0
                > state.continuity.refresh_strength.0
        );
    }

    #[test]
    fn tempo_state_defers_unstable_interpretation() {
        let interpretation = synthetic_tempo_interpretation(
            super::TempoRecommendation::Defer,
            super::TempoTrustLevel::Tentative,
            super::TempoInterpretationReason::UnstableTempo,
            89.9,
            None,
            0.38,
            0.03,
            0.8,
            0.3,
        );
        let state = super::tempo_state_recommendation(
            interpretation,
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(state.action, super::TempoStateAction::Defer);
        assert_eq!(state.reason, super::TempoStateReason::TempoDeferred);
        assert!(state.confidence.0 > 0.4);
        assert_eq!(state.continuity.action, super::TempoContinuityAction::Clear);
        assert_eq!(
            state.continuity.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(state.continuity.expiry.clear_after_beats, 0);
        assert_eq!(
            state.continuity.lifecycle.refresh.action,
            super::TempoContinuityAction::Clear
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.provenance,
            super::TempoContinuityProvenance::NoTempo
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            state.continuity.lifecycle.refresh.history,
            super::TempoContinuityHistory::Degrading
        );
        assert_eq!(state.continuity.refresh_strength.0, 0.0);
    }

    #[test]
    fn tempo_continuity_calibrates_severity_history_and_refresh_strength() {
        let integer = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::SnapInteger,
                super::TempoTrustLevel::Stable,
                super::TempoInterpretationReason::NearIntegerPulse,
                120.0,
                Some(120.0),
                0.86,
                0.08,
                0.22,
                0.82,
            ),
            super::Confidence::new(0.9),
            super::Confidence::new(0.12),
        );
        let core_window = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseCoreWindow,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableCoreWindow,
                90.0,
                None,
                0.64,
                0.07,
                0.72,
                0.64,
            ),
            super::Confidence::new(0.72),
            super::Confidence::new(0.18),
        );
        let guarded_refined = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::UseRefined,
                super::TempoTrustLevel::Guarded,
                super::TempoInterpretationReason::StableRefinedPulse,
                117.8,
                None,
                0.61,
                0.09,
                0.32,
                0.62,
            ),
            super::Confidence::new(0.71),
            super::Confidence::new(0.21),
        );
        let deferred = super::tempo_state_recommendation(
            synthetic_tempo_interpretation(
                super::TempoRecommendation::Defer,
                super::TempoTrustLevel::Tentative,
                super::TempoInterpretationReason::UnstableTempo,
                89.9,
                None,
                0.38,
                0.03,
                0.8,
                0.3,
            ),
            super::Confidence::new(0.42),
            super::Confidence::new(0.55),
        );

        assert_eq!(
            integer.continuity.severity,
            super::TempoContinuitySeverity::Confirmed
        );
        assert_eq!(
            integer.continuity.history,
            super::TempoContinuityHistory::Reinforcing
        );
        assert_eq!(
            core_window.continuity.severity,
            super::TempoContinuitySeverity::Guarded
        );
        assert_eq!(
            core_window.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(
            guarded_refined.continuity.severity,
            super::TempoContinuitySeverity::Fragile
        );
        assert_eq!(
            guarded_refined.continuity.history,
            super::TempoContinuityHistory::Preserving
        );
        assert_eq!(
            deferred.continuity.severity,
            super::TempoContinuitySeverity::Cleared
        );
        assert_eq!(
            deferred.continuity.history,
            super::TempoContinuityHistory::Degrading
        );
        assert!(integer.continuity.refresh_strength.0 > core_window.continuity.refresh_strength.0);
        assert!(core_window.continuity.refresh_strength.0 > deferred.continuity.refresh_strength.0);
        assert!(
            guarded_refined
                .continuity
                .lifecycle
                .refresh
                .refresh_strength
                .0
                > guarded_refined.continuity.refresh_strength.0
        );
        assert_eq!(
            deferred.continuity.lifecycle.decay[1].refresh_strength.0,
            0.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_confidence_between_steady_and_dropout_sections() {
        let sample_rate = 48_000;
        let bpm = 120.0;

        let mut steady_fixture = FixtureBuilder::new();
        steady_fixture.push_four_four_section(GrooveSection {
            bars: 6,
            beat_pattern: [0.5, 0.26, 0.38, 0.24],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
            chord_every_bars: 1,
            section_marker: None,
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[],
        });
        let steady = analyze_fixture(&steady_fixture.build(sample_rate, bpm));

        let mut dropout_fixture = FixtureBuilder::new();
        dropout_fixture.push_four_four_section(GrooveSection {
            bars: 6,
            beat_pattern: [0.5, 0.26, 0.38, 0.24],
            chord_cycle: &[CHORD_A, CHORD_B, CHORD_C],
            chord_every_bars: 1,
            section_marker: Some((8, CHORD_D, 0.75)),
            bar_patterns: None,
            bar_chords: None,
            dropout_bars: &[2],
        });
        let dropout = analyze_fixture(&dropout_fixture.build(sample_rate, bpm));

        let steady_meter = steady.meter.as_ref().expect("steady meter");
        let dropout_meter = dropout.meter.as_ref().expect("dropout meter");
        assert_eq!(steady_meter.beats_per_bar, 4);
        assert_eq!(dropout_meter.beats_per_bar, 4);
        assert!(steady_meter.confidence.0 > dropout_meter.confidence.0);
    }

    #[test]
    fn beat_tracker_handles_fill_bar_with_harmonic_rhythm_changes() {
        let preset = RhythmPreset::FillTransition124(FillDensityVariant::Medium);
        let (bpm, result) = analyze_preset(preset);
        let meter = assert_meter(preset, &result, 4, 0.18);

        assert_detected_bpm(preset, &result, bpm, 3.0);
        assert!(result.confidence.0 > result.tempo_ambiguity.0);
        assert!(meter.downbeat_positions_seconds.len() >= 2);
    }

    #[test]
    fn beat_tracker_prefers_unknown_meter_for_dropout_heavy_transition_fixture() {
        let preset = RhythmPreset::Dropout120(DropoutVariant::Heavy);
        let (bpm, result) = analyze_preset(preset);
        assert_detected_bpm(preset, &result, bpm, 3.0);
        assert!(result.meter.is_none());
    }

    #[test]
    fn beat_tracker_matches_named_preset_surface_expectations() {
        let cases = [
            (RhythmPreset::NeutralClick120, 120.0, None, 0.85, 0.05),
            (
                RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Active),
                120.0,
                Some(4),
                0.75,
                0.25,
            ),
            (RhythmPreset::AmbiguousSubdivision90, 90.0, None, 0.45, 0.2),
            (
                RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Sparse),
                120.0,
                None,
                0.85,
                0.3,
            ),
            (RhythmPreset::WeakBackbeat118, 118.0, Some(4), 0.55, 0.15),
            (
                RhythmPreset::SectionTransition122,
                122.0,
                Some(4),
                0.55,
                0.1,
            ),
            (
                RhythmPreset::FillTransition124(FillDensityVariant::Medium),
                124.0,
                Some(4),
                0.55,
                0.1,
            ),
            (
                RhythmPreset::FillTransition124(FillDensityVariant::Dense),
                124.0,
                Some(4),
                0.5,
                0.12,
            ),
            (
                RhythmPreset::Dropout120(DropoutVariant::Light),
                120.0,
                None,
                0.85,
                0.05,
            ),
            (
                RhythmPreset::Dropout120(DropoutVariant::Medium),
                120.0,
                None,
                0.82,
                0.05,
            ),
            (
                RhythmPreset::Dropout120(DropoutVariant::Heavy),
                120.0,
                None,
                0.4,
                0.05,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::Pickup),
                120.0,
                Some(4),
                0.7,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::LateShift),
                120.0,
                Some(4),
                0.7,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::MixedLength),
                120.0,
                None,
                0.45,
                0.08,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::Modulation),
                120.0,
                None,
                0.45,
                0.08,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::Reentry),
                120.0,
                Some(4),
                0.65,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::CadentialElongation),
                120.0,
                None,
                0.45,
                0.08,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ReentryHarmonicShift),
                120.0,
                Some(4),
                0.65,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ReentryDenseFill),
                120.0,
                Some(4),
                0.6,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ReentryAcceleratingHarmony),
                120.0,
                Some(4),
                0.6,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ReentryDeceleratingHarmony),
                120.0,
                Some(4),
                0.6,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
                ),
                120.0,
                Some(4),
                0.58,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
                ),
                120.0,
                Some(4),
                0.58,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
                ),
                120.0,
                None,
                0.48,
                0.14,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
                ),
                120.0,
                None,
                0.48,
                0.14,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryAcceleratingHarmonyReset,
                ),
                120.0,
                None,
                0.54,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryDeceleratingHarmonyReset,
                ),
                120.0,
                None,
                0.54,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
                ),
                120.0,
                Some(4),
                0.56,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
                ),
                120.0,
                Some(4),
                0.56,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
                ),
                120.0,
                None,
                0.56,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(
                    BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
                ),
                120.0,
                None,
                0.56,
                0.12,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ModulationDenseFill),
                120.0,
                None,
                0.45,
                0.1,
            ),
            (
                RhythmPreset::BarTransition120(BarTransitionVariant::ModulationDenseFillExtended),
                120.0,
                None,
                0.42,
                0.12,
            ),
        ];

        for (preset, bpm, expected_meter, min_confidence, min_ambiguity) in cases {
            let (_, result) = analyze_preset(preset);
            assert_detected_bpm(preset, &result, bpm, 3.0);
            assert!(
                result.confidence.0 > min_confidence,
                "preset {:?} confidence {}",
                preset,
                result.confidence.0
            );
            assert!(
                result.tempo_ambiguity.0 >= min_ambiguity,
                "preset {:?} ambiguity {}",
                preset,
                result.tempo_ambiguity.0
            );

            match expected_meter {
                Some(beats_per_bar) => {
                    assert_meter(preset, &result, beats_per_bar, 0.18);
                }
                None => assert!(
                    result.meter.is_none(),
                    "preset {:?} should be meterless",
                    preset
                ),
            }
        }
    }

    #[test]
    fn beat_tracker_calibrates_named_preset_families() {
        let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, structured_sparse) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Sparse,
        ));
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, section) = analyze_preset(RhythmPreset::SectionTransition122);
        let (_, fill) = analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Medium));
        let (_, fill_dense) =
            analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Dense));
        let (_, dropout_light) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Light));
        let (_, dropout_medium) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Medium));
        let (_, dropout) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, late_shift) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::LateShift,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));
        let (_, modulation) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::Modulation,
        ));
        let (_, reentry) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::Reentry,
        ));
        let (_, cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::CadentialElongation,
        ));
        let (_, reentry_harmonic) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryHarmonicShift,
        ));
        let (_, reentry_fill) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDenseFill,
        ));
        let (_, reentry_accelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmony,
        ));
        let (_, reentry_decelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmony,
        ));
        let (_, reentry_accelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
        ));
        let (_, reentry_decelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
        ));
        let (_, reentry_accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
        ));
        let (_, reentry_decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
        ));
        let (_, reentry_accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));
        let (_, reentry_decelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyReset,
        ));
        let (_, reentry_accelerating_sustained_reset) =
            analyze_preset(RhythmPreset::BarTransition120(
                BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
            ));
        let (_, reentry_decelerating_sustained_reset) =
            analyze_preset(RhythmPreset::BarTransition120(
                BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
            ));
        let (_, reentry_accelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
        ));
        let (_, reentry_decelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
        ));
        let (_, modulation_fill) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFill,
        ));
        let (_, modulation_fill_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        let structured_meter = structured.meter.as_ref().expect("structured meter");
        let section_meter = section.meter.as_ref().expect("section meter");
        let fill_meter = fill.meter.as_ref().expect("fill meter");
        let fill_dense_meter = fill_dense.meter.as_ref().expect("dense fill meter");
        let pickup_meter = pickup.meter.as_ref().expect("pickup meter");
        let late_shift_meter = late_shift.meter.as_ref().expect("late-shift meter");
        let reentry_meter = reentry.meter.as_ref().expect("reentry meter");
        let reentry_harmonic_meter = reentry_harmonic
            .meter
            .as_ref()
            .expect("reentry harmonic meter");
        let reentry_fill_meter = reentry_fill.meter.as_ref().expect("reentry fill meter");
        let reentry_accelerating_meter = reentry_accelerating
            .meter
            .as_ref()
            .expect("reentry accelerating meter");
        let reentry_decelerating_meter = reentry_decelerating
            .meter
            .as_ref()
            .expect("reentry decelerating meter");
        let reentry_accelerating_dense_meter = reentry_accelerating_dense
            .meter
            .as_ref()
            .expect("reentry accelerating dense meter");
        let reentry_decelerating_dense_meter = reentry_decelerating_dense
            .meter
            .as_ref()
            .expect("reentry decelerating dense meter");
        let reentry_accelerating_sustained_reset_meter = reentry_accelerating_sustained_reset
            .meter
            .as_ref()
            .expect("reentry accelerating sustained reset meter");
        let reentry_decelerating_sustained_reset_meter = reentry_decelerating_sustained_reset
            .meter
            .as_ref()
            .expect("reentry decelerating sustained reset meter");
        assert!(neutral.meter.is_none());
        assert!(structured_sparse.meter.is_none());
        assert!(dropout_light.meter.is_none());
        assert!(dropout_medium.meter.is_none());
        assert!(dropout.meter.is_none());
        assert!(mixed_length.meter.is_none());
        assert!(modulation.meter.is_none());
        assert!(cadential.meter.is_none());
        assert!(reentry_accelerating_accent.meter.is_none());
        assert!(reentry_decelerating_accent.meter.is_none());
        assert!(reentry_accelerating_reset.meter.is_none());
        assert!(reentry_decelerating_reset.meter.is_none());
        assert!(reentry_accelerating_cadential.meter.is_none());
        assert!(reentry_decelerating_cadential.meter.is_none());
        assert!(modulation_fill.meter.is_none());
        assert!(modulation_fill_extended.meter.is_none());
        assert!(ambiguous.tempo_ambiguity.0 > neutral.tempo_ambiguity.0);
        assert!(ambiguous.tempo_ambiguity.0 > fill.tempo_ambiguity.0);
        assert!(structured_meter.confidence.0 > 0.2);
        assert_eq!(
            structured_meter.detection_kind,
            super::MeterDetectionKind::WholeTrack
        );
        assert!(structured_meter.recovery.is_none());
        assert!(
            structured_meter.support_profile.whole_track_strength.0
                > structured_meter.support_profile.segment_recovery_strength.0
        );
        assert!(structured.confidence.0 >= structured_sparse.confidence.0 - 0.05);
        assert!(section_meter.confidence.0 > 0.2);
        assert!(fill_meter.confidence.0 > 0.18);
        assert!(fill_dense_meter.confidence.0 > 0.18);
        assert!(fill_dense.tempo_ambiguity.0 > fill.tempo_ambiguity.0);
        assert!(dropout_light.confidence.0 > dropout_medium.confidence.0);
        assert!(dropout.confidence.0 > 0.6);
        assert_eq!(pickup_meter.beats_per_bar, 4);
        assert_eq!(late_shift_meter.beats_per_bar, 4);
        assert_eq!(reentry_meter.beats_per_bar, 4);
        assert_eq!(reentry_harmonic_meter.beats_per_bar, 4);
        assert_eq!(reentry_fill_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_dense_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_dense_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_sustained_reset_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_sustained_reset_meter.beats_per_bar, 4);
        assert!(pickup_meter.confidence.0 >= late_shift_meter.confidence.0 - 0.1);
        assert!(reentry_meter.confidence.0 > 0.18);
        assert!(reentry_harmonic_meter.confidence.0 > 0.18);
        assert!(reentry_fill_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_dense_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_dense_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_sustained_reset_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_sustained_reset_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_cadential.confidence.0 > 0.18);
        assert!(reentry_decelerating_cadential.confidence.0 > 0.18);
        assert!(late_shift.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
        assert!(mixed_length.confidence.0 < pickup.confidence.0);
        assert!(reentry.confidence.0 > modulation.confidence.0);
        assert!(reentry_accelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(reentry_decelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(
            reentry_accelerating_dense.confidence.0 > reentry_accelerating_dense.tempo_ambiguity.0
        );
        assert!(
            reentry_decelerating_dense.confidence.0 > reentry_decelerating_dense.tempo_ambiguity.0
        );
        assert!(
            reentry_accelerating_accent.tempo_ambiguity.0
                >= reentry_accelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(
            reentry_decelerating_accent.tempo_ambiguity.0
                >= reentry_decelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(reentry_accelerating_reset.confidence.0 > 0.18);
        assert!(reentry_decelerating_reset.confidence.0 > 0.18);
        assert!(
            reentry_accelerating_sustained_reset.confidence.0
                >= reentry_accelerating_reset.confidence.0 - 0.03
        );
        assert!(
            reentry_decelerating_sustained_reset.confidence.0
                >= reentry_decelerating_reset.confidence.0 - 0.03
        );
        assert!(
            reentry_accelerating_cadential.confidence.0
                >= reentry_accelerating_reset.confidence.0 - 0.05
        );
        assert!(
            reentry_decelerating_cadential.confidence.0
                >= reentry_decelerating_reset.confidence.0 - 0.05
        );
        assert!(modulation_fill.tempo_ambiguity.0 >= reentry_harmonic.tempo_ambiguity.0);
        assert!(modulation_fill.tempo_ambiguity.0 >= reentry_fill.tempo_ambiguity.0 - 0.05);
        assert!(modulation_fill_extended.tempo_ambiguity.0 >= modulation_fill.tempo_ambiguity.0);
        assert!(cadential.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.05);
        assert!(section.confidence.0 > section.tempo_ambiguity.0);
        assert!(fill.confidence.0 > fill.tempo_ambiguity.0);
        assert!(section.confidence.0 > ambiguous.confidence.0 - 0.1);
    }

    #[test]
    fn beat_tracker_calibrates_dropout_variant_monotonicity() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, light) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Light));
        let (_, medium) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Medium));
        let (_, heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));

        let structured_meter = structured.meter.as_ref().expect("structured meter");

        assert_eq!(structured_meter.beats_per_bar, 4);
        assert!(light.meter.is_none());
        assert!(medium.meter.is_none());
        assert!(heavy.meter.is_none());
        assert!(light.confidence.0 > medium.confidence.0);
        assert!(heavy.confidence.0 > 0.6);
        assert!(structured.confidence.0 >= light.confidence.0 - 0.05);
    }

    #[test]
    fn beat_tracker_calibrates_fill_density_variant_monotonicity() {
        let (_, medium) =
            analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Medium));
        let (_, dense) = analyze_preset(RhythmPreset::FillTransition124(FillDensityVariant::Dense));

        let medium_meter = medium.meter.as_ref().expect("medium fill meter");
        let dense_meter = dense.meter.as_ref().expect("dense fill meter");

        assert_eq!(medium_meter.beats_per_bar, 4);
        assert_eq!(dense_meter.beats_per_bar, 4);
        assert!(dense.tempo_ambiguity.0 > medium.tempo_ambiguity.0);
        assert!(medium_meter.confidence.0 >= dense_meter.confidence.0 - 0.12);
        assert!(dense.confidence.0 > dense.tempo_ambiguity.0);
    }

    #[test]
    fn beat_tracker_calibrates_harmonic_rhythm_variant_monotonicity() {
        let (_, sparse) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Sparse,
        ));
        let (_, active) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));

        let active_meter = active.meter.as_ref().expect("active harmonic meter");

        assert!(sparse.meter.is_none());
        assert_eq!(active_meter.beats_per_bar, 4);
        assert!(active.confidence.0 >= sparse.confidence.0 - 0.05);
        assert!(active.tempo_ambiguity.0 <= sparse.tempo_ambiguity.0 + 0.1);
    }

    #[test]
    fn beat_tracker_calibrates_bar_transition_variant_monotonicity() {
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, late_shift) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::LateShift,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));
        let (_, modulation) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::Modulation,
        ));
        let (_, reentry) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::Reentry,
        ));
        let (_, cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::CadentialElongation,
        ));
        let (_, reentry_harmonic) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryHarmonicShift,
        ));
        let (_, reentry_fill) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDenseFill,
        ));
        let (_, reentry_accelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmony,
        ));
        let (_, reentry_decelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmony,
        ));
        let (_, reentry_accelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
        ));
        let (_, reentry_decelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
        ));
        let (_, reentry_accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
        ));
        let (_, reentry_decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
        ));
        let (_, reentry_accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));
        let (_, reentry_decelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyReset,
        ));
        let (_, reentry_accelerating_sustained_reset) =
            analyze_preset(RhythmPreset::BarTransition120(
                BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
            ));
        let (_, reentry_decelerating_sustained_reset) =
            analyze_preset(RhythmPreset::BarTransition120(
                BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
            ));
        let (_, reentry_accelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
        ));
        let (_, reentry_decelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
        ));
        let (_, modulation_fill) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFill,
        ));
        let (_, modulation_fill_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        let pickup_meter = pickup.meter.as_ref().expect("pickup meter");
        let late_shift_meter = late_shift.meter.as_ref().expect("late-shift meter");
        let reentry_meter = reentry.meter.as_ref().expect("reentry meter");
        let reentry_harmonic_meter = reentry_harmonic
            .meter
            .as_ref()
            .expect("reentry harmonic meter");
        let reentry_fill_meter = reentry_fill.meter.as_ref().expect("reentry fill meter");
        let reentry_accelerating_meter = reentry_accelerating
            .meter
            .as_ref()
            .expect("reentry accelerating meter");
        let reentry_decelerating_meter = reentry_decelerating
            .meter
            .as_ref()
            .expect("reentry decelerating meter");
        let reentry_accelerating_dense_meter = reentry_accelerating_dense
            .meter
            .as_ref()
            .expect("reentry accelerating dense meter");
        let reentry_decelerating_dense_meter = reentry_decelerating_dense
            .meter
            .as_ref()
            .expect("reentry decelerating dense meter");
        let reentry_accelerating_sustained_reset_meter = reentry_accelerating_sustained_reset
            .meter
            .as_ref()
            .expect("reentry accelerating sustained reset meter");
        let reentry_decelerating_sustained_reset_meter = reentry_decelerating_sustained_reset
            .meter
            .as_ref()
            .expect("reentry decelerating sustained reset meter");
        assert_eq!(pickup_meter.beats_per_bar, 4);
        assert_eq!(late_shift_meter.beats_per_bar, 4);
        assert_eq!(reentry_meter.beats_per_bar, 4);
        assert_eq!(reentry_harmonic_meter.beats_per_bar, 4);
        assert_eq!(reentry_fill_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_dense_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_dense_meter.beats_per_bar, 4);
        assert_eq!(reentry_accelerating_sustained_reset_meter.beats_per_bar, 4);
        assert_eq!(reentry_decelerating_sustained_reset_meter.beats_per_bar, 4);
        assert!(mixed_length.meter.is_none());
        assert!(modulation.meter.is_none());
        assert!(cadential.meter.is_none());
        assert!(reentry_accelerating_accent.meter.is_none());
        assert!(reentry_decelerating_accent.meter.is_none());
        assert!(reentry_accelerating_reset.meter.is_none());
        assert!(reentry_decelerating_reset.meter.is_none());
        assert!(reentry_accelerating_cadential.meter.is_none());
        assert!(reentry_decelerating_cadential.meter.is_none());
        assert!(modulation_fill.meter.is_none());
        assert!(modulation_fill_extended.meter.is_none());
        assert!(pickup_meter.confidence.0 > 0.2);
        assert!(late_shift_meter.confidence.0 > 0.18);
        assert!(reentry_meter.confidence.0 > 0.18);
        assert!(reentry_harmonic_meter.confidence.0 > 0.18);
        assert!(reentry_fill_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_dense_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_dense_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_sustained_reset_meter.confidence.0 > 0.18);
        assert!(reentry_decelerating_sustained_reset_meter.confidence.0 > 0.18);
        assert!(reentry_accelerating_cadential.confidence.0 > 0.18);
        assert!(reentry_decelerating_cadential.confidence.0 > 0.18);
        assert!(pickup_meter.confidence.0 >= late_shift_meter.confidence.0 - 0.12);
        assert!(late_shift.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
        assert!(pickup.confidence.0 > mixed_length.confidence.0);
        assert!(reentry.confidence.0 > modulation.confidence.0);
        assert!(reentry_accelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(reentry_decelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(
            reentry_accelerating_dense.confidence.0 > reentry_accelerating_dense.tempo_ambiguity.0
        );
        assert!(
            reentry_decelerating_dense.confidence.0 > reentry_decelerating_dense.tempo_ambiguity.0
        );
        assert!(
            reentry_accelerating_accent.tempo_ambiguity.0
                >= reentry_accelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(
            reentry_decelerating_accent.tempo_ambiguity.0
                >= reentry_decelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(reentry_accelerating_reset.confidence.0 > 0.18);
        assert!(reentry_decelerating_reset.confidence.0 > 0.18);
        assert!(
            reentry_accelerating_sustained_reset.confidence.0
                >= reentry_accelerating_reset.confidence.0 - 0.03
        );
        assert!(
            reentry_decelerating_sustained_reset.confidence.0
                >= reentry_decelerating_reset.confidence.0 - 0.03
        );
        assert!(
            reentry_accelerating_cadential.confidence.0
                >= reentry_accelerating_reset.confidence.0 - 0.05
        );
        assert!(
            reentry_decelerating_cadential.confidence.0
                >= reentry_decelerating_reset.confidence.0 - 0.05
        );
        assert!(modulation_fill.tempo_ambiguity.0 >= reentry_harmonic.tempo_ambiguity.0);
        assert!(modulation_fill.tempo_ambiguity.0 >= reentry_fill.tempo_ambiguity.0 - 0.05);
        assert!(modulation_fill_extended.tempo_ambiguity.0 >= modulation_fill.tempo_ambiguity.0);
        assert!(cadential.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.05);
        assert!(modulation.tempo_ambiguity.0 >= pickup.tempo_ambiguity.0);
    }

    #[test]
    fn beat_tracker_calibrates_multi_stage_reentry_harmonic_drift() {
        let (_, reentry) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::Reentry,
        ));
        let (_, accelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmony,
        ));
        let (_, decelerating) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmony,
        ));

        let accelerating_meter = accelerating
            .meter
            .as_ref()
            .expect("accelerating recovery meter");
        let decelerating_meter = decelerating
            .meter
            .as_ref()
            .expect("decelerating recovery meter");

        assert_eq!(accelerating_meter.beats_per_bar, 4);
        assert_eq!(decelerating_meter.beats_per_bar, 4);
        assert!(accelerating_meter.confidence.0 > 0.18);
        assert!(decelerating_meter.confidence.0 > 0.18);
        assert!(accelerating.confidence.0 > accelerating.tempo_ambiguity.0);
        assert!(decelerating.confidence.0 > decelerating.tempo_ambiguity.0);
        assert!(accelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(decelerating.tempo_ambiguity.0 >= reentry.tempo_ambiguity.0 - 0.03);
        assert!(
            decelerating_meter.confidence.0 >= accelerating_meter.confidence.0 - 0.12,
            "decelerating confidence {} accelerating {}",
            decelerating_meter.confidence.0,
            accelerating_meter.confidence.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_multistage_reentry_density_vs_accent_drift() {
        let (_, accelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyDenseFill,
        ));
        let (_, decelerating_dense) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyDenseFill,
        ));
        let (_, accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
        ));
        let (_, decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
        ));

        let accelerating_dense_meter = accelerating_dense
            .meter
            .as_ref()
            .expect("accelerating dense recovery meter");
        let decelerating_dense_meter = decelerating_dense
            .meter
            .as_ref()
            .expect("decelerating dense recovery meter");

        assert_eq!(accelerating_dense_meter.beats_per_bar, 4);
        assert_eq!(decelerating_dense_meter.beats_per_bar, 4);
        assert!(accelerating_accent.meter.is_none());
        assert!(decelerating_accent.meter.is_none());
        assert!(accelerating_dense.tempo_ambiguity.0 > 0.12);
        assert!(decelerating_dense.tempo_ambiguity.0 > 0.12);
        assert!(
            accelerating_accent.tempo_ambiguity.0 >= accelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(
            decelerating_accent.tempo_ambiguity.0 >= decelerating_dense.tempo_ambiguity.0 - 0.03
        );
        assert!(accelerating_dense.confidence.0 >= accelerating_accent.confidence.0 - 0.05);
        assert!(decelerating_dense.confidence.0 >= decelerating_accent.confidence.0 - 0.05);
    }

    #[test]
    fn beat_tracker_calibrates_reanchor_recovery_after_destabilized_window() {
        let (_, accelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyAccentShift,
        ));
        let (_, decelerating_accent) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyAccentShift,
        ));
        let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));
        let (_, decelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyReset,
        ));
        let (_, accelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, decelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
        ));
        let (_, accelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
        ));
        let (_, decelerating_cadential) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonyCadentialReanchor,
        ));

        assert!(accelerating_accent.meter.is_none());
        assert!(decelerating_accent.meter.is_none());
        assert!(accelerating_reset.meter.is_none());
        assert!(decelerating_reset.meter.is_none());
        assert!(accelerating_cadential.meter.is_none());
        assert!(decelerating_cadential.meter.is_none());
        assert_eq!(
            accelerating_sustained_reset
                .meter
                .as_ref()
                .expect("accelerating sustained reset meter")
                .beats_per_bar,
            4
        );
        assert_eq!(
            decelerating_sustained_reset
                .meter
                .as_ref()
                .expect("decelerating sustained reset meter")
                .beats_per_bar,
            4
        );
        assert!(accelerating_reset.confidence.0 > 0.18);
        assert!(decelerating_reset.confidence.0 > 0.18);
        assert!(
            accelerating_sustained_reset.confidence.0 >= accelerating_reset.confidence.0 - 0.03
        );
        assert!(
            decelerating_sustained_reset.confidence.0 >= decelerating_reset.confidence.0 - 0.03
        );
        assert!(accelerating_cadential.confidence.0 >= accelerating_reset.confidence.0 - 0.05);
        assert!(decelerating_cadential.confidence.0 >= decelerating_reset.confidence.0 - 0.05);
        assert!(accelerating_cadential.confidence.0 > 0.18);
        assert!(decelerating_cadential.confidence.0 > 0.18);
    }

    #[test]
    fn beat_tracker_calibrates_sustained_segment_recovery_vs_prolonged_modulation() {
        let (_, accelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, decelerating_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryDeceleratingHarmonySustainedReset,
        ));
        let (_, prolonged_modulation) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        let accelerating_meter = accelerating_sustained_reset
            .meter
            .as_ref()
            .expect("accelerating sustained recovery meter");
        let decelerating_meter = decelerating_sustained_reset
            .meter
            .as_ref()
            .expect("decelerating sustained recovery meter");

        assert_eq!(accelerating_meter.beats_per_bar, 4);
        assert_eq!(decelerating_meter.beats_per_bar, 4);
        assert!(accelerating_meter.confidence.0 > 0.18);
        assert!(decelerating_meter.confidence.0 > 0.18);
        assert_eq!(
            accelerating_meter.detection_kind,
            super::MeterDetectionKind::SegmentRecovery
        );
        assert_eq!(accelerating_meter.trust, super::MeterTrustLevel::Recovering);
        assert_eq!(
            decelerating_meter.detection_kind,
            super::MeterDetectionKind::SegmentRecovery
        );
        assert_eq!(decelerating_meter.trust, super::MeterTrustLevel::Recovering);
        let accelerating_recovery = accelerating_meter
            .recovery
            .as_ref()
            .expect("accelerating recovery context");
        let decelerating_recovery = decelerating_meter
            .recovery
            .as_ref()
            .expect("decelerating recovery context");
        assert!(accelerating_recovery.recovered_beats >= 8);
        assert!(decelerating_recovery.recovered_beats >= 8);
        assert!(accelerating_recovery.supporting_windows >= 2);
        assert!(decelerating_recovery.supporting_windows >= 2);
        assert!(accelerating_recovery.end_seconds > accelerating_recovery.start_seconds);
        assert!(decelerating_recovery.end_seconds > decelerating_recovery.start_seconds);
        assert!(
            accelerating_meter
                .support_profile
                .segment_recovery_strength
                .0
                > accelerating_meter.support_profile.whole_track_strength.0
        );
        assert!(
            decelerating_meter
                .support_profile
                .segment_recovery_strength
                .0
                > decelerating_meter.support_profile.whole_track_strength.0
        );
        assert!(
            accelerating_meter
                .support_profile
                .recovery_duration_strength
                .0
                > 0.5
        );
        assert!(
            decelerating_meter
                .support_profile
                .recovery_duration_strength
                .0
                > 0.5
        );
        assert!(prolonged_modulation.meter.is_none());
        assert!(
            prolonged_modulation.tempo_ambiguity.0
                >= accelerating_sustained_reset.tempo_ambiguity.0 - 0.02
        );
        assert!(
            prolonged_modulation.tempo_ambiguity.0
                >= decelerating_sustained_reset.tempo_ambiguity.0 - 0.02
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_trust_levels_across_public_categories() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));

        let structured_meter = structured.meter.as_ref().expect("structured meter");
        let weak_backbeat_meter = weak_backbeat.meter.as_ref().expect("weak backbeat meter");
        let sustained_reset_meter = sustained_reset
            .meter
            .as_ref()
            .expect("sustained reset meter");

        assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
        assert_eq!(
            structured_meter.recommendation,
            super::MeterRecommendation::Lock
        );
        assert_eq!(structured.meter_state.action, super::MeterStateAction::Lock);
        assert_eq!(
            structured.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[1].action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(weak_backbeat_meter.trust, super::MeterTrustLevel::Tentative);
        assert_eq!(
            weak_backbeat_meter.recommendation,
            super::MeterRecommendation::Defer
        );
        assert_eq!(
            weak_backbeat.meter_state.action,
            super::MeterStateAction::Hold
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            sustained_reset_meter.trust,
            super::MeterTrustLevel::Recovering
        );
        assert_eq!(
            sustained_reset_meter.recommendation,
            super::MeterRecommendation::Monitor
        );
        assert_eq!(
            sustained_reset.meter_state.action,
            super::MeterStateAction::Watch
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert!(
            structured_meter.support_profile.whole_track_strength.0
                >= weak_backbeat_meter.support_profile.whole_track_strength.0
        );
        assert!(
            sustained_reset_meter
                .support_profile
                .segment_recovery_strength
                .0
                > sustained_reset_meter.support_profile.whole_track_strength.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_recommendations_across_action_categories() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));

        let structured_meter = structured.meter.as_ref().expect("structured meter");
        let weak_backbeat_meter = weak_backbeat.meter.as_ref().expect("weak backbeat meter");
        let sustained_reset_meter = sustained_reset
            .meter
            .as_ref()
            .expect("sustained reset meter");

        assert_eq!(
            structured_meter.recommendation,
            super::MeterRecommendation::Lock
        );
        assert_eq!(
            structured.meter_state.reason,
            super::MeterStateReason::StableMeter
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset_meter.recommendation,
            super::MeterRecommendation::Monitor
        );
        assert_eq!(
            sustained_reset.meter_state.reason,
            super::MeterStateReason::RecoveringMeter
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            weak_backbeat_meter.recommendation,
            super::MeterRecommendation::Defer
        );
        assert_eq!(
            weak_backbeat.meter_state.reason,
            super::MeterStateReason::TentativeMeter
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert!(structured_meter.confidence.0 > weak_backbeat_meter.confidence.0);
        assert!(
            sustained_reset_meter
                .support_profile
                .recovery_duration_strength
                .0
                > 0.5
        );
        assert!(weak_backbeat_meter.recovery.is_none());
    }

    #[test]
    fn beat_tracker_calibrates_transition_meter_state_actions_for_meterless_cases() {
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, accelerating_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert!(dropout_heavy.meter.is_none());
        assert!(accelerating_reset.meter.is_none());
        assert!(modulation_extended.meter.is_none());

        assert_eq!(
            dropout_heavy.meter_state.action,
            super::MeterStateAction::Hold
        );
        assert_eq!(
            dropout_heavy.meter_state.reason,
            super::MeterStateReason::DestabilizedHold
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::PriorMeter
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            accelerating_reset.meter_state.action,
            super::MeterStateAction::Watch
        );
        assert_eq!(
            accelerating_reset.meter_state.reason,
            super::MeterStateReason::RecoveryEmerging
        );
        assert_eq!(
            accelerating_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            accelerating_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            accelerating_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.action,
            super::MeterStateAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.reason,
            super::MeterStateReason::MeterCleared
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::Cleared
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert!(
            dropout_heavy.meter_state.confidence.0
                >= modulation_extended.meter_state.confidence.0 - 0.05
        );
        assert!(
            accelerating_reset.meter_state.confidence.0
                > modulation_extended.meter_state.confidence.0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_across_transition_families() {
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, cadential_reanchor) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyCadentialReanchor,
        ));

        assert!(pickup.meter.is_some());
        assert!(mixed_length.meter.is_none());
        assert!(sustained_reset.meter.is_some());
        assert!(cadential_reanchor.meter.is_none());

        assert_eq!(
            pickup.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            pickup
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            mixed_length.meter_state.continuity.downbeat_phase.action,
            super::MeterContinuityAction::Clear
        );
        assert_eq!(
            cadential_reanchor.meter_state.continuity.bar_length.action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            cadential_reanchor
                .meter_state
                .continuity
                .downbeat_phase
                .action,
            super::MeterContinuityAction::Reacquire
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_provenance_and_expiry_windows() {
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));

        assert!(dropout_heavy.meter.is_none());
        assert!(dropout_extended.meter.is_none());
        assert!(pickup.meter.is_some());
        assert!(pickup_extended.meter.is_some());
        assert!(sustained_reset.meter.is_some());
        assert!(long_sustained_reset.meter.is_some());

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::PriorMeter
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Retain
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .trusted_beats
                <= dropout_heavy
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .after_beats
                >= dropout_extended
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trusted_beats
                <= dropout_heavy
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .trusted_beats
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .source,
            super::MeterContinuitySource::RecoveryWindow
        );

        assert_eq!(
            pickup.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.source,
            super::MeterContinuitySource::CurrentMeter
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.trusted_beats,
            0
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trusted_beats,
            0
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .revalidate_after_beats
                <= pickup
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .revalidate_after_beats
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .action,
            super::MeterContinuityAction::Lock
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .action,
            super::MeterContinuityAction::Reacquire
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .source,
            super::MeterContinuitySource::RecoveryWindow
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .trusted_beats
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .trusted_beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .revalidate_after_beats
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .revalidate_after_beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .after_beats
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[1]
                    .after_beats
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_severity_across_lifecycle_stages() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Confirmed
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[1].severity,
            super::MeterContinuitySeverity::Cleared
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .severity,
            super::MeterContinuitySeverity::Cleared
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            dropout_heavy.meter_state.continuity.downbeat_phase.severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .downbeat_phase
                .severity,
            super::MeterContinuitySeverity::Fragile
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .severity,
            super::MeterContinuitySeverity::Confirmed
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .severity,
            super::MeterContinuitySeverity::Guarded
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .severity,
            super::MeterContinuitySeverity::Fragile
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.severity,
            super::MeterContinuitySeverity::Cleared
        );
        assert_eq!(
            mixed_length
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .severity,
            super::MeterContinuitySeverity::Cleared
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_reason_and_confidence_surface() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::StableEvidence
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );
        assert_eq!(
            structured.meter_state.continuity.bar_length.lifecycle.decay[0].reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::TentativeEvidence
        );
        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::PriorStateCarry
        );
        assert_eq!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .reason,
            super::MeterContinuityReason::RevalidationDecay
        );

        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::RecoveryWindowSupport
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::RecoveryWindowSupport
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );

        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.reason,
            super::MeterContinuityReason::PhaseDisplacement
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .reason,
            super::MeterContinuityReason::StableEvidence
        );

        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.reason,
            super::MeterContinuityReason::InsufficientEvidence
        );
        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.confidence.0,
            0.0
        );

        assert!(
            structured.meter_state.continuity.bar_length.confidence.0
                > weak_backbeat.meter_state.continuity.bar_length.confidence.0
        );
        assert!(
            weak_backbeat.meter_state.continuity.bar_length.confidence.0
                > weak_backbeat
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .confidence
                    .0
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .confidence
                .0
                > dropout_heavy.meter_state.continuity.bar_length.confidence.0
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .refresh
                .confidence
                .0
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .confidence
                    .0
        );
        assert!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .confidence
                .0
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[0]
                    .confidence
                    .0
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .confidence
                .0
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .confidence
                    .0
        );
        assert!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .confidence
                .0
                > dropout_heavy
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[1]
                    .confidence
                    .0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_triggers_and_unresolved_spans() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, pickup) =
            analyze_preset(RhythmPreset::BarTransition120(BarTransitionVariant::Pickup));
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_heavy) = analyze_preset(RhythmPreset::Dropout120(DropoutVariant::Heavy));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, mixed_length) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::MixedLength,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::StableRevalidation
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations,
            0
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats,
            0
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::TentativeCarry
        );
        assert!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= 1
        );

        assert_eq!(
            pickup.meter_state.continuity.downbeat_phase.trigger,
            super::MeterContinuityTrigger::PhaseRecovery
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .trigger,
            super::MeterContinuityTrigger::PhaseRecovery
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .unresolved
                .beats
                > pickup
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .unresolved
                    .beats
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .unresolved
                .failed_revalidations
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .lifecycle
                    .decay[0]
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            dropout_heavy.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::PriorStateDrift
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert!(
            dropout_heavy
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= 1
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[1]
                .unresolved
                .failed_revalidations
                > dropout_extended
                    .meter_state
                    .continuity
                    .bar_length
                    .lifecycle
                    .decay[0]
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .trigger,
            super::MeterContinuityTrigger::RecoveryWindowDrift
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .unresolved
                    .beats
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .failed_revalidations
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .unresolved
                    .failed_revalidations
        );

        assert_eq!(
            mixed_length.meter_state.continuity.bar_length.trigger,
            super::MeterContinuityTrigger::EvidenceLoss
        );
        assert_eq!(
            mixed_length
                .meter_state
                .continuity
                .bar_length
                .unresolved
                .beats,
            0
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_cause_stacks_for_stacked_instability() {
        let contains_cause = |stack: super::MeterContinuityCauseStack,
                              cause: super::MeterContinuityCause| {
            stack.primary == cause
                || stack
                    .secondary
                    .into_iter()
                    .flatten()
                    .any(|entry| entry == cause)
        };

        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(contains_cause(
            ambiguous.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::SparseMeterSupport,
        ));
        assert!(ambiguous.meter_state.continuity.bar_length.causes.count >= 2);

        assert!(contains_cause(
            pickup_extended.meter_state.continuity.downbeat_phase.causes,
            super::MeterContinuityCause::PhaseDisplacement,
        ));
        assert!(contains_cause(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .causes
                .count
                >= 2
        );

        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::RecoveryWindowInstability,
        ));
        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(contains_cause(
            dropout_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::IrregularBarStructure,
        ));
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .causes
                .count
                >= 2
        );

        assert!(contains_cause(
            modulation_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::EvidenceLoss,
        ));
        assert!(contains_cause(
            modulation_extended.meter_state.continuity.bar_length.causes,
            super::MeterContinuityCause::TempoAmbiguity,
        ));
        assert!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .causes
                .count
                >= 2
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_history_across_transition_families() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Reinforcing
        );
        assert_eq!(
            structured
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );

        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.downbeat_phase.history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .history,
            super::MeterContinuityHistory::Degrading
        );
        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .lifecycle
                .decay[1]
                .history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .decay[0]
                .history,
            super::MeterContinuityHistory::Degrading
        );

        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .history,
            super::MeterContinuityHistory::Preserving
        );
        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .lifecycle
                .refresh
                .history,
            super::MeterContinuityHistory::Reinforcing
        );

        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .history,
            super::MeterContinuityHistory::Degrading
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_arcs_across_transition_families() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            weak_backbeat.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Stalling
        );
        assert_eq!(
            ambiguous.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Collapsing
        );
        assert_eq!(
            pickup_extended.meter_state.continuity.downbeat_phase.arc,
            super::MeterContinuityArc::Collapsing
        );
        assert_eq!(
            dropout_extended.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Stalling
        );
        assert_eq!(
            sustained_reset.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            long_sustained_reset.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Recovering
        );
        assert_eq!(
            modulation_extended.meter_state.continuity.bar_length.arc,
            super::MeterContinuityArc::Collapsing
        );
    }

    #[test]
    fn beat_tracker_calibrates_meter_continuity_arc_rationales_and_support() {
        let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
            HarmonicRhythmVariant::Active,
        ));
        let (_, weak_backbeat) = analyze_preset(RhythmPreset::WeakBackbeat118);
        let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);
        let (_, pickup_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::PickupExtended,
        ));
        let (_, dropout_extended) =
            analyze_preset(RhythmPreset::Dropout120(DropoutVariant::ExtendedHeavy));
        let (_, sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonySustainedReset,
        ));
        let (_, long_sustained_reset) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset,
        ));
        let (_, modulation_extended) = analyze_preset(RhythmPreset::BarTransition120(
            BarTransitionVariant::ModulationDenseFillExtended,
        ));

        assert_eq!(
            structured.meter_state.continuity.bar_length.arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert!(
            structured
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
                > structured
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .drift_pressure
                    .0
        );

        assert_eq!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::UnresolvedDrift
        );
        assert!(
            weak_backbeat
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .drift_pressure
                .0
                > weak_backbeat
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .refresh_strength
                    .0
        );

        assert_eq!(
            ambiguous.meter_state.continuity.bar_length.arc_rationale,
            super::MeterContinuityArcRationale::EvidenceLoss
        );
        assert!(
            ambiguous
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                >= 0.5
        );

        assert_eq!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_rationale,
            super::MeterContinuityArcRationale::UnresolvedDrift
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_support
                .refresh_strength
                .0
                > 0.8
        );
        assert!(
            pickup_extended
                .meter_state
                .continuity
                .downbeat_phase
                .arc_support
                .drift_pressure
                .0
                > pickup_extended
                    .meter_state
                    .continuity
                    .downbeat_phase
                    .arc_support
                    .structural_pressure
                    .0
        );

        assert_eq!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::StructuralInstability
        );
        assert!(
            dropout_extended
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                > sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .structural_pressure
                    .0
        );

        assert_eq!(
            sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert_eq!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::RefreshStrength
        );
        assert!(
            long_sustained_reset
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .refresh_strength
                .0
                >= sustained_reset
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .refresh_strength
                    .0
        );

        assert_eq!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .arc_rationale,
            super::MeterContinuityArcRationale::EvidenceLoss
        );
        assert!(
            modulation_extended
                .meter_state
                .continuity
                .bar_length
                .arc_support
                .structural_pressure
                .0
                >= ambiguous
                    .meter_state
                    .continuity
                    .bar_length
                    .arc_support
                    .structural_pressure
                    .0
        );
    }
}
