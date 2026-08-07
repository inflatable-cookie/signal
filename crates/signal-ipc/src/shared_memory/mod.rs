mod broker;
mod error;
mod metadata;
mod permissions;
mod region;

pub use broker::SharedMemoryBroker;
pub use error::{
    SharedMemoryRegionLifecycleError, SharedMemoryRegionLifecycleErrorKind,
    SharedMemoryRegionOperation,
};
pub use region::MappedSharedMemoryRegion;
