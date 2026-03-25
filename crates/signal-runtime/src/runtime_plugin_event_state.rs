use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginEventState {
    continuity: EventPacketContinuityReport,
    last_processing_epoch: Option<u64>,
    last_block_sequence: Option<u64>,
    last_generated_event_bytes: u32,
    last_batch_summary: EventPacketSummary,
}

impl RuntimePluginEventState {
    pub(crate) fn record_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
        generated_event_bytes: u32,
        summary: EventPacketSummary,
    ) {
        self.continuity.record(processing_epoch, lease_id, summary);
        self.last_processing_epoch = Some(processing_epoch);
        self.last_block_sequence = Some(block_sequence);
        self.last_generated_event_bytes = generated_event_bytes;
        self.last_batch_summary = summary;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn snapshot(&self) -> RuntimePluginEventSnapshot {
        let aggregate = self.continuity.aggregate();
        RuntimePluginEventSnapshot {
            last_processing_epoch: self.last_processing_epoch,
            last_block_sequence: self.last_block_sequence,
            last_generated_event_bytes: self.last_generated_event_bytes,
            last_batch_total_events: self.last_batch_summary.total_events,
            last_batch_parameter_value_events: self.last_batch_summary.parameter_value_events,
            last_batch_parameter_modulation_events: self
                .last_batch_summary
                .parameter_modulation_events,
            last_batch_parameter_gesture_events: self.last_batch_summary.parameter_gesture_events,
            last_batch_note_events: self.last_batch_summary.note_events,
            last_batch_note_expression_events: self.last_batch_summary.note_expression_events,
            last_batch_note_expression_pressure_events: self
                .last_batch_summary
                .note_expression_pressure_events,
            last_batch_note_expression_timbre_events: self
                .last_batch_summary
                .note_expression_timbre_events,
            last_batch_note_expression_tuning_events: self
                .last_batch_summary
                .note_expression_tuning_events,
            last_batch_midi_events: self.last_batch_summary.midi_events,
            total_events: aggregate.total_events,
            parameter_value_events: aggregate.parameter_value_events,
            parameter_modulation_events: aggregate.parameter_modulation_events,
            parameter_gesture_events: aggregate.parameter_gesture_events,
            note_events: aggregate.note_events,
            note_expression_events: aggregate.note_expression_events,
            note_expression_pressure_events: aggregate.note_expression_pressure_events,
            note_expression_timbre_events: aggregate.note_expression_timbre_events,
            note_expression_tuning_events: aggregate.note_expression_tuning_events,
            midi_events: aggregate.midi_events,
            mpe_posture: if aggregate.note_expression_events > 0 {
                RuntimeControllerExpressionMpePosture::Guarded
            } else {
                RuntimeControllerExpressionMpePosture::Unsupported
            },
            midi2_posture: if aggregate.note_expression_tuning_events > 0 {
                RuntimeControllerExpressionMidi2Posture::Guarded
            } else {
                RuntimeControllerExpressionMidi2Posture::Unsupported
            },
            first_epoch: self.continuity.first_epoch(),
            last_epoch: self.continuity.last_epoch(),
            segment_count: self.continuity.segment_count(),
            segment_epochs: self.continuity.segment_epochs(),
            lease_rollovers: self.continuity.lease_rollovers,
        }
    }
}
