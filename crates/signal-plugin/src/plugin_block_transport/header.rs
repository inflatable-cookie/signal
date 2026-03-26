#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockProcessingHeader {
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub channel_count: u16,
    pub frame_count: u32,
}

impl BlockProcessingHeader {
    pub const ENCODED_BYTES: usize = 24;

    pub fn write_to_slice(&self, bytes: &mut [u8]) -> Result<(), &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("render-context region is too small for encoded block header");
        }

        bytes[0..8].copy_from_slice(&self.processing_epoch.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.block_sequence.to_le_bytes());
        bytes[16..18].copy_from_slice(&self.channel_count.to_le_bytes());
        bytes[18..20].fill(0);
        bytes[20..24].copy_from_slice(&self.frame_count.to_le_bytes());
        Ok(())
    }

    pub fn read_from_slice(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("render-context region is too small for encoded block header");
        }

        Ok(Self {
            processing_epoch: u64::from_le_bytes(
                bytes[0..8]
                    .try_into()
                    .map_err(|_| "processing_epoch decode")?,
            ),
            block_sequence: u64::from_le_bytes(
                bytes[8..16]
                    .try_into()
                    .map_err(|_| "block_sequence decode")?,
            ),
            channel_count: u16::from_le_bytes(
                bytes[16..18]
                    .try_into()
                    .map_err(|_| "channel_count decode")?,
            ),
            frame_count: u32::from_le_bytes(
                bytes[20..24].try_into().map_err(|_| "frame_count decode")?,
            ),
        })
    }
}
