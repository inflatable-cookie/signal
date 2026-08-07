use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::SharedMemoryTransportPayload;

use super::error::{
    lifecycle_error, SharedMemoryRegionLifecycleError, SharedMemoryRegionLifecycleErrorKind,
    SharedMemoryRegionOperation,
};
use super::permissions::tighten_path_permissions;

#[derive(Debug)]
pub(super) struct StoredRegionMetadata {
    region_id: String,
    lease_id: String,
    total_bytes: u32,
    owner_pid: u32,
}

impl StoredRegionMetadata {
    pub(super) fn new(
        region_id: String,
        lease_id: String,
        total_bytes: u32,
        owner_pid: u32,
    ) -> Self {
        Self {
            region_id,
            lease_id,
            total_bytes,
            owner_pid,
        }
    }
}

pub(super) fn metadata_path_for_transport(transport: &SharedMemoryTransportPayload) -> PathBuf {
    metadata_path_for_backing_path(Path::new(&transport.backing_path))
}

pub(super) fn metadata_path_for_backing_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.meta", path.display()))
}

pub(super) fn read_region_metadata(
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

pub(super) fn validate_region_metadata(
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

pub(super) fn write_region_metadata(
    path: &Path,
    metadata: &StoredRegionMetadata,
) -> io::Result<()> {
    let payload = format!(
        "region_id={}\nlease_id={}\ntotal_bytes={}\nowner_pid={}\n",
        metadata.region_id, metadata.lease_id, metadata.total_bytes, metadata.owner_pid
    );
    fs::write(path, payload)?;
    tighten_path_permissions(path)?;
    Ok(())
}
