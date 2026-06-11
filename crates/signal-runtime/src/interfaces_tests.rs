use super::*;
use signal_hardware::{AudioSampleFormat, BackendHealth, HardwareBackendIdentity};

#[path = "interfaces_tests/device_supervision.rs"]
mod device_supervision;
#[path = "interfaces_tests/external_io.rs"]
mod external_io;
#[path = "interfaces_tests/fixtures.rs"]
mod fixtures;
#[path = "interfaces_tests/multichannel_topology.rs"]
mod multichannel_topology;
#[path = "interfaces_tests/stretch_reporting.rs"]
mod stretch_reporting;

use fixtures::host_io_summary;
