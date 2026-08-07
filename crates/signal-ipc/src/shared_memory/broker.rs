use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

use super::error::{
    lifecycle_error, SharedMemoryRegionLifecycleError, SharedMemoryRegionLifecycleErrorKind,
    SharedMemoryRegionOperation,
};
use super::metadata::{
    metadata_path_for_backing_path, metadata_path_for_transport, read_region_metadata,
    validate_region_metadata, write_region_metadata, StoredRegionMetadata,
};
use super::permissions::{
    map_file, sanitize_identifier, tighten_directory_permissions, tighten_file_permissions,
};
use super::region::MappedSharedMemoryRegion;

static PROCESS_WIDE_REGION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Creates, attaches, and destroys named shared-memory regions backed by
/// memory-mapped files.
///
/// All regions are stored under a configurable root directory (default:
/// `$TMPDIR/signal-shared-memory`). The broker writes a `.meta` sidecar file
/// alongside each data file containing the region ID, lease ID, byte count, and
/// owner PID so that the attach and destroy paths can validate the transport
/// descriptor before mapping or removing the files.
///
/// On Unix the broker tightens directory permissions to `0700` and file
/// permissions to `0600` at creation time.
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
    /// Construct a broker rooted at the given directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            next_region: AtomicU64::new(1),
        }
    }

    /// Returns the root directory where region files are stored.
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Create a new shared-memory region of `total_bytes` bytes.
    ///
    /// `lease_id` is used as a human-readable prefix in the backing file name.
    /// The returned [`MappedSharedMemoryRegion`] is zeroed and ready to use.
    /// Call [`destroy_region`][Self::destroy_region] with the region's transport
    /// payload when the region is no longer needed.
    pub fn create_region(
        &self,
        lease_id: &str,
        total_bytes: u32,
    ) -> Result<MappedSharedMemoryRegion, SharedMemoryRegionLifecycleError> {
        fs::create_dir_all(&self.root).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                "",
                self.root.display().to_string(),
                "failed to create broker root",
                Some(error),
            )
        })?;
        tighten_directory_permissions(&self.root).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                "",
                self.root.display().to_string(),
                "failed to tighten broker root permissions",
                Some(error),
            )
        })?;

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
            .open(&path)
            .map_err(|error| {
                lifecycle_error(
                    SharedMemoryRegionOperation::Create,
                    SharedMemoryRegionLifecycleErrorKind::IoFailure,
                    &region_id,
                    path.display().to_string(),
                    "failed to create shared-memory backing file",
                    Some(error),
                )
            })?;
        tighten_file_permissions(&file).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                &region_id,
                path.display().to_string(),
                "failed to tighten shared-memory backing file permissions",
                Some(error),
            )
        })?;
        file.set_len(total_bytes as u64).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                &region_id,
                path.display().to_string(),
                "failed to size shared-memory backing file",
                Some(error),
            )
        })?;
        let metadata_path = metadata_path_for_backing_path(&path);
        write_region_metadata(
            &metadata_path,
            &StoredRegionMetadata::new(
                region_id.clone(),
                lease_id.to_string(),
                total_bytes,
                process::id(),
            ),
        )
        .map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                &region_id,
                metadata_path.display().to_string(),
                "failed to write shared-memory metadata sidecar",
                Some(error),
            )
        })?;
        let mut map = map_file(&file, total_bytes as usize).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Create,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                &region_id,
                path.display().to_string(),
                "failed to memory-map shared-memory backing file",
                Some(error),
            )
        })?;
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

    /// Attach to an existing region identified by `transport`.
    ///
    /// Validates the sidecar metadata and file size before mapping. Returns an
    /// error if the backing file is missing, the metadata does not match, or the
    /// size differs from the transport descriptor.
    pub fn attach_region(
        &self,
        transport: &SharedMemoryTransportPayload,
    ) -> Result<MappedSharedMemoryRegion, SharedMemoryRegionLifecycleError> {
        let backing_path = PathBuf::from(&transport.backing_path);
        let metadata_path = metadata_path_for_transport(transport);
        let stored = read_region_metadata(
            SharedMemoryRegionOperation::Attach,
            transport,
            &metadata_path,
        )?;
        validate_region_metadata(
            SharedMemoryRegionOperation::Attach,
            transport,
            &stored,
            &backing_path,
        )?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&transport.backing_path)
            .map_err(|error| {
                let kind = if error.kind() == io::ErrorKind::NotFound {
                    SharedMemoryRegionLifecycleErrorKind::MissingBackingFile
                } else {
                    SharedMemoryRegionLifecycleErrorKind::IoFailure
                };
                lifecycle_error(
                    SharedMemoryRegionOperation::Attach,
                    kind,
                    &transport.region_id,
                    transport.backing_path.clone(),
                    "failed to open shared-memory backing file",
                    Some(error),
                )
            })?;
        let file_len = file
            .metadata()
            .map_err(|error| {
                lifecycle_error(
                    SharedMemoryRegionOperation::Attach,
                    SharedMemoryRegionLifecycleErrorKind::IoFailure,
                    &transport.region_id,
                    transport.backing_path.clone(),
                    "failed to stat shared-memory backing file",
                    Some(error),
                )
            })?
            .len();
        if file_len != transport.total_bytes as u64 {
            return Err(lifecycle_error(
                SharedMemoryRegionOperation::Attach,
                SharedMemoryRegionLifecycleErrorKind::SizeMismatch,
                &transport.region_id,
                transport.backing_path.clone(),
                format!(
                    "shared-memory region expected {} bytes but backing file has {}",
                    transport.total_bytes, file_len
                ),
                None,
            ));
        }

        let map = map_file(&file, transport.total_bytes as usize).map_err(|error| {
            lifecycle_error(
                SharedMemoryRegionOperation::Attach,
                SharedMemoryRegionLifecycleErrorKind::IoFailure,
                &transport.region_id,
                transport.backing_path.clone(),
                "failed to memory-map shared-memory backing file",
                Some(error),
            )
        })?;
        Ok(MappedSharedMemoryRegion {
            metadata: transport.clone(),
            file,
            map,
        })
    }

    /// Remove the backing file and metadata sidecar for the region identified by
    /// `transport`. Any live mappings obtained via [`attach_region`][Self::attach_region]
    /// become invalid after this call.
    pub fn destroy_region(
        &self,
        transport: &SharedMemoryTransportPayload,
    ) -> Result<(), SharedMemoryRegionLifecycleError> {
        let backing_path = PathBuf::from(&transport.backing_path);
        let metadata_path = metadata_path_for_transport(transport);
        let stored = read_region_metadata(
            SharedMemoryRegionOperation::Destroy,
            transport,
            &metadata_path,
        )?;
        validate_region_metadata(
            SharedMemoryRegionOperation::Destroy,
            transport,
            &stored,
            &backing_path,
        )?;

        fs::remove_file(&backing_path).map_err(|error| {
            let kind = if error.kind() == io::ErrorKind::NotFound {
                SharedMemoryRegionLifecycleErrorKind::MissingBackingFile
            } else {
                SharedMemoryRegionLifecycleErrorKind::IoFailure
            };
            lifecycle_error(
                SharedMemoryRegionOperation::Destroy,
                kind,
                &transport.region_id,
                transport.backing_path.clone(),
                "failed to remove shared-memory backing file",
                Some(error),
            )
        })?;
        fs::remove_file(&metadata_path).map_err(|error| {
            let kind = if error.kind() == io::ErrorKind::NotFound {
                SharedMemoryRegionLifecycleErrorKind::MissingMetadata
            } else {
                SharedMemoryRegionLifecycleErrorKind::IoFailure
            };
            lifecycle_error(
                SharedMemoryRegionOperation::Destroy,
                kind,
                &transport.region_id,
                metadata_path.display().to_string(),
                "failed to remove shared-memory metadata sidecar",
                Some(error),
            )
        })?;
        Ok(())
    }
}
