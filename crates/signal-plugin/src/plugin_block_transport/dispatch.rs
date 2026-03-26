use crate::{
    PluginInstanceId, PluginIoLayout, PluginRenderContext, SandboxTransport, SharedMemoryLayout,
};

use super::header::BlockProcessingHeader;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockDispatch {
    pub instance_id: PluginInstanceId,
    pub header: BlockProcessingHeader,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub layout: SharedMemoryLayout,
    pub render_context: PluginRenderContext,
}

impl BlockDispatch {
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
        self.render_context.write_to_slice(
            &mut render_region[BlockProcessingHeader::ENCODED_BYTES..packet_bytes],
        )?;
        Ok(())
    }

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
        let render_context = PluginRenderContext::read_from_slice(
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
