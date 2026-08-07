use super::super::{CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_REQUESTED_SIZE};

pub(crate) fn extensions_fragment(
    instrument: bool,
    midi_outputs: u16,
    audio_output_count: u32,
) -> String {
    format!(
        r#"// ── clap.gui (offscreen bookkeeping only) ──────────────────────────────────

unsafe extern "C" fn gui_is_api_supported(
    _plugin: *const clap_plugin,
    _api: *const c_char,
    is_floating: bool,
) -> bool {{
    // Embedded on every window API (nothing is dereferenced), floating
    // unsupported: matches the phase-1 host path.
    !is_floating
}}

unsafe extern "C" fn gui_create(
    _plugin: *const clap_plugin,
    _api: *const c_char,
    is_floating: bool,
) -> bool {{
    if is_floating {{
        return false;
    }}
    GUI_WIDTH.store({gui_initial_width}, std::sync::atomic::Ordering::SeqCst);
    GUI_HEIGHT.store({gui_initial_height}, std::sync::atomic::Ordering::SeqCst);
    GUI_CREATED.store(true, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_destroy(_plugin: *const clap_plugin) {{
    GUI_CREATED.store(false, std::sync::atomic::Ordering::SeqCst);
    GUI_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
    GUI_PARENTED.store(false, std::sync::atomic::Ordering::SeqCst);
}}

unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {{
    true
}}

unsafe extern "C" fn gui_get_size(
    _plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) || width.is_null() || height.is_null()
    {{
        return false;
    }}
    *width = GUI_WIDTH.load(std::sync::atomic::Ordering::SeqCst);
    *height = GUI_HEIGHT.load(std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {{
    true
}}

unsafe extern "C" fn gui_adjust_size(
    _plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {{
    !width.is_null() && !height.is_null()
}}

unsafe extern "C" fn gui_set_size(_plugin: *const clap_plugin, width: u32, height: u32) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) {{
        return false;
    }}
    GUI_WIDTH.store(width, std::sync::atomic::Ordering::SeqCst);
    GUI_HEIGHT.store(height, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_set_parent(
    _plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool {{
    // The parent handle is recorded, never dereferenced (offscreen test
    // plugin): any non-null handle parents successfully.
    if window.is_null() || (*window).specific.is_null() {{
        return false;
    }}
    GUI_PARENTED.store(true, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn gui_show(_plugin: *const clap_plugin) -> bool {{
    if !GUI_CREATED.load(std::sync::atomic::Ordering::SeqCst) {{
        return false;
    }}
    GUI_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
    // Stand-in editor tweak: the next processed block pushes a Gain
    // PARAM_VALUE out-event for the plugin→host sync proof (g12.024).
    PENDING_PARAM_OUT.store(true, std::sync::atomic::Ordering::SeqCst);
    let host = HOST.load(std::sync::atomic::Ordering::SeqCst);
    if !host.is_null() {{
        if let Some(get_extension) = (*host).get_extension {{
            let extension = get_extension(host, STATE_ID.as_ptr().cast());
            if !extension.is_null() {{
                if let Some(mark_dirty) = (*(extension as *const clap_host_state)).mark_dirty {{
                    mark_dirty(host);
                }}
            }}
        }}
    }}
    // Exercise the host-callback path: ask the host for a resize.
    let host = HOST.load(std::sync::atomic::Ordering::SeqCst);
    if !host.is_null() {{
        if let Some(get_extension) = (*host).get_extension {{
            let extension = get_extension(host, GUI_ID.as_ptr() as *const c_char);
            if !extension.is_null() {{
                let host_gui = extension as *const clap_host_gui;
                if let Some(request_resize) = (*host_gui).request_resize {{
                    let _ = request_resize(host, {gui_request_width}, {gui_request_height});
                }}
            }}
            // Exercise the host clap.params wiring too (g12.024): an
            // editor tweak conventionally asks the host for a flush.
            let params_extension = get_extension(host, PARAMS_ID.as_ptr() as *const c_char);
            if !params_extension.is_null() {{
                let host_params = params_extension as *const clap_host_params;
                if let Some(request_flush) = (*host_params).request_flush {{
                    request_flush(host);
                }}
            }}
        }}
    }}
    true
}}

unsafe extern "C" fn gui_hide(_plugin: *const clap_plugin) -> bool {{
    GUI_VISIBLE.store(false, std::sync::atomic::Ordering::SeqCst);
    true
}}

unsafe extern "C" fn audio_port_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {{
    if is_input {{ {audio_input_count} }} else {{ {audio_output_count} }}
}}
unsafe extern "C" fn latency_get(_plugin: *const clap_plugin) -> u32 {{ 0 }}
unsafe extern "C" fn audio_port_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {{
    if (is_input && index != 0) || (!is_input && index >= {audio_output_count}) {{ return false; }}
    let name: &[u8] = if is_input {{ b"Main Input\0".as_slice() }} else {{ b"Main Output\0".as_slice() }};
    let channel_count = 2;
    let mut port = clap_audio_port_info {{
        id: if is_input {{ 1 }} else {{ 2 + index }},
        name: [0; 256],
        flags: if index == 0 {{ CLAP_AUDIO_PORT_IS_MAIN }} else {{ 0 }},
        channel_count,
        port_type: ptr::null(),
        in_place_pair: u32::MAX,
    }};
    for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = port;
    true
}}

unsafe extern "C" fn note_port_count(_plugin: *const clap_plugin, is_input: bool) -> u32 {{
    if is_input {{ 1 }} else {{ {midi_outputs} }}
}}

unsafe extern "C" fn note_port_get(
    _plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool {{
    if index != 0 {{ return false; }}
    let name: &[u8] = if is_input {{ b"MIDI In\0".as_slice() }} else {{ b"MIDI Out\0".as_slice() }};
    let mut port = clap_note_port_info {{
        id: if is_input {{ 11 }} else {{ 12 }},
        supported_dialects: CLAP_NOTE_DIALECT_MIDI,
        preferred_dialect: CLAP_NOTE_DIALECT_MIDI,
        name: [0; 256],
    }};
    for (slot, value) in port.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = port;
    true
}}

unsafe extern "C" fn param_count(_plugin: *const clap_plugin) -> u32 {{ 2 }}
unsafe extern "C" fn param_get_info(
    _plugin: *const clap_plugin,
    index: u32,
    info: *mut clap_param_info,
) -> bool {{
    let (id, name, flags, default_value) = match index {{
        0 => (4096u32, b"Gain\0".as_slice(), CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE, FIXTURE_GAIN as f64),
        1 => (0u32, b"Bypass\0".as_slice(), CLAP_PARAM_IS_BYPASS | CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_STEPPED, 0.0f64),
        _ => return false,
    }};
    let mut param = clap_param_info {{
        id,
        flags,
        cookie: ptr::null_mut(),
        name: [0; 256],
        module: [0; 1024],
        min_value: 0.0,
        max_value: 1.0,
        default_value,
    }};
    for (slot, value) in param.name.iter_mut().zip(name.iter().copied()) {{
        *slot = value as c_char;
    }}
    *info = param;
    true
}}

unsafe extern "C" fn param_get_value(
    _plugin: *const clap_plugin,
    param_id: u32,
    out_value: *mut f64,
) -> bool {{
    if out_value.is_null() {{ return false; }}
    *out_value = if param_id == 4096 {{
        f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst)) as f64
    }} else {{
        0.0
    }};
    true
}}"#,
        gui_initial_width = CLAP_FIXTURE_GUI_INITIAL_SIZE.0,
        gui_initial_height = CLAP_FIXTURE_GUI_INITIAL_SIZE.1,
        gui_request_width = CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
        gui_request_height = CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
        audio_input_count = if instrument { 0 } else { 1 },
        audio_output_count = audio_output_count,
        midi_outputs = midi_outputs,
    )
}
