pub(super) fn sample(
    input: &[f32],
    input_frames: usize,
    channels: usize,
    channel: usize,
    numerator: i128,
    denominator: i128,
) -> f64 {
    let integer = numerator.div_euclid(denominator);
    let remainder = numerator.rem_euclid(denominator);
    let fraction = remainder as f64 / denominator as f64;
    let left = frame(input, input_frames, channels, channel, integer);
    let right = frame(input, input_frames, channels, channel, integer + 1);
    (1.0 - fraction) * left + fraction * right
}

fn frame(input: &[f32], input_frames: usize, channels: usize, channel: usize, frame: i128) -> f64 {
    let Ok(frame) = usize::try_from(frame) else {
        return 0.0;
    };
    if frame >= input_frames {
        return 0.0;
    }
    f64::from(input[frame * channels + channel])
}
