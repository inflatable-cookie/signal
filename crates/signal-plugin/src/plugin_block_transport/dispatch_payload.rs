use crate::{
    read_event_from_slice, write_event_to_slice, AudioBlock, BlockDispatch, BlockPayload,
    EventPacket, PluginEvent, SharedMemoryRegion,
};

impl BlockDispatch {
    /// Encodes `block` into the audio-input region of the shared-memory buffer.
    pub fn write_audio_input(
        &self,
        bytes: &mut [u8],
        block: &AudioBlock,
    ) -> Result<(), &'static str> {
        self.write_audio_region(bytes, self.layout.audio_input, block)
    }

    /// Decodes an [`AudioBlock`] from the audio-input region of the shared-memory buffer.
    pub fn read_audio_input(&self, bytes: &[u8]) -> Result<AudioBlock, &'static str> {
        self.read_audio_region(bytes, self.layout.audio_input)
    }

    /// Encodes `block` into the audio-output region of the shared-memory buffer.
    pub fn write_audio_output(
        &self,
        bytes: &mut [u8],
        block: &AudioBlock,
    ) -> Result<(), &'static str> {
        self.write_audio_region(bytes, self.layout.audio_output, block)
    }

    /// Decodes an [`AudioBlock`] from the audio-output region of the shared-memory buffer.
    pub fn read_audio_output(&self, bytes: &[u8]) -> Result<AudioBlock, &'static str> {
        self.read_audio_region(bytes, self.layout.audio_output)
    }

    /// Encodes `packet` into the event-input region of the shared-memory buffer.
    pub fn write_event_input(
        &self,
        bytes: &mut [u8],
        packet: &EventPacket,
    ) -> Result<(), &'static str> {
        self.write_event_region(bytes, self.layout.event_input, packet)
    }

    /// Decodes an [`EventPacket`] from the event-input region of the shared-memory buffer.
    pub fn read_event_input(&self, bytes: &[u8]) -> Result<EventPacket, &'static str> {
        self.read_event_region(bytes, self.layout.event_input)
    }

    /// Encodes `packet` into the event-output region of the shared-memory buffer.
    pub fn write_event_output(
        &self,
        bytes: &mut [u8],
        packet: &EventPacket,
    ) -> Result<(), &'static str> {
        self.write_event_region(bytes, self.layout.event_output, packet)
    }

    /// Decodes an [`EventPacket`] from the event-output region of the shared-memory buffer.
    pub fn read_event_output(&self, bytes: &[u8]) -> Result<EventPacket, &'static str> {
        self.read_event_region(bytes, self.layout.event_output)
    }

    /// Encodes both the audio and event portions of `payload` into the input regions.
    pub fn write_input_payload(
        &self,
        bytes: &mut [u8],
        payload: &BlockPayload,
    ) -> Result<(), &'static str> {
        self.write_audio_input(bytes, &payload.audio)?;
        self.write_event_input(bytes, &payload.events)?;
        Ok(())
    }

    /// Decodes the audio and event input regions into a [`BlockPayload`].
    pub fn read_input_payload(&self, bytes: &[u8]) -> Result<BlockPayload, &'static str> {
        Ok(BlockPayload::new(
            self.read_audio_input(bytes)?,
            self.read_event_input(bytes)?,
        ))
    }

    /// Encodes both the audio and event portions of `payload` into the output regions.
    pub fn write_output_payload(
        &self,
        bytes: &mut [u8],
        payload: &BlockPayload,
    ) -> Result<(), &'static str> {
        self.write_audio_output(bytes, &payload.audio)?;
        self.write_event_output(bytes, &payload.events)?;
        Ok(())
    }

    /// Decodes the audio and event output regions into a [`BlockPayload`].
    pub fn read_output_payload(&self, bytes: &[u8]) -> Result<BlockPayload, &'static str> {
        Ok(BlockPayload::new(
            self.read_audio_output(bytes)?,
            self.read_event_output(bytes)?,
        ))
    }

    fn write_audio_region(
        &self,
        bytes: &mut [u8],
        region: SharedMemoryRegion,
        block: &AudioBlock,
    ) -> Result<(), &'static str> {
        if block.channel_count != self.header.channel_count
            || block.frame_count != self.header.frame_count
        {
            return Err("audio block dimensions do not match dispatch header");
        }

        let region_bytes = self.layout.region_slice_mut(bytes, region)?;
        let expected_bytes = block.samples.len() * core::mem::size_of::<f32>();
        if region_bytes.len() < expected_bytes {
            return Err("audio region is too small for encoded samples");
        }
        region_bytes.fill(0);
        for (index, sample) in block.samples.iter().enumerate() {
            let start = index * 4;
            region_bytes[start..start + 4].copy_from_slice(&sample.to_le_bytes());
        }
        Ok(())
    }

    fn read_audio_region(
        &self,
        bytes: &[u8],
        region: SharedMemoryRegion,
    ) -> Result<AudioBlock, &'static str> {
        let region_bytes = self.layout.region_slice(bytes, region)?;
        let sample_count = self.header.channel_count as usize * self.header.frame_count as usize;
        let expected_bytes = sample_count * core::mem::size_of::<f32>();
        if region_bytes.len() < expected_bytes {
            return Err("audio region is too small for encoded samples");
        }

        let mut samples = Vec::with_capacity(sample_count);
        for chunk in region_bytes[..expected_bytes].chunks_exact(4) {
            samples.push(f32::from_le_bytes(
                chunk.try_into().map_err(|_| "audio sample decode")?,
            ));
        }
        AudioBlock::new(self.header.channel_count, self.header.frame_count, samples)
    }

    fn write_event_region(
        &self,
        bytes: &mut [u8],
        region: SharedMemoryRegion,
        packet: &EventPacket,
    ) -> Result<(), &'static str> {
        let region_bytes = self.layout.region_slice_mut(bytes, region)?;
        let header_bytes = 4usize;
        let required_bytes = header_bytes + packet.events.len() * PluginEvent::ENCODED_BYTES;
        if region_bytes.len() < required_bytes {
            return Err("event region is too small for encoded events");
        }

        region_bytes.fill(0);
        region_bytes[0..4].copy_from_slice(&(packet.events.len() as u32).to_le_bytes());
        for (index, event) in packet.events.iter().enumerate() {
            let start = header_bytes + index * PluginEvent::ENCODED_BYTES;
            let end = start + PluginEvent::ENCODED_BYTES;
            write_event_to_slice(event, &mut region_bytes[start..end])?;
        }
        Ok(())
    }

    fn read_event_region(
        &self,
        bytes: &[u8],
        region: SharedMemoryRegion,
    ) -> Result<EventPacket, &'static str> {
        let region_bytes = self.layout.region_slice(bytes, region)?;
        if region_bytes.len() < 4 {
            return Err("event region is too small for encoded packet header");
        }

        let event_count = u32::from_le_bytes(
            region_bytes[0..4]
                .try_into()
                .map_err(|_| "event count decode")?,
        ) as usize;
        let required_bytes = 4 + event_count * PluginEvent::ENCODED_BYTES;
        if region_bytes.len() < required_bytes {
            return Err("event region is too small for encoded packet body");
        }

        let mut events = Vec::with_capacity(event_count);
        for index in 0..event_count {
            let start = 4 + index * PluginEvent::ENCODED_BYTES;
            let end = start + PluginEvent::ENCODED_BYTES;
            events.push(read_event_from_slice(&region_bytes[start..end])?);
        }
        Ok(EventPacket::new(events))
    }
}
