pub(crate) fn state_fragment() -> String {
    r#"// ── clap.state ─────────────────────────────────────────────────────────────

unsafe extern "C" fn state_save(
    _plugin: *const clap_plugin,
    stream: *const clap_ostream,
) -> bool {
    if stream.is_null() {
        return false;
    }
    let Some(write) = (*stream).write else { return false };
    let mut state = [0u8; 8];
    state[..4].copy_from_slice(&GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst).to_le_bytes());
    state[4..].copy_from_slice(&NOTE_LEVEL_BITS.load(std::sync::atomic::Ordering::SeqCst).to_le_bytes());
    let mut offset = 0usize;
    while offset < state.len() {
        let written = write(
            stream,
            state.as_ptr().add(offset).cast(),
            (state.len() - offset) as u64,
        );
        if written <= 0 || written as usize > state.len() - offset {
            return false;
        }
        offset += written as usize;
    }
    true
}

unsafe extern "C" fn state_load(
    _plugin: *const clap_plugin,
    stream: *const clap_istream,
) -> bool {
    if stream.is_null() {
        return false;
    }
    let Some(read) = (*stream).read else { return false };
    let mut state = [0u8; 8];
    let mut offset = 0usize;
    while offset < state.len() {
        let count = read(
            stream,
            state.as_mut_ptr().add(offset).cast(),
            (state.len() - offset) as u64,
        );
        if count <= 0 || count as usize > state.len() - offset {
            return false;
        }
        offset += count as usize;
    }
    GAIN_BITS.store(
        u32::from_le_bytes(state[..4].try_into().unwrap()),
        std::sync::atomic::Ordering::SeqCst,
    );
    NOTE_LEVEL_BITS.store(
        u32::from_le_bytes(state[4..].try_into().unwrap()),
        std::sync::atomic::Ordering::SeqCst,
    );
    true
}"#.to_string()
}
