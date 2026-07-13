use super::super::super::study_local_schedule::{schedule::Schedule, BASE_HOP};
use super::Frame;

pub(super) struct Ownership {
    events: Vec<usize>,
}

impl Ownership {
    pub(super) fn for_frame(frame: &Frame, events: &[usize], schedule: &Schedule) -> Option<Self> {
        let overlapping = events
            .iter()
            .copied()
            .filter(|event| frame.source.abs_diff(*event as isize) < frame.length / 2)
            .collect::<Vec<_>>();
        if overlapping.len() < 2 || overlapping.contains(&(frame.source as usize)) {
            return None;
        }
        let separated = overlapping.windows(2).any(|pair| {
            super::super::anchors::projected(schedule, pair[0])
                .abs_diff(super::super::anchors::projected(schedule, pair[1]))
                > frame.length
        });
        separated.then_some(Self {
            events: overlapping,
        })
    }

    pub(super) fn sample(&self, input: &[f64], sample: isize) -> Option<f64> {
        let radius = BASE_HOP as isize / 2;
        self.events.iter().find_map(|event| {
            let event = *event as isize;
            if sample <= event - radius || sample >= event + radius {
                return None;
            }
            let left = event - radius;
            let right = event + radius;
            let fraction = (sample - left) as f64 / (right - left) as f64;
            Some(
                (1.0 - fraction) * super::reflected(input, left)
                    + fraction * super::reflected(input, right),
            )
        })
    }
}
