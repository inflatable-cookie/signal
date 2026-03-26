use crate::{
    EventPacket, EventPacketSummary, NoteExpressionKind, ParameterAutomationSummary,
    ParameterGesturePhase, PluginEvent,
};

impl EventPacket {
    pub fn summary(&self) -> EventPacketSummary {
        let mut summary = EventPacketSummary {
            total_events: self.events.len(),
            ..EventPacketSummary::default()
        };
        for event in &self.events {
            if event.is_parameter_value() {
                summary.parameter_value_events += 1;
            }
            if event.is_parameter_modulation() {
                summary.parameter_modulation_events += 1;
            }
            if event.is_parameter_gesture() {
                summary.parameter_gesture_events += 1;
            }
            if event.is_note() {
                summary.note_events += 1;
            }
            if event.is_note_expression() {
                summary.note_expression_events += 1;
                if let PluginEvent::NoteExpression(expression) = event {
                    match expression.expression {
                        NoteExpressionKind::Pressure => {
                            summary.note_expression_pressure_events += 1;
                        }
                        NoteExpressionKind::Timbre => {
                            summary.note_expression_timbre_events += 1;
                        }
                        NoteExpressionKind::Tuning => {
                            summary.note_expression_tuning_events += 1;
                        }
                    }
                }
            }
            if event.is_midi() {
                summary.midi_events += 1;
            }
        }
        summary
    }

    pub fn parameter_automation_summary(&self, parameter_id: u32) -> ParameterAutomationSummary {
        let mut summary = ParameterAutomationSummary {
            parameter_id,
            ..ParameterAutomationSummary::default()
        };

        for event in &self.events {
            match event {
                PluginEvent::ParameterValue(event) if event.parameter_id == parameter_id => {
                    summary.value_events += 1;
                    if summary.first_value.is_none() {
                        summary.first_value = Some(event.normalized_value);
                    }
                    summary.last_value = Some(event.normalized_value);
                }
                PluginEvent::ParameterModulation(event) if event.parameter_id == parameter_id => {
                    summary.modulation_events += 1;
                    summary.last_modulation = Some(event.amount);
                }
                PluginEvent::ParameterGesture(event) if event.parameter_id == parameter_id => {
                    match event.phase {
                        ParameterGesturePhase::Begin => summary.gesture_begin_events += 1,
                        ParameterGesturePhase::End => summary.gesture_end_events += 1,
                    }
                }
                _ => {}
            }
        }

        summary
    }
}
