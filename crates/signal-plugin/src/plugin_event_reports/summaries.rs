/// Event-type breakdown for a single event packet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventPacketSummary {
    /// Total number of events in the packet.
    pub total_events: usize,
    /// Number of [`ParameterValue`](crate::PluginEvent::ParameterValue) events.
    pub parameter_value_events: usize,
    /// Number of [`ParameterModulation`](crate::PluginEvent::ParameterModulation) events.
    pub parameter_modulation_events: usize,
    /// Number of [`ParameterGesture`](crate::PluginEvent::ParameterGesture) events.
    pub parameter_gesture_events: usize,
    /// Number of [`Note`](crate::PluginEvent::Note) events.
    pub note_events: usize,
    /// Number of [`NoteExpression`](crate::PluginEvent::NoteExpression) events (all dimensions).
    pub note_expression_events: usize,
    /// Number of [`NoteExpression`](crate::PluginEvent::NoteExpression) events with `Pressure` dimension.
    pub note_expression_pressure_events: usize,
    /// Number of [`NoteExpression`](crate::PluginEvent::NoteExpression) events with `Timbre` dimension.
    pub note_expression_timbre_events: usize,
    /// Number of [`NoteExpression`](crate::PluginEvent::NoteExpression) events with `Tuning` dimension.
    pub note_expression_tuning_events: usize,
    /// Number of [`Midi`](crate::PluginEvent::Midi) events.
    pub midi_events: usize,
}

impl EventPacketSummary {
    /// Accumulates the counts from `other` into `self`.
    pub fn merge(&mut self, other: Self) {
        self.total_events += other.total_events;
        self.parameter_value_events += other.parameter_value_events;
        self.parameter_modulation_events += other.parameter_modulation_events;
        self.parameter_gesture_events += other.parameter_gesture_events;
        self.note_events += other.note_events;
        self.note_expression_events += other.note_expression_events;
        self.note_expression_pressure_events += other.note_expression_pressure_events;
        self.note_expression_timbre_events += other.note_expression_timbre_events;
        self.note_expression_tuning_events += other.note_expression_tuning_events;
        self.midi_events += other.midi_events;
    }
}

/// Aggregated automation statistics for a single parameter across one or more event packets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParameterAutomationSummary {
    /// Identifies the parameter this summary covers.
    pub parameter_id: u32,
    /// Number of absolute value events observed.
    pub value_events: usize,
    /// Number of modulation events observed.
    pub modulation_events: usize,
    /// Number of gesture-begin events observed.
    pub gesture_begin_events: usize,
    /// Number of gesture-end events observed.
    pub gesture_end_events: usize,
    /// First normalized value seen, or `None` if no value events were observed.
    pub first_value: Option<f32>,
    /// Last normalized value seen, or `None` if no value events were observed.
    pub last_value: Option<f32>,
    /// Last modulation amount seen, or `None` if no modulation events were observed.
    pub last_modulation: Option<f32>,
}

impl ParameterAutomationSummary {
    /// Accumulates statistics from `other` into `self`, preserving first/last value ordering.
    pub fn merge(&mut self, other: Self) {
        if self.parameter_id == 0 {
            self.parameter_id = other.parameter_id;
        }

        self.value_events += other.value_events;
        self.modulation_events += other.modulation_events;
        self.gesture_begin_events += other.gesture_begin_events;
        self.gesture_end_events += other.gesture_end_events;

        if self.first_value.is_none() {
            self.first_value = other.first_value;
        }
        if other.last_value.is_some() {
            self.last_value = other.last_value;
        }
        if other.last_modulation.is_some() {
            self.last_modulation = other.last_modulation;
        }
    }
}
