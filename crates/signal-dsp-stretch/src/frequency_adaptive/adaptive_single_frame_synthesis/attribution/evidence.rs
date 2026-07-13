use super::super::super::HASH_OFFSET;
use super::super::quality::control::Control;
use super::super::render::Mode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TraceMode {
    Ordinary,
    Combined,
}

impl TraceMode {
    pub(super) fn render_mode(self) -> Mode {
        match self {
            Self::Ordinary => Mode::Ordinary,
            Self::Combined => Mode::Both,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    PassingAblation,
    GlobalTimeMap,
    PhysicalFrequencyPhaseTransport,
    EventCorrection,
    VerticalLocking,
    DiagonalDualSynthesis,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ToneEvidence {
    pub(super) output_angular_error: f64,
    pub(super) maximum_frequency_error: f64,
    pub(super) maximum_transport_advance_error: f64,
    pub(super) maximum_final_advance_error: f64,
    pub(super) peak_owner_changes: usize,
    pub(super) event_assignments: usize,
    pub(super) vertical_assignments: usize,
    pub(super) resolution_error: [f64; 2],
    pub(super) frames: Vec<ToneFrameEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ToneFrameEvidence {
    pub(super) source: isize,
    pub(super) output: isize,
    pub(super) length: usize,
    pub(super) hops: [f64; 2],
    pub(super) bins: [usize; 3],
    pub(super) frequency_error: f64,
    pub(super) advance_error: [f64; 2],
    pub(super) assignments: [bool; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ContributionEvidence {
    pub(super) source: isize,
    pub(super) output: isize,
    pub(super) length: usize,
    pub(super) energy: f64,
    pub(super) energy_center: f64,
    pub(super) peak_output: isize,
    pub(super) peak_magnitude: f64,
    pub(super) hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct EventEvidence {
    pub(super) source: usize,
    pub(super) scheduled: usize,
    pub(super) selected: bool,
    pub(super) centered: bool,
    pub(super) overlapping_frames: usize,
    pub(super) event_assignments: usize,
    pub(super) vertical_assignments: usize,
    pub(super) dominant_frame: [isize; 3],
    pub(super) displacement: [usize; 3],
    pub(super) replica_peaks: [usize; 3],
    pub(super) contributions: Vec<ContributionEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RowEvidence {
    pub(super) control: Control,
    pub(super) ratio: f64,
    pub(super) mode: TraceMode,
    pub(super) hard_failure: bool,
    pub(super) stage: Stage,
    pub(super) tone: Option<ToneEvidence>,
    pub(super) events: Vec<EventEvidence>,
    pub(super) dense_errors: [usize; 2],
    pub(super) dense_unmatched: usize,
    pub(super) hashes: [u64; 9],
}

pub(super) fn row_hash(row: &RowEvidence) -> u64 {
    let mut state = HASH_OFFSET;
    hash(&mut state, row.control as u64);
    hash(&mut state, row.ratio.to_bits());
    hash(&mut state, row.mode as u64);
    hash(&mut state, row.hard_failure as u64);
    hash(&mut state, row.stage as u64);
    if let Some(tone) = &row.tone {
        for value in [
            tone.output_angular_error,
            tone.maximum_frequency_error,
            tone.maximum_transport_advance_error,
            tone.maximum_final_advance_error,
            tone.resolution_error[0],
            tone.resolution_error[1],
        ] {
            hash(&mut state, value.to_bits());
        }
        for value in [
            tone.peak_owner_changes,
            tone.event_assignments,
            tone.vertical_assignments,
        ] {
            hash(&mut state, value as u64);
        }
        for frame in &tone.frames {
            for value in [frame.source, frame.output] {
                hash(&mut state, value as i64 as u64);
            }
            hash(&mut state, frame.length as u64);
            for value in frame.hops {
                hash(&mut state, value.to_bits());
            }
            for value in frame.bins {
                hash(&mut state, value as u64);
            }
            hash(&mut state, frame.frequency_error.to_bits());
            for value in frame.advance_error {
                hash(&mut state, value.to_bits());
            }
            for value in frame.assignments {
                hash(&mut state, value as u64);
            }
        }
    }
    for event in &row.events {
        for value in [event.source, event.scheduled, event.overlapping_frames]
            .into_iter()
            .chain(event.displacement)
            .chain(event.replica_peaks)
        {
            hash(&mut state, value as u64);
        }
        hash(&mut state, event.selected as u64);
        hash(&mut state, event.centered as u64);
        for value in event.dominant_frame {
            hash(&mut state, value as i64 as u64);
        }
        for contribution in &event.contributions {
            hash(&mut state, contribution.source as i64 as u64);
            hash(&mut state, contribution.output as i64 as u64);
            hash(&mut state, contribution.length as u64);
            hash(&mut state, contribution.energy.to_bits());
            hash(&mut state, contribution.energy_center.to_bits());
            hash(&mut state, contribution.peak_output as i64 as u64);
            hash(&mut state, contribution.peak_magnitude.to_bits());
            hash(&mut state, contribution.hash);
        }
    }
    for value in &row.hashes[..8] {
        hash(&mut state, *value);
    }
    state
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
