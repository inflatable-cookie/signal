use super::{plan, render, schedule, synthesis, CandidateError, Plan, Request};

const RATE: u32 = 44_100;
const NEUTRAL_CYCLE_US: u32 = 48_000;

fn mono_input(frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|frame| {
            (0.4 * (std::f64::consts::TAU * 220.0 * frame as f64 / f64::from(RATE)).sin()) as f32
        })
        .collect()
}

fn request<'a>(
    input: &'a [f32],
    channels: usize,
    target_frames: usize,
    cycle_us: u32,
) -> Request<'a> {
    Request {
        input,
        channels,
        sample_rate: RATE,
        target_frames,
        cycle_us,
    }
}

#[test]
fn identity_and_empty_requests_are_exact() {
    let input = mono_input(4096);
    assert_eq!(
        render(request(&input, 1, input.len(), NEUTRAL_CYCLE_US)),
        Ok(input)
    );
    assert_eq!(render(request(&[], 1, 0, NEUTRAL_CYCLE_US)), Ok(Vec::new()));
}

#[test]
fn admitted_ratios_and_cycle_extremes_are_finite_and_deterministic() {
    let input = mono_input(4096);
    for ratio in [2, 4, 8] {
        for cycle_us in [plan::MIN_CYCLE_US, NEUTRAL_CYCLE_US, plan::MAX_CYCLE_US] {
            let first =
                render(request(&input, 1, input.len() * ratio, cycle_us)).expect("first render");
            let second =
                render(request(&input, 1, input.len() * ratio, cycle_us)).expect("repeat render");
            assert_eq!(first, second);
            assert_eq!(first.len(), input.len() * ratio);
            assert!(first.iter().all(|sample| sample.is_finite()));
            assert!(
                first
                    .iter()
                    .map(|sample| sample.abs())
                    .fold(0.0_f32, f32::max)
                    <= 0.400_002
            );
        }
    }
}

#[test]
fn linked_stereo_preserves_duplicate_antiphase_and_common_negation() {
    let mono = mono_input(4096);
    for relation in ["duplicate", "anti-phase"] {
        let mut input = Vec::with_capacity(mono.len() * 2);
        for sample in &mono {
            input.push(*sample);
            input.push(if relation == "duplicate" {
                *sample
            } else {
                -*sample
            });
        }
        let output =
            render(request(&input, 2, mono.len() * 8, NEUTRAL_CYCLE_US)).expect("stereo render");
        for frame in output.chunks_exact(2) {
            let error = if relation == "duplicate" {
                frame[0] - frame[1]
            } else {
                frame[0] + frame[1]
            };
            assert!(error.abs() <= 1e-6);
        }

        let negated: Vec<_> = input.iter().map(|sample| -*sample).collect();
        let negative =
            render(request(&negated, 2, mono.len() * 8, NEUTRAL_CYCLE_US)).expect("negated render");
        for (positive, negative) in output.iter().zip(negative) {
            assert!((*positive + negative).abs() <= 1e-6);
        }
    }
}

#[test]
fn invalid_requests_return_exact_errors() {
    let input = mono_input(32);
    assert_eq!(
        render(request(&input, 0, 64, NEUTRAL_CYCLE_US)),
        Err(CandidateError::InvalidChannels)
    );
    assert_eq!(
        render(request(&input, 3, 64, NEUTRAL_CYCLE_US)),
        Err(CandidateError::InvalidChannels)
    );
    assert_eq!(
        render(request(&[0.0, 0.0, 0.0], 2, 4, NEUTRAL_CYCLE_US)),
        Err(CandidateError::PartialFrame)
    );
    assert_eq!(
        render(Request {
            sample_rate: plan::MIN_RATE - 1,
            ..request(&input, 1, 64, NEUTRAL_CYCLE_US)
        }),
        Err(CandidateError::InvalidSampleRate)
    );
    assert_eq!(
        render(request(&input, 1, 64, plan::MIN_CYCLE_US - 1)),
        Err(CandidateError::InvalidCycle)
    );
    assert_eq!(
        render(request(&input, 1, 16, NEUTRAL_CYCLE_US)),
        Err(CandidateError::UnsupportedCompression)
    );
    assert_eq!(
        render(request(&input, 1, 32 * 16, NEUTRAL_CYCLE_US)),
        Err(CandidateError::UnsupportedRatio)
    );
    assert_eq!(
        render(request(&[f32::NAN], 1, 2, NEUTRAL_CYCLE_US)),
        Err(CandidateError::NonFiniteInput)
    );
}

#[test]
fn exact_sixteen_rejects_before_output_allocation() {
    let input = mono_input(32);
    synthesis::reset_output_allocation_count();
    assert_eq!(
        render(request(&input, 1, input.len() * 16, NEUTRAL_CYCLE_US)),
        Err(CandidateError::UnsupportedRatio)
    );
    assert_eq!(synthesis::output_allocation_count(), 0);
}

#[test]
fn map_window_and_memory_laws_remain_bounded() {
    let input = mono_input(4096);
    let plan = Plan::new(request(&input, 1, input.len() * 8, plan::MAX_CYCLE_US)).expect("plan");
    assert!(plan::working_bytes(plan.cycle_frames) <= plan::MAX_WORKING_BYTES);
    assert_eq!(plan.window[0], 0.0);
    assert_eq!(plan.window[plan.cycle_frames], 1.0);
    for index in 0..=plan.cycle_frames {
        assert!((plan.window[index] + plan.window[plan.cycle_frames - index] - 1.0).abs() <= 1e-12);
    }
    let mut previous = i128::MIN;
    for output in 0..plan.target_frames {
        let mapped = schedule::ideal_map_numerator(&plan, output).expect("ideal map");
        assert!(mapped > previous);
        previous = mapped;
    }
}
