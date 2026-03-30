use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount};
use signal_runtime::{RuntimeError, RuntimeErrorKind};

use super::super::LocalRuntimeHost;

impl LocalRuntimeHost {
    pub(crate) fn process_engine_block_through_output_pump(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<signal_runtime::RuntimeEngineBlockResult, RuntimeError> {
        let Some(stream) = self.active_output_stream.clone() else {
            self.audio_pump.fault();
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "local host audio pump has no negotiated output stream",
            ));
        };
        let input = AudioBuffer::new(
            self.runtime.config().sample_rate,
            ChannelLayout::Count(ChannelCount(stream.output_channels as usize)),
            FrameCount(stream.buffer_size),
        );
        let result = self
            .runtime
            .process_engine_block(processing_epoch, block_sequence, input)
            .inspect_err(|_| self.audio_pump.fault())?;
        self.audio_pump.record_callback(
            &stream,
            block_sequence,
            &result.output,
            result.snapshot.graph_id.as_deref(),
        );
        Ok(result)
    }
}
