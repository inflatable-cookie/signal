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

/// Fixed bank in alias-block conditioning attribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridConditioningBank {
    /// Untightened Rule 26 boundary bank.
    Raw,
    /// Exact pointwise inverse-energy diagnostic bank.
    ExactPointwise,
    /// Rejected endpoint-even diagnostic bank.
    EndpointEven,
}

/// Direction selected by alias-block conditioning attribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridConditioningDirection {
    /// Numerical attribution did not meet its proof gates.
    Inconclusive,
    /// Return to boundary-filter geometry.
    BoundaryGeometry,
    /// Contract later block-aware boundary preconditioner research.
    BlockAwareBoundary,
}

/// Per-residue extremal frame evidence for one conditioning bank.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridConditioningResidueEvidence {
    /// Bank under review.
    pub bank: StretchCommonGridConditioningBank,
    /// Alias residue index.
    pub residue: usize,
    /// Number of frequency bins in the residue block.
    pub bin_count: usize,
    /// Minimum and maximum frame eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Normalized residuals for minimum and maximum eigenvectors.
    pub residuals: [f64; 2],
    /// Stable hashes for bins, matrix, and extremal eigenvectors.
    pub hashes: [u64; 4],
}

/// Bounded bin contribution to one extremal conditioning mode.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridConditioningBinEvidence {
    /// FFT bin index.
    pub bin: usize,
    /// Normalized eigenvector weight.
    pub weight: f64,
}

/// Bounded channel contribution to one extremal conditioning mode.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridConditioningChannelEvidence {
    /// Filter-bank channel index.
    pub channel: usize,
    /// Total quadratic contribution.
    pub total: f64,
    /// Diagonal contribution.
    pub diagonal: f64,
    /// Signed cross contribution.
    pub cross: f64,
}

/// Attribution for one bank's global minimum or maximum frame mode.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridConditioningModeEvidence {
    /// Bank under review.
    pub bank: StretchCommonGridConditioningBank,
    /// Whether this is the maximum rather than minimum mode.
    pub maximum: bool,
    /// Limiting residue and eigenvalue.
    pub residue: usize,
    /// Limiting eigenvalue.
    pub eigenvalue: f64,
    /// Rayleigh quotient under raw, exact-pointwise, and endpoint-even banks.
    pub cross_bank_rayleigh: [f64; 3],
    /// Eigenvector mass in DC, interior, and Nyquist regions.
    pub region_mass: [f64; 3],
    /// Sixteen largest bin weights.
    pub top_bins: Vec<StretchCommonGridConditioningBinEvidence>,
    /// Sixteen largest total channel contributions.
    pub top_total_channels: Vec<StretchCommonGridConditioningChannelEvidence>,
    /// Sixteen largest absolute cross-term channel contributions.
    pub top_cross_channels: Vec<StretchCommonGridConditioningChannelEvidence>,
    /// Total, diagonal, cross, and relative closure error.
    pub contribution_sums: [f64; 4],
}

/// Complete report-only alias-block conditioning attribution.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridConditioningReview {
    /// All three banks across all eleven residues.
    pub residues: Vec<StretchCommonGridConditioningResidueEvidence>,
    /// Global minimum and maximum attribution for every bank.
    pub modes: Vec<StretchCommonGridConditioningModeEvidence>,
    /// Raw, exact multiplier, endpoint multiplier, and evidence hashes.
    pub hashes: [u64; 4],
    /// Largest eigenpair residual and contribution closure error.
    pub maximum_errors: [f64; 2],
    /// Direction selected by the frozen contract.
    pub direction: StretchCommonGridConditioningDirection,
}

/// Diagnostic frame operator in the Nyquist-completion coupling ablation.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridNyquistAblationOperator {
    /// Complete exact-pointwise frame operator.
    Full,
    /// Operator with the complete channel-1535 rank-one term removed.
    CompletionRemoved,
    /// Operator retaining channel-1535 diagonal energy but not cross-bin terms.
    CompletionDiagonalized,
}

/// Research direction selected by the Nyquist-completion coupling ablation.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridNyquistAblationDirection {
    /// A numerical or matrix-identity proof gate failed.
    Inconclusive,
    /// Research an orthogonal or multi-row Nyquist completion.
    OrthogonalOrMultiRowCompletion,
    /// Replace the Nyquist completion family, including its diagonal energy.
    ReplacementCompletion,
    /// Broaden research to the complete high-edge channel geometry.
    CompleteHighEdgeGeometry,
}

/// Per-residue evidence for one Nyquist-completion ablation operator.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridNyquistAblationResidueEvidence {
    /// Operator under review.
    pub operator: StretchCommonGridNyquistAblationOperator,
    /// Alias residue index.
    pub residue: usize,
    /// Number of frequency bins in the residue block.
    pub bin_count: usize,
    /// Minimum and maximum eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Per-residue maximum-to-minimum eigenvalue ratio.
    pub condition_ratio: f64,
    /// Proven Jacobi solve evidence.
    pub jacobi: StretchCommonGridJacobiEvidence,
    /// Channel-1535 diagonal energy and off-diagonal Frobenius energy.
    pub completion_energy: [f64; 2],
    /// Stable hashes of bins and matrix.
    pub hashes: [u64; 2],
}

/// Global frame extrema for one Nyquist-completion ablation operator.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridNyquistAblationGlobalEvidence {
    /// Operator under review.
    pub operator: StretchCommonGridNyquistAblationOperator,
    /// Global minimum and maximum eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Residues owning the global minimum and maximum.
    pub residues: [usize; 2],
    /// Global maximum-to-minimum condition ratio.
    pub condition_ratio: f64,
}

/// Frozen exact-pointwise extremal-mode response to all three operators.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridNyquistAblationModeEvidence {
    /// Whether this is the maximum rather than minimum full-operator mode.
    pub maximum: bool,
    /// Residue containing the frozen mode.
    pub residue: usize,
    /// Eigenvalue of the complete exact-pointwise operator.
    pub eigenvalue: f64,
    /// Rayleigh quotients for full, removed, and diagonalized operators.
    pub rayleigh: [f64; 3],
    /// Removed-minus-full and diagonalized-minus-full Rayleigh changes.
    pub changes: [f64; 2],
    /// Relative closure errors for complete and off-diagonal subtraction.
    pub closure_errors: [f64; 2],
    /// Stable hash of the frozen eigenvector.
    pub vector_hash: u64,
}

/// Complete report-only Nyquist-completion alias-coupling ablation.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridNyquistAblationReview {
    /// All three operators across all eleven residues.
    pub residues: Vec<StretchCommonGridNyquistAblationResidueEvidence>,
    /// Global extrema and condition ratio for all three operators.
    pub globals: [StretchCommonGridNyquistAblationGlobalEvidence; 3],
    /// Frozen full-operator minimum and maximum modes.
    pub modes: [StretchCommonGridNyquistAblationModeEvidence; 2],
    /// Maximum residual, orthogonality, trace, Frobenius, and closure errors.
    pub maximum_errors: [f64; 5],
    /// Exact-pointwise filter hash and complete evidence hash.
    pub hashes: [u64; 2],
    /// Geometry research direction selected by the frozen gates.
    pub direction: StretchCommonGridNyquistAblationDirection,
}

/// Outcome of the three-row Nyquist-completion matrix proof.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridThreeRowNyquistDirection {
    /// The construction and complete frame matrix pass every frozen gate.
    IdentityReconstructionProof,
    /// The candidate fails and returns to boundary-geometry research.
    BoundaryGeometry,
}

/// Per-residue evidence for the three-row Nyquist-completion candidate.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridThreeRowNyquistResidueEvidence {
    /// Alias residue index.
    pub residue: usize,
    /// Number of frequency bins in the residue block.
    pub bin_count: usize,
    /// Minimum and maximum frame eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Per-residue maximum-to-minimum condition ratio.
    pub condition_ratio: f64,
    /// Proven Jacobi solve evidence.
    pub jacobi: StretchCommonGridJacobiEvidence,
    /// Stable hashes of bins and complete frame matrix.
    pub hashes: [u64; 2],
}

/// Complete release-only three-row Nyquist-completion matrix proof.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridThreeRowNyquistReview {
    /// Candidate analysis-row count.
    pub row_count: usize,
    /// Uniform coefficient hop.
    pub hop_frames: usize,
    /// Completion-row delays in source frames.
    pub completion_delays: [i32; 3],
    /// Hash of preserved raw channels `0..1534`.
    pub preserved_hash: u64,
    /// Stable hash of each completion row.
    pub completion_hashes: [u64; 3],
    /// Support, diagonal-energy, off-diagonal, and real-Nyquist errors.
    pub construction_errors: [f64; 4],
    /// All eleven complete frame matrices.
    pub residues: Vec<StretchCommonGridThreeRowNyquistResidueEvidence>,
    /// Global minimum and maximum eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Residues owning the global minimum and maximum.
    pub limiting_residues: [usize; 2],
    /// Global maximum-to-minimum condition ratio.
    pub condition_ratio: f64,
    /// Maximum residual, orthogonality, trace, and Frobenius errors.
    pub maximum_proof_errors: [f64; 4],
    /// Stable hash of the complete report evidence.
    pub evidence_hash: u64,
    /// Next direction selected by the frozen gates.
    pub direction: StretchCommonGridThreeRowNyquistDirection,
}

/// Matrix operator in residual boundary-geometry attribution.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridResidualBoundaryOperator {
    /// Complete rejected three-row candidate.
    Full,
    /// DC-row cross terms removed while retaining diagonal energy.
    DcDiagonalized,
    /// Preserved high-edge cross terms removed while retaining diagonal energy.
    HighEdgeDiagonalized,
    /// DC and preserved high-edge cross terms removed together.
    BothBoundaryDiagonalized,
}

/// Geometry direction selected by residual boundary attribution.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridResidualBoundaryDirection {
    /// Numerical, closure, or repeat evidence failed.
    Inconclusive,
    /// Preserved high-edge rows own the remaining condition failure.
    HighEdgeGeometry,
    /// DC lowpass rows own the remaining condition failure.
    DcGeometry,
    /// Both boundary groups jointly own the failure.
    JointBoundaryGeometry,
    /// Boundary cross terms are insufficient; broaden to the complete raw bank.
    CompleteRawBank,
}

/// Per-residue evidence for one residual-boundary operator.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridResidualBoundaryResidueEvidence {
    /// Operator under review.
    pub operator: StretchCommonGridResidualBoundaryOperator,
    /// Alias residue index.
    pub residue: usize,
    /// Minimum and maximum eigenvalues.
    pub eigenvalues: [f64; 2],
    /// Per-residue condition ratio.
    pub condition_ratio: f64,
    /// Jacobi proof evidence.
    pub jacobi: StretchCommonGridJacobiEvidence,
    /// Stable bin and matrix hashes.
    pub hashes: [u64; 2],
    /// Relative matrix-subtraction closure.
    pub closure_error: f64,
}

/// Contribution from one frozen channel group.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridResidualBoundaryGroupEvidence {
    /// Inclusive-exclusive candidate-row range.
    pub rows: [usize; 2],
    /// Total, diagonal, signed cross, and relative arithmetic closure.
    pub contributions: [f64; 4],
}

/// Attribution of one frozen full-candidate extremal mode.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridResidualBoundaryModeEvidence {
    /// Whether this is the maximum rather than minimum mode.
    pub maximum: bool,
    /// Frozen residue and eigenvalue.
    pub residue: usize,
    /// Full-candidate eigenvalue.
    pub eigenvalue: f64,
    /// Eigenvector mass in DC, interior, and Nyquist regions.
    pub region_mass: [f64; 3],
    /// Sixteen largest bin weights.
    pub top_bins: Vec<StretchCommonGridConditioningBinEvidence>,
    /// Sixteen largest total row contributions.
    pub top_total_channels: Vec<StretchCommonGridConditioningChannelEvidence>,
    /// Sixteen largest absolute row cross contributions.
    pub top_cross_channels: Vec<StretchCommonGridConditioningChannelEvidence>,
    /// DC, interior, preserved-high-edge, and completion contributions.
    pub groups: [StretchCommonGridResidualBoundaryGroupEvidence; 4],
    /// Rayleigh quotients for all four operators.
    pub rayleigh: [f64; 4],
    /// DC, high-edge, and both-boundary changes from full.
    pub changes: [f64; 3],
    /// Relative total-contribution closure.
    pub closure_error: f64,
    /// Stable eigenvector hash.
    pub vector_hash: u64,
}

/// Complete release-only residual boundary-geometry attribution.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridResidualBoundaryReview {
    /// All four operators across all eleven residues.
    pub residues: Vec<StretchCommonGridResidualBoundaryResidueEvidence>,
    /// Global condition ratios in operator order.
    pub conditions: [f64; 4],
    /// Frozen full-candidate minimum and maximum modes.
    pub modes: [StretchCommonGridResidualBoundaryModeEvidence; 2],
    /// Maximum residual, orthogonality, trace, Frobenius, and closure errors.
    pub maximum_errors: [f64; 5],
    /// Stable complete evidence hash.
    pub evidence_hash: u64,
    /// Selected geometry direction.
    pub direction: StretchCommonGridResidualBoundaryDirection,
}

/// Direction selected by complete canonical block-tightener feasibility.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCommonGridCanonicalTightenerDirection {
    /// Every row preserves support and may advance to large-probe localization.
    LargeProbeLocalization,
    /// Support or endpoint damage closes the common-grid family.
    TransformFamilyReassessment,
    /// Numerical proof failed before a localization decision.
    Inconclusive,
}

/// Complete release-only canonical block-tightener feasibility report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridCanonicalTightenerReview {
    /// Number of transformed rows evaluated before passage or first violation.
    pub evaluated_rows: usize,
    /// First violating row, or `usize::MAX` when every row passes.
    pub first_violating_row: usize,
    /// Global transformed-frame minimum, maximum, and condition ratio.
    pub frame_values: [f64; 3],
    /// Maximum residual, orthogonality, trace, Frobenius, and identity errors.
    pub maximum_proof_errors: [f64; 5],
    /// Maximum relative support leakage, out-of-support peak, and endpoint error.
    pub localization_errors: [f64; 3],
    /// Original and transformed nonzero-bin counts for the limiting row.
    pub limiting_support_bins: [usize; 2],
    /// Stable hashes of input filters, block tighteners, evaluated rows, and report.
    pub hashes: [u64; 4],
    /// Selected continuation direction.
    pub direction: StretchCommonGridCanonicalTightenerDirection,
}

/// Direction selected by dense painless common-lattice feasibility.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchDensePainlessDirection {
    /// Every frozen gate passes and phase-topology research may be contracted.
    PhaseTopologyContract,
    /// A frozen feasibility gate fails and requires operator review.
    OperatorReview,
}

/// Complete release-only dense painless common-lattice feasibility report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchDensePainlessReview {
    /// FFT length, band count, common coefficient count, and common hop.
    pub geometry: [usize; 4],
    /// Unequal and dense coefficient totals.
    pub coefficient_counts: [usize; 2],
    /// Dense coefficient growth and redundancy relative to source frames.
    pub coefficient_cost: [f64; 2],
    /// Minimum, maximum, and condition ratio of the diagonal frame operator.
    pub frame_values: [f64; 3],
    /// Uncovered bins, support violations, and non-finite values.
    pub structural_failures: [usize; 3],
    /// Real-spectrum closure and peak, RMS, head, and tail reconstruction errors.
    pub reconstruction_errors: [f64; 5],
    /// Whole-common-hop localization radii through the frozen cap.
    pub localization_radii: Vec<usize>,
    /// Maximum analysis and dual excluded-energy ratios at each radius.
    pub localization_curves: Vec<[f64; 2]>,
    /// Maximum required analysis and dual radii, or `usize::MAX` on failure.
    pub required_radii: [usize; 2],
    /// Bands limiting analysis and dual localization.
    pub limiting_bands: [usize; 2],
    /// Unequal/dense filter, frame, and dual hashes plus complete evidence hash.
    pub hashes: [u64; 7],
    /// Selected continuation direction.
    pub direction: StretchDensePainlessDirection,
}

/// Direction selected by time-adaptive painless identity reconstruction.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchTimeAdaptivePainlessDirection {
    /// Every identity gate passes and automatic selection may be contracted.
    AutomaticSelectionContract,
    /// A schedule or reconstruction gate fails and must be redesigned.
    ScheduleRedesign,
}

/// Evidence for one declared time-adaptive window schedule.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchTimeAdaptiveScheduleEvidence {
    /// Schedule family index and frame count.
    pub family_and_frames: [usize; 2],
    /// Counts of 512, 1024, 2048, and 4096-sample windows.
    pub window_counts: [usize; 4],
    /// Minimum and maximum adjacent source-center hops.
    pub hop_extrema: [usize; 2],
    /// Reflected source reads and total complex coefficients per control.
    pub work_counts: [usize; 2],
    /// Frame-operator minimum, maximum, and condition ratio.
    pub frame_values: [f64; 3],
    /// Uncovered padded/source frames, illegal transitions, and support failures.
    pub structural_failures: [usize; 4],
    /// Conjugate symmetry, imaginary residue, peak, RMS, head, and tail errors.
    pub maximum_errors: [f64; 6],
    /// Non-finite coefficient, dual, and output values.
    pub non_finite_values: usize,
    /// Schedule, window, dual, coefficient, output, and evidence hashes.
    pub hashes: [u64; 6],
}

/// Complete release-only time-adaptive painless identity report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchTimeAdaptivePainlessReview {
    /// Evidence for all five declared schedules.
    pub schedules: Vec<StretchTimeAdaptiveScheduleEvidence>,
    /// Whether empty input remains exactly empty.
    pub empty_input_exact: bool,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchTimeAdaptivePainlessDirection,
}

/// Direction selected by automatic Rényi time-resolution evidence.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenyiSelectorDirection {
    /// Every selector gate passes and variable-hop phase may be contracted.
    VariableHopPhaseContract,
    /// At least one selector gate fails and selector research must continue.
    SelectorResearch,
}

/// Evidence for one automatic Rényi selector control.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiControlEvidence {
    /// Frozen control index.
    pub control: usize,
    /// Minimum-entropy winner at every decision anchor.
    pub raw_winners: Vec<u8>,
    /// Legal minimum-total-entropy resolution path.
    pub selected_levels: Vec<u8>,
    /// Per-anchor energies for 512, 1024, 2048, and 4096 windows.
    pub energies: Vec<[f64; 4]>,
    /// Per-anchor normalized Rényi entropies in the same order.
    pub entropies: Vec<[f64; 4]>,
    /// Selected counts by resolution level.
    pub level_counts: [usize; 4],
    /// Transition count and minimum/maximum derived hop.
    pub path_shape: [usize; 3],
    /// Reflected reads and non-finite values.
    pub structural_counts: [usize; 2],
    /// Maximum linked-channel energy closure error.
    pub channel_energy_closure: f64,
    /// Total selected path cost.
    pub path_cost: f64,
    /// Input, entropy, path, and complete evidence hashes.
    pub hashes: [u64; 4],
}

/// Complete release-only automatic Rényi resolution-selection report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiSelectorReview {
    /// Frozen base-control evidence.
    pub controls: Vec<StretchRenyiControlEvidence>,
    /// Steady, event, dense, boundary, chirp/noise/mixed, and equivalence failures.
    pub gate_failures: [usize; 7],
    /// Maximum perturbation path-change fraction.
    pub maximum_perturbation_change: f64,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchRenyiSelectorDirection,
}

/// Direction selected by Rényi selector-failure attribution.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenyiAttributionDirection {
    /// Time-region geometry owns the failed selector evidence.
    ComparisonRegionContract,
    /// One folded-frequency region owns the failed selector evidence.
    FrequencyEvidenceContract,
    /// Attribution is split, ambiguous, or does not satisfy either boundary.
    Inconclusive,
}

/// Effect of removing one diagnostic time or frequency region.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiRegionRemovalEvidence {
    /// Entropy change for the four resolution levels.
    pub entropy_deltas: [f64; 4],
    /// Removed energy fraction for the four resolution levels.
    pub energy_fractions: [f64; 4],
    /// Removed alpha-mass fraction for the four resolution levels.
    pub alpha_fractions: [f64; 4],
    /// Longest-minimum raw winner after diagnostic removal.
    pub raw_winner: u8,
}

/// Exact partition evidence for one selector decision anchor.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiAttributionAnchorEvidence {
    /// Source-frame decision anchor.
    pub anchor: usize,
    /// Time-slice coefficient counts for each resolution.
    pub time_counts: [[usize; 8]; 4],
    /// Time-slice energy sums for each resolution.
    pub time_energies: [[f64; 8]; 4],
    /// Time-slice alpha-mass sums for each resolution.
    pub time_alpha_sums: [[f64; 8]; 4],
    /// Folded-frequency coefficient counts for each resolution.
    pub frequency_counts: [[usize; 8]; 4],
    /// Folded-frequency energy sums for each resolution.
    pub frequency_energies: [[f64; 8]; 4],
    /// Folded-frequency alpha-mass sums for each resolution.
    pub frequency_alpha_sums: [[f64; 8]; 4],
    /// Leave-one-time-slice-out diagnostic evidence.
    pub time_removals: [StretchRenyiRegionRemovalEvidence; 8],
    /// Leave-one-frequency-region-out diagnostic evidence.
    pub frequency_removals: [StretchRenyiRegionRemovalEvidence; 8],
}

/// Partition evidence for one unchanged Batch 29.6AK control.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiAttributionControlEvidence {
    /// Frozen control index.
    pub control: usize,
    /// Evidence for all `64` decision anchors.
    pub anchors: Vec<StretchRenyiAttributionAnchorEvidence>,
    /// Maximum time-count, time-sum, frequency-count, and frequency-sum errors.
    pub closure_errors: [f64; 4],
    /// Non-finite values and non-silent empty-removal anomalies.
    pub structural_failures: [usize; 2],
    /// Full-region energy or entropy fields differing from Batch 29.6AK.
    pub baseline_drift: usize,
    /// Stable complete attribution hash.
    pub evidence_hash: u64,
}

/// Complete release-only Rényi selector-failure attribution report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiAttributionReview {
    /// Unchanged Batch 29.6AK selector report.
    pub baseline: StretchRenyiSelectorReview,
    /// Exact partition and counterfactual evidence for every control.
    pub controls: Vec<StretchRenyiAttributionControlEvidence>,
    /// Applicable isolated, mixed-event, and mixed-negative anchor counts.
    pub diagnostic_counts: [usize; 3],
    /// Passing geometry and frequency-region candidate counts.
    pub candidate_counts: [usize; 2],
    /// Restored isolated anchors and changed mixed negative controls.
    pub geometry_effects: [usize; 2],
    /// Mixed event anchors restored by each folded-frequency removal.
    pub frequency_event_restorations: [usize; 8],
    /// Mixed negative controls changed by each folded-frequency removal.
    pub frequency_negative_changes: [usize; 8],
    /// Linear-chirp raw-winner changes by time and frequency region.
    pub linear_chirp_changes: [[usize; 8]; 2],
    /// Stable aggregate attribution hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchRenyiAttributionDirection,
}

/// Direction selected by the final Rényi attribution reassessment.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenyiReassessmentDirection {
    /// Declared-event support isolates comparison-region geometry.
    ComparisonRegionContract,
    /// One low-frequency subregion isolates frequency evidence.
    FrequencyEvidenceContract,
    /// Both mechanisms pass and require one joint localized evidence contract.
    LocalizedTimeFrequencyContract,
    /// Evidence remains ambiguous or invalid and requires operator review.
    OperatorReview,
}

/// Final support and low-band attribution for one decision anchor.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiRefinedAnchorEvidence {
    /// Source-frame decision anchor.
    pub anchor: usize,
    /// Event-excluding and event-owning coefficient counts by resolution.
    pub support_counts: [[usize; 2]; 4],
    /// Event-excluding and event-owning energy sums by resolution.
    pub support_energies: [[f64; 2]; 4],
    /// Event-excluding and event-owning alpha-mass sums by resolution.
    pub support_alpha_sums: [[f64; 2]; 4],
    /// Raw winner after removing every event-owning coefficient frame.
    pub support_removed_winner: u8,
    /// Folded low-band subregion coefficient counts by resolution.
    pub low_counts: [[usize; 8]; 4],
    /// Folded low-band subregion energy sums by resolution.
    pub low_energies: [[f64; 8]; 4],
    /// Folded low-band subregion alpha-mass sums by resolution.
    pub low_alpha_sums: [[f64; 8]; 4],
    /// Untouched-complement count, energy, and alpha-mass by resolution.
    pub complement_counts: [usize; 4],
    /// Untouched-complement energy by resolution.
    pub complement_energies: [f64; 4],
    /// Untouched-complement alpha-mass by resolution.
    pub complement_alpha_sums: [f64; 4],
    /// Raw winners after removing each fixed low-band subregion.
    pub low_removed_winners: [u8; 8],
}

/// Final attribution evidence for one frozen control.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiRefinedControlEvidence {
    /// Frozen Batch 29.6AK control index.
    pub control: usize,
    /// Evidence for all `64` anchors.
    pub anchors: Vec<StretchRenyiRefinedAnchorEvidence>,
    /// Maximum support-count, support-sum, low-count, and low-sum closure errors.
    pub closure_errors: [f64; 4],
    /// Non-finite, empty-removal, and parent-drift failures.
    pub structural_failures: [usize; 3],
    /// Stable control evidence hash.
    pub evidence_hash: u64,
}

/// Complete final Rényi selector-attribution reassessment.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiReassessmentReview {
    /// Unchanged Batch 29.6AM attribution report.
    pub prior: StretchRenyiAttributionReview,
    /// Isolated-impulse, linear-chirp, and mixed-control refinement evidence.
    pub controls: Vec<StretchRenyiRefinedControlEvidence>,
    /// Restored isolated anchors and changed mixed negative controls.
    pub support_effects: [usize; 2],
    /// Restored mixed event anchors for each low-band subregion.
    pub low_event_restorations: [usize; 8],
    /// Changed mixed negative controls for each low-band subregion.
    pub low_negative_changes: [usize; 8],
    /// Changed linear-chirp winners for each low-band subregion.
    pub linear_chirp_changes: [usize; 8],
    /// Passing support boundary and low-band candidate counts.
    pub candidate_counts: [usize; 2],
    /// Stable aggregate refinement hash.
    pub evidence_hash: u64,
    /// Selected terminal direction.
    pub direction: StretchRenyiReassessmentDirection,
}

/// Direction selected by anchor-local Rényi geometry evidence.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchRenyiGeometryDirection {
    /// Every selector gate passes and variable-hop phase may be contracted.
    VariableHopPhaseContract,
    /// At least one terminal selector gate fails and operator review is required.
    OperatorReview,
}

/// Complete release-only anchor-local Rényi geometry report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRenyiGeometryReview {
    /// Selector evidence for all twelve frozen controls.
    pub controls: Vec<StretchRenyiControlEvidence>,
    /// Per-resolution minimum and maximum centre offsets from each anchor.
    pub support_extrema: [[isize; 2]; 4],
    /// Membership count failures and complete-window support escapes.
    pub geometry_failures: [usize; 2],
    /// Steady, event, dense, boundary, chirp/noise/mixed, and equivalence failures.
    pub gate_failures: [usize; 7],
    /// Maximum perturbation path-change fraction.
    pub maximum_perturbation_change: f64,
    /// Perturbation path-change fraction for each frozen control.
    pub perturbation_changes: Vec<f64>,
    /// Gain, polarity, and linked-stereo path-equivalence failures.
    pub equivalence_failures: usize,
    /// Stable membership hash.
    pub membership_hash: u64,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected terminal direction.
    pub direction: StretchRenyiGeometryDirection,
}

/// Direction selected by transient-evidence measurement.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchTransientEvidenceDirection {
    /// Every detector gate passes and occupancy-to-window mapping may be contracted.
    OccupancyMappingContract,
    /// At least one detector gate fails and operator review is required.
    OperatorReview,
}

/// Mixed-phase transient evidence at one decision anchor.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchTransientAnchorEvidence {
    /// Logical source-frame anchor.
    pub anchor: usize,
    /// Eligible and percussive channel-bin cell counts.
    pub cell_counts: [usize; 2],
    /// Percussive numerator and complete eligible denominator.
    pub magnitude_sums: [f64; 2],
    /// Linked percussive occupancy ratio.
    pub occupancy: f64,
}

/// Detector evidence for one frozen control.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchTransientControlEvidence {
    /// Frozen control index.
    pub control: usize,
    /// Evidence for all `64` decision anchors.
    pub anchors: Vec<StretchTransientAnchorEvidence>,
    /// Logical source-frame peak positions.
    pub peaks: Vec<usize>,
    /// Nearest-peak distance for each declared event, or no value when absent.
    pub event_offsets: Vec<Option<usize>>,
    /// Reflected reads and non-finite values.
    pub structural_counts: [usize; 2],
    /// Input, mask, ratio, peak, and complete evidence hashes.
    pub hashes: [u64; 5],
}

/// Complete release-only mixed-phase transient detector report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchTransientEvidenceReview {
    /// Evidence for all twelve frozen controls.
    pub controls: Vec<StretchTransientControlEvidence>,
    /// False-positive, event, dense, mixed, equivalence, perturbation, and structural failures.
    pub gate_failures: [usize; 7],
    /// Maximum occupancy change under deterministic perturbation.
    pub maximum_perturbation_change: f64,
    /// Occupancy change under perturbation for each frozen control.
    pub perturbation_changes: Vec<f64>,
    /// Maximum matched-peak displacement in decision anchors.
    pub maximum_peak_displacement: usize,
    /// Matched-peak displacement per control, absent when peak counts differ.
    pub peak_displacements: Vec<Option<usize>>,
    /// Controls whose perturbation changed the number of peaks.
    pub unmatched_perturbation_peaks: usize,
    /// Maximum gain/polarity/stereo occupancy difference.
    pub maximum_equivalence_error: f64,
    /// Maximum gain/polarity/stereo occupancy difference per control.
    pub equivalence_errors: Vec<f64>,
    /// Gain/polarity/stereo variants whose peak indices changed.
    pub equivalence_peak_failures: usize,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchTransientEvidenceDirection,
}

/// Direction selected by the mixed-phase distribution audit.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchMixedPhaseDistributionDirection {
    /// At least one frozen audit pair separates every event and negative family.
    Calibratable,
    /// Structurally valid event and negative distributions overlap.
    Overlapping,
    /// Cell accounting, finiteness, equivalence, or repeat evidence failed.
    StructuralFailure,
}

/// One normalized-magnitude band from the mixed-phase distribution audit.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMixedPhaseBandEvidence {
    /// Inclusive band index in ascending normalized magnitude.
    pub band: usize,
    /// Whether the samples belong to a declared-event neighborhood.
    pub event: bool,
    /// Assigned cell count.
    pub cell_count: usize,
    /// Sum of anchor-frame magnitudes.
    pub magnitude_sum: f64,
    /// Mixed-phase quantiles at the nine frozen probabilities.
    pub quantiles: [f64; 9],
}

/// Distribution evidence for one frozen control and perturbation state.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMixedPhaseControlEvidence {
    /// Frozen control index.
    pub control: usize,
    /// Whether deterministic relative-noise perturbation was applied.
    pub perturbed: bool,
    /// Ten event/non-event normalized-magnitude bands.
    pub bands: Vec<StretchMixedPhaseBandEvidence>,
    /// Nonzero-energy cells, assigned cells, reflected reads, and non-finite values.
    pub structural_counts: [usize; 4],
    /// Input and distribution hashes.
    pub hashes: [u64; 2],
}

/// One cutoff/radius pair in the fixed mixed-phase audit lattice.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMixedPhaseAuditPairEvidence {
    /// Normalized-magnitude lower cutoff.
    pub magnitude_cutoff: f64,
    /// Radius around ideal impulsive mixed phase `1`.
    pub mixed_phase_radius: f64,
    /// Minimum event recall for impulse, dense, boundary, and mixed families.
    pub event_recall: [f64; 4],
    /// Maximum leakage for steady, chirp, and noise families.
    pub negative_leakage: [f64; 3],
    /// Whether this pair meets the frozen separation limits.
    pub separates: bool,
}

/// Complete release-only mixed-phase distribution audit.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMixedPhaseDistributionReview {
    /// Base and perturbed evidence for all twelve frozen controls.
    pub controls: Vec<StretchMixedPhaseControlEvidence>,
    /// The fixed five-by-five audit lattice.
    pub audit_pairs: Vec<StretchMixedPhaseAuditPairEvidence>,
    /// Accounting, quantile-order, non-finite, and equivalence failures.
    pub structural_failures: [usize; 4],
    /// Maximum normalized distribution-signature equivalence error.
    pub maximum_equivalence_error: f64,
    /// Maximum gain, polarity, and stereo signature error per control.
    pub equivalence_errors: Vec<f64>,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchMixedPhaseDistributionDirection,
}

/// Direction selected by median-HPSS evidence measurement.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchMedianHpssDirection {
    /// Every evidence gate passes and occupancy mapping may be contracted.
    OccupancyMappingContract,
    /// At least one evidence gate fails and operator review is required.
    OperatorReview,
}

/// Median-HPSS evidence at one decision anchor.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMedianHpssAnchorEvidence {
    /// Logical source-frame anchor.
    pub anchor: usize,
    /// Linked, harmonic-median, percussive-median, and masked magnitude sums.
    pub magnitude_sums: [f64; 4],
    /// Magnitude-weighted soft percussive occupancy.
    pub occupancy: f64,
}

/// Median-HPSS evidence for one frozen control.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMedianHpssControlEvidence {
    /// Frozen control index.
    pub control: usize,
    /// Evidence for all `64` decision anchors.
    pub anchors: Vec<StretchMedianHpssAnchorEvidence>,
    /// Logical source-frame evidence peaks.
    pub peaks: Vec<usize>,
    /// Nearest-peak distance for each declared event, or no value when absent.
    pub event_offsets: Vec<Option<usize>>,
    /// Reflected sample reads, reflected median cells, and non-finite values.
    pub structural_counts: [usize; 3],
    /// Input, magnitude, median, mask, occupancy, peak, and complete hashes.
    pub hashes: [u64; 7],
}

/// Complete release-only median-HPSS evidence report.
#[cfg(all(test, not(debug_assertions)))]
#[derive(Clone, Debug, PartialEq)]
pub struct StretchMedianHpssReview {
    /// Evidence for all twelve frozen controls.
    pub controls: Vec<StretchMedianHpssControlEvidence>,
    /// False-positive, event, dense, mixed, equivalence, perturbation, and structural failures.
    pub gate_failures: [usize; 7],
    /// Maximum occupancy change under deterministic perturbation.
    pub maximum_perturbation_change: f64,
    /// Occupancy change under perturbation for each control.
    pub perturbation_changes: Vec<f64>,
    /// Maximum matched-peak displacement in decision anchors.
    pub maximum_peak_displacement: usize,
    /// Matched-peak displacement per control, absent when peak counts differ.
    pub peak_displacements: Vec<Option<usize>>,
    /// Controls whose perturbation changed the number of peaks.
    pub unmatched_perturbation_peaks: usize,
    /// Maximum gain, polarity, or linked-stereo occupancy difference.
    pub maximum_equivalence_error: f64,
    /// Maximum equivalence difference per control.
    pub equivalence_errors: Vec<f64>,
    /// Variants whose peak indices changed.
    pub equivalence_peak_failures: usize,
    /// Stable aggregate evidence hash.
    pub evidence_hash: u64,
    /// Selected continuation direction.
    pub direction: StretchMedianHpssDirection,
}

/// Numerical evidence for one bounded Hermitian Jacobi solve.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridJacobiEvidence {
    /// Matrix dimension.
    pub size: usize,
    /// Completed sweeps and applied rotations.
    pub sweeps_and_rotations: [usize; 2],
    /// Whether the frozen off-diagonal tolerance was reached.
    pub converged: bool,
    /// Hermitian error and final off-diagonal ratio.
    pub structural_errors: [f64; 2],
    /// Eigenpair residual, orthogonality, trace, and Frobenius errors.
    pub proof_errors: [f64; 4],
    /// Stable eigenvalue and eigenvector hashes.
    pub hashes: [u64; 2],
    /// Smallest and largest eigenvalues.
    pub extrema: [f64; 2],
}

/// Complete analytic and alias-matrix Hermitian Jacobi proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCommonGridJacobiReview {
    /// Analytic scalar, two-by-two, diagonal, repeated, and clustered controls.
    pub controls: Vec<StretchCommonGridJacobiEvidence>,
    /// All thirty-three frozen alias-block matrices.
    pub alias_blocks: Vec<StretchCommonGridJacobiEvidence>,
    /// Maximum residual, orthogonality, trace, and Frobenius errors.
    pub maximum_errors: [f64; 4],
    /// Stable complete evidence hash.
    pub evidence_hash: u64,
    /// Whether every frozen proof gate passed.
    pub passed: bool,
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
