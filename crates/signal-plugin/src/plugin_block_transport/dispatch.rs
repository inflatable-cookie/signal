use crate::{
    write_render_context_to_slice, PluginInstanceId, PluginIoLayout, PluginRenderContext,
    SandboxTransport, SharedMemoryLayout,
};

use super::header::BlockProcessingHeader;

/// All information the sandbox needs to process a single audio block.
///
/// `BlockDispatch` is serialised into the shared-memory render-context region before the host
/// signals the sandbox to begin processing, and deserialised by the sandbox process at the start
/// of each block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDispatch {
    /// Instance that should process this block.
    pub instance_id: PluginInstanceId,
    /// Block identity and channel/frame geometry.
    pub header: BlockProcessingHeader,
    /// Audio and MIDI bus counts for this block.
    pub io_layout: PluginIoLayout,
    /// IPC transport to use when reading/writing audio and events.
    pub transport: SandboxTransport,
    /// Byte-range map of the shared-memory regions for this block.
    pub layout: SharedMemoryLayout,
    /// Transport and timing context for this block.
    pub render_context: PluginRenderContext,
}

impl BlockDispatch {
    /// Constructs a `BlockDispatch` for shared-memory transport, computing the audio byte budget
    /// from `io_layout` and `frame_count`.
    pub fn new(
        instance_id: PluginInstanceId,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        io_layout: PluginIoLayout,
        render_context: PluginRenderContext,
        event_bytes: u32,
    ) -> Self {
        let channel_count = io_layout.audio_channels();
        let audio_bytes = channel_count as u32 * frame_count * core::mem::size_of::<f32>() as u32;

        Self {
            instance_id,
            header: BlockProcessingHeader {
                processing_epoch,
                block_sequence,
                channel_count,
                frame_count,
            },
            io_layout,
            transport: SandboxTransport::SharedMemory,
            layout: SharedMemoryLayout::single_block(audio_bytes, event_bytes),
            render_context,
        }
    }

    /// Encodes the block header and render context into the shared-memory render-context region.
    pub fn write_to_shared_memory(&self, bytes: &mut [u8]) -> Result<(), &'static str> {
        let render_region = self
            .layout
            .region_slice_mut(bytes, self.layout.render_context)?;
        let packet_bytes =
            BlockProcessingHeader::ENCODED_BYTES + PluginRenderContext::ENCODED_BYTES;
        if render_region.len() < packet_bytes {
            return Err("render-context region is too small for block packet");
        }

        self.header
            .write_to_slice(&mut render_region[..BlockProcessingHeader::ENCODED_BYTES])?;
        write_render_context_to_slice(
            &self.render_context,
            &mut render_region[BlockProcessingHeader::ENCODED_BYTES..packet_bytes],
        )?;
        Ok(())
    }

    /// Decodes a `BlockDispatch` from the shared-memory render-context region.
    pub fn read_from_shared_memory(
        instance_id: PluginInstanceId,
        io_layout: PluginIoLayout,
        layout: SharedMemoryLayout,
        bytes: &[u8],
    ) -> Result<Self, &'static str> {
        let render_region = layout.region_slice(bytes, layout.render_context)?;
        let packet_bytes =
            BlockProcessingHeader::ENCODED_BYTES + PluginRenderContext::ENCODED_BYTES;
        if render_region.len() < packet_bytes {
            return Err("render-context region is too small for block packet");
        }

        let header = BlockProcessingHeader::read_from_slice(
            &render_region[..BlockProcessingHeader::ENCODED_BYTES],
        )?;
        let render_context = crate::read_render_context_from_slice(
            &render_region[BlockProcessingHeader::ENCODED_BYTES..packet_bytes],
        )?;

        Ok(Self {
            instance_id,
            header,
            io_layout,
            transport: SandboxTransport::SharedMemory,
            layout,
            render_context,
        })
    }
}
