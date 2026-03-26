use super::super::*;
use super::{
    RuntimeEnginePreworkCache, PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW, PREWORK_QUEUE_CAPACITY,
};

impl RuntimeEngineState {
    pub(crate) fn admit_prework_for_block(
        &mut self,
        context: GraphExecutionContext,
        transport: Option<TransportProjection>,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        let (graph_id, planning) = {
            let graph = self.graph.as_ref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?;
            (
                graph.graph_id().to_string(),
                graph.planning_summary(context.anticipative_enabled),
            )
        };
        if planning
            .dispatches
            .iter()
            .all(|dispatch| dispatch.lane != signal_graph::GraphExecutionLane::Anticipative)
        {
            self.snapshot.prework_cache_state = if context.anticipative_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        }

        let input_signature = hash_audio_buffer(&buffer);
        let already_matching = self.prework_queue.iter().any(|cache| {
            self.prework_cache_matches(cache, graph_id.as_str(), &context, &buffer, input_signature)
        });
        if already_matching {
            return Ok(true);
        }
        self.retire_prework_entries_matching(
            |cache| cache.source_block_sequence == context.block_sequence,
            RuntimePreworkInvalidationReason::SupersededByAdmission,
        );

        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let Some(prepared) = graph.prepare_anticipative(&buffer, &context, None) else {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        };

        self.snapshot.prework_cache_admissions =
            self.snapshot.prework_cache_admissions.saturating_add(1);
        if admitted_from_block_sequence < context.block_sequence {
            self.snapshot.prework_cache_queued_admissions = self
                .snapshot
                .prework_cache_queued_admissions
                .saturating_add(1);
        }
        self.snapshot.last_prework_admission_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_admission_block_sequence = Some(context.block_sequence);
        self.snapshot.last_prework_admitted_from_block_sequence =
            Some(admitted_from_block_sequence);
        self.prework_queue.push_back(RuntimeEnginePreworkCache {
            graph_id,
            projection_epoch: context.projection_epoch,
            parameter_epoch: context.parameter_epoch,
            transport: transport.unwrap_or_else(|| transport_projection_from_context(&context)),
            block_size: context.configured_block_size,
            frame_count: buffer.frames().0,
            channel_count: buffer.channel_count().0,
            input_signature,
            prepared,
            valid_until_processing_epoch: context.processing_epoch.saturating_add(1),
            valid_until_block_sequence: context
                .block_sequence
                .saturating_add(PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW),
            source_processing_epoch: context.processing_epoch,
            source_block_sequence: context.block_sequence,
            admitted_from_block_sequence,
            consumption_count: 0,
        });
        self.prework_queue
            .make_contiguous()
            .sort_by_key(|cache| cache.source_block_sequence);
        while self.prework_queue.len() > PREWORK_QUEUE_CAPACITY {
            let cache = self.prework_queue.pop_front().expect("queue not empty");
            self.retire_prework_entry(
                cache,
                RuntimePreworkInvalidationReason::QueueCapacityExceeded,
            );
        }
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Admitted;
        self.snapshot.last_prework_source_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_source_block_sequence = Some(context.block_sequence);
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(true)
    }

    pub(crate) fn snapshot(&self) -> RuntimeEngineBlockSnapshot {
        self.snapshot.clone()
    }

    pub(crate) fn prework_cache_matches(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> bool {
        context.anticipative_enabled
            && cache.graph_id == graph_id
            && cache.projection_epoch == context.projection_epoch
            && cache.parameter_epoch == context.parameter_epoch
            && cache.transport.playing == context.transport_playing
            && cache.transport.tempo_bpm == context.transport_tempo_bpm
            && cache.transport.timeline_position_samples == context.timeline_position_samples
            && cache.block_size == context.configured_block_size
            && cache.frame_count == buffer.frames().0
            && cache.channel_count == buffer.channel_count().0
            && cache.input_signature == input_signature
            && context.processing_epoch <= cache.valid_until_processing_epoch
            && context.block_sequence <= cache.valid_until_block_sequence
    }

    pub(crate) fn prework_cache_mismatch_reason(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<RuntimePreworkInvalidationReason> {
        if !context.anticipative_enabled {
            return Some(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if cache.graph_id != graph_id || cache.projection_epoch != context.projection_epoch {
            return Some(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        }
        if cache.parameter_epoch != context.parameter_epoch {
            return Some(RuntimePreworkInvalidationReason::ParameterBatchApplied);
        }
        if cache.transport.playing != context.transport_playing
            || cache.transport.tempo_bpm != context.transport_tempo_bpm
            || cache.transport.timeline_position_samples != context.timeline_position_samples
        {
            return Some(classify_transport_invalidation_reason(
                Some(cache.transport),
                transport_projection_from_context(context),
            ));
        }
        if context.processing_epoch > cache.valid_until_processing_epoch {
            return Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired);
        }
        if context.block_sequence > cache.valid_until_block_sequence {
            return Some(RuntimePreworkInvalidationReason::BlockSequenceExpired);
        }
        if cache.block_size != context.configured_block_size
            || cache.frame_count != buffer.frames().0
            || cache.channel_count != buffer.channel_count().0
            || cache.input_signature != input_signature
        {
            return Some(RuntimePreworkInvalidationReason::InputSignatureChanged);
        }
        None
    }

    pub(crate) fn prework_freshness_state(
        &self,
        cache: Option<&RuntimeEnginePreworkCache>,
        current_block_sequence: Option<u64>,
    ) -> RuntimePreworkFreshnessState {
        if !self.snapshot.prework_cache_enabled {
            return RuntimePreworkFreshnessState::Disabled;
        }
        if matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            return RuntimePreworkFreshnessState::Invalidated;
        }
        let Some(cache) = cache else {
            return RuntimePreworkFreshnessState::Empty;
        };
        let Some(current_block_sequence) = current_block_sequence else {
            return RuntimePreworkFreshnessState::Fresh;
        };
        let remaining = cache
            .valid_until_block_sequence
            .saturating_sub(current_block_sequence);
        match remaining {
            0 => RuntimePreworkFreshnessState::Exhausted,
            1 => RuntimePreworkFreshnessState::Expiring,
            _ => RuntimePreworkFreshnessState::Fresh,
        }
    }
}
