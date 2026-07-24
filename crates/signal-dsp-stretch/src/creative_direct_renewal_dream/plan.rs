use super::{CandidateError, CandidateRequest};

pub(super) const MIN_SAMPLE_RATE: u32 = 8_000;
pub(super) const MAX_SAMPLE_RATE: u32 = 192_000;
pub(super) const MIN_FFT_SIZE: usize = 8_192;
pub(super) const MAX_FFT_SIZE: usize = 131_072;
pub(crate) const MAX_EXACT_INTEGER: usize = (1_u64 << 53) as usize - 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderPlan {
    pub(crate) channels: usize,
    pub(crate) sample_rate: u32,
    pub(crate) source_frames: usize,
    pub(crate) target_frames: usize,
    pub(crate) fft_size: usize,
    pub(crate) hop: usize,
    pub(crate) blocks: usize,
    pub(crate) head_extent: usize,
    pub(crate) tail_extent: usize,
    pub(crate) output_samples: usize,
}

impl RenderPlan {
    pub(crate) fn new(request: &CandidateRequest<'_>) -> Result<Self, CandidateError> {
        if !matches!(request.channels, 1 | 2) {
            return Err(CandidateError::InvalidChannels);
        }
        if !(MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&request.sample_rate) {
            return Err(CandidateError::InvalidSampleRate);
        }
        if request.input.len() % request.channels != 0 {
            return Err(CandidateError::PartialFrame);
        }
        if !request.input.iter().all(|sample| sample.is_finite()) {
            return Err(CandidateError::NonFiniteInput);
        }
        if !request.space.is_finite() || !(0.0..=1.0).contains(&request.space) {
            return Err(CandidateError::InvalidSpace);
        }

        let source_frames = request.input.len() / request.channels;
        validate_dimensions(source_frames, request.target_frames)?;
        if source_frames == 0 {
            return Ok(Self {
                channels: request.channels,
                sample_rate: request.sample_rate,
                source_frames: 0,
                target_frames: 0,
                fft_size: fft_size(request.sample_rate),
                hop: fft_size(request.sample_rate) / 2,
                blocks: 0,
                head_extent: 0,
                tail_extent: 0,
                output_samples: 0,
            });
        }

        let fft_size = fft_size(request.sample_rate);
        let hop = fft_size / 2;
        let blocks = request
            .target_frames
            .checked_add(hop - 1)
            .ok_or(CandidateError::SizeOverflow)?
            / hop;
        let output_samples = request
            .target_frames
            .checked_mul(request.channels)
            .ok_or(CandidateError::SizeOverflow)?;
        let head_extent =
            ((request.sample_rate as usize + 199) / 200).min(request.target_frames / 4);
        let tail_extent = hop
            .checked_mul(2)
            .ok_or(CandidateError::SizeOverflow)?
            .min(request.target_frames / 4);

        Ok(Self {
            channels: request.channels,
            sample_rate: request.sample_rate,
            source_frames,
            target_frames: request.target_frames,
            fft_size,
            hop,
            blocks,
            head_extent,
            tail_extent,
            output_samples,
        })
    }

    pub(crate) fn source_center(&self, block: usize) -> Result<f64, CandidateError> {
        let output_frame = block
            .checked_mul(self.hop)
            .ok_or(CandidateError::SizeOverflow)?;
        let doubled_plus_one = (output_frame as u128)
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or(CandidateError::SizeOverflow)?;
        let numerator = doubled_plus_one
            .checked_mul(self.source_frames as u128)
            .ok_or(CandidateError::SizeOverflow)?;
        let denominator = (self.target_frames as u128)
            .checked_mul(2)
            .ok_or(CandidateError::SizeOverflow)?;
        Ok(numerator as f64 / denominator as f64 - 0.5)
    }
}

pub(crate) fn validate_dimensions(
    source_frames: usize,
    target_frames: usize,
) -> Result<(), CandidateError> {
    if source_frames > MAX_EXACT_INTEGER || target_frames > MAX_EXACT_INTEGER {
        return Err(CandidateError::SizeOverflow);
    }
    if source_frames == 0 {
        return if target_frames == 0 {
            Ok(())
        } else {
            Err(CandidateError::EmptyInput)
        };
    }
    if target_frames == 0 {
        return Err(CandidateError::ZeroTarget);
    }
    let minimum = source_frames
        .checked_mul(4)
        .ok_or(CandidateError::SizeOverflow)?;
    let maximum = source_frames
        .checked_mul(16)
        .ok_or(CandidateError::SizeOverflow)?;
    if !(minimum..=maximum).contains(&target_frames) {
        return Err(CandidateError::UnsupportedRatio);
    }
    Ok(())
}

pub(crate) fn round_half_up_two_thirds(sample_rate: u32) -> usize {
    ((sample_rate as u64 * 4 + 3) / 6) as usize
}

pub(crate) fn nearest_power_of_two_ties_up(value: usize) -> usize {
    if value <= 1 {
        return 1;
    }
    let upper = value.next_power_of_two();
    let lower = upper / 2;
    if upper - value <= value - lower {
        upper
    } else {
        lower
    }
}

pub(crate) fn fft_size(sample_rate: u32) -> usize {
    nearest_power_of_two_ties_up(round_half_up_two_thirds(sample_rate))
        .clamp(MIN_FFT_SIZE, MAX_FFT_SIZE)
}
