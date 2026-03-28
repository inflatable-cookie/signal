use crate::GraphParameterApplicationStrategy;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageParameterEvent {
    pub sample_offset: usize,
    pub value: f32,
}

pub fn bounded_stage_events(
    events: &[StageParameterEvent],
    strategy: GraphParameterApplicationStrategy,
) -> (Vec<StageParameterEvent>, usize) {
    match strategy {
        GraphParameterApplicationStrategy::SplitAtEvents { max_sub_blocks } => {
            let max_boundaries = max_sub_blocks.saturating_sub(1);
            if events.len() <= max_boundaries {
                return (events.to_vec(), 0);
            }

            if max_boundaries == 0 {
                let final_value = events.last().map(|event| event.value).unwrap_or(0.0);
                return (
                    vec![StageParameterEvent {
                        sample_offset: 0,
                        value: final_value,
                    }],
                    events.len(),
                );
            }

            let last_exact_index = max_boundaries.saturating_sub(1);
            let last_boundary = events[last_exact_index].sample_offset;
            let mut bounded = events[..max_boundaries].to_vec();
            if let Some(last) = bounded.last_mut() {
                last.value = events
                    .iter()
                    .skip(last_exact_index)
                    .last()
                    .map(|event| event.value)
                    .unwrap_or(last.value);
                last.sample_offset = last_boundary;
            }
            (bounded, events.len().saturating_sub(max_boundaries))
        }
    }
}
