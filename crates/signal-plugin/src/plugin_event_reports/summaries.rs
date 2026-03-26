#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventPacketSummary {
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
}

impl EventPacketSummary {
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParameterAutomationSummary {
    pub parameter_id: u32,
    pub value_events: usize,
    pub modulation_events: usize,
    pub gesture_begin_events: usize,
    pub gesture_end_events: usize,
    pub first_value: Option<f32>,
    pub last_value: Option<f32>,
    pub last_modulation: Option<f32>,
}

impl ParameterAutomationSummary {
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
