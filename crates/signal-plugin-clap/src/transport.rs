use signal_plugin::SharedMemoryLayout;

use crate::protocol::ClapBlockProtocol;

mod block_io;
mod lifecycle;

impl ClapBlockProtocol {
    fn shared_memory_layout(&self, max_block_frames: u32) -> SharedMemoryLayout {
        let audio_bytes = self.io_layout.audio_channels() as u32
            * max_block_frames
            * core::mem::size_of::<f32>() as u32;
        SharedMemoryLayout::single_block(audio_bytes, self.event_capacity_bytes)
    }
}
