use signal_plugin::{BlockPayload, CompletionState, LoopRange, PluginEvent, PluginRenderContext};
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout};
use signal_runtime::{
    PluginNodeRender, PluginNodeRenderBatch, RuntimeError, RuntimePluginDispatchState,
};

use super::super::{
    LocalRuntimeHost, LOCAL_DEMO_GRAPH_ID, LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES,
    LOCAL_DEMO_PLUGIN_NODE_ID, LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
};
use super::{
    demo_interaction_parameter_step, plugin_automation_value_from_runtime_batch,
    LifecycleRunSummary,
};

impl LocalRuntimeHost {
    pub(crate) fn build_plugin_block_request(
        &self,
        protocol: &ClapBlockProtocol,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        plugin_dispatch_state: &RuntimePluginDispatchState,
    ) -> Result<(signal_plugin::BlockDispatch, BlockPayload), RuntimeError> {
        let dispatch = protocol.block_dispatch(
            processing_epoch,
            block_sequence,
            frame_count,
            self.plugin_render_context(frame_count, plugin_dispatch_state),
        );
        let payload = self.plugin_input_payload(
            protocol,
            block_sequence,
            frame_count,
            plugin_dispatch_state.parameter_batch.as_ref(),
        )?;
        Ok((dispatch, payload))
    }

    pub(crate) fn plugin_render_context(
        &self,
        frame_count: u32,
        plugin_dispatch_state: &RuntimePluginDispatchState,
    ) -> PluginRenderContext {
        let transport = plugin_dispatch_state.transport;
        PluginRenderContext {
            sample_rate_hz: self.runtime.config().sample_rate.0,
            tempo_bpm: transport
                .map(|transport| transport.tempo_bpm)
                .unwrap_or(120.0),
            timeline_position_samples: transport
                .map(|transport| transport.timeline_position_samples)
                .unwrap_or(0),
            playing: transport
                .map(|transport| transport.playing)
                .unwrap_or(false),
            bypassed: false,
            loop_range: transport
                .and_then(|transport| transport.loop_state)
                .map(|loop_state| LoopRange {
                    start_samples: loop_state.start_samples,
                    end_samples: loop_state.end_samples,
                }),
            deadline_frames: frame_count,
        }
    }

    pub(crate) fn plugin_input_payload(
        &self,
        protocol: &ClapBlockProtocol,
        block_sequence: u64,
        frame_count: u32,
        parameter_batch: Option<&signal_runtime::ParameterBatch>,
    ) -> Result<BlockPayload, RuntimeError> {
        let mut payload = protocol.test_input_payload(block_sequence, frame_count);
        let automation_parameter_id = protocol.automation_parameter_id();
        let automation_value =
            plugin_automation_value_from_runtime_batch(automation_parameter_id, parameter_batch)
                .or_else(|| demo_interaction_parameter_step(automation_parameter_id));
        for event in &mut payload.events.events {
            if let (PluginEvent::ParameterValue(existing), Some(automation_value)) =
                (event, automation_value)
            {
                if existing.parameter_id == automation_parameter_id {
                    *existing = automation_value;
                }
            }
        }

        let expected_audio_samples =
            payload.audio.channel_count as usize * payload.audio.frame_count as usize;
        if payload.audio.samples.len() != expected_audio_samples {
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::PluginFailure,
                "plugin input payload audio shape became invalid",
            ));
        }
        Ok(payload)
    }

    pub(crate) fn apply_plugin_node_render_for_block(
        &mut self,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        stored_result: &BrokeredBlockOutcome,
    ) -> Result<(), RuntimeError> {
        let render_bypassed = stored_result.result.slot.state != CompletionState::Completed;
        if render_bypassed {
            run.plugin_render_bypass_count = run.plugin_render_bypass_count.saturating_add(1);
        }
        run.last_plugin_render_bypassed = render_bypassed;
        run.last_plugin_render_latency_samples = LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES;
        run.last_plugin_render_tail_samples = LOCAL_DEMO_PLUGIN_TAIL_SAMPLES;
        let channel_layout = match stored_result.output.audio.channel_count {
            1 => ChannelLayout::Mono,
            2 => ChannelLayout::Stereo,
            count => ChannelLayout::Count(ChannelCount(count as usize)),
        };
        let output = AudioBuffer::from_interleaved(
            self.runtime.config().sample_rate,
            channel_layout,
            stored_result.output.audio.samples.clone(),
        );
        self.runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: LOCAL_DEMO_GRAPH_ID.into(),
                processing_epoch: run.processing_epoch,
                block_sequence,
                renders: vec![PluginNodeRender {
                    node_id: LOCAL_DEMO_PLUGIN_NODE_ID.into(),
                    sandbox_id: run.sandbox_id.clone(),
                    output,
                    latency_samples: LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES,
                    tail_samples: LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
                    bypassed: render_bypassed,
                }],
            })
    }
}
