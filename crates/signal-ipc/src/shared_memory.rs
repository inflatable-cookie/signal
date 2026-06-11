use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use memmap2::{MmapMut, MmapOptions};

use crate::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

static PROCESS_WIDE_REGION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// A live memory-mapped shared-memory region.
///
/// Created by [`SharedMemoryBroker::create_region`] or attached via
/// [`SharedMemoryBroker::attach_region`]. The mapping stays live for the
/// lifetime of this value; drop it to release the mapping (the backing file is
/// not removed on drop — call [`SharedMemoryBroker::destroy_region`] for that).
pub struct MappedSharedMemoryRegion {
    metadata: SharedMemoryTransportPayload,
    file: File,
    map: MmapMut,
}

/// Which lifecycle operation a shared-memory error occurred during.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedMemoryRegionOperation {
    /// The region was being created.
    Create,
    /// The region was being attached.
    Attach,
    /// The region was being destroyed.
    Destroy,
}

/// Reason category for a shared-memory lifecycle failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedMemoryRegionLifecycleErrorKind {
    /// The metadata sidecar file was not found.
    MissingMetadata,
    /// The backing data file was not found.
    MissingBackingFile,
    /// The sidecar file exists but could not be parsed.
    InvalidMetadata,
    /// The sidecar file contents do not match the transport descriptor.
    MetadataMismatch,
    /// The backing file size does not match the expected byte count.
    SizeMismatch,
    /// A filesystem or OS I/O call failed.
    IoFailure,
}

/// Error returned by shared-memory lifecycle operations.
#[derive(Debug)]
pub struct SharedMemoryRegionLifecycleError {
    operation: SharedMemoryRegionOperation,
    kind: SharedMemoryRegionLifecycleErrorKind,
    region_id: String,
    backing_path: String,
    detail: String,
    source: Option<io::Error>,
}

impl SharedMemoryRegionLifecycleError {
    /// Which lifecycle operation failed.
    pub fn operation(&self) -> SharedMemoryRegionOperation {
        self.operation
    }

    /// Category of the failure.
    pub fn kind(&self) -> SharedMemoryRegionLifecycleErrorKind {
        self.kind
    }

    /// Region ID associated with the failure.
    pub fn region_id(&self) -> &str {
        &self.region_id
    }

    /// Path to the backing file involved in the failure.
    pub fn backing_path(&self) -> &str {
        &self.backing_path
    }

    /// Human-readable explanation of the failure.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl core::fmt::Display for SharedMemoryRegionLifecycleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "shared-memory {:?} failed for region {} at {}: {} ({:?})",
            self.operation, self.region_id, self.backing_path, self.detail, self.kind
        )
    }
}

impl std::error::Error for SharedMemoryRegionLifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl From<SharedMemoryRegionLifecycleError> for io::Error {
    fn from(error: SharedMemoryRegionLifecycleError) -> Self {
        io::Error::other(error)
    }
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
            &StoredRegionMetadata {
                region_id: region_id.clone(),
                lease_id: lease_id.to_string(),
                total_bytes,
                owner_pid: process::id(),
            },
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

#[derive(Debug)]
struct StoredRegionMetadata {
    region_id: String,
    lease_id: String,
    total_bytes: u32,
    owner_pid: u32,
}

fn lifecycle_error(
    operation: SharedMemoryRegionOperation,
    kind: SharedMemoryRegionLifecycleErrorKind,
    region_id: impl Into<String>,
    backing_path: impl Into<String>,
    detail: impl Into<String>,
    source: Option<io::Error>,
) -> SharedMemoryRegionLifecycleError {
    SharedMemoryRegionLifecycleError {
        operation,
        kind,
        region_id: region_id.into(),
        backing_path: backing_path.into(),
        detail: detail.into(),
        source,
    }
}

fn metadata_path_for_transport(transport: &SharedMemoryTransportPayload) -> PathBuf {
    metadata_path_for_backing_path(Path::new(&transport.backing_path))
}

fn metadata_path_for_backing_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.meta", path.display()))
}

fn read_region_metadata(
    operation: SharedMemoryRegionOperation,
    transport: &SharedMemoryTransportPayload,
    metadata_path: &Path,
) -> Result<StoredRegionMetadata, SharedMemoryRegionLifecycleError> {
    let raw = fs::read_to_string(metadata_path).map_err(|error| {
        let kind = if error.kind() == io::ErrorKind::NotFound {
            SharedMemoryRegionLifecycleErrorKind::MissingMetadata
        } else {
            SharedMemoryRegionLifecycleErrorKind::IoFailure
        };
        lifecycle_error(
            operation,
            kind,
            &transport.region_id,
            metadata_path.display().to_string(),
            "failed to read shared-memory metadata sidecar",
            Some(error),
        )
    })?;
    parse_region_metadata(operation, transport, metadata_path, &raw)
}

fn parse_region_metadata(
    operation: SharedMemoryRegionOperation,
    transport: &SharedMemoryTransportPayload,
    metadata_path: &Path,
    raw: &str,
) -> Result<StoredRegionMetadata, SharedMemoryRegionLifecycleError> {
    let mut region_id = None;
    let mut lease_id = None;
    let mut total_bytes = None;
    let mut owner_pid = None;
    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "region_id" => region_id = Some(value.trim().to_string()),
            "lease_id" => lease_id = Some(value.trim().to_string()),
            "total_bytes" => {
                total_bytes = value.trim().parse::<u32>().ok();
            }
            "owner_pid" => {
                owner_pid = value.trim().parse::<u32>().ok();
            }
            _ => {}
        }
    }
    match (region_id, lease_id, total_bytes, owner_pid) {
        (Some(region_id), Some(lease_id), Some(total_bytes), Some(owner_pid)) => {
            Ok(StoredRegionMetadata {
                region_id,
                lease_id,
                total_bytes,
                owner_pid,
            })
        }
        _ => Err(lifecycle_error(
            operation,
            SharedMemoryRegionLifecycleErrorKind::InvalidMetadata,
            &transport.region_id,
            metadata_path.display().to_string(),
            "shared-memory metadata sidecar is malformed",
            None,
        )),
    }
}

fn validate_region_metadata(
    operation: SharedMemoryRegionOperation,
    transport: &SharedMemoryTransportPayload,
    stored: &StoredRegionMetadata,
    backing_path: &Path,
) -> Result<(), SharedMemoryRegionLifecycleError> {
    if stored.region_id != transport.region_id {
        return Err(lifecycle_error(
            operation,
            SharedMemoryRegionLifecycleErrorKind::MetadataMismatch,
            &transport.region_id,
            backing_path.display().to_string(),
            format!(
                "shared-memory metadata region_id {} did not match transport {}",
                stored.region_id, transport.region_id
            ),
            None,
        ));
    }
    if stored.total_bytes != transport.total_bytes {
        return Err(lifecycle_error(
            operation,
            SharedMemoryRegionLifecycleErrorKind::SizeMismatch,
            &transport.region_id,
            backing_path.display().to_string(),
            format!(
                "shared-memory metadata expected {} bytes but transport expected {}",
                stored.total_bytes, transport.total_bytes
            ),
            None,
        ));
    }
    Ok(())
}

fn write_region_metadata(path: &Path, metadata: &StoredRegionMetadata) -> io::Result<()> {
    let payload = format!(
        "region_id={}\nlease_id={}\ntotal_bytes={}\nowner_pid={}\n",
        metadata.region_id, metadata.lease_id, metadata.total_bytes, metadata.owner_pid
    );
    fs::write(path, payload)?;
    tighten_path_permissions(path)?;
    Ok(())
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

#[cfg(unix)]
fn tighten_directory_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn tighten_directory_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn tighten_file_permissions(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn tighten_file_permissions(_file: &File) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn tighten_path_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn tighten_path_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}
