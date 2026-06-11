/// Per-parameter automation snapshot: lane counts, batch statistics, event counts, and segment lease state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    /// Total number of automation lanes for this parameter.
    pub lane_count: usize,
    /// Total number of automation breakpoint points.
    pub point_count: usize,
    /// Number of projected automation segments.
    pub projected_segment_count: usize,
    /// Number of lanes that are mapped to a parameter.
    pub mapped_lane_count: usize,
    /// Number of lanes that are not mapped to any parameter.
    pub unmapped_lane_count: usize,
    /// Number of lanes using hold (step) interpolation.
    pub hold_lane_count: usize,
    /// Number of lanes using linear interpolation.
    pub linear_lane_count: usize,
    /// Processing epoch of the last dispatched automation batch.
    pub last_batch_epoch: Option<u64>,
    /// Number of events in the last dispatched batch.
    pub last_batch_event_count: usize,
    /// Number of events ignored in the last batch.
    pub last_batch_ignored_event_count: usize,
    /// Number of sub-blocks generated in the last batch.
    pub last_batch_sub_block_count: usize,
    /// Number of events coalesced in the last batch.
    pub last_batch_coalesced_event_count: usize,
    /// Maximum sub-blocks allowed by the current batch strategy.
    pub last_batch_strategy_max_sub_blocks: usize,
    /// Minimum ramp step size in samples for the last batch.
    pub last_batch_min_ramp_step_samples: Option<usize>,
    /// Maximum sample offset of any event in the last batch.
    pub last_batch_max_sample_offset: Option<usize>,
    /// Block sequence of the most recent block that received automation.
    pub last_block_sequence: Option<u64>,
    /// Timeline position in samples at the most recent block.
    pub last_timeline_position_samples: Option<i64>,
    /// Whether the transport was playing at the most recent block.
    pub transport_playing: Option<bool>,
    /// ID of the parameter this snapshot describes.
    pub parameter_id: u32,
    /// Total value change events dispatched for this parameter.
    pub value_events: usize,
    /// Total modulation events dispatched for this parameter.
    pub modulation_events: usize,
    /// Total gesture-begin events dispatched for this parameter.
    pub gesture_begin_events: usize,
    /// Total gesture-end events dispatched for this parameter.
    pub gesture_end_events: usize,
    /// First parameter value dispatched.
    pub first_value: Option<f32>,
    /// Most recent parameter value dispatched.
    pub last_value: Option<f32>,
    /// Most recent modulation value dispatched.
    pub last_modulation: Option<f32>,
    /// Processing epoch of the first event for this parameter.
    pub first_epoch: Option<u64>,
    /// Processing epoch of the most recent event for this parameter.
    pub last_epoch: Option<u64>,
    /// Number of lease segments that have been allocated for this parameter.
    pub segment_count: usize,
    /// Processing epochs of all lease segments.
    pub segment_epochs: Vec<u64>,
    /// Number of lease rollovers that have occurred for this parameter.
    pub lease_rollovers: usize,
}

/// Per-plugin event snapshot: batch event counts by type (parameter value/modulation/gesture, note, note expression, MIDI) plus MPE/MIDI2 posture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginEventSnapshot {
    /// Processing epoch of the most recently dispatched event batch.
    pub last_processing_epoch: Option<u64>,
    /// Block sequence of the most recently dispatched event batch.
    pub last_block_sequence: Option<u64>,
    /// Number of serialised event bytes generated in the last batch.
    pub last_generated_event_bytes: u32,
    /// Total events of all types in the last batch.
    pub last_batch_total_events: usize,
    /// Parameter value events in the last batch.
    pub last_batch_parameter_value_events: usize,
    /// Parameter modulation events in the last batch.
    pub last_batch_parameter_modulation_events: usize,
    /// Parameter gesture events (begin + end combined) in the last batch.
    pub last_batch_parameter_gesture_events: usize,
    /// Note on/off events in the last batch.
    pub last_batch_note_events: usize,
    /// Note expression events (all sub-types) in the last batch.
    pub last_batch_note_expression_events: usize,
    /// Note expression pressure events in the last batch.
    pub last_batch_note_expression_pressure_events: usize,
    /// Note expression timbre events in the last batch.
    pub last_batch_note_expression_timbre_events: usize,
    /// Note expression tuning events in the last batch.
    pub last_batch_note_expression_tuning_events: usize,
    /// Raw MIDI events in the last batch.
    pub last_batch_midi_events: usize,
    /// Cumulative total of all events dispatched to this plugin.
    pub total_events: usize,
    /// Cumulative parameter value events dispatched.
    pub parameter_value_events: usize,
    /// Cumulative parameter modulation events dispatched.
    pub parameter_modulation_events: usize,
    /// Cumulative parameter gesture events dispatched.
    pub parameter_gesture_events: usize,
    /// Cumulative note events dispatched.
    pub note_events: usize,
    /// Cumulative note expression events dispatched.
    pub note_expression_events: usize,
    /// Cumulative note expression pressure events dispatched.
    pub note_expression_pressure_events: usize,
    /// Cumulative note expression timbre events dispatched.
    pub note_expression_timbre_events: usize,
    /// Cumulative note expression tuning events dispatched.
    pub note_expression_tuning_events: usize,
    /// Cumulative MIDI events dispatched.
    pub midi_events: usize,
    /// MPE support posture reported for this plugin.
    pub mpe_posture: RuntimeControllerExpressionMpePosture,
    /// MIDI 2.0 support posture reported for this plugin.
    pub midi2_posture: RuntimeControllerExpressionMidi2Posture,
    /// Processing epoch of the first event dispatched to this plugin.
    pub first_epoch: Option<u64>,
    /// Processing epoch of the most recent event dispatched to this plugin.
    pub last_epoch: Option<u64>,
    /// Number of lease segments allocated for this plugin's event transport.
    pub segment_count: usize,
    /// Processing epochs of all lease segments.
    pub segment_epochs: Vec<u64>,
    /// Number of lease rollovers that have occurred.
    pub lease_rollovers: usize,
}

/// MPE (MIDI Polyphonic Expression) support posture for a controller or plugin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeControllerExpressionMpePosture {
    /// MPE is not supported by this controller or plugin.
    #[default]
    Unsupported,
    /// MPE support is present but gated (may have compatibility caveats).
    Guarded,
}

/// MIDI 2.0 support posture for a controller or plugin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeControllerExpressionMidi2Posture {
    /// MIDI 2.0 is not supported by this controller or plugin.
    #[default]
    Unsupported,
    /// MIDI 2.0 support is present but gated (may have compatibility caveats).
    Guarded,
}
