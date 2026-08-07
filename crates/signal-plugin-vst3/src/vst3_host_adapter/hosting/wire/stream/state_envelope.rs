//! Component/controller state envelope encoding.

pub(crate) const STATE_ENVELOPE_MAGIC: &[u8; 8] = b"SCV3ST\0\x01";

pub(crate) fn encode_state_envelope(component: &[u8], controller: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(24 + component.len() + controller.len());
    result.extend_from_slice(STATE_ENVELOPE_MAGIC);
    result.extend_from_slice(&(component.len() as u64).to_le_bytes());
    result.extend_from_slice(&(controller.len() as u64).to_le_bytes());
    result.extend_from_slice(component);
    result.extend_from_slice(controller);
    result
}

pub(crate) fn decode_state_envelope(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    if bytes.len() < 24 || &bytes[..8] != STATE_ENVELOPE_MAGIC {
        return None;
    }
    let component_len = u64::from_le_bytes(bytes[8..16].try_into().ok()?) as usize;
    let controller_len = u64::from_le_bytes(bytes[16..24].try_into().ok()?) as usize;
    let component_end = 24usize.checked_add(component_len)?;
    let controller_end = component_end.checked_add(controller_len)?;
    if controller_end != bytes.len() {
        return None;
    }
    Some((
        &bytes[24..component_end],
        &bytes[component_end..controller_end],
    ))
}
