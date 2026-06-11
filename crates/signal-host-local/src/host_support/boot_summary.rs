use signal_hardware::HardwareStreamConfig;
use signal_runtime::RuntimeEngineBlockResult;

use super::super::{
    LocalEngineSummary, LocalHardwareSummary, LocalRuntimeHost, LocalRuntimeHostSummary,
};

impl LocalRuntimeHost {
    pub(crate) fn summarize_boot_outcome(
        &self,
        hardware_stream: &HardwareStreamConfig,
        last_engine_result: Option<(u64, RuntimeEngineBlockResult)>,
    ) -> LocalRuntimeHostSummary {
        let observation = self.observation_report();
        LocalRuntimeHostSummary {
            backend_name: self.hardware.backend_name(),
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
                backend_diagnostics: self.hardware.diagnostics(),
            },
            audio_pump: self.audio_pump.summary(),
            scan_roots: self.supervisor.last_scan_roots.clone(),
            engine: LocalEngineSummary {
                processed_blocks: self.audio_pump.summary().callback_count,
                last_block_sequence: last_engine_result
                    .as_ref()
                    .map(|(block_sequence, _)| *block_sequence),
                last_graph_id: last_engine_result
                    .as_ref()
                    .and_then(|(_, result)| result.snapshot.graph_id.clone()),
                last_output_peak: last_engine_result
                    .as_ref()
                    .and_then(|(_, result)| result.snapshot.last_output_peak),
                last_output_rms: last_engine_result
                    .as_ref()
                    .and_then(|(_, result)| result.snapshot.last_output_rms),
            },
            topology: observation.execution_topology_summary.clone(),
        }
    }
}
