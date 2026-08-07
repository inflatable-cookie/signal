//! LV2 hosting error surface, C ABI, and urid:map feature.

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use std::sync::Mutex;

use super::super::introspection::URID_MAP_FEATURE;

/// token suitable for broker receipt details (mirrors `ClapHostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2HostingError {
    /// Stable snake_case failure token (e.g. `library_open_failed`).
    pub token: String,
}

impl Lv2HostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for Lv2HostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for Lv2HostingError {}

// ── LV2 C ABI ───────────────────────────────────────────────────────────────

/// `LV2_Feature`.
#[repr(C)]
pub(crate) struct Lv2Feature {
    pub(crate) uri: *const c_char,
    pub(crate) data: *mut c_void,
}

/// `LV2_Descriptor` (lv2core.h layout).
#[repr(C)]
pub(crate) struct Lv2DescriptorRaw {
    pub(crate) uri: *const c_char,
    pub(crate) instantiate: Option<
        unsafe extern "C" fn(
            *const Lv2DescriptorRaw,
            f64,
            *const c_char,
            *const *const Lv2Feature,
        ) -> *mut c_void,
    >,
    pub(crate) connect_port: Option<unsafe extern "C" fn(*mut c_void, u32, *mut c_void)>,
    pub(crate) activate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub(crate) run: Option<unsafe extern "C" fn(*mut c_void, u32)>,
    pub(crate) deactivate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub(crate) cleanup: Option<unsafe extern "C" fn(*mut c_void)>,
    pub(crate) extension_data: Option<unsafe extern "C" fn(*const c_char) -> *const c_void>,
}

/// `const LV2_Descriptor* lv2_descriptor(uint32_t index)`.
pub(crate) type Lv2DescriptorProc = unsafe extern "C" fn(u32) -> *const Lv2DescriptorRaw;

/// `LV2_URID_Map` (urid.h layout).
#[repr(C)]
pub(crate) struct Lv2UridMap {
    pub(crate) handle: *mut c_void,
    pub(crate) map: unsafe extern "C" fn(*mut c_void, *const c_char) -> u32,
}

// ── urid:map feature ────────────────────────────────────────────────────────

/// Interned string→u32 URID map state. URIDs start at 1 (0 is the LV2
/// reserved "no URID" value).
pub(crate) struct UridMapState {
    pub(crate) interned: Mutex<HashMap<Vec<u8>, u32>>,
}

unsafe extern "C" fn urid_map_callback(handle: *mut c_void, uri: *const c_char) -> u32 {
    if handle.is_null() || uri.is_null() {
        return 0;
    }
    // Safety: `handle` is the boxed `UridMapState` owned by the hosting
    // instance, alive for the plugin's whole lifetime; `uri` is a
    // NUL-terminated string per the LV2 URID contract.
    let state = unsafe { &*(handle as *const UridMapState) };
    let key = unsafe { CStr::from_ptr(uri) }.to_bytes().to_vec();
    let Ok(mut interned) = state.interned.lock() else {
        return 0;
    };
    let next = interned.len() as u32 + 1;
    *interned.entry(key).or_insert(next)
}

/// The urid:map feature bundle: boxed state, boxed `LV2_URID_Map`, boxed
/// `LV2_Feature`, and the NULL-terminated features array — all owned here
/// so every pointer the plugin may retain stays valid for the instance
/// lifetime (boxes give stable addresses even when this struct moves).
pub(crate) struct UridMapFeatureSet {
    pub(crate) _state: Box<UridMapState>,
    pub(crate) _map: Box<Lv2UridMap>,
    pub(crate) _uri: CString,
    pub(crate) _feature: Box<Lv2Feature>,
    pub(crate) features: Vec<*const Lv2Feature>,
}

impl UridMapFeatureSet {
    pub(crate) fn new() -> Self {
        let state = Box::new(UridMapState {
            interned: Mutex::new(HashMap::new()),
        });
        let map = Box::new(Lv2UridMap {
            handle: (&*state as *const UridMapState) as *mut c_void,
            map: urid_map_callback,
        });
        let uri = CString::new(URID_MAP_FEATURE).expect("static feature URI has no NUL");
        let feature = Box::new(Lv2Feature {
            uri: uri.as_ptr(),
            data: (&*map as *const Lv2UridMap) as *mut c_void,
        });
        let features = vec![&*feature as *const Lv2Feature, ptr::null()];
        Self {
            _state: state,
            _map: map,
            _uri: uri,
            _feature: feature,
            features,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const *const Lv2Feature {
        self.features.as_ptr()
    }
}
