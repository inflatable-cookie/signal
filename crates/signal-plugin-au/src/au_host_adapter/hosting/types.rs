//! AU hosting error surface, registry sentinel, and FourCC load keys.

use crate::au_host_adapter::AuHostPlatform;

/// Sentinel `.component` path for registry-resolved AU entries: the file is
/// never opened — the load key alone rebuilds the `AudioComponentDescription`
/// and the system registry resolves the component. The `.component`
/// extension is what routes the sandbox broker to the AU hosting branch.
pub const AU_REGISTRY_COMPONENT_PATH: &str = "au-registry.component";

/// Error surface for AU hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `Vst3HostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuHostingError {
    /// Stable snake_case failure token (e.g. `component_not_found`).
    pub token: String,
}

impl AuHostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for AuHostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for AuHostingError {}

/// The build-target AU platform (macOS is the only one that can host).
pub const fn current_au_platform() -> AuHostPlatform {
    AuHostPlatform::MacOs
}

// ── FourCC codes ────────────────────────────────────────────────────────────

/// Encode a 4-character OSType code (`"aufx"`) as its big-endian `u32`.
/// `None` when the string is not exactly four ASCII bytes.
pub(crate) fn fourcc_from_str(code: &str) -> Option<u32> {
    let bytes = code.as_bytes();
    if bytes.len() != 4 || !bytes.iter().all(u8::is_ascii) {
        return None;
    }
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Decode an OSType `u32` back into its 4-character code. Non-printable
/// bytes render as `?` (real registry OSTypes are printable ASCII).
pub(crate) fn fourcc_to_string(code: u32) -> String {
    code.to_be_bytes()
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '?'
            }
        })
        .collect()
}

/// Parse a catalog load key — the colon-separated fourcc triple
/// `{type}:{subtype}:{manufacturer}` (e.g. `aufx:dely:appl`) — into the
/// three OSType codes.
pub(crate) fn parse_load_key(load_key: &str) -> Option<(u32, u32, u32)> {
    let mut parts = load_key.split(':');
    let component_type = fourcc_from_str(parts.next()?)?;
    let component_subtype = fourcc_from_str(parts.next()?)?;
    let manufacturer = fourcc_from_str(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((component_type, component_subtype, manufacturer))
}
