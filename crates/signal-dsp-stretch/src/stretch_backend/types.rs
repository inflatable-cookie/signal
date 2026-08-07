//! Stretch backend tiers, plan contract, and window/hop constants.

/// Quality tier of a stretch backend (memo 013 vocabulary). One tier exists
/// today; real-time and offline production tiers land with the library
/// evaluation (P-TS-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQuality {
    /// Draft-quality phase vocoder: pitch-preserving, but transients smear
    /// and no formant handling. Offline use only.
    Draft,
    /// Bounded-latency preview quality. Implemented as a control-side
    /// prototype; direct audio-thread processing is still unsupported.
    RealtimePreview,
    /// Highest-quality deterministic offline/export quality. Product-facing
    /// use is still promotion-gated per artifact.
    OfflineHighQuality,
}

/// Signal-owned stretch execution tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendTier {
    /// Existing render-plane varispeed path. Tempo changes also shift pitch.
    Repitch,
    /// Prototype bounded-latency preview tier for live audition and playback.
    RealtimePreview,
    /// Deterministic high-quality tier for exports, freeze, and cached
    /// post-warp artifacts.
    OfflineHighQuality,
}

/// Implementation status for one tier in the Signal-native stretch program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendStatus {
    /// The tier is implemented in Signal today.
    Implemented,
    /// The tier has an implemented DSP path, but it has not yet satisfied the
    /// full product-facing backend contract or corpus promotion gate.
    Prototype,
    /// The tier is designed but not implemented.
    Planned,
}

/// Clean-room architecture contract for one Signal-owned tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchBackendPlan {
    /// Signal-owned execution tier.
    pub tier: StretchBackendTier,
    /// Current implementation status.
    pub status: StretchBackendStatus,
    /// Whether tempo and pitch can be controlled independently.
    pub independent_tempo_and_pitch: bool,
    /// Whether stretch ratio may change within one render.
    pub dynamic_ratio: bool,
    /// Whether transient preservation is part of the tier contract.
    pub transient_preservation: bool,
    /// Whether stereo or multichannel vertical coherence is part of the tier
    /// contract.
    pub vertical_phase_coherence: bool,
    /// Whether the tier promises sample-accurate or near-sample-accurate
    /// timeline alignment.
    pub alignment_promised: bool,
    /// Whether processing may run on the realtime audio thread.
    pub audio_thread_safe: bool,
    /// Whether rendered output is deterministic enough for cache identity,
    /// export reuse, and regression comparison.
    pub deterministic_output: bool,
}

/// Signal-owned tier plan. This is a code-level mirror of the roadmap
/// contract so callers can gate behavior without vendor-specific names.
pub const SIGNAL_STRETCH_BACKEND_PLAN: [StretchBackendPlan; 3] = [
    StretchBackendPlan {
        tier: StretchBackendTier::Repitch,
        status: StretchBackendStatus::Implemented,
        independent_tempo_and_pitch: false,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: true,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::RealtimePreview,
        status: StretchBackendStatus::Prototype,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::OfflineHighQuality,
        status: StretchBackendStatus::Implemented,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
];

/// Returns the Signal-owned plan for `tier`.
pub fn stretch_backend_plan(tier: StretchBackendTier) -> StretchBackendPlan {
    SIGNAL_STRETCH_BACKEND_PLAN
        .iter()
        .copied()
        .find(|plan| plan.tier == tier)
        .expect("all StretchBackendTier variants are represented")
}

/// Offline high-quality renderer path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfflineHighQualityPath {
    /// Current production-candidate OfflineHighQuality path.
    Default,
    /// Compression-only selector that switches to a shorter STFT window when
    /// the current path misses transients or exceeds the current-smear gate.
    CompressionShortWindowSelector,
    /// Expansion-only selector that switches to a shorter STFT window when
    /// the current path misses transients or regresses versus the draft
    /// transient-smear baseline.
    ExpansionShortWindowSelector,
}

/// Default STFT window: 2048 samples (~43 ms at 48 kHz).
pub const DEFAULT_WINDOW_SIZE: usize = 2_048;
/// Default analysis hop: window / 4 (75% overlap).
pub const DEFAULT_ANALYSIS_HOP: usize = DEFAULT_WINDOW_SIZE / 4;
/// Short-window selector STFT size for compression material.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE: usize = 1_024;
/// Short-window selector analysis hop.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE / 4;
/// RealtimePreview prototype STFT size.
pub const REALTIME_PREVIEW_WINDOW_SIZE: usize = 512;
/// RealtimePreview prototype analysis hop.
pub const REALTIME_PREVIEW_ANALYSIS_HOP: usize = REALTIME_PREVIEW_WINDOW_SIZE / 4;
/// Short-window selector gate: current path must miss at least this many
/// source transients before the selector may switch.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES: usize = 1;
/// Short-window selector gate: current path must exceed this transient-smear
/// value before the selector may switch.
pub const COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES: f64 = 64.0;
/// Short-window selector STFT size for expansion material.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE;
/// Short-window selector analysis hop for expansion material.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP;
/// Expansion short-window selector gate: current path must miss at least this
/// many source transients before the selector may switch.
pub const EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES: usize =
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES;
