use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum MaterialPhaseDirection {
    ListeningCheckpoint,
    ArchitectureReview,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct SyntheticReview {
    pub(in crate::frequency_adaptive) state_counts: StateCounts,
    pub(in crate::frequency_adaptive) structural_failures: usize,
    pub(in crate::frequency_adaptive) hidden_gain_failures: usize,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct MaterialPhaseReview {
    pub(in crate::frequency_adaptive) synthetic: SyntheticReview,
    pub(in crate::frequency_adaptive) stereo: PeakRegionReview,
    pub(in crate::frequency_adaptive) mechanics: SharedRotationMechanicsReview,
    pub(in crate::frequency_adaptive) corpus: SharedRotationCorpusReview,
    pub(in crate::frequency_adaptive) direction: MaterialPhaseDirection,
}

pub(in crate::frequency_adaptive) fn review() -> MaterialPhaseReview {
    let synthetic = synthetic_review();
    let stereo = peak_region_feasibility::review_candidate(
        "stretch-frequency-adaptive-material-phase",
        stereo_adapter,
    );
    let mechanics = mechanics::review(render);
    let corpus = corpus::review(render, "frequency-adaptive-material-phase");
    let mechanics_pass = mechanics.repeated
        && mechanics.structural_failures == 0
        && mechanics.identity_mismatches == 0
        && mechanics.errors.iter().all(|error| *error <= 1.0e-12)
        && mechanics.silent_peer_peak == 0.0
        && mechanics.states.tracked > 0
        && mechanics.states.reset > 0
        && mechanics.states.silent > 0
        && mechanics.states.shoulder > 0
        && mechanics.states.locked > 0
        && mechanics.states.diffuse > 0;
    let passed = synthetic.repeated
        && synthetic.structural_failures == 0
        && synthetic.hidden_gain_failures == 0
        && stereo.repeated
        && stereo.candidate_failures == 0
        && stereo.local_consistency_failures == 0
        && mechanics_pass
        && corpus.repeated
        && corpus.candidate_hard_failures == 0
        && corpus.row_complete_regressions == 0;
    let direction = if passed {
        MaterialPhaseDirection::ListeningCheckpoint
    } else {
        MaterialPhaseDirection::ArchitectureReview
    };
    write_summary(&synthetic, &stereo, &mechanics, &corpus, direction);
    MaterialPhaseReview {
        synthetic,
        stereo,
        mechanics,
        corpus,
        direction,
    }
}

fn stereo_adapter(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    let rendered = render(inputs, ratio, sample_rate);
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: 0,
        shared_fallback: 0,
        unilateral_non_silent_completions: 0,
        reference_bins: [0; 2],
        active_reference_ties: 0,
        reference_switches: 0,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.regions,
            rendered.states.tracked,
            rendered.states.reset,
            rendered.states.silent,
        ],
        tracked_peak_phase_trace: Default::default(),
        hash: rendered.hash,
    }
}

fn synthetic_review() -> SyntheticReview {
    let run = || {
        let sample_rate = 48_000;
        let frames = 16_384;
        let silence = vec![0.0; frames];
        let tone = (0..frames)
            .map(|index| {
                (std::f64::consts::TAU * 440.0 * index as f64 / sample_rate as f64).sin() * 0.25
            })
            .collect::<Vec<_>>();
        let noise = (0..frames)
            .map(|index| deterministic_unit(index, Scale::Middle, index % 97) * 0.1)
            .collect::<Vec<_>>();
        let mut impulse = vec![0.0; frames];
        impulse[frames / 2] = 1.0;
        let mixed = tone
            .iter()
            .zip(&noise)
            .map(|(tone, noise)| tone + noise)
            .collect::<Vec<_>>();
        let transient = tone
            .iter()
            .zip(&impulse)
            .map(|(tone, impulse)| tone + impulse)
            .collect::<Vec<_>>();
        let mut counts = StateCounts::default();
        let mut structural_failures = 0;
        let mut hidden_gain_failures = 0;
        let mut hash = HASH_OFFSET;
        for source in [&silence, &tone, &noise, &impulse, &mixed, &transient] {
            for ratio in RATIOS {
                let rendered = render([source, source], ratio, sample_rate);
                structural_failures +=
                    rendered.uncovered + rendered.non_finite + rendered.boundary_failures;
                hidden_gain_failures += usize::from(
                    source.iter().any(|sample| *sample != 0.0)
                        && rendered.channels[0]
                            .iter()
                            .map(|sample| sample.abs())
                            .fold(0.0, f64::max)
                            > 4.0,
                );
                add_counts(&mut counts, rendered.states);
                hash = (hash ^ rendered.hash).wrapping_mul(0x100_0000_01b3);
            }
        }
        (counts, structural_failures, hidden_gain_failures, hash)
    };
    let first = run();
    let second = run();
    SyntheticReview {
        state_counts: first.0,
        structural_failures: first.1,
        hidden_gain_failures: first.2,
        repeated: first == second,
        hash: first.3,
    }
}

fn add_counts(target: &mut StateCounts, source: StateCounts) {
    target.tracked += source.tracked;
    target.reset += source.reset;
    target.silent += source.silent;
    target.regions += source.regions;
    target.owner_switches += source.owner_switches;
    target.shoulder += source.shoulder;
    target.locked += source.locked;
    target.diffuse += source.diffuse;
}

fn write_summary(
    synthetic: &SyntheticReview,
    stereo: &PeakRegionReview,
    mechanics: &SharedRotationMechanicsReview,
    corpus: &SharedRotationCorpusReview,
    direction: MaterialPhaseDirection,
) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-frequency-adaptive-material-phase");
    fs::create_dir_all(&root).expect("create material-phase report directory");
    let report = format!(
        "direction\t{direction:?}\ngeometry\t{},{},{}\ncrossovers_hz\t{},{}\nsynthetic_failures\t{},{}\nsynthetic_states\t{},{},{},{},{},{},{},{}\nsynthetic_hash\t{:016x}\nstereo_failures\t{}\nstereo_local_failures\t{}\nstereo_hash\t{:016x}\nmechanics_failures\t{},{}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nmechanics_hash\t{:016x}\nmono_hard_failures\t{}\nmono_row_complete_regressions\t{}\nmono_hash\t{:016x}\n",
        SUPPORT_FRAMES[0], SUPPORT_FRAMES[1], SUPPORT_FRAMES[2],
        CROSSOVER_HZ[0], CROSSOVER_HZ[1], synthetic.structural_failures,
        synthetic.hidden_gain_failures, synthetic.state_counts.tracked,
        synthetic.state_counts.reset, synthetic.state_counts.silent,
        synthetic.state_counts.regions, synthetic.state_counts.shoulder,
        synthetic.state_counts.locked, synthetic.state_counts.diffuse,
        synthetic.state_counts.owner_switches, synthetic.hash,
        stereo.candidate_failures, stereo.local_consistency_failures,
        stereo.evidence_hash, mechanics.structural_failures,
        mechanics.identity_mismatches, mechanics.errors[0], mechanics.errors[1],
        mechanics.errors[2], mechanics.errors[3], mechanics.errors[4], mechanics.hash,
        corpus.candidate_hard_failures, corpus.row_complete_regressions, corpus.hash,
    );
    fs::write(root.join("proof-summary.tsv"), report).expect("write material-phase summary");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires frozen exact-source and Rubber Band corpus pack"]
    fn frequency_adaptive_material_phase_stage_b_objective_gate() {
        let result = review();
        eprintln!("frequency_adaptive_material_phase_stage_b {result:#?}");
        assert!(result.synthetic.repeated);
        assert_eq!(result.synthetic.structural_failures, 0);
        assert_eq!(result.synthetic.hidden_gain_failures, 0);
        assert!(result.synthetic.state_counts.silent > 0);
        assert!(result.synthetic.state_counts.shoulder > 0);
        assert!(result.synthetic.state_counts.locked > 0);
        assert!(result.synthetic.state_counts.diffuse > 0);
        assert!(result.stereo.repeated);
        assert_eq!(result.stereo.rows.len(), 48);
        assert_eq!(result.stereo.candidate_failures, 0);
        assert_eq!(result.stereo.local_consistency_failures, 0);
        assert!(result.mechanics.repeated);
        assert_eq!(result.mechanics.structural_failures, 0);
        assert_eq!(result.mechanics.identity_mismatches, 0);
        assert!(result
            .mechanics
            .errors
            .iter()
            .all(|error| *error <= 1.0e-12));
        assert_eq!(result.mechanics.silent_peer_peak, 0.0);
        assert!(result.corpus.repeated);
        assert_eq!(result.corpus.rows.len(), 6);
        assert_eq!(result.corpus.candidate_hard_failures, 0);
        assert_eq!(result.corpus.row_complete_regressions, 0);
        assert_eq!(
            result.direction,
            MaterialPhaseDirection::ListeningCheckpoint
        );
    }
}
