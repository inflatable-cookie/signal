//! Bus, processing, and parameter constants.

use super::super::com::Tresult;

pub(crate) const K_AUDIO: i32 = 0;
pub(crate) const K_INPUT: i32 = 0;
pub(crate) const K_OUTPUT: i32 = 1;
pub(crate) const K_MAIN: i32 = 0;
pub(crate) const K_REALTIME: i32 = 0;
pub(crate) const K_SAMPLE32: i32 = 0;
pub(crate) const K_PROJECT_TIME_MUSIC_VALID: u32 = 1 << 9;
pub(crate) const K_TEMPO_VALID: u32 = 1 << 10;
pub(crate) const K_BAR_POSITION_VALID: u32 = 1 << 11;
pub(crate) const K_TIME_SIG_VALID: u32 = 1 << 13;
pub(crate) const K_CONT_TIME_VALID: u32 = 1 << 17;
/// `kSpeakerL | kSpeakerR`.
pub(crate) const STEREO_ARRANGEMENT: u64 = 0x3;

// ParameterInfo flags.
pub(crate) const PARAM_CAN_AUTOMATE: i32 = 1;
pub(crate) const PARAM_IS_READ_ONLY: i32 = 1 << 1;
pub(crate) const PARAM_IS_HIDDEN: i32 = 1 << 4;
pub(crate) const PARAM_IS_BYPASS: i32 = 1 << 16;
/// `RestartFlags::kLatencyChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_LATENCY_CHANGED: u32 = 1 << 3;
/// `RestartFlags::kIoChanged` from `ivsteditcontroller.h`.
pub const VST3_RESTART_IO_CHANGED: u32 = 1 << 1;
pub(crate) const RESTART_PROCESSING_MASK: u32 =
    VST3_RESTART_IO_CHANGED | VST3_RESTART_LATENCY_CHANGED;

/// `kNotImplemented` (platform-dependent: COM `E_NOTIMPL` on Windows).
#[cfg(target_os = "windows")]
pub(crate) const K_NOT_IMPLEMENTED: Tresult = 0x8000_4001_u32 as i32;
#[cfg(not(target_os = "windows"))]
pub(crate) const K_NOT_IMPLEMENTED: Tresult = 3;
