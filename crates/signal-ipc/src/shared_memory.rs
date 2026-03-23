use std::{
    fs::{self, File, OpenOptions},
    io,
    path::PathBuf,
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use memmap2::{MmapMut, MmapOptions};

use crate::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

static PROCESS_WIDE_REGION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct MappedSharedMemoryRegion {
    metadata: SharedMemoryTransportPayload,
    file: File,
    map: MmapMut,
}

impl core::fmt::Debug for MappedSharedMemoryRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MappedSharedMemoryRegion")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl MappedSharedMemoryRegion {
    pub fn metadata(&self) -> &SharedMemoryTransportPayload {
        &self.metadata
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.map
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.map
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.map.flush()
    }

    pub fn total_bytes(&self) -> u32 {
        self.metadata.total_bytes
    }

    pub fn file_len(&self) -> io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }
}

#[derive(Debug)]
pub struct SharedMemoryBroker {
    root: PathBuf,
    next_region: AtomicU64,
}

impl Default for SharedMemoryBroker {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("signal-shared-memory"))
    }
}

impl SharedMemoryBroker {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            next_region: AtomicU64::new(1),
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn create_region(
        &self,
        lease_id: &str,
        total_bytes: u32,
    ) -> io::Result<MappedSharedMemoryRegion> {
        fs::create_dir_all(&self.root)?;

        let sequence = self.next_region.fetch_add(1, Ordering::Relaxed);
        let process_wide_sequence = PROCESS_WIDE_REGION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let region_id = format!(
            "region-{}-{timestamp}-{sequence}-{process_wide_sequence}",
            process::id()
        );
        let filename = format!("{}-{region_id}.signal-shm", sanitize_identifier(lease_id));
        let path = self.root.join(filename);
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)?;
        file.set_len(total_bytes as u64)?;
        let mut map = map_file(&file, total_bytes as usize)?;
        map.fill(0);

        Ok(MappedSharedMemoryRegion {
            metadata: SharedMemoryTransportPayload {
                region_id,
                transport_kind: SharedMemoryTransportKind::MappedFile,
                backing_path: path.display().to_string(),
                total_bytes,
            },
            file,
            map,
        })
    }

    pub fn attach_region(
        &self,
        transport: &SharedMemoryTransportPayload,
    ) -> io::Result<MappedSharedMemoryRegion> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&transport.backing_path)?;
        let file_len = file.metadata()?.len();
        if file_len < transport.total_bytes as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "shared-memory region {} expected {} bytes but file only has {}",
                    transport.region_id, transport.total_bytes, file_len
                ),
            ));
        }

        let map = map_file(&file, transport.total_bytes as usize)?;
        Ok(MappedSharedMemoryRegion {
            metadata: transport.clone(),
            file,
            map,
        })
    }

    pub fn destroy_region(&self, transport: &SharedMemoryTransportPayload) -> io::Result<()> {
        match fs::remove_file(&transport.backing_path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

fn map_file(file: &File, len: usize) -> io::Result<MmapMut> {
    // SAFETY: the file is explicitly sized by the broker before mapping, and
    // callers pass the exact byte length they expect to use.
    unsafe { MmapOptions::new().len(len).map_mut(file) }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect()
}
