//! VST3 COM primitives.

// ── COM primitives ──────────────────────────────────────────────────────────

/// Steinberg `tresult`.
pub(crate) type Tresult = i32;

/// `kResultOk` / `kResultTrue` (0 on every platform).
pub(crate) const K_RESULT_OK: Tresult = 0;
pub(crate) const K_RESULT_FALSE: Tresult = 1;

/// `kNoInterface` (platform-dependent: COM `E_NOINTERFACE` on Windows).
#[cfg(target_os = "windows")]
pub(crate) const K_NO_INTERFACE: Tresult = 0x8000_4002_u32 as i32;
#[cfg(not(target_os = "windows"))]
pub(crate) const K_NO_INTERFACE: Tresult = -1;

/// 16-byte Steinberg TUID.
pub(crate) type Tuid = [u8; 16];

/// Build a TUID from the four canonical `u32` fields with the platform's
/// `INLINE_UID` byte layout (see module docs).
pub(crate) const fn tuid_from_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> Tuid {
    if cfg!(target_os = "windows") {
        [
            (l1 & 0xFF) as u8,
            ((l1 >> 8) & 0xFF) as u8,
            ((l1 >> 16) & 0xFF) as u8,
            ((l1 >> 24) & 0xFF) as u8,
            ((l2 >> 16) & 0xFF) as u8,
            ((l2 >> 24) & 0xFF) as u8,
            (l2 & 0xFF) as u8,
            ((l2 >> 8) & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8,
            ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8,
            (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8,
            ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8,
            (l4 & 0xFF) as u8,
        ]
    } else {
        [
            ((l1 >> 24) & 0xFF) as u8,
            ((l1 >> 16) & 0xFF) as u8,
            ((l1 >> 8) & 0xFF) as u8,
            (l1 & 0xFF) as u8,
            ((l2 >> 24) & 0xFF) as u8,
            ((l2 >> 16) & 0xFF) as u8,
            ((l2 >> 8) & 0xFF) as u8,
            (l2 & 0xFF) as u8,
            ((l3 >> 24) & 0xFF) as u8,
            ((l3 >> 16) & 0xFF) as u8,
            ((l3 >> 8) & 0xFF) as u8,
            (l3 & 0xFF) as u8,
            ((l4 >> 24) & 0xFF) as u8,
            ((l4 >> 16) & 0xFF) as u8,
            ((l4 >> 8) & 0xFF) as u8,
            (l4 & 0xFF) as u8,
        ]
    }
}

/// Decode a catalog load key (raw in-memory TUID hex on non-Windows, the
/// canonical class-ID hex everywhere) into the in-memory TUID.
pub(crate) fn tuid_from_class_id_hex(class_id_hex: &str) -> Option<Tuid> {
    let hex = class_id_hex.trim();
    if hex.len() != 32 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (index, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    if cfg!(target_os = "windows") {
        // Canonical hex → COM in-memory layout: swap the first 4-byte field
        // and the two following 2-byte fields.
        let mut swapped = bytes;
        swapped[0..4].reverse();
        swapped[4..6].reverse();
        swapped[6..8].reverse();
        Some(swapped)
    } else {
        Some(bytes)
    }
}

// Interface IIDs (canonical field values from the published VST3 interface
// definitions; encoded per-platform by `tuid_from_uid`).
pub(crate) const FUNKNOWN_IID: Tuid = tuid_from_uid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
pub(crate) const ICOMPONENT_IID: Tuid =
    tuid_from_uid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
pub(crate) const IAUDIO_PROCESSOR_IID: Tuid =
    tuid_from_uid(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);
pub(crate) const IEDIT_CONTROLLER_IID: Tuid =
    tuid_from_uid(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E);
pub(crate) const IPLUGIN_FACTORY_3_IID: Tuid =
    tuid_from_uid(0x4555A2AB, 0xC1234E57, 0x9B122910, 0x36878931);
pub(crate) const ICOMPONENT_HANDLER_IID: Tuid =
    tuid_from_uid(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6);
pub(crate) const IHOST_APPLICATION_IID: Tuid =
    tuid_from_uid(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5);
pub(crate) const IMESSAGE_IID: Tuid = tuid_from_uid(0x936F033B, 0xC6C047DB, 0xBB0882F8, 0x13C1E613);
pub(crate) const IATTRIBUTE_LIST_IID: Tuid =
    tuid_from_uid(0x1E5F0AEB, 0xCC7F4533, 0xA2544011, 0x38AD5EE4);
// ivstparameterchanges.h (published interface definitions).
pub(crate) const IPARAMETER_CHANGES_IID: Tuid =
    tuid_from_uid(0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D);
pub(crate) const IPARAM_VALUE_QUEUE_IID: Tuid =
    tuid_from_uid(0x01263A18, 0xED074F6F, 0x98C9D356, 0x4686F9BA);
// ivstevents.h / ivstmidicontrollers.h (published interface definitions).
pub(crate) const IEVENT_LIST_IID: Tuid =
    tuid_from_uid(0x3A2C4214, 0x346349FE, 0xB2C4F397, 0xB9695A44);
pub(crate) const IMIDI_MAPPING_IID: Tuid =
    tuid_from_uid(0xDF695DF2, 0x8B4B47EB, 0xAB3EF8FB, 0x2D1F6BB2);
pub(crate) const ICONNECTION_POINT_IID: Tuid =
    tuid_from_uid(0x70A4156F, 0x6E6E4026, 0x989148BF, 0xAA60D8D1);
pub(crate) const IBSTREAM_IID: Tuid = tuid_from_uid(0xC3BF6EA2, 0x30994752, 0x9B6BF990, 0x1EE33E9B);
