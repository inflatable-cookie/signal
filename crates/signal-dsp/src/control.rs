//! Control signal smoothing and automation playback.
//!
//! This module provides [`SmoothedValue`] for parameter interpolation,
//! [`ControlSegment`] for scheduled automation events, and [`ControlPlan`]
//! for reusable automation schedules.

mod schedule;
mod smoothing;

pub use schedule::{ControlPlan, ControlSegment, ControlSegmentPlayer, ControlSegmentShape};
pub use smoothing::SmoothedValue;

#[cfg(test)]
mod tests {
    use super::{ControlPlan, ControlSegment, ControlSegmentPlayer, SmoothedValue};

    #[test]
    fn smoothed_value_fills_block_sample_accurately() {
        let mut value = SmoothedValue::new(0.0);
        value.set_linear_target(1.0, 4);
        let mut block = [0.0; 4];

        value.fill_block(&mut block);

        assert_eq!(block, [0.25, 0.5, 0.75, 1.0]);
        assert!(!value.is_smoothing());
    }

    #[test]
    fn control_segment_player_renders_sample_accurate_segments_across_blocks() {
        let segments = [
            ControlSegment::step(0, 0.0),
            ControlSegment::linear(2, 4, 1.0),
            ControlSegment::step(8, 0.25),
        ];
        let mut player = ControlSegmentPlayer::new(0.0, &segments);
        let mut first = [0.0; 5];
        let mut second = [0.0; 5];

        player.render_block(&mut first);
        player.render_block(&mut second);

        assert_eq!(first, [0.0, 0.0, 0.25, 0.5, 0.75]);
        assert_eq!(second, [1.0, 1.0, 1.0, 0.25, 0.25]);
        assert_eq!(player.current_frame(), 10);
        assert_eq!(player.current_value(), 0.25);
    }

    #[test]
    fn control_plan_renders_same_values_for_offset_block() {
        let segments = [
            ControlSegment::step(0, 0.0),
            ControlSegment::linear(2, 4, 1.0),
            ControlSegment::step(8, 0.25),
        ];
        let plan = ControlPlan::new(0.0, &segments);
        let mut sequential_player = plan.player();
        let mut first = [0.0; 5];
        let mut expected_second = [0.0; 5];
        let mut actual_second = [0.0; 5];

        sequential_player.render_block(&mut first);
        sequential_player.render_block(&mut expected_second);
        plan.render_block(5, &mut actual_second);

        assert_eq!(expected_second, actual_second);
    }
}
