use crate::Sample;

pub(crate) const TAIL_ANCHOR_REVIEW_FRAMES: usize = 256;

pub(crate) fn anchor_output_tail_to_source(input: &[Sample], output: &mut [Sample]) {
    let (Some(source_tail), Some(output_tail)) = (input.last(), output.last()) else {
        return;
    };
    if output_tail.abs() <= source_tail.abs() {
        return;
    }

    let frame_count = TAIL_ANCHOR_REVIEW_FRAMES.min(output.len());
    if frame_count == 1 {
        output[0] = *source_tail;
        return;
    }
    let correction = *source_tail - *output_tail;
    let start = output.len() - frame_count;
    for (offset, sample) in output[start..].iter_mut().enumerate() {
        let phase = std::f32::consts::PI * offset as f32 / (frame_count - 1) as f32;
        let weight = 0.5 - 0.5 * phase.cos();
        *sample += correction * weight;
    }
    *output.last_mut().expect("non-empty output") = *source_tail;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OfflineHighQualityStretcher;

    #[test]
    fn tail_anchor_matches_quieter_source_endpoint_inside_bounded_span() {
        let mut input = vec![0.0; 1_024];
        input[1_023] = 0.1;
        let mut output = vec![0.25; 2_048];
        let prefix = output[..output.len() - TAIL_ANCHOR_REVIEW_FRAMES].to_vec();

        anchor_output_tail_to_source(&input, &mut output);

        assert_eq!(output.last(), input.last());
        assert_eq!(
            &output[..output.len() - TAIL_ANCHOR_REVIEW_FRAMES],
            prefix.as_slice()
        );
        assert_eq!(output[output.len() - TAIL_ANCHOR_REVIEW_FRAMES], 0.25);
    }

    #[test]
    fn tail_anchor_keeps_output_when_source_endpoint_is_louder() {
        let input = vec![0.5; 1_024];
        let mut output = vec![0.25; 2_048];
        let original = output.clone();

        anchor_output_tail_to_source(&input, &mut output);

        assert_eq!(output, original);
    }

    #[test]
    fn tail_anchor_handles_empty_and_single_frame_outputs() {
        let mut empty = Vec::new();
        anchor_output_tail_to_source(&[0.5], &mut empty);
        assert!(empty.is_empty());

        let mut single = vec![0.5];
        anchor_output_tail_to_source(&[0.0], &mut single);
        assert_eq!(single, vec![0.0]);
    }

    #[test]
    fn tail_anchor_review_path_is_deterministic_and_honors_output_contract() {
        let mut input = (0..48_000)
            .map(|frame| (std::f32::consts::TAU * 440.0 * frame as f32 / 48_000.0).sin())
            .collect::<Vec<_>>();
        *input.last_mut().expect("input tail") = 0.0;
        let mut first = OfflineHighQualityStretcher::new(1.25);
        let mut repeated = OfflineHighQualityStretcher::new(1.25);

        let first_output = first.stretch_tail_anchor_review_mono(&input);
        let repeated_output = repeated.stretch_tail_anchor_review_mono(&input);

        assert_eq!(first_output.len(), 60_000);
        assert_eq!(first_output, repeated_output);
        assert_eq!(first_output.last(), input.last());
    }
}
