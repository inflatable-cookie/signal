use signal_hardware::HardwareStreamConfig;

use super::super::{LocalHardwareSummary, LocalRuntimeHost, LocalRuntimeHostSummary};
use super::LocalAudioPumpSummary;

impl LocalRuntimeHost {
    pub(crate) fn summarize_boot_outcome(
        &self,
        hardware_stream: &HardwareStreamConfig,
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
            audio_pump: LocalAudioPumpSummary {
                stream_state: self.stream_state,
            },
            scan_roots: self.supervisor.last_scan_roots.clone(),
            topology: observation.execution_topology_summary.clone(),
        }
    }
}
