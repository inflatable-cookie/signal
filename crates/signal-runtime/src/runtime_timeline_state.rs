#[path = "runtime_timeline_state/execution.rs"]
mod execution;

use super::*;

pub(crate) fn classify_transport_transition(
    previous: Option<TransportProjection>,
    next: TransportProjection,
) -> Option<RuntimeTransportTransitionKind> {
    let Some(previous) = previous else {
        return Some(if next.playing {
            RuntimeTransportTransitionKind::Started
        } else {
            RuntimeTransportTransitionKind::Initial
        });
    };
    if previous.playing != next.playing {
        return Some(if next.playing {
            RuntimeTransportTransitionKind::Started
        } else {
            RuntimeTransportTransitionKind::Stopped
        });
    }
    if previous.timeline_position_samples != next.timeline_position_samples {
        return Some(RuntimeTransportTransitionKind::Seeked);
    }
    if previous.tempo_bpm != next.tempo_bpm {
        return Some(RuntimeTransportTransitionKind::TempoChanged);
    }
    if previous.loop_state != next.loop_state {
        return Some(RuntimeTransportTransitionKind::LoopStateChanged);
    }
    None
}

pub(crate) fn classify_transport_invalidation_reason(
    previous: Option<TransportProjection>,
    next: TransportProjection,
) -> RuntimePreworkInvalidationReason {
    match classify_transport_transition(previous, next) {
        Some(RuntimeTransportTransitionKind::Started)
        | Some(RuntimeTransportTransitionKind::Initial) => {
            RuntimePreworkInvalidationReason::TransportStarted
        }
        Some(RuntimeTransportTransitionKind::Stopped) => {
            RuntimePreworkInvalidationReason::TransportStopped
        }
        Some(RuntimeTransportTransitionKind::Seeked) => {
            RuntimePreworkInvalidationReason::TransportSeeked
        }
        Some(RuntimeTransportTransitionKind::TempoChanged) => {
            RuntimePreworkInvalidationReason::TransportTempoChanged
        }
        Some(RuntimeTransportTransitionKind::LoopStateChanged) => {
            RuntimePreworkInvalidationReason::TransportLoopStateChanged
        }
        Some(RuntimeTransportTransitionKind::LoopWrapped) => {
            RuntimePreworkInvalidationReason::TransportLoopWrapped
        }
        None => RuntimePreworkInvalidationReason::TransportSeeked,
    }
}

pub(crate) fn transport_projection_from_context(
    context: &GraphExecutionContext,
) -> TransportProjection {
    TransportProjection {
        playing: context.transport_playing,
        timeline_position_samples: context.timeline_position_samples,
        tempo_bpm: context.transport_tempo_bpm,
        loop_state: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RuntimePendingTransportTransition {
    pub(crate) kind: RuntimeTransportTransitionKind,
    pub(crate) effective_block_sequence: Option<u64>,
    pub(crate) transport_epoch: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeEngineTransportAdvance {
    pub(crate) start_samples: Option<i64>,
    pub(crate) end_samples: Option<i64>,
    pub(crate) loop_wrapped: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeTimelineState {
    pub(crate) next_block_sequence: u64,
    pub(crate) continuity: BlockSequenceContinuityReport,
    pub(crate) transport_epoch: u64,
    pub(crate) last_transport_transition: Option<RuntimeTransportTransitionKind>,
    pub(crate) last_transport_transition_processing_epoch: Option<u64>,
    pub(crate) last_transport_transition_block_sequence: Option<u64>,
    pub(crate) pending_transport_transition: Option<RuntimePendingTransportTransition>,
    pub(crate) last_transport_playing: Option<bool>,
    pub(crate) last_transport_tempo_bpm: Option<f64>,
    pub(crate) last_transport_timeline_position_samples: Option<i64>,
    pub(crate) last_transport_loop_start_samples: Option<i64>,
    pub(crate) last_transport_loop_end_samples: Option<i64>,
    pub(crate) last_engine_block_start_samples: Option<i64>,
    pub(crate) last_engine_block_end_samples: Option<i64>,
    pub(crate) loop_wrap_count: u64,
}

impl RuntimeTimelineState {
    pub(crate) fn allocate_block_sequence(&mut self) -> u64 {
        let block_sequence = self.next_block_sequence;
        self.next_block_sequence = self.next_block_sequence.saturating_add(1);
        block_sequence
    }

    pub(crate) fn record_block_sequence(
        &mut self,
        sandbox_id: &str,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) -> Option<LeaseRolloverRecord> {
        let lease_id = lease_id.into();
        let previous = self.continuity.segments.last().cloned();
        self.continuity
            .record(processing_epoch, lease_id.clone(), block_sequence);
        previous.and_then(|segment| {
            (segment.lease_id != lease_id).then(|| LeaseRolloverRecord {
                sandbox_id: sandbox_id.to_string(),
                previous_lease_id: segment.lease_id,
                lease_id,
                processing_epoch,
                first_block_sequence: block_sequence,
            })
        })
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn record_transport_projection(
        &mut self,
        kind: RuntimeTransportTransitionKind,
        effective_block_sequence: Option<u64>,
        processing_epoch: Option<u64>,
        projection: TransportProjection,
    ) -> u64 {
        self.transport_epoch = self.transport_epoch.saturating_add(1);
        self.last_transport_transition = Some(kind);
        self.last_transport_transition_processing_epoch = processing_epoch;
        self.last_transport_transition_block_sequence = effective_block_sequence;
        self.pending_transport_transition = Some(RuntimePendingTransportTransition {
            kind,
            effective_block_sequence,
            transport_epoch: self.transport_epoch,
        });
        self.update_transport_state(projection);
        self.transport_epoch
    }

    pub(crate) fn consume_pending_transport_transition(
        &mut self,
        block_sequence: u64,
    ) -> Option<RuntimePendingTransportTransition> {
        match self.pending_transport_transition {
            Some(pending)
                if pending
                    .effective_block_sequence
                    .is_none_or(|effective| effective == block_sequence) =>
            {
                self.pending_transport_transition = None;
                Some(pending)
            }
            _ => None,
        }
    }

    pub(crate) fn record_loop_wrap(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        projection: TransportProjection,
    ) -> u64 {
        self.transport_epoch = self.transport_epoch.saturating_add(1);
        self.loop_wrap_count = self.loop_wrap_count.saturating_add(1);
        self.last_transport_transition = Some(RuntimeTransportTransitionKind::LoopWrapped);
        self.last_transport_transition_processing_epoch = Some(processing_epoch);
        self.last_transport_transition_block_sequence = Some(block_sequence);
        self.update_transport_state(projection);
        self.transport_epoch
    }

    pub(crate) fn record_engine_block_window(
        &mut self,
        start_samples: Option<i64>,
        end_samples: Option<i64>,
    ) {
        self.last_engine_block_start_samples = start_samples;
        self.last_engine_block_end_samples = end_samples;
    }

    pub(crate) fn update_transport_state(&mut self, projection: TransportProjection) {
        self.last_transport_playing = Some(projection.playing);
        self.last_transport_tempo_bpm = Some(projection.tempo_bpm);
        self.last_transport_timeline_position_samples = Some(projection.timeline_position_samples);
        self.last_transport_loop_start_samples = projection
            .loop_state
            .map(|loop_region| loop_region.start_samples);
        self.last_transport_loop_end_samples = projection
            .loop_state
            .map(|loop_region| loop_region.end_samples);
    }

    pub(crate) fn snapshot(&self) -> RuntimeTimelineSnapshot {
        RuntimeTimelineSnapshot {
            next_block_sequence: self.next_block_sequence,
            block_sequence_continuity: self.continuity.clone(),
            transport_epoch: self.transport_epoch,
            last_transport_transition: self.last_transport_transition,
            last_transport_transition_processing_epoch: self
                .last_transport_transition_processing_epoch,
            last_transport_transition_block_sequence: self.last_transport_transition_block_sequence,
            last_transport_playing: self.last_transport_playing,
            last_transport_tempo_bpm: self.last_transport_tempo_bpm,
            last_transport_timeline_position_samples: self.last_transport_timeline_position_samples,
            last_transport_loop_start_samples: self.last_transport_loop_start_samples,
            last_transport_loop_end_samples: self.last_transport_loop_end_samples,
            last_engine_block_start_samples: self.last_engine_block_start_samples,
            last_engine_block_end_samples: self.last_engine_block_end_samples,
            loop_wrap_count: self.loop_wrap_count,
        }
    }
}
