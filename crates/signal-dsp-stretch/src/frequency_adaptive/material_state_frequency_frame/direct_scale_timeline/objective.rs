use std::{fs, path::PathBuf};

use super::{geometry::CAPACITY, *};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    quality::gate_calibration::peak_region_feasibility,
    render::{StereoRender, TrackedPeakPhaseTrace},
};

const SAMPLE_RATE: usize = 48_000;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Debug, PartialEq)]
struct SyntheticReview {
    structural_failures: usize,
    mechanics_errors: [f64; 4],
    state_counts: [usize; 5],
    borrowed_regions: usize,
    local_regions: usize,
    maximum_pending_ticks: usize,
    maximum_guidance_ticks: usize,
    maximum_output_samples: usize,
    non_finite_values: usize,
    repeated: bool,
    hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct StereoReview {
    rows: usize,
    calibrated_failures: usize,
    improved_windows: usize,
    signal_relative_local_failures: usize,
    maximum_normalized_gram_residual: f64,
    structural_failures: usize,
    repeated: bool,
    evidence_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct SyntheticRun {
    structural_failures: usize,
    mechanics_errors: [f64; 4],
    state_counts: [usize; 5],
    borrowed_regions: usize,
    local_regions: usize,
    maximum_pending_ticks: usize,
    maximum_guidance_ticks: usize,
    maximum_output_samples: usize,
    non_finite_values: usize,
    hash: u64,
}

fn synthetic_review() -> SyntheticReview {
    let first = synthetic_run();
    let second = synthetic_run();
    SyntheticReview {
        structural_failures: first.structural_failures,
        mechanics_errors: first.mechanics_errors,
        state_counts: first.state_counts,
        borrowed_regions: first.borrowed_regions,
        local_regions: first.local_regions,
        maximum_pending_ticks: first.maximum_pending_ticks,
        maximum_guidance_ticks: first.maximum_guidance_ticks,
        maximum_output_samples: first.maximum_output_samples,
        non_finite_values: first.non_finite_values,
        repeated: first == second,
        hash: first.hash,
    }
}

fn synthetic_run() -> SyntheticRun {
    let frames = 16_384;
    let silence = vec![0.0; frames];
    let tone = (0..frames)
        .map(|index| {
            (std::f64::consts::TAU * 440.0 * index as f64 / SAMPLE_RATE as f64).sin() * 0.25
        })
        .collect::<Vec<_>>();
    let noise = (0..frames)
        .map(|index| deterministic_noise(index) * 0.1)
        .collect::<Vec<_>>();
    let mut impulse = vec![0.0; frames];
    impulse[frames / 2] = 1.0;
    let mixed = sum(&tone, &noise);
    let transient = sum(&tone, &impulse);
    let sources = [&silence, &tone, &noise, &impulse, &mixed, &transient];
    let mut result = SyntheticRun {
        structural_failures: 0,
        mechanics_errors: [0.0; 4],
        state_counts: [0; 5],
        borrowed_regions: 0,
        local_regions: 0,
        maximum_pending_ticks: 0,
        maximum_guidance_ticks: 0,
        maximum_output_samples: 0,
        non_finite_values: 0,
        hash: HASH_OFFSET,
    };
    for source in sources {
        for ratio in RATIOS {
            let rendered = render::render([source, source], ratio, SAMPLE_RATE);
            result.structural_failures += rendered.uncovered
                + rendered.boundary_failures
                + usize::from(rendered.target_length != rendered.channels[0].len())
                + usize::from(rendered.target_length != rendered.channels[1].len())
                + usize::from(rendered.maximum_pending_ticks > PENDING_TICKS)
                + usize::from(rendered.maximum_guidance_ticks != GUIDANCE_TICKS)
                + usize::from(rendered.maximum_output_samples > CAPACITY.output_samples);
            result.non_finite_values += rendered.non_finite;
            for (target, count) in result.state_counts.iter_mut().zip(rendered.states) {
                *target += count;
            }
            result.borrowed_regions += rendered.borrowed_regions;
            result.local_regions += rendered.local_regions;
            result.maximum_pending_ticks = result
                .maximum_pending_ticks
                .max(rendered.maximum_pending_ticks);
            result.maximum_guidance_ticks = result
                .maximum_guidance_ticks
                .max(rendered.maximum_guidance_ticks);
            result.maximum_output_samples = result
                .maximum_output_samples
                .max(rendered.maximum_output_samples);
            hash_u64(&mut result.hash, rendered.hash);
        }
    }
    result.mechanics_errors = mechanics_errors(&tone, &noise);
    for error in result.mechanics_errors {
        hash_u64(&mut result.hash, error.to_bits());
    }
    result
}

fn stereo_review() -> StereoReview {
    let review = peak_region_feasibility::review_candidate(
        "stretch-direct-scale-timeline-stereo",
        stereo_adapter,
    );
    StereoReview {
        rows: review.rows.len(),
        calibrated_failures: review.candidate_failures,
        improved_windows: review
            .rows
            .iter()
            .map(|row| row.local_windows_improved)
            .sum(),
        signal_relative_local_failures: review.local_consistency_failures,
        maximum_normalized_gram_residual: review
            .rows
            .iter()
            .map(|row| row.maximum_local_residuals[1])
            .fold(0.0_f64, f64::max),
        structural_failures: review.rows.iter().map(|row| row.structural_failures).sum(),
        repeated: review.repeated,
        evidence_hash: review.evidence_hash,
    }
}

fn stereo_adapter(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    let rendered = render::render(inputs, ratio, sample_rate);
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: rendered.borrowed_regions,
        shared_fallback: rendered.local_regions,
        unilateral_non_silent_completions: 0,
        reference_bins: [0; 2],
        active_reference_ties: 0,
        reference_switches: rendered.owner_switches,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.borrowed_regions + rendered.local_regions,
            rendered.states[TerminalState::Locked.index()],
            rendered.states[TerminalState::Reset.index()]
                + rendered.states[TerminalState::Attack.index()],
            0,
        ],
        tracked_peak_phase_trace: TrackedPeakPhaseTrace::default(),
        hash: rendered.hash,
    }
}

fn mechanics_errors(primary: &[f64], secondary: &[f64]) -> [f64; 4] {
    let silence = vec![0.0; primary.len()];
    let mut errors = [0.0_f64; 4];
    for ratio in RATIOS {
        let duplicate = render::render([primary, primary], ratio, SAMPLE_RATE).channels;
        let mono = render::render([primary, &silence], ratio, SAMPLE_RATE).channels;
        let ordinary = render::render([primary, secondary], ratio, SAMPLE_RATE).channels;
        let swapped = render::render([secondary, primary], ratio, SAMPLE_RATE).channels;
        errors[0] = errors[0].max(maximum_error(&duplicate[0], &duplicate[1]));
        errors[1] = errors[1].max(maximum_error(&duplicate[0], &mono[0]));
        errors[2] = errors[2].max(maximum_norm(&mono[1]));
        errors[3] = errors[3]
            .max(maximum_error(&ordinary[0], &swapped[1]))
            .max(maximum_error(&ordinary[1], &swapped[0]));
    }
    errors
}

fn deterministic_noise(index: usize) -> f64 {
    let mut value = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    (value as f64 / u64::MAX as f64) * 2.0 - 1.0
}

fn sum(left: &[f64], right: &[f64]) -> Vec<f64> {
    left.iter()
        .zip(right)
        .map(|(left, right)| left + right)
        .collect()
}

fn maximum_error(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f64, f64::max)
}

fn maximum_norm(samples: &[f64]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max)
}

fn report_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/stretch-direct-scale-timeline")
}

fn write_synthetic(review: &SyntheticReview) {
    let root = report_root();
    fs::create_dir_all(&root).expect("create direct objective report directory");
    let text = format!(
        "stage\tsynthetic\nstructural_failures\t{}\nmechanics_errors\t{:e},{:e},{:e},{:e}\nstate_counts\t{:?}\nregions\t{},{}\nmaximum_live\t{},{},{}\nnon_finite\t{}\nrepeated\t{}\nhash\t{:016x}\n",
        review.structural_failures,
        review.mechanics_errors[0],
        review.mechanics_errors[1],
        review.mechanics_errors[2],
        review.mechanics_errors[3],
        review.state_counts,
        review.borrowed_regions,
        review.local_regions,
        review.maximum_pending_ticks,
        review.maximum_guidance_ticks,
        review.maximum_output_samples,
        review.non_finite_values,
        review.repeated,
        review.hash,
    );
    fs::write(root.join("synthetic.tsv"), text).expect("write direct synthetic report");
}

fn write_stereo(review: &StereoReview) {
    let root = report_root();
    fs::create_dir_all(&root).expect("create direct objective report directory");
    let text = format!(
        "stage\tstereo\nrows\t{}\ncalibrated_failures\t{}\nimproved_windows\t{}\nsignal_relative_local_failures\t{}\nmaximum_normalized_gram_residual\t{:.14}\nstructural_failures\t{}\nrepeated\t{}\nevidence_hash\t{:016x}\n",
        review.rows,
        review.calibrated_failures,
        review.improved_windows,
        review.signal_relative_local_failures,
        review.maximum_normalized_gram_residual,
        review.structural_failures,
        review.repeated,
        review.evidence_hash,
    );
    fs::write(root.join("stereo.tsv"), text).expect("write direct stereo report");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires one release-only direct synthetic objective run"]
    fn direct_scale_timeline_rule_31z_objective_synthetic_gate() {
        let review = synthetic_review();
        write_synthetic(&review);
        eprintln!("direct_scale_timeline_objective_synthetic {review:#?}");
        assert_eq!(review.structural_failures, 0, "{review:#?}");
        assert_eq!(review.non_finite_values, 0, "{review:#?}");
        assert!(review.repeated, "{review:#?}");
        assert!(
            review.mechanics_errors.iter().all(|error| *error <= 1.0e-6),
            "{review:#?}"
        );
        assert!(
            review.state_counts[TerminalState::Reset.index()] > 0,
            "{review:#?}"
        );
        assert!(
            review.state_counts[TerminalState::Attack.index()] > 0,
            "{review:#?}"
        );
        assert!(
            review.state_counts[TerminalState::Unlocked.index()] > 0,
            "{review:#?}"
        );
        assert!(
            review.state_counts[TerminalState::Locked.index()] > 0,
            "{review:#?}"
        );
        assert!(review.borrowed_regions > 0, "{review:#?}");
        assert!(review.local_regions > 0, "{review:#?}");
        assert_eq!(review.maximum_pending_ticks, PENDING_TICKS, "{review:#?}");
        assert_eq!(review.maximum_guidance_ticks, GUIDANCE_TICKS, "{review:#?}");
        assert!(
            review.maximum_output_samples <= CAPACITY.output_samples,
            "{review:#?}"
        );
    }

    #[test]
    #[ignore = "requires one release-only corrected 48-row direct stereo run"]
    fn direct_scale_timeline_rule_31z_objective_stereo_gate() {
        let review = stereo_review();
        write_stereo(&review);
        eprintln!("direct_scale_timeline_objective_stereo {review:#?}");
        assert_eq!(review.rows, 48, "{review:#?}");
        assert_eq!(review.structural_failures, 0, "{review:#?}");
        assert!(review.repeated, "{review:#?}");
        assert_eq!(review.calibrated_failures, 0, "{review:#?}");
        assert!(review.improved_windows >= 245, "{review:#?}");
        assert!(review.signal_relative_local_failures <= 13, "{review:#?}");
        assert!(
            review.maximum_normalized_gram_residual <= 0.017_446_938_152_60,
            "{review:#?}"
        );
    }
}
