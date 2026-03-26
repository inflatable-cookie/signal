use super::super::super::super::*;

impl RuntimeEngineState {
    pub(super) fn prepare_dispatch_for_block(
        &mut self,
        graph_id: &str,
        context: &GraphExecutionContext,
        transport: Option<TransportProjection>,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Result<(Option<GraphPreparedDispatch>, bool, bool), RuntimeError> {
        let cache_hit_index =
            self.matching_prework_index(graph_id, context, buffer, input_signature);
        let cache_hit = cache_hit_index.is_some();

        let prepared = if cache_hit {
            self.snapshot.prework_cache_hits = self.snapshot.prework_cache_hits.saturating_add(1);
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            if self.prework_queue[cache_hit_index.expect("cache hit index present")]
                .admitted_from_block_sequence
                < context.block_sequence
            {
                self.snapshot.prework_cache_queued_consumptions = self
                    .snapshot
                    .prework_cache_queued_consumptions
                    .saturating_add(1);
            }
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            let cache = &mut self.prework_queue[cache_hit_index.expect("cache hit index present")];
            self.snapshot.last_prework_source_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            self.snapshot.last_prework_consumed_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            cache.consumption_count = cache.consumption_count.saturating_add(1);
            Some(cache.prepared.clone())
        } else {
            let anticipative_dispatches_present = self
                .graph
                .as_ref()
                .map(|graph| {
                    graph
                        .planning_summary(context.anticipative_enabled)
                        .dispatches
                        .iter()
                        .any(|dispatch| {
                            dispatch.lane == signal_graph::GraphExecutionLane::Anticipative
                        })
                })
                .unwrap_or(false);
            let planning = self
                .graph
                .as_ref()
                .map(|graph| graph.planning_summary(context.anticipative_enabled))
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "no executable graph has been applied",
                    )
                })?;
            if anticipative_dispatches_present {
                self.snapshot.prework_cache_misses =
                    self.snapshot.prework_cache_misses.saturating_add(1);
            }
            let admitted = self.admit_prework_for_block(
                context.clone(),
                transport,
                context.block_sequence,
                buffer.clone(),
            )?;
            let prepared = if admitted {
                self.prework_queue
                    .iter()
                    .rev()
                    .find(|cache| cache.source_block_sequence == context.block_sequence)
                    .map(|cache| cache.prepared.clone())
            } else {
                None
            };
            self.snapshot.prework_cache_state = if !self.prework_queue.is_empty() {
                RuntimePreworkCacheState::Admitted
            } else if planning
                .dispatches
                .iter()
                .any(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
            {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            prepared
        };
        let prepared_was_used = prepared.is_some();
        Ok((prepared, cache_hit, prepared_was_used))
    }

    pub(super) fn realize_plugin_node_renders(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Vec<GraphNodeRenderOverride> {
        self.take_plugin_node_render_batch(processing_epoch, block_sequence)
            .map(|batch| {
                for render in &batch.renders {
                    self.latest_plugin_node_renders.insert(
                        render.node_id.clone(),
                        RuntimePluginRenderedNodeState {
                            sandbox_id: render.sandbox_id.clone(),
                            output: render.output.clone(),
                            latency_samples: render.latency_samples,
                            tail_samples: render.tail_samples,
                            bypassed: render.bypassed,
                            processing_epoch: batch.processing_epoch,
                            block_sequence: batch.block_sequence,
                        },
                    );
                }
                batch
                    .renders
                    .into_iter()
                    .map(|render| GraphNodeRenderOverride {
                        node_id: render.node_id,
                        buffer: render.output,
                        latency_samples: render.latency_samples,
                        tail_samples: render.tail_samples,
                        bypassed: render.bypassed,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}
