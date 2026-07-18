use std::fs;

use super::super::{
    metrics::{self, evaluate},
    relation_repair::transform::local_residuals,
    SAMPLE_RATE,
};
use super::{
    add_render_hashes, inputs::PreparedRow, mechanics_measurement, mix, specimen::RubberBand, Run,
    StereoRow,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    coherent_representation, render,
};

pub(super) fn run(
    specimen: &RubberBand,
    root: &std::path::Path,
    prepared: &[PreparedRow],
    mechanics_inputs: &[Vec<f64>; 3],
) -> Run {
    fs::create_dir_all(root).expect("create comparator run root");
    let trim = coherent_representation::source_geometry(SAMPLE_RATE)[0];
    let mut rows = Vec::with_capacity(48);
    let mut hashes = [0xcbf2_9ce4_8422_2325_u64; 4];
    for (index, item) in prepared.iter().enumerate() {
        let rendered = specimen.render(
            root,
            &format!("stereo-{index:02}"),
            &[&item.source[0], &item.source[1]],
            item.ratio,
            SAMPLE_RATE,
        );
        add_render_hashes(&mut hashes, &rendered);
        let candidate: [Vec<f64>; 2] = rendered.channels.try_into().expect("stereo render");
        let input: [Vec<f64>; 2] = rendered.input.try_into().expect("stereo input");
        let current = render::linked([&input[0], &input[1]], item.ratio, SAMPLE_RATE);
        let source_trim = ((trim as f64 / item.ratio).ceil() as usize).min(item.source_frames / 3);
        let output_trim = trim.min(candidate[0].len() / 3);
        let local_residuals = local_residuals(&input, &current.channels, &candidate);
        let local_windows_improved = local_residuals[0]
            .iter()
            .zip(local_residuals[1])
            .filter(|(before, after)| *after < **before)
            .count();
        let maximum_local_residuals =
            local_residuals.map(|values| values.into_iter().fold(0.0_f64, f64::max));
        let row = StereoRow {
            ratio: item.ratio,
            source_frames: item.source_frames,
            phase: item.phase,
            bin_aligned: item.bin_aligned,
            control: item.kind.name(),
            whole: evaluate(&input, &candidate, item.frequency, SAMPLE_RATE),
            interior: evaluate(
                &metrics::crop(&input, source_trim),
                &metrics::crop(&candidate, output_trim),
                item.frequency,
                SAMPLE_RATE,
            ),
            structural_failures: 0,
            local_windows_improved,
            maximum_local_residuals,
            local_residuals,
            input_hash: rendered.input_hash,
            output_hash: rendered.output_hash,
        };
        mix_metrics(&mut hashes[3], &row);
        rows.push(row);
    }
    let mechanics_errors =
        mechanics_measurement::measure(specimen, root, mechanics_inputs, &mut hashes);
    for error in mechanics_errors {
        mix(&mut hashes[3], error.to_bits());
    }
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325;
    for hash in hashes {
        mix(&mut evidence_hash, hash);
    }
    Run {
        rows,
        mechanics_errors,
        input_hash: hashes[0],
        output_hash: hashes[1],
        command_hash: hashes[2],
        measurement_hash: hashes[3],
        evidence_hash,
    }
}

fn mix_metrics(hash: &mut u64, row: &StereoRow) {
    for metric in [row.whole, row.interior] {
        for value in [
            metric.ipd_error_radians,
            metric.mid_side_delta_db,
            metric.correlation_delta,
            metric.relation_residual,
        ] {
            mix(hash, value.to_bits());
        }
    }
    mix(hash, row.local_windows_improved as u64);
    for value in row.maximum_local_residuals {
        mix(hash, value.to_bits());
    }
    for values in row.local_residuals {
        for value in values {
            mix(hash, value.to_bits());
        }
    }
}
