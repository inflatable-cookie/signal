use signal_primitives::Sample;

pub(super) struct OlaTimeStretchRender {
    pub(super) samples: Vec<Sample>,
    pub(super) synthesis_positions: Vec<usize>,
    pub(super) uncovered_output_frames: usize,
}

pub(super) fn normalized_ola_time_stretch(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> OlaTimeStretchRender {
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);
    let output_start = window_size / 2;
    let output_end = output_start + target_len;
    let frame_count = padded_input
        .len()
        .saturating_sub(window_size)
        .div_euclid(analysis_hop)
        + 1;
    let synthesis_positions = (0..frame_count)
        .map(|frame| (frame as f64 * analysis_hop as f64 * ratio).round() as usize)
        .collect::<Vec<_>>();
    let ola_len = synthesis_positions
        .last()
        .copied()
        .unwrap_or_default()
        .saturating_add(window_size)
        .saturating_add(1)
        .max(output_end);
    let window = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect::<Vec<_>>();
    let mut output = vec![0.0_f32; ola_len];
    let mut normalization = vec![0.0_f32; ola_len];
    for (frame, synthesis_start) in synthesis_positions.iter().copied().enumerate() {
        let analysis_start = frame * analysis_hop;
        for index in 0..window_size {
            let weight = window[index];
            output[synthesis_start + index] +=
                padded_input[analysis_start + index] * weight * weight;
            normalization[synthesis_start + index] += weight * weight;
        }
    }
    let uncovered_output_frames = normalization[output_start..output_end]
        .iter()
        .filter(|weight| **weight <= 1.0e-3)
        .count();
    for (sample, weight) in output.iter_mut().zip(&normalization) {
        if *weight > 1.0e-3 {
            *sample /= *weight;
        }
    }
    OlaTimeStretchRender {
        samples: output[output_start..output_end].to_vec(),
        synthesis_positions,
        uncovered_output_frames,
    }
}
