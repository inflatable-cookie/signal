use super::super::hosting::{tuid_from_uid, Tuid};

/// `IPlugView` IID (published interface definition, iplugview.h).
pub(crate) const IPLUG_VIEW_IID: Tuid =
    tuid_from_uid(0x5BC32507, 0xD06049EA, 0xA6151B52, 0x2B755B29);
/// `IPlugFrame` IID (published interface definition, iplugview.h).
pub(crate) const IPLUG_FRAME_IID: Tuid =
    tuid_from_uid(0x367FAF01, 0xAFA94693, 0x8D4DA2A0, 0xED0882A3);

/// The platform window type this build hands to `attached` /
/// `isPlatformTypeSupported` (`kPlatformTypeNSView` on macOS).
#[cfg(target_os = "macos")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"NSView";
#[cfg(target_os = "linux")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"X11EmbedWindowID";
#[cfg(target_os = "windows")]
pub(crate) const PLATFORM_TYPE: &std::ffi::CStr = c"HWND";

/// The view name passed to `IEditController::createView` (`ViewType::kEditor`).
pub(crate) const VIEW_TYPE_EDITOR: &std::ffi::CStr = c"editor";
