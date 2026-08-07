use std::io;

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

pub(super) fn lifecycle_error(
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
