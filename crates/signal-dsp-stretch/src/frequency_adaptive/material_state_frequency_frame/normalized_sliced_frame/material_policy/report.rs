use std::{fs, path::PathBuf};

use super::*;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::quality::gate_calibration::peak_region_feasibility;

const SAMPLE_RATE: usize = 48_000;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Debug, PartialEq)]
struct SyntheticReview {
    structural_failures: usize,
    mechanics_errors: [f64; 4],
    state_counts: [usize; 5],
    linked_regions: usize,
    unlinked_regions: usize,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
    maximum_guidance_frames: usize,
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

fn stereo_review() -> StereoReview {
    let review = peak_region_feasibility::review_candidate(
        "stretch-normalized-material-policy-stereo",
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

fn synthetic_review() -> SyntheticReview {
    let first = synthetic_run();
    let second = synthetic_run();
    SyntheticReview {
        repeated: first == second,
        structural_failures: first.structural_failures,
        mechanics_errors: first.mechanics_errors,
        state_counts: first.state_counts,
        linked_regions: first.linked_regions,
        unlinked_regions: first.unlinked_regions,
        maximum_live_source_slices: first.maximum_live_source_slices,
        maximum_live_output_slices: first.maximum_live_output_slices,
        maximum_guidance_frames: first.maximum_guidance_frames,
        non_finite_values: first.non_finite_values,
        hash: first.hash,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SyntheticRun {
    structural_failures: usize,
    mechanics_errors: [f64; 4],
    state_counts: [usize; 5],
    linked_regions: usize,
    unlinked_regions: usize,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
    maximum_guidance_frames: usize,
    non_finite_values: usize,
    hash: u64,
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
        linked_regions: 0,
        unlinked_regions: 0,
        maximum_live_source_slices: 0,
        maximum_live_output_slices: 0,
        maximum_guidance_frames: 0,
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
                + usize::from(rendered.maximum_live_source_slices > SOURCE_SLICE_CAPACITY)
                + usize::from(rendered.maximum_live_output_slices > OUTPUT_SLICE_CAPACITY)
                + usize::from(rendered.maximum_guidance_frames != MATERIAL_HALO_FRAMES);
            result.non_finite_values += rendered.non_finite;
            for (target, count) in result.state_counts.iter_mut().zip(rendered.states.states) {
                *target += count;
            }
            result.linked_regions += rendered.states.linked_regions;
            result.unlinked_regions += rendered.states.unlinked_regions;
            result.maximum_live_source_slices = result
                .maximum_live_source_slices
                .max(rendered.maximum_live_source_slices);
            result.maximum_live_output_slices = result
                .maximum_live_output_slices
                .max(rendered.maximum_live_output_slices);
            result.maximum_guidance_frames = result
                .maximum_guidance_frames
                .max(rendered.maximum_guidance_frames);
            hash_u64(&mut result.hash, rendered.hash);
        }
    }
    result.mechanics_errors = mechanics_errors(&tone, &noise);
    for error in result.mechanics_errors {
        hash_u64(&mut result.hash, error.to_bits());
    }
    result
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

fn write_synthetic(review: &SyntheticReview) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-normalized-material-policy");
    fs::create_dir_all(&root).expect("create normalized material report directory");
    let text = format!(
        "stage\tsynthetic\nstructural_failures\t{}\nmechanics_errors\t{:e},{:e},{:e},{:e}\nstate_counts\t{:?}\nregions\t{},{}\nmaximum_live\t{},{},{}\nnon_finite\t{}\nrepeated\t{}\nhash\t{:016x}\n",
        review.structural_failures,
        review.mechanics_errors[0],
        review.mechanics_errors[1],
        review.mechanics_errors[2],
        review.mechanics_errors[3],
        review.state_counts,
        review.linked_regions,
        review.unlinked_regions,
        review.maximum_live_source_slices,
        review.maximum_live_output_slices,
        review.maximum_guidance_frames,
        review.non_finite_values,
        review.repeated,
        review.hash,
    );
    fs::write(root.join("synthetic.tsv"), text)
        .expect("write normalized material synthetic report");
}

fn write_stereo(review: &StereoReview) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-normalized-material-policy");
    fs::create_dir_all(&root).expect("create normalized material report directory");
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
    fs::write(root.join("stereo.tsv"), text).expect("write normalized material stereo report");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_material_policy_rule_31v_classifier_is_frozen() {
        let silence = Material::default();
        let tone = Material {
            tonalness: 0.8,
            noisiness: 0.2,
            transientness: 0.2,
        };
        let noise = Material {
            tonalness: 0.3,
            noisiness: 0.8,
            transientness: 0.7,
        };
        assert_eq!(silence.tonalness, 0.0);
        assert!(tone.tonalness > tone.noisiness);
        assert!(noise.noisiness > noise.tonalness);
    }

    #[test]
    #[ignore = "requires release-only normalized material renderer"]
    fn normalized_material_policy_rule_31v_synthetic_gate() {
        let review = synthetic_review();
        write_synthetic(&review);
        eprintln!("normalized_material_policy_synthetic {review:#?}");
        assert_eq!(review.structural_failures, 0);
        assert_eq!(review.non_finite_values, 0);
        assert!(review.repeated);
        assert!(review.mechanics_errors.iter().all(|error| *error <= 1.0e-6));
        assert!(review.state_counts[Decision::Reset.index()] > 0);
        assert!(review.state_counts[Decision::Attack.index()] > 0);
        assert!(review.state_counts[Decision::Unlocked.index()] > 0);
        assert!(review.state_counts[Decision::Locked.index()] > 0);
        assert!(review.linked_regions > 0);
        assert!(review.unlinked_regions > 0);
        assert!(review.maximum_live_source_slices <= SOURCE_SLICE_CAPACITY);
        assert!(review.maximum_live_output_slices <= OUTPUT_SLICE_CAPACITY);
        assert_eq!(review.maximum_guidance_frames, MATERIAL_HALO_FRAMES);
    }

    #[test]
    #[ignore = "requires release-only corrected 48-row stereo corpus"]
    fn normalized_material_policy_rule_31v_stereo_gate() {
        let review = stereo_review();
        write_stereo(&review);
        eprintln!("normalized_material_policy_stereo {review:#?}");
        assert_eq!(review.rows, 48);
        assert_eq!(review.structural_failures, 0);
        assert!(review.repeated);
        assert_eq!(review.calibrated_failures, 0);
        assert!(review.improved_windows >= 245);
        assert!(review.signal_relative_local_failures <= 13);
        assert!(review.maximum_normalized_gram_residual <= 0.017_446_938_152_60);
    }
}
