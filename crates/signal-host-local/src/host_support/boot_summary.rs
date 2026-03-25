use signal_hardware::{HardwareBackend, HardwareStreamConfig};
use signal_plugin_clap::ClapBlockProtocol;

use super::super::{
    LifecycleRunSummary, LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary,
    LocalPayloadSummary, LocalPluginDispatchSummary, LocalRuntimeHost, LocalRuntimeHostSummary,
    LocalTransportSummary,
};

impl LocalRuntimeHost {
    pub(crate) fn summarize_boot_outcome(
        &self,
        hardware_stream: &HardwareStreamConfig,
        sandbox_id: &str,
        protocol: &ClapBlockProtocol,
        run: LifecycleRunSummary,
    ) -> LocalRuntimeHostSummary {
        let header = protocol.block_header(
            run.processing_epoch,
            run.last_block_sequence,
            self.runtime.config().graph.block_size as u32,
        );
        let observation = self.observation_report();
        LocalRuntimeHostSummary {
            backend_name: self.coreaudio.backend_name(),
            hardware: LocalHardwareSummary {
                device_id: hardware_stream.device.device_id.clone(),
                device_name: hardware_stream.device.name.clone(),
                sample_rate: hardware_stream.sample_rate.0,
                buffer_size: hardware_stream.buffer_size,
                input_channels: hardware_stream.input_channels,
                output_channels: hardware_stream.output_channels,
                sample_format: hardware_stream.sample_format,
                lifecycle: hardware_stream.lifecycle,
                simulated: hardware_stream.simulated,
                backend_diagnostics: self.coreaudio.diagnostics(),
            },
            audio_pump: self.audio_pump.summary(),
            scan_roots: self.supervisor.last_scan_roots.clone(),
            execution: LocalExecutionSummary {
                control_requests: run.control_requests,
                control_responses: run.control_responses,
                heartbeat_responses: run.heartbeat_responses,
                processed_blocks: run.processed_blocks,
                engine_processed_blocks: run.engine_processed_blocks,
                last_control_message: run.last_control_message.clone(),
                last_completion_state: run.last_completion_state,
                last_block_sequence: run.last_block_sequence,
                last_engine_graph_id: run.last_engine_graph_id.clone(),
                last_engine_output_peak: run.last_engine_output_peak,
                last_engine_output_rms: run.last_engine_output_rms,
                processing_epoch: run.processing_epoch,
                restart_count: self.supervisor.restarts,
                teardown_count: self.supervisor.teardowns,
                last_recovery_intent: self.supervisor.last_recovery_intent,
                last_stop_reason: self.supervisor.last_stop_reason,
                last_plugin_state: run.last_plugin_state.clone(),
            },
            transport: LocalTransportSummary {
                sandbox_id: self
                    .supervisor
                    .last_sandbox_id
                    .clone()
                    .unwrap_or_else(|| sandbox_id.to_string()),
                shared_memory_lease_id: run.shared_memory_lease_id,
                shared_memory_region_id: run
                    .transport
                    .as_ref()
                    .map(|transport| transport.region_id.clone())
                    .unwrap_or_default(),
                shared_memory_path: run
                    .transport
                    .as_ref()
                    .map(|transport| transport.backing_path.clone())
                    .unwrap_or_default(),
                shared_memory_bytes: header.layout.total_bytes(),
            },
            topology: observation.execution_topology_summary.clone(),
            plugin_dispatch: run.last_plugin_render_context.clone().map(|render_context| {
                LocalPluginDispatchSummary {
                    processing_epoch: run.processing_epoch,
                    block_sequence: run.last_block_sequence,
                    render_context,
                    automation_value: run.last_plugin_automation_value,
                    render_bypass_count: run.plugin_render_bypass_count,
                    last_render_bypassed: run.last_plugin_render_bypassed,
                    last_render_latency_samples: run.last_plugin_render_latency_samples,
                    last_render_tail_samples: run.last_plugin_render_tail_samples,
                }
            }),
            last_payload: LocalPayloadSummary {
                event_count: run.last_output_event_count,
                parameter_event_count: run.last_parameter_event_count,
                parameter_gesture_event_count: run.last_parameter_gesture_event_count,
                parameter_modulation_event_count: run.last_parameter_modulation_event_count,
                note_event_count: run.last_note_event_count,
                note_expression_event_count: run.last_note_expression_event_count,
                midi_event_count: run.last_midi_event_count,
                generated_event_bytes: run.last_generated_event_bytes,
                first_output_sample: run.last_output_first_sample,
            },
            faults: LocalFaultSummary {
                deadline_misses: run.deadline_misses,
                heartbeat_misses: run.heartbeat_misses,
                watchdog_triggered: run.watchdog_triggered,
                watchdog_trigger_reason: run.watchdog_trigger_reason,
            },
        }
    }
}
