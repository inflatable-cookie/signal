//! Plugin inventory, activation, and editor receipt types.

use super::receipt::{SandboxBrokerReceiptLine, SandboxBrokerReceiptState};
use super::wire::decode_wire_token;

/// One plugin parameter from the child's load-time inventory (read-only
/// phase 1; descriptor fields enriched in g12.013).
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginParameter {
    /// Stable plugin-format parameter id.
    pub parameter_id: u32,
    /// Human-readable parameter name.
    pub name: String,
    /// Minimum plain value.
    pub min_value: f32,
    /// Maximum plain value.
    pub max_value: f32,
    /// Default value (normalized).
    pub default_value: f32,
    /// Display unit (e.g. "dB", "Hz"); `None` when the format reports none.
    pub unit: Option<String>,
    /// Discrete step count across the plain range (`Some(1)` = toggle);
    /// `None` for continuous parameters.
    pub step_count: Option<u32>,
    /// Whether the host may automate this parameter. Legacy receipts
    /// without a flags token parse as automatable (the pre-g12 assumption).
    pub is_automatable: bool,
    /// Whether this is the plugin's bypass parameter.
    pub is_bypass: bool,
}

/// Receipt of a successful `load-plugin`: the child's parameter inventory
/// and port summary.
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginInventory {
    /// Parameters enumerated by the child at load.
    pub parameters: Vec<SandboxPluginParameter>,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Receipt of a successful `activate`: everything the parent needs to attach
/// the shared-memory audio block region.
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginAudioLease {
    /// Region identifier assigned by the child's shared-memory broker.
    pub region_id: String,
    /// Lease identifier for the audio block region.
    pub lease_id: String,
    /// Filesystem path of the region's backing file.
    pub shm_path: String,
    /// Total region size in bytes.
    pub shm_bytes: u32,
    /// Largest block the region carries.
    pub max_frames: u32,
    /// Interleaved channel count (2 in v1).
    pub channels: u32,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Outcome of an `activate` request.
#[derive(Clone, Debug, PartialEq)]
pub enum SandboxPluginActivateOutcome {
    /// The plugin activated; the audio block region is ready to attach.
    Activated(SandboxPluginAudioLease),
    /// The plugin's main-port layout is unsupported in phase 1; the caller
    /// should compile the chain as passthrough.
    LayoutUnsupported {
        /// Human-readable detail from the receipt.
        detail: String,
    },
}

/// Parse the `params=` inventory blob:
/// `id:name:min:max:default[:unit:steps:flags];...`.
///
/// The three descriptor tokens are additive (g12.013) and version-tolerant
/// both ways: a legacy five-field entry parses with `None`/legacy defaults
/// (automatable, not bypass), and entries with unknown trailing tokens
/// parse by ignoring them.
pub(crate) fn parse_parameter_inventory(blob: &str) -> Vec<SandboxPluginParameter> {
    blob.split(';')
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| {
            let mut fields = entry.split(':');
            let parameter_id = fields.next()?.parse::<u32>().ok()?;
            let name = decode_wire_token(fields.next()?);
            let min_value = fields.next()?.parse::<f32>().ok()?;
            let max_value = fields.next()?.parse::<f32>().ok()?;
            let default_value = fields.next()?.parse::<f32>().ok()?;
            let unit = fields
                .next()
                .map(decode_wire_token)
                .filter(|unit| !unit.is_empty());
            let step_count = fields.next().and_then(|value| value.parse::<u32>().ok());
            let (is_automatable, is_bypass) = match fields.next() {
                Some(flags) => (flags.contains('a'), flags.contains('b')),
                None => (true, false),
            };
            Some(SandboxPluginParameter {
                parameter_id,
                name,
                min_value,
                max_value,
                default_value,
                unit,
                step_count,
                is_automatable,
                is_bypass,
            })
        })
        .collect()
}

/// Receipt of a successful `open-editor` (g13.027): the child-owned
/// editor window is up, sized to the plugin's initial content size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxEditorOpened {
    /// Initial editor content width (logical units).
    pub width: u32,
    /// Initial editor content height (logical units).
    pub height: u32,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Receipt of a `close-editor` (g13.027). Tolerant wire: `closed` is
/// `false` when no editor with that instance was open (the user closed it
/// first, or it never opened).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxEditorClosed {
    /// Whether an open editor was actually closed by this command.
    pub closed: bool,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// The decoded editor instance of a spontaneous user-close notification,
/// or `None` for ordinary command receipts.
pub(crate) fn user_closed_editor_instance(receipt: &SandboxBrokerReceiptLine) -> Option<String> {
    if receipt.state != SandboxBrokerReceiptState::EditorClosed {
        return None;
    }
    if receipt.extra_value("reason") != Some("user_closed") {
        return None;
    }
    Some(
        receipt
            .extra_value("editor_instance")
            .map(decode_wire_token)
            .unwrap_or_default(),
    )
}
