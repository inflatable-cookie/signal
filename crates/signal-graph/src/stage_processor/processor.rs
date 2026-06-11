use crate::GraphStageSpec;

pub(super) struct GraphStageProcessor {
    stage: GraphStageProcessorKind,
}

enum GraphStageProcessorKind {
    Gain { linear: f32 },
    Bias { amount: f32 },
    TanhDrive { drive: f32 },
    StereoBalance { balance: f32 },
    HardClip { threshold: f32 },
}

impl GraphStageProcessor {
    pub(super) fn new(stage: &GraphStageSpec) -> Self {
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
