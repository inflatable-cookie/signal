use super::super::super::super::*;
use super::super::PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;

impl RuntimeEngineState {
    pub(super) fn apply_graph_block_report(
        &mut self,
        GraphBlockReport {
            graph_id,
            context,
            node_count,
            stateful_node_count,
            latency_node_count,
            plugin_backed_node_count,
            phase_count,
            anticipative_phase_count,
            phase_order,
            lane_count,
            anticipative_lane_count,
            lane_order,
            dispatch_count,
            dispatch_boundary_count,
            dispatch_order,
            prepared_dispatch_count,
            realtime_dispatch_count,
            dispatch_handoff_count,
            stage_count,
            dynamic_kernel_stage_count,
            dynamic_stage_state_model,
            total_latency_samples,
            max_node_latency_samples,
            total_tail_samples,
            max_node_tail_samples,
            output_tail_samples,
            max_bus_tail_samples,
            parameter_epoch,
            parameter_event_count,
            parameter_targeted_node_count,
            parameter_ignored_event_count,
            parameter_sub_block_count,
            parameter_coalesced_event_count,
            frame_count,
            channel_count,
            input_peak,
            prework_output_peak,
            realtime_input_peak,
            output_peak,
            output_rms,
            bus_levels,
            first_output_sample,
            ..
        }: GraphBlockReport,
        contract: &signal_graph::GraphContractSummary,
        cache_hit: bool,
        prepared_was_used: bool,
    ) -> Vec<RuntimeMeterSourceSnapshot> {
        self.snapshot.graph_id = Some(graph_id);
        self.snapshot.node_count = node_count;
        self.snapshot.stateful_node_count = stateful_node_count;
        self.snapshot.latency_node_count = latency_node_count;
        self.snapshot.plugin_backed_node_count = plugin_backed_node_count;
        self.snapshot.phase_count = phase_count;
        self.snapshot.anticipative_phase_count = anticipative_phase_count;
        self.snapshot.phase_order = phase_order;
        self.snapshot.lane_count = lane_count;
        self.snapshot.anticipative_lane_count = anticipative_lane_count;
        self.snapshot.lane_order = lane_order;
        self.snapshot.dispatch_count = dispatch_count;
        self.snapshot.dispatch_boundary_count = dispatch_boundary_count;
        self.snapshot.dispatch_order = dispatch_order;
        self.snapshot.prepared_dispatch_count = prepared_dispatch_count;
        self.snapshot.realtime_dispatch_count = realtime_dispatch_count;
        self.snapshot.dispatch_handoff_count = dispatch_handoff_count;
        self.snapshot.prework_cache_enabled = prepared_dispatch_count > 0;
        if !self.snapshot.prework_cache_enabled {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
        } else if !self.prework_queue.is_empty() {
            self.snapshot.prework_cache_state = if cache_hit {
                RuntimePreworkCacheState::Consumed
            } else {
                RuntimePreworkCacheState::Admitted
            };
        } else if !matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
        }
        self.snapshot.last_prework_cache_hit = cache_hit;
        self.snapshot.prework_cache_block_freshness_window = PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        if !cache_hit && self.prework_queue.is_empty() {
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
        }
        if !cache_hit && prepared_was_used {
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            if let Some(cache) = self
                .prework_queue
                .iter_mut()
                .rev()
                .find(|cache| cache.source_block_sequence == context.block_sequence)
            {
                self.snapshot.last_prework_source_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_source_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admission_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_admission_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admitted_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                self.snapshot.last_prework_consumed_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                cache.consumption_count = cache.consumption_count.saturating_add(1);
            }
        } else if !cache_hit {
            self.snapshot.last_prework_consumed_from_block_sequence = None;
        }
        self.snapshot.stage_count = stage_count;
        self.snapshot.dynamic_kernel_stage_count = dynamic_kernel_stage_count;
        self.snapshot.dynamic_stage_state_model = dynamic_stage_state_model;
        self.snapshot.total_latency_samples = total_latency_samples;
        self.snapshot.max_node_latency_samples = max_node_latency_samples;
        self.snapshot.total_tail_samples = total_tail_samples;
        self.snapshot.max_node_tail_samples = max_node_tail_samples;
        self.snapshot.output_tail_samples = output_tail_samples;
        self.snapshot.max_bus_tail_samples = max_bus_tail_samples;
        self.snapshot.parameter_epoch = parameter_epoch;
        self.snapshot.parameter_event_count = parameter_event_count;
        self.snapshot.parameter_targeted_node_count = parameter_targeted_node_count;
        self.snapshot.parameter_ignored_event_count = parameter_ignored_event_count;
        self.snapshot.parameter_sub_block_count = parameter_sub_block_count;
        self.snapshot.parameter_coalesced_event_count = parameter_coalesced_event_count;
        self.snapshot.processed_blocks = self.snapshot.processed_blocks.saturating_add(1);
        self.snapshot.last_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_block_sequence = Some(context.block_sequence);
        self.snapshot.last_frame_count = frame_count;
        self.snapshot.last_channel_count = channel_count;
        self.snapshot.last_input_peak = Some(input_peak);
        self.snapshot.last_prework_output_peak = prework_output_peak;
        self.snapshot.last_realtime_input_peak = realtime_input_peak;
        self.snapshot.last_output_peak = Some(output_peak);
        self.snapshot.last_output_rms = Some(output_rms);
        self.snapshot.last_first_output_sample = first_output_sample;
        self.snapshot.last_execution_context = Some(context);

        bus_levels
            .into_iter()
            .map(|level| {
                let bus_id = level.bus_id.clone();
                let RuntimeMeterContractMetadata {
                    topology_role,
                    track_lane_id,
                    bus_group_id,
                    console_group_id,
                    send_return_id,
                    producer_node_ids,
                } = RuntimeMeteringStateModel::meter_contract_metadata(
                    contract,
                    level.bus_id.as_str(),
                );
                RuntimeMeterSourceSnapshot {
                    bus_id: bus_id.clone(),
                    topology_role,
                    track_lane_id,
                    bus_group_id,
                    console_group_id,
                    send_return_id,
                    producer_node_ids: producer_node_ids.clone(),
                    peak_level: level.peak,
                    rms_level: level.rms,
                    latency_samples: level.latency_samples,
                    tail_samples: level.tail_samples,
                }
            })
            .collect()
    }
}
