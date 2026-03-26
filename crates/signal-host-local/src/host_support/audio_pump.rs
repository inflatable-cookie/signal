use signal_hardware::HardwareStreamConfig;
use signal_primitives::AudioBuffer;

use super::{
    transfer_runtime_output_to_host_buffer, LocalAudioPumpSummary, LocalAudioStreamState,
    LocalAudioTransferPolicy,
};

#[derive(Clone, Debug)]
pub(crate) struct LocalAudioPumpState {
    summary: LocalAudioPumpSummary,
}

impl Default for LocalAudioPumpState {
    fn default() -> Self {
        Self {
            summary: LocalAudioPumpSummary {
                stream_state: LocalAudioStreamState::Stopped,
                transfer_policy: LocalAudioTransferPolicy {
                    max_callback_frames: 0,
                    max_transfer_channels: 0,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 0,
                last_callback_index: None,
                total_callback_frames: 0,
                total_runtime_output_frames: 0,
                copied_output_samples: 0,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: None,
                last_runtime_graph_id: None,
            },
        }
    }
}

impl LocalAudioPumpState {
    pub(crate) fn reset_for_stream(&mut self, stream: &HardwareStreamConfig) {
        let transfer_policy = LocalAudioTransferPolicy {
            max_callback_frames: stream.buffer_size,
            max_transfer_channels: stream.output_channels,
            zero_fill_unwritten_output: true,
        };
        self.summary = LocalAudioPumpSummary {
            stream_state: LocalAudioStreamState::Stopped,
            transfer_policy,
            ..Self::default().summary
        };
    }

    pub(crate) fn stop(&mut self) {
        self.summary.stream_state = LocalAudioStreamState::Stopped;
    }

    pub(crate) fn fault(&mut self) {
        self.summary.stream_state = LocalAudioStreamState::Faulted;
        self.summary.last_runtime_graph_id = None;
    }

    pub(crate) fn summary(&self) -> LocalAudioPumpSummary {
        self.summary.clone()
    }

    pub(crate) fn record_callback(
        &mut self,
        stream: &HardwareStreamConfig,
        callback_index: u64,
        runtime_output: &AudioBuffer,
        runtime_graph_id: Option<&str>,
    ) {
        self.summary.stream_state = LocalAudioStreamState::Running;
        self.summary.last_callback_index = Some(callback_index);
        self.summary.callback_count = self.summary.callback_count.saturating_add(1);
        self.summary.total_callback_frames = self
            .summary
            .total_callback_frames
            .saturating_add(stream.buffer_size as u64);
        self.summary.total_runtime_output_frames = self
            .summary
            .total_runtime_output_frames
            .saturating_add(runtime_output.frames().0 as u64);
        if let Some(graph_id) = runtime_graph_id {
            self.summary.last_runtime_graph_id = Some(graph_id.to_string());
        }

        let transfer = transfer_runtime_output_to_host_buffer(
            runtime_output,
            stream,
            self.summary.transfer_policy.into(),
        );
        self.summary.copied_output_samples = self
            .summary
            .copied_output_samples
            .saturating_add(transfer.outcome.copied_samples as u64);
        self.summary.zero_filled_output_samples = self
            .summary
            .zero_filled_output_samples
            .saturating_add(transfer.outcome.zero_filled_samples as u64);
        self.summary.dropped_output_samples = self
            .summary
            .dropped_output_samples
            .saturating_add(transfer.outcome.dropped_samples as u64);
        self.summary.last_callback_output_peak = Some(transfer.output_peak);
    }
}
