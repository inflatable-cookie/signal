use super::{CandidateError, Request};

pub(super) const MIN_RATE: u32 = 8_000;
pub(super) const MAX_RATE: u32 = 192_000;
pub(super) const MIN_CYCLE_US: u32 = 5_000;
pub(super) const MAX_CYCLE_US: u32 = 90_000;
pub(super) const EXACT_INTEGER_LIMIT: usize = 9_007_199_254_740_991;
pub(super) const MAX_WORKING_BYTES: usize = 256 * 1024;

#[derive(Debug)]
pub(super) struct Plan {
    pub(super) input_frames: usize,
    pub(super) target_frames: usize,
    pub(super) channels: usize,
    pub(super) cycle_frames: usize,
    pub(super) denominator: i128,
    pub(super) window: Vec<f64>,
}

impl Plan {
    pub(super) fn new(request: Request<'_>) -> Result<Self, CandidateError> {
        if !matches!(request.channels, 1 | 2) {
            return Err(CandidateError::InvalidChannels);
        }
        if !request.input.len().is_multiple_of(request.channels) {
            return Err(CandidateError::PartialFrame);
        }
        if !(MIN_RATE..=MAX_RATE).contains(&request.sample_rate) {
            return Err(CandidateError::InvalidSampleRate);
        }
        if !(MIN_CYCLE_US..=MAX_CYCLE_US).contains(&request.cycle_us) {
            return Err(CandidateError::InvalidCycle);
        }
        if request.input.iter().any(|sample| !sample.is_finite()) {
            return Err(CandidateError::NonFiniteInput);
        }
        let input_frames = request.input.len() / request.channels;
        validate_dimensions(input_frames, request.target_frames, request.channels)?;
        let cycle_frames = cycle_frames(request.sample_rate, request.cycle_us)?;
        let denominator = i128::try_from(request.target_frames)
            .map_err(|_| CandidateError::ArithmeticOverflow)?
            .checked_mul(2)
            .ok_or(CandidateError::ArithmeticOverflow)?;
        let mut window = Vec::with_capacity(cycle_frames + 1);
        for index in 0..=cycle_frames {
            let value = if index == 0 {
                0.0
            } else if index == cycle_frames {
                1.0
            } else {
                0.5 - 0.5 * (std::f64::consts::PI * index as f64 / cycle_frames as f64).cos()
            };
            window.push(value);
        }
        if working_bytes(cycle_frames) > MAX_WORKING_BYTES {
            return Err(CandidateError::AllocationOverflow);
        }
        Ok(Self {
            input_frames,
            target_frames: request.target_frames,
            channels: request.channels,
            cycle_frames,
            denominator,
            window,
        })
    }
}

pub(super) fn validate_dimensions(
    input_frames: usize,
    target_frames: usize,
    channels: usize,
) -> Result<(), CandidateError> {
    if input_frames > EXACT_INTEGER_LIMIT || target_frames > EXACT_INTEGER_LIMIT {
        return Err(CandidateError::ExactIntegerLimit);
    }
    if input_frames == 0 || target_frames == 0 {
        return if input_frames == 0 && target_frames == 0 {
            Ok(())
        } else {
            Err(CandidateError::InvalidEmptyTarget)
        };
    }
    if target_frames < input_frames {
        return Err(CandidateError::UnsupportedCompression);
    }
    let maximum = input_frames
        .checked_mul(8)
        .ok_or(CandidateError::ArithmeticOverflow)?;
    if target_frames > maximum {
        return Err(CandidateError::UnsupportedRatio);
    }
    target_frames
        .checked_mul(channels)
        .and_then(|samples| samples.checked_mul(std::mem::size_of::<f32>()))
        .ok_or(CandidateError::AllocationOverflow)?;
    Ok(())
}

pub(super) fn cycle_frames(sample_rate: u32, cycle_us: u32) -> Result<usize, CandidateError> {
    let numerator = u128::from(sample_rate)
        .checked_mul(u128::from(cycle_us))
        .and_then(|value| value.checked_add(500_000))
        .ok_or(CandidateError::ArithmeticOverflow)?;
    usize::try_from(numerator / 1_000_000).map_err(|_| CandidateError::ArithmeticOverflow)
}

pub(super) const fn working_bytes(cycle_frames: usize) -> usize {
    (cycle_frames + 1) * std::mem::size_of::<f64>() + std::mem::size_of::<Plan>()
}
