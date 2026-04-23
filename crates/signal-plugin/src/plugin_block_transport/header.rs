/// Fixed-size metadata written at the start of every block's render-context region.
///
/// The header identifies the block within its epoch and describes the audio geometry so the
/// sandbox can validate the shared-memory layout before processing begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockProcessingHeader {
    /// Processing epoch this block belongs to; changes on activate/reset.
    pub processing_epoch: u64,
    /// Monotonically increasing counter within the current epoch.
    pub block_sequence: u64,
    /// Number of audio channels in the block's audio buffers.
    pub channel_count: u16,
    /// Number of audio frames in this block.
    pub frame_count: u32,
}

impl BlockProcessingHeader {
    /// Encoded size of the header in the shared-memory wire format.
    pub const ENCODED_BYTES: usize = 24;

    /// Encodes this header into `bytes`.
    ///
    /// Returns an error if `bytes.len() < ENCODED_BYTES`.
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

    /// Decodes a `BlockProcessingHeader` from `bytes`.
    ///
    /// Returns an error if `bytes.len() < ENCODED_BYTES` or a field cannot be decoded.
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
