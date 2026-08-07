use std::{fs::File, io};

use memmap2::MmapMut;

use crate::SharedMemoryTransportPayload;

/// A live memory-mapped shared-memory region.
///
/// Created by [`SharedMemoryBroker::create_region`] or attached via
/// [`SharedMemoryBroker::attach_region`]. The mapping stays live for the
/// lifetime of this value; drop it to release the mapping (the backing file is
/// not removed on drop — call [`SharedMemoryBroker::destroy_region`] for that).
pub struct MappedSharedMemoryRegion {
    pub(super) metadata: SharedMemoryTransportPayload,
    pub(super) file: File,
    pub(super) map: MmapMut,
}

impl core::fmt::Debug for MappedSharedMemoryRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MappedSharedMemoryRegion")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl MappedSharedMemoryRegion {
    /// Returns the transport descriptor for this region.
    pub fn metadata(&self) -> &SharedMemoryTransportPayload {
        &self.metadata
    }

    /// Returns the mapped bytes as an immutable slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.map
    }

    /// Returns the mapped bytes as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.map
    }

    /// Flush dirty pages to the backing file.
    pub fn flush(&mut self) -> io::Result<()> {
        self.map.flush()
    }

    /// Returns the total size of this region in bytes as declared in its metadata.
    pub fn total_bytes(&self) -> u32 {
        self.metadata.total_bytes
    }

    /// Returns the current size of the backing file on disk.
    pub fn file_len(&self) -> io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }
}
