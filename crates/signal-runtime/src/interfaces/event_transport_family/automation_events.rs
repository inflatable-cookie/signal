#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    pub lane_count: usize,
    pub point_count: usize,
    pub projected_segment_count: usize,
    pub mapped_lane_count: usize,
    pub unmapped_lane_count: usize,
    pub hold_lane_count: usize,
    pub linear_lane_count: usize,
    pub last_batch_epoch: Option<u64>,
    pub last_batch_event_count: usize,
    pub last_batch_ignored_event_count: usize,
    pub last_batch_sub_block_count: usize,
    pub last_batch_coalesced_event_count: usize,
    pub last_batch_strategy_max_sub_blocks: usize,
    pub last_batch_min_ramp_step_samples: Option<usize>,
    pub last_batch_max_sample_offset: Option<usize>,
    pub last_block_sequence: Option<u64>,
    pub last_timeline_position_samples: Option<i64>,
    pub transport_playing: Option<bool>,
    pub parameter_id: u32,
    pub value_events: usize,
    pub modulation_events: usize,
    pub gesture_begin_events: usize,
    pub gesture_end_events: usize,
    pub first_value: Option<f32>,
    pub last_value: Option<f32>,
    pub last_modulation: Option<f32>,
    pub first_epoch: Option<u64>,
    pub last_epoch: Option<u64>,
    pub segment_count: usize,
    pub segment_epochs: Vec<u64>,
    pub lease_rollovers: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginEventSnapshot {
    pub last_processing_epoch: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub last_generated_event_bytes: u32,
    pub last_batch_total_events: usize,
    pub last_batch_parameter_value_events: usize,
    pub last_batch_parameter_modulation_events: usize,
    pub last_batch_parameter_gesture_events: usize,
    pub last_batch_note_events: usize,
    pub last_batch_note_expression_events: usize,
    pub last_batch_note_expression_pressure_events: usize,
    pub last_batch_note_expression_timbre_events: usize,
    pub last_batch_note_expression_tuning_events: usize,
    pub last_batch_midi_events: usize,
    pub total_events: usize,
    pub parameter_value_events: usize,
    pub parameter_modulation_events: usize,
    pub parameter_gesture_events: usize,
    pub note_events: usize,
    pub note_expression_events: usize,
    pub note_expression_pressure_events: usize,
    pub note_expression_timbre_events: usize,
    pub note_expression_tuning_events: usize,
    pub midi_events: usize,
    pub mpe_posture: RuntimeControllerExpressionMpePosture,
    pub midi2_posture: RuntimeControllerExpressionMidi2Posture,
    pub first_epoch: Option<u64>,
    pub last_epoch: Option<u64>,
    pub segment_count: usize,
    pub segment_epochs: Vec<u64>,
    pub lease_rollovers: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeControllerExpressionMpePosture {
    #[default]
    Unsupported,
    Guarded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeControllerExpressionMidi2Posture {
    #[default]
    Unsupported,
    Guarded,
}
