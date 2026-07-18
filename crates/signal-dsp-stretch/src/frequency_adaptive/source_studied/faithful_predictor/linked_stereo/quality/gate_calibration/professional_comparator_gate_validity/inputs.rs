use super::super::{
    metrics::{control, ControlKind},
    ALIGNMENTS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::coherent_representation;

#[derive(Clone)]
pub(super) struct PreparedRow {
    pub(super) ratio: f64,
    pub(super) source_frames: usize,
    pub(super) phase: f64,
    pub(super) frequency: f64,
    pub(super) bin_aligned: bool,
    pub(super) kind: ControlKind,
    pub(super) source: [Vec<f64>; 2],
}

pub(super) fn prepare() -> Vec<PreparedRow> {
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut rows = Vec::with_capacity(48);
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        rows.push(PreparedRow {
                            ratio,
                            source_frames,
                            phase,
                            frequency,
                            bin_aligned,
                            kind,
                            source: source.clone(),
                        });
                    }
                }
            }
        }
    }
    assert_eq!(rows.len(), 48);
    rows
}
