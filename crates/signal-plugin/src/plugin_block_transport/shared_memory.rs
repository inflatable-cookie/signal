use signal_ipc::SharedMemoryTransportPayload;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryRegion {
    pub offset_bytes: u32,
    pub size_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryLayout {
    pub audio_input: SharedMemoryRegion,
    pub audio_output: SharedMemoryRegion,
    pub event_input: SharedMemoryRegion,
    pub event_output: SharedMemoryRegion,
    pub render_context: SharedMemoryRegion,
    pub completion: SharedMemoryRegion,
}

impl SharedMemoryLayout {
    pub fn single_block(total_audio_bytes: u32, total_event_bytes: u32) -> Self {
        let audio_input = SharedMemoryRegion {
            offset_bytes: 0,
            size_bytes: total_audio_bytes,
        };
        let audio_output = SharedMemoryRegion {
            offset_bytes: audio_input.offset_bytes + audio_input.size_bytes,
            size_bytes: total_audio_bytes,
        };
        let event_input = SharedMemoryRegion {
            offset_bytes: audio_output.offset_bytes + audio_output.size_bytes,
            size_bytes: total_event_bytes,
        };
        let event_output = SharedMemoryRegion {
            offset_bytes: event_input.offset_bytes + event_input.size_bytes,
            size_bytes: total_event_bytes,
        };
        let render_context = SharedMemoryRegion {
            offset_bytes: event_output.offset_bytes + event_output.size_bytes,
            size_bytes: 256,
        };
        let completion = SharedMemoryRegion {
            offset_bytes: render_context.offset_bytes + render_context.size_bytes,
            size_bytes: 64,
        };

        Self {
            audio_input,
            audio_output,
            event_input,
            event_output,
            render_context,
            completion,
        }
    }

    pub fn total_bytes(self) -> u32 {
        self.completion.offset_bytes + self.completion.size_bytes
    }

    pub fn region_slice<'a>(
        &self,
        bytes: &'a [u8],
        region: SharedMemoryRegion,
    ) -> Result<&'a [u8], &'static str> {
        let start = region.offset_bytes as usize;
        let end = start + region.size_bytes as usize;
        bytes
            .get(start..end)
            .ok_or("shared-memory region exceeds mapped transport size")
    }

    pub fn region_slice_mut<'a>(
        &self,
        bytes: &'a mut [u8],
        region: SharedMemoryRegion,
    ) -> Result<&'a mut [u8], &'static str> {
        let start = region.offset_bytes as usize;
        let end = start + region.size_bytes as usize;
        bytes
            .get_mut(start..end)
            .ok_or("shared-memory region exceeds mapped transport size")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedMemoryLease {
    pub lease_id: String,
    pub processing_epoch: u64,
    pub layout: SharedMemoryLayout,
    transport: Option<SharedMemoryTransportPayload>,
    bytes: Vec<u8>,
    invalidated_epochs: Vec<u64>,
}

impl SharedMemoryLease {
    pub fn new(
        lease_id: impl Into<String>,
        processing_epoch: u64,
        layout: SharedMemoryLayout,
    ) -> Self {
        Self {
            lease_id: lease_id.into(),
            processing_epoch,
            layout,
            transport: None,
            bytes: vec![0; layout.total_bytes() as usize],
            invalidated_epochs: Vec::new(),
        }
    }

    pub fn with_transport(mut self, transport: SharedMemoryTransportPayload) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn total_bytes(&self) -> u32 {
        self.layout.total_bytes()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn invalidate_epoch(&mut self, epoch: u64) -> bool {
        if !self.invalidated_epochs.contains(&epoch) {
            self.invalidated_epochs.push(epoch);
            return true;
        }
        false
    }

    pub fn is_epoch_valid(&self, epoch: u64) -> bool {
        self.processing_epoch == epoch && !self.invalidated_epochs.contains(&epoch)
    }

    pub fn invalidated_epochs(&self) -> &[u64] {
        &self.invalidated_epochs
    }

    pub fn bind_transport(&mut self, transport: SharedMemoryTransportPayload) {
        self.transport = Some(transport);
    }

    pub fn transport(&self) -> Option<&SharedMemoryTransportPayload> {
        self.transport.as_ref()
    }
}
