use signal_primitives::Sample;

/// Geometry evidence for one band in the frequency-adaptive reconstruction proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveBandEvidence {
    /// Centre bin in the full transform's FFT ordering.
    pub center_bin: usize,
    /// Absolute centre frequency.
    pub center_frequency_hz: f64,
    /// Number of nonzero frequency samples in the compact analysis filter.
    pub support_bins: usize,
    /// Source-frame spacing between adjacent coefficients in this band.
    pub decimation_frames: usize,
    /// Number of complex coefficients produced by this band.
    pub coefficient_count: usize,
    /// Peak position of the zero-phase band impulse response.
    pub impulse_peak_frame: usize,
}

/// Transform and reconstruction evidence from the frequency-adaptive proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveEvidence {
    /// Full transform length.
    pub fft_frames: usize,
    /// Number of analysis bands, including mirrored and completion bands.
    pub band_count: usize,
    /// Total complex coefficient count.
    pub coefficient_count: usize,
    /// Smallest diagonal frame-operator value.
    pub frame_operator_min: f64,
    /// Largest diagonal frame-operator value.
    pub frame_operator_max: f64,
    /// Ratio of maximum to minimum frame-operator values.
    pub frame_condition_ratio: f64,
    /// Frequency samples with no nonzero analysis filter.
    pub uncovered_frequency_bins: usize,
    /// Frequency samples owned by more than one nonzero analysis filter.
    pub multiply_covered_frequency_bins: usize,
    /// Bands whose compact support exceeds their decimation.
    pub painless_support_violations: usize,
    /// Input sample count.
    pub source_frames: usize,
    /// Reconstructed sample count.
    pub output_frames: usize,
    /// Largest absolute reconstruction error.
    pub reconstruction_peak_error: f64,
    /// Root-mean-square reconstruction error.
    pub reconstruction_rms_error: f64,
    /// Absolute reconstruction error at the first sample.
    pub reconstruction_head_error: f64,
    /// Absolute reconstruction error at the final sample.
    pub reconstruction_tail_error: f64,
    /// Non-finite analysis coefficients.
    pub non_finite_coefficients: usize,
    /// Non-finite reconstructed samples.
    pub non_finite_output_samples: usize,
    /// Largest band impulse offset from the declared zero-phase origin.
    pub max_band_impulse_delay_frames: usize,
    /// Stable FNV-1a hash of analysis-filter geometry and values.
    pub filter_hash: u64,
    /// Stable FNV-1a hash of complex coefficient bits.
    pub coefficient_hash: u64,
    /// Stable FNV-1a hash of reconstructed sample bits.
    pub reconstruction_hash: u64,
    /// Per-band geometry and delay evidence.
    pub bands: Vec<StretchFrequencyAdaptiveBandEvidence>,
}

/// Report-only identity reconstruction through a frequency-adaptive frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveReview {
    /// Exact-length reconstructed mono samples.
    pub samples: Vec<Sample>,
    /// Transform geometry and reconstruction evidence.
    pub evidence: StretchFrequencyAdaptiveEvidence,
}

/// Evidence from the report-only common-grid wavelet reconstruction proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridWaveletEvidence {
    /// Number of nonnegative-frequency analysis channels.
    pub channel_count: usize,
    /// Number of lowpass completion channels.
    pub lowpass_channel_count: usize,
    /// Uniform coefficient spacing in source frames.
    pub hop_frames: usize,
    /// Complex-coefficient redundancy relative to real source samples.
    pub redundancy: f64,
    /// Stable hash of the deterministic channel delays.
    pub delay_hash: u64,
    /// Estimated lower frame bound.
    pub frame_bound_min: f64,
    /// Estimated upper frame bound.
    pub frame_bound_max: f64,
    /// Ratio of upper to lower frame bounds.
    pub frame_condition_ratio: f64,
    /// Largest relative residual from canonical-dual block solves.
    pub canonical_dual_residual: f64,
    /// Number of analyzed complex coefficients.
    pub analysis_coefficient_count: usize,
    /// Number of synthesized complex coefficients.
    pub synthesis_coefficient_count: usize,
    /// Largest absolute reconstruction error.
    pub reconstruction_peak_error: f64,
    /// Root-mean-square reconstruction error.
    pub reconstruction_rms_error: f64,
    /// First-sample reconstruction error.
    pub reconstruction_head_error: f64,
    /// Final-sample reconstruction error.
    pub reconstruction_tail_error: f64,
    /// Number of non-finite coefficients or output samples.
    pub non_finite_values: usize,
    /// Stable source-sample hash.
    pub source_hash: u64,
    /// Stable reconstructed-sample hash.
    pub output_hash: u64,
    /// Stable complex-coefficient hash.
    pub coefficient_hash: u64,
}

/// Exact-length identity reconstruction through the common-grid wavelet frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridWaveletReview {
    /// Reconstructed mono samples.
    pub samples: Vec<Sample>,
    /// Common-grid frame and reconstruction evidence.
    pub evidence: StretchCommonGridWaveletEvidence,
}

/// Identity reconstruction through the untightened boundary-completion candidate.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridBoundaryReview {
    /// Complete canonical-dual identity reconstruction.
    pub reconstruction: StretchCommonGridWaveletReview,
    /// Stable hash of preserved raw channels `0..1534`.
    pub preserved_filter_hash: u64,
    /// Stable hash of the replacement Nyquist completion.
    pub nyquist_completion_hash: u64,
    /// Stable hash of the complete raw boundary bank.
    pub raw_filter_hash: u64,
}

/// Identity reconstruction through the endpoint-even preconditioned frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridPreconditionedReview {
    /// Complete canonical-dual identity reconstruction.
    pub reconstruction: StretchCommonGridWaveletReview,
    /// Stable hash of the complete raw boundary bank.
    pub raw_filter_hash: u64,
    /// Stable hash of the common scalar preconditioner.
    pub multiplier_hash: u64,
}

/// Steady-tone evidence for common-grid delay compensation and phase scale.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridTonePhaseEvidence {
    /// Expected tone angular frequency in radians per sample.
    pub expected_angular_frequency: f64,
    /// Largest horizontal instantaneous-frequency error on qualified coefficients.
    pub max_angular_frequency_error: f64,
    /// Largest adjacent-channel phase residual after delay compensation.
    pub max_compensated_phase_residual: f64,
    /// Number of qualified horizontal differences.
    pub horizontal_measurements: usize,
    /// Number of qualified adjacent-channel comparisons.
    pub vertical_measurements: usize,
    /// Whether all measured derivatives and residuals are finite.
    pub all_values_finite: bool,
    /// Coefficients skipped because their energy did not qualify the ratio.
    pub zero_energy_skips: usize,
    /// Stable hash of the auxiliary derivative coefficients.
    pub auxiliary_hash: u64,
    /// Stable hash of the diagnostic trace.
    pub trace_hash: u64,
}

/// Evidence from exact common-grid field projection and bounded phase assignment.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridProjectedPhaseEvidence {
    /// Fixed-ratio source-to-output duration multiplier.
    pub ratio: f64,
    /// Exact rounded target length in sample frames.
    pub target_frames: usize,
    /// Source coefficient columns available before logical boundary extension.
    pub source_columns: usize,
    /// Projected output columns, including terminal coverage.
    pub output_columns: usize,
    /// Largest reconstruction error for the authoritative fractional coordinate.
    pub max_coordinate_error: f64,
    /// Whether projected coordinates increase strictly.
    pub coordinates_monotonic: bool,
    /// Projected columns with a non-integral source coordinate.
    pub fractional_columns: usize,
    /// Logical interpolation reads served by boundary extension.
    pub boundary_pad_reads: usize,
    /// Magnitude, frequency, and vertical-derivative values projected.
    pub projected_field_values: usize,
    /// Significant phase cells seeded in column zero.
    pub seed_assignments: usize,
    /// Significant cells assigned from the preceding output column.
    pub horizontal_assignments: usize,
    /// Significant cells assigned from an adjacent current-column channel.
    pub vertical_assignments: usize,
    /// Significant cells assigned more than once.
    pub duplicate_assignments: usize,
    /// Significant cells left without an assignment.
    pub missing_assignments: usize,
    /// Projected cells below the relative magnitude threshold.
    pub insignificant_cells: usize,
    /// Largest number of live heap candidates.
    pub heap_high_water: usize,
    /// Fixed heap-entry capacity contract.
    pub heap_capacity: usize,
    /// Number of projected fields or assigned phases that were non-finite.
    pub non_finite_values: usize,
    /// Stable hash of projected field values.
    pub projected_field_hash: u64,
    /// Stable hash of phase-assignment decisions and values.
    pub assignment_hash: u64,
}

/// Evidence from the canonical-dual synthesis-guard stop gate.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridDualGuardEvidence {
    /// Transform length used to measure complete canonical-dual atoms.
    pub probe_fft_frames: usize,
    /// Total positive-frequency channels in the finalized bank.
    pub channel_count: usize,
    /// Channels evaluated before passage or fail-fast rejection.
    pub evaluated_channels: usize,
    /// Maximum permitted two-sided guard in sample frames.
    pub guard_cap_frames: usize,
    /// Passing guard, or first whole-hop lower bound beyond the legal cap.
    pub required_guard_lower_bound_frames: usize,
    /// Largest excluded-energy ratio at the selected legal guard.
    pub max_tail_energy_ratio: f64,
    /// Channel that produced the limiting tail measurement.
    pub limiting_channel: usize,
    /// Largest canonical-dual block-solve residual.
    pub max_dual_residual: f64,
    /// Non-finite dual spectrum or atom values.
    pub non_finite_values: usize,
    /// Whether every channel passed within the guard cap.
    pub passed: bool,
    /// Stable hash of evaluated canonical-dual atoms.
    pub dual_atom_hash: u64,
}

/// Response stage measured by the common-grid tail-attribution diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StretchCommonGridTailStage {
    /// Finalized analysis response before per-bin tightening.
    RawAnalysis,
    /// Analysis response after per-bin tightening.
    TightenedAnalysis,
    /// Exact complete-frame canonical-dual response after tightening.
    CanonicalDual,
}

/// Spectrum form measured by the common-grid tail-attribution diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StretchCommonGridTailForm {
    /// Positive-frequency response with an empty negative-frequency half.
    Analytic,
    /// Explicit conjugate mirror used for real-output synthesis.
    RealMirrored,
}

/// Tail evidence for one channel, response stage, and spectrum form.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridTailAtomEvidence {
    /// Filter-bank channel index.
    pub channel: usize,
    /// Response stage.
    pub stage: StretchCommonGridTailStage,
    /// Spectrum form.
    pub form: StretchCommonGridTailForm,
    /// Peak sample in circular atom ordering.
    pub peak_frame: usize,
    /// Total squared atom energy.
    pub total_energy: f64,
    /// Excluded-energy ratios at the report's fixed radii.
    pub tail_energy_ratios: Vec<f64>,
    /// First passing whole-hop guards, or lower bounds beyond the probe radius.
    pub guard_lower_bounds: Vec<usize>,
    /// Canonical-dual residual, or zero for analysis stages.
    pub dual_residual: f64,
    /// Non-finite spectrum or atom values.
    pub non_finite_values: usize,
    /// Stable hash of the complex atom.
    pub atom_hash: u64,
}

/// Complete fixed-matrix common-grid tail-attribution report.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridTailAttributionEvidence {
    /// Probe transform length.
    pub probe_fft_frames: usize,
    /// Fixed whole-hop tail radii.
    pub radii_frames: Vec<usize>,
    /// Fixed excluded-energy thresholds.
    pub thresholds: Vec<f64>,
    /// All five-channel, three-stage, two-form atom reports.
    pub atoms: Vec<StretchCommonGridTailAtomEvidence>,
    /// Real-output tightening ratios in channel order.
    pub tightening_ratios: Vec<f64>,
    /// Real-output canonical-dual ratios in channel order.
    pub dualization_ratios: Vec<f64>,
    /// Real/analytic ratios in atom stage/channel order.
    pub mirroring_ratios: Vec<f64>,
    /// Canonical-dual real-output channel-zero/channel-sixteen ratio.
    pub lowpass_to_first_wavelet_ratio: f64,
    /// Canonical-dual real-output channel-zero/interior ratio.
    pub lowpass_to_interior_ratio: f64,
    /// Largest exact canonical-dual block-solve residual.
    pub max_dual_residual: f64,
    /// Total non-finite spectrum or atom values.
    pub non_finite_values: usize,
    /// Stable hash of the complete evidence matrix.
    pub report_hash: u64,
}
