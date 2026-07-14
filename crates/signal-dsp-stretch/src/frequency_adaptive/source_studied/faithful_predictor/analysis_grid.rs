use super::{
    chord_control, hash_samples, pinned_source, render_stage_with_grid, spectrum_metrics,
    TraceStage, TransformGrid, CHORD_FREQUENCIES, SAMPLE_RATE,
};

const SOURCE_RELATIVE_CEILING_DB: f64 = 1.0;
const IDENTITY_CEILING: f64 = 1.0e-10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ModifiedGridToneEvidence {
    pub(in crate::frequency_adaptive) frequency_hz: f64,
    pub(in crate::frequency_adaptive) pinned_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) baseline_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) output_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) minus_pinned_db: f64,
    pub(in crate::frequency_adaptive) minus_baseline_db: f64,
    pub(in crate::frequency_adaptive) peak_error_hz: f64,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum ModifiedGridDirection {
    SourceParityClosed,
    SourceParityImproved,
    GridRejected,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct ModifiedGridReview {
    pub(in crate::frequency_adaptive) geometry: [usize; 4],
    pub(in crate::frequency_adaptive) identity_maximum_error: f64,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 6],
    pub(in crate::frequency_adaptive) tones: [ModifiedGridToneEvidence; 4],
    pub(in crate::frequency_adaptive) chord_pinned_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_baseline_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_output_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_minus_pinned_db: f64,
    pub(in crate::frequency_adaptive) chord_minus_baseline_db: f64,
    pub(in crate::frequency_adaptive) chord_peak_error_hz: f64,
    pub(in crate::frequency_adaptive) chord_hash: u64,
    pub(in crate::frequency_adaptive) baseline_source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: ModifiedGridDirection,
}

pub(in crate::frequency_adaptive) fn review() -> ModifiedGridReview {
    let pinned = pinned_source::review();
    assert_eq!(pinned.source_relative_failures, [3, 1]);
    let first = run(&pinned);
    let second = run(&pinned);
    let repeated = first == second;
    let failures = first.source_relative_failures.into_iter().sum::<usize>();
    let baseline_failures = first
        .baseline_source_relative_failures
        .into_iter()
        .sum::<usize>();
    let structurally_valid = first.structural_failures == [0; 6]
        && first.identity_maximum_error <= IDENTITY_CEILING
        && repeated;
    ModifiedGridReview {
        repeated,
        direction: if structurally_valid && failures == 0 {
            ModifiedGridDirection::SourceParityClosed
        } else if structurally_valid && failures < baseline_failures {
            ModifiedGridDirection::SourceParityImproved
        } else {
            ModifiedGridDirection::GridRejected
        },
        ..first
    }
}

fn run(pinned: &pinned_source::PinnedSourceReview) -> ModifiedGridReview {
    let identity_input = quantized(chord_control());
    let identity = render_stage_with_grid(
        &identity_input,
        1.0,
        SAMPLE_RATE,
        TraceStage::Analysis,
        TransformGrid::ModifiedHalfBin,
    );
    let identity_maximum_error = identity
        .samples
        .iter()
        .zip(&identity_input)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max);
    let mut structural_failures = [
        usize::from(identity.samples.len() != identity_input.len()),
        identity.non_finite,
        identity.uncovered,
        identity.boundary_failures,
        usize::from(identity_maximum_error > IDENTITY_CEILING),
        0,
    ];

    let tones = std::array::from_fn(|tone| {
        let frequency = CHORD_FREQUENCIES[tone];
        let amplitude = 0.16 - tone as f64 * 0.015;
        let input = quantized(
            (0..SAMPLE_RATE * 2)
                .map(|index| {
                    amplitude
                        * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64)
                            .sin()
                })
                .collect(),
        );
        let render = modified_render(&input);
        structural_failures[5] += render.structural_failures;
        let metrics = spectrum_metrics(
            &render.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        ModifiedGridToneEvidence {
            frequency_hz: frequency,
            pinned_out_of_band_db: pinned.tones[tone].output_out_of_band_db,
            baseline_out_of_band_db: pinned.tones[tone].signal_out_of_band_db,
            output_out_of_band_db: metrics.out_of_band_db,
            minus_pinned_db: metrics.out_of_band_db - pinned.tones[tone].output_out_of_band_db,
            minus_baseline_db: metrics.out_of_band_db - pinned.tones[tone].signal_out_of_band_db,
            peak_error_hz: metrics.maximum_peak_error_hz,
            hash: render.hash,
        }
    });

    let chord_input = quantized(chord_control());
    let chord = modified_render(&chord_input);
    structural_failures[5] += chord.structural_failures;
    let chord_metrics = spectrum_metrics(
        &chord.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
        &CHORD_FREQUENCIES,
    );
    let source_relative_failures = [
        tones
            .iter()
            .filter(|tone| tone.minus_pinned_db > SOURCE_RELATIVE_CEILING_DB)
            .count(),
        usize::from(
            chord_metrics.out_of_band_db - pinned.chord_output_out_of_band_db
                > SOURCE_RELATIVE_CEILING_DB,
        ),
    ];
    ModifiedGridReview {
        geometry: [960, 240, 1_024, 512],
        identity_maximum_error,
        structural_failures,
        tones,
        chord_pinned_out_of_band_db: pinned.chord_output_out_of_band_db,
        chord_baseline_out_of_band_db: pinned.chord_signal_out_of_band_db,
        chord_output_out_of_band_db: chord_metrics.out_of_band_db,
        chord_minus_pinned_db: chord_metrics.out_of_band_db - pinned.chord_output_out_of_band_db,
        chord_minus_baseline_db: chord_metrics.out_of_band_db - pinned.chord_signal_out_of_band_db,
        chord_peak_error_hz: chord_metrics.maximum_peak_error_hz,
        chord_hash: chord.hash,
        baseline_source_relative_failures: pinned.source_relative_failures,
        source_relative_failures,
        repeated: false,
        direction: ModifiedGridDirection::GridRejected,
    }
}

struct ModifiedRender {
    samples: Vec<f64>,
    structural_failures: usize,
    hash: u64,
}

fn modified_render(input: &[f64]) -> ModifiedRender {
    let render = render_stage_with_grid(
        input,
        2.0,
        SAMPLE_RATE,
        TraceStage::Complete,
        TransformGrid::ModifiedHalfBin,
    );
    ModifiedRender {
        structural_failures: usize::from(render.samples.len() != input.len() * 2)
            + render.non_finite
            + render.uncovered
            + render.boundary_failures,
        hash: hash_samples(&render.samples),
        samples: render.samples,
    }
}

fn quantized(samples: Vec<f64>) -> Vec<f64> {
    samples
        .into_iter()
        .map(|sample| {
            let value =
                (sample.clamp(-1.0, f64::from(i16::MAX) / 32_768.0) * 32_768.0).round() as i16;
            f64::from(value) / 32_768.0
        })
        .collect()
}
