use super::{
    analysis_grid, analysis_window, chord_control, hash_samples, pinned_source,
    render_stage_with_grid_and_window, spectrum_metrics, stage_trace, TraceStage, TransformGrid,
    CHORD_FREQUENCIES, SAMPLE_RATE,
};

const SOURCE_RELATIVE_CEILING_DB: f64 = 1.0;
const IDENTITY_CEILING: f64 = 1.0e-10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AnalysisInteractionEvidence {
    pub(in crate::frequency_adaptive) frequency_hz: f64,
    pub(in crate::frequency_adaptive) baseline_db: f64,
    pub(in crate::frequency_adaptive) grid_only_db: f64,
    pub(in crate::frequency_adaptive) window_only_db: f64,
    pub(in crate::frequency_adaptive) combined_db: f64,
    pub(in crate::frequency_adaptive) pinned_db: f64,
    pub(in crate::frequency_adaptive) interaction_db: f64,
    pub(in crate::frequency_adaptive) combined_minus_pinned_db: f64,
    pub(in crate::frequency_adaptive) peak_error_hz: f64,
    pub(in crate::frequency_adaptive) hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum AnalysisInteractionDirection {
    SourceParityClosed,
    SourceParityImproved,
    InteractionRejected,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct AnalysisInteractionReview {
    pub(in crate::frequency_adaptive) geometry: [usize; 4],
    pub(in crate::frequency_adaptive) identity_maximum_error: f64,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 6],
    pub(in crate::frequency_adaptive) tones: [AnalysisInteractionEvidence; 4],
    pub(in crate::frequency_adaptive) chord: AnalysisInteractionEvidence,
    pub(in crate::frequency_adaptive) baseline_source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) grid_source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) window_source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) source_relative_failures: [usize; 2],
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: AnalysisInteractionDirection,
}

pub(in crate::frequency_adaptive) fn review() -> AnalysisInteractionReview {
    let window = stage_trace::pinned_window();
    let pinned = pinned_source::review();
    let grid = analysis_grid::run(&pinned);
    let window_only = analysis_window::run(&window, &pinned);
    let first = run(&window.analysis, &pinned, &grid, &window_only);
    let second = run(&window.analysis, &pinned, &grid, &window_only);
    let repeated = first == second;
    let failures = first.source_relative_failures.into_iter().sum::<usize>();
    let baseline_failures = first
        .baseline_source_relative_failures
        .into_iter()
        .sum::<usize>();
    let structurally_valid =
        first.structural_failures == [0; 6] && first.identity_maximum_error <= IDENTITY_CEILING;
    AnalysisInteractionReview {
        repeated,
        direction: if repeated && structurally_valid && failures == 0 {
            AnalysisInteractionDirection::SourceParityClosed
        } else if repeated && structurally_valid && failures < baseline_failures {
            AnalysisInteractionDirection::SourceParityImproved
        } else {
            AnalysisInteractionDirection::InteractionRejected
        },
        ..first
    }
}

fn run(
    window: &[f64],
    pinned: &pinned_source::PinnedSourceReview,
    grid: &analysis_grid::ModifiedGridReview,
    window_only: &analysis_window::KaiserWindowReview,
) -> AnalysisInteractionReview {
    let identity_input = quantized(chord_control());
    let identity = render_stage_with_grid_and_window(
        &identity_input,
        1.0,
        SAMPLE_RATE,
        TraceStage::Analysis,
        TransformGrid::ModifiedHalfBin,
        window,
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
        let render = combined_render(&input, window);
        structural_failures[5] += render.structural_failures;
        let metrics = spectrum_metrics(
            &render.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        evidence(
            frequency,
            pinned.tones[tone].signal_out_of_band_db,
            grid.tones[tone].output_out_of_band_db,
            window_only.tones[tone].output_out_of_band_db,
            metrics.out_of_band_db,
            pinned.tones[tone].output_out_of_band_db,
            metrics.maximum_peak_error_hz,
            render.hash,
        )
    });

    let chord_input = quantized(chord_control());
    let chord_render = combined_render(&chord_input, window);
    structural_failures[5] += chord_render.structural_failures;
    let chord_metrics = spectrum_metrics(
        &chord_render.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
        &CHORD_FREQUENCIES,
    );
    let chord = evidence(
        0.0,
        pinned.chord_signal_out_of_band_db,
        grid.chord_output_out_of_band_db,
        window_only.chord_output_out_of_band_db,
        chord_metrics.out_of_band_db,
        pinned.chord_output_out_of_band_db,
        chord_metrics.maximum_peak_error_hz,
        chord_render.hash,
    );
    let source_relative_failures = [
        tones
            .iter()
            .filter(|tone| tone.combined_minus_pinned_db > SOURCE_RELATIVE_CEILING_DB)
            .count(),
        usize::from(chord.combined_minus_pinned_db > SOURCE_RELATIVE_CEILING_DB),
    ];

    AnalysisInteractionReview {
        geometry: [960, 240, 1_024, 512],
        identity_maximum_error,
        structural_failures,
        tones,
        chord,
        baseline_source_relative_failures: pinned.source_relative_failures,
        grid_source_relative_failures: grid.source_relative_failures,
        window_source_relative_failures: window_only.source_relative_failures,
        source_relative_failures,
        repeated: false,
        direction: AnalysisInteractionDirection::InteractionRejected,
    }
}

#[allow(clippy::too_many_arguments)]
fn evidence(
    frequency_hz: f64,
    baseline_db: f64,
    grid_only_db: f64,
    window_only_db: f64,
    combined_db: f64,
    pinned_db: f64,
    peak_error_hz: f64,
    hash: u64,
) -> AnalysisInteractionEvidence {
    AnalysisInteractionEvidence {
        frequency_hz,
        baseline_db,
        grid_only_db,
        window_only_db,
        combined_db,
        pinned_db,
        interaction_db: combined_db - grid_only_db - window_only_db + baseline_db,
        combined_minus_pinned_db: combined_db - pinned_db,
        peak_error_hz,
        hash,
    }
}

struct CombinedRender {
    samples: Vec<f64>,
    structural_failures: usize,
    hash: u64,
}

fn combined_render(input: &[f64], window: &[f64]) -> CombinedRender {
    let render = render_stage_with_grid_and_window(
        input,
        2.0,
        SAMPLE_RATE,
        TraceStage::Complete,
        TransformGrid::ModifiedHalfBin,
        window,
    );
    CombinedRender {
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
