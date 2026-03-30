use super::*;
use crate::{RuntimeConfig, SignalRuntime};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind,
};

#[path = "interfaces_tests/control_surfaces.rs"]
mod control_surfaces;
#[path = "interfaces_tests/device_supervision.rs"]
mod device_supervision;
#[path = "interfaces_tests/external_io.rs"]
mod external_io;
#[path = "interfaces_tests/external_midi.rs"]
mod external_midi;
#[path = "interfaces_tests/fixtures.rs"]
mod fixtures;
#[path = "interfaces_tests/jack_coordination.rs"]
mod jack_coordination;
#[path = "interfaces_tests/linux_host_parity.rs"]
mod linux_host_parity;
#[path = "interfaces_tests/linux_sessions.rs"]
mod linux_sessions;
#[path = "interfaces_tests/multichannel_topology.rs"]
mod multichannel_topology;
#[path = "interfaces_tests/stretch_reporting.rs"]
mod stretch_reporting;

use fixtures::{host_io_summary, linux_host_io_summary, transport_session_summary};
