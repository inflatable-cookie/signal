use super::*;

pub(super) fn forced_render(
    inputs: [&[f64]; CHANNEL_CAPACITY],
) -> ([Vec<f64>; 2], StateCounts, usize, usize) {
    let analysis = analyse_for_stage_a(inputs, SAMPLE_RATE_HZ);
    let (coefficients, counts, region_high_water) = forced_transport(&analysis);
    let (channels, non_finite) =
        synthesise_for_stage_a(&analysis.representation, coefficients, inputs[0].len());
    (channels, counts, region_high_water, non_finite)
}

fn forced_transport(
    analysis: &Analysis,
) -> ([Vec<Vec<Complex64>>; CHANNEL_CAPACITY], StateCounts, usize) {
    let representation = &analysis.representation;
    let positive = representation
        .bands
        .iter()
        .enumerate()
        .filter(|(_, band)| band.center <= representation.fft_frames / 2)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let frequencies_hz = positive
        .iter()
        .map(|band| {
            absolute_bin(
                representation.bands[*band].center,
                representation.fft_frames,
            ) as f64
                * SAMPLE_RATE_HZ as f64
                / representation.fft_frames as f64
        })
        .collect::<Vec<_>>();
    let mut output = std::array::from_fn(|_| {
        vec![
            vec![Complex64::default(); representation.common_coefficients];
            representation.bands.len()
        ]
    });
    let mut workspace = Workspace::new();
    let decisions = [
        Decision::Reset,
        Decision::Attack,
        Decision::Ordinary,
        Decision::Unlocked,
        Decision::Locked,
    ];
    for time in 0..representation.common_coefficients {
        let current = std::array::from_fn(|channel| {
            positive
                .iter()
                .map(|band| analysis.coefficients[channel][*band][time])
                .collect::<Vec<_>>()
        });
        let next = workspace
            .process(
                &current,
                &frequencies_hz,
                decisions[time.min(decisions.len() - 1)],
            )
            .expect("frozen Stage A capacity");
        for channel in 0..CHANNEL_CAPACITY {
            for (local, band) in positive.iter().enumerate() {
                output[channel][*band][time] = next[channel][local];
            }
        }
    }
    mirror_coefficients(representation, &positive, &mut output);
    (output, workspace.counts, workspace.region_high_water)
}
