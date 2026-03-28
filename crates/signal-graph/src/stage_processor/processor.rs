use crate::GraphStageSpec;
use signal_dsp::{
    process_delay_with_feedback_control, process_low_pass_with_cutoff_control, DelayLine,
    OnePoleLowPass,
};
use signal_primitives::SampleRate;

pub(super) struct GraphStageProcessor {
    stage: GraphStageProcessorKind,
}

enum GraphStageProcessorKind {
    Gain {
        linear: f32,
    },
    Bias {
        amount: f32,
    },
    TanhDrive {
        drive: f32,
    },
    StereoBalance {
        balance: f32,
    },
    HardClip {
        threshold: f32,
    },
    LowPass {
        cutoff_hz: f32,
        filters: Vec<OnePoleLowPass>,
    },
    Delay {
        feedback: f32,
        delay_samples: usize,
        lines: Vec<DelayLine>,
    },
}

impl GraphStageProcessor {
    pub(super) fn new(
        stage: &GraphStageSpec,
        sample_rate: SampleRate,
        channel_count: usize,
    ) -> Self {
        let stage = match *stage {
            GraphStageSpec::Gain { linear } => GraphStageProcessorKind::Gain { linear },
            GraphStageSpec::Bias { amount } => GraphStageProcessorKind::Bias { amount },
            GraphStageSpec::TanhDrive { drive } => GraphStageProcessorKind::TanhDrive { drive },
            GraphStageSpec::StereoBalance { balance } => {
                GraphStageProcessorKind::StereoBalance { balance }
            }
            GraphStageSpec::HardClip { threshold } => {
                GraphStageProcessorKind::HardClip { threshold }
            }
            GraphStageSpec::LowPass { cutoff_hz } => GraphStageProcessorKind::LowPass {
                cutoff_hz,
                filters: (0..channel_count)
                    .map(|_| {
                        OnePoleLowPass::new(sample_rate, signal_primitives::FrequencyHz(cutoff_hz))
                    })
                    .collect(),
            },
            GraphStageSpec::Delay {
                delay_samples,
                feedback,
            } => GraphStageProcessorKind::Delay {
                feedback,
                delay_samples,
                lines: (0..channel_count)
                    .map(|_| {
                        let mut delay = DelayLine::with_max_delay(delay_samples.max(1));
                        delay.set_delay_samples(delay_samples);
                        delay.set_feedback(feedback);
                        delay
                    })
                    .collect(),
            },
        };
        Self { stage }
    }

    pub(super) fn set_parameter(&mut self, value: f32) {
        match &mut self.stage {
            GraphStageProcessorKind::Gain { linear } => *linear = value,
            GraphStageProcessorKind::Bias { amount } => *amount = value,
            GraphStageProcessorKind::TanhDrive { drive } => *drive = value,
            GraphStageProcessorKind::StereoBalance { balance } => *balance = value,
            GraphStageProcessorKind::HardClip { threshold } => *threshold = value.abs(),
            GraphStageProcessorKind::LowPass { cutoff_hz, filters } => {
                *cutoff_hz = value.max(0.0);
                for filter in filters {
                    filter.set_cutoff_hz(signal_primitives::FrequencyHz(*cutoff_hz));
                }
            }
            GraphStageProcessorKind::Delay {
                feedback, lines, ..
            } => {
                *feedback = value;
                for line in lines {
                    line.set_feedback(value);
                }
            }
        }
    }

    pub(super) fn process_interleaved(&mut self, samples: &mut [f32], channel_count: usize) {
        match &mut self.stage {
            GraphStageProcessorKind::Gain { linear } => {
                for sample in samples {
                    *sample *= *linear;
                }
            }
            GraphStageProcessorKind::Bias { amount } => {
                for sample in samples {
                    *sample += *amount;
                }
            }
            GraphStageProcessorKind::TanhDrive { drive } => {
                let drive = drive.max(0.0);
                for sample in samples {
                    *sample = (*sample * drive).tanh();
                }
            }
            GraphStageProcessorKind::StereoBalance { balance } => {
                apply_stereo_balance_interleaved(samples, channel_count, *balance);
            }
            GraphStageProcessorKind::HardClip { threshold } => {
                let threshold = threshold.abs();
                for sample in samples {
                    *sample = sample.clamp(-threshold, threshold);
                }
            }
            GraphStageProcessorKind::LowPass { cutoff_hz, filters } => {
                process_low_pass_interleaved(samples, channel_count, filters, *cutoff_hz);
            }
            GraphStageProcessorKind::Delay {
                feedback,
                delay_samples,
                lines,
            } => {
                process_delay_interleaved(samples, channel_count, lines, *delay_samples, *feedback);
            }
        }
    }
}

fn apply_stereo_balance_interleaved(samples: &mut [f32], channel_count: usize, balance: f32) {
    if channel_count != 2 {
        return;
    }

    let balance = balance.clamp(-1.0, 1.0);
    let left_gain = if balance >= 0.0 { 1.0 - balance } else { 1.0 };
    let right_gain = if balance <= 0.0 { 1.0 + balance } else { 1.0 };

    for frame in samples.chunks_exact_mut(channel_count) {
        frame[0] *= left_gain;
        frame[1] *= right_gain;
    }
}

fn process_low_pass_interleaved(
    samples: &mut [f32],
    channel_count: usize,
    filters: &mut [OnePoleLowPass],
    cutoff_hz: f32,
) {
    if channel_count == 0 {
        return;
    }

    for (channel_index, filter) in filters.iter_mut().enumerate().take(channel_count) {
        let mut mono = samples
            .chunks_exact(channel_count)
            .map(|frame| frame[channel_index])
            .collect::<Vec<_>>();
        let cutoff = vec![cutoff_hz; mono.len()];
        process_low_pass_with_cutoff_control(filter, &mut mono, &cutoff);
        for (frame, sample) in samples
            .chunks_exact_mut(channel_count)
            .zip(mono.into_iter())
        {
            frame[channel_index] = sample;
        }
    }
}

fn process_delay_interleaved(
    samples: &mut [f32],
    channel_count: usize,
    lines: &mut [DelayLine],
    delay_samples: usize,
    feedback: f32,
) {
    if channel_count == 0 {
        return;
    }

    for (channel_index, delay) in lines.iter_mut().enumerate().take(channel_count) {
        delay.set_delay_samples(delay_samples);
        let mut mono = samples
            .chunks_exact(channel_count)
            .map(|frame| frame[channel_index])
            .collect::<Vec<_>>();
        let feedback_block = vec![feedback; mono.len()];
        process_delay_with_feedback_control(delay, &mut mono, &feedback_block);
        for (frame, sample) in samples
            .chunks_exact_mut(channel_count)
            .zip(mono.into_iter())
        {
            frame[channel_index] = sample;
        }
    }
}
