use super::super::CLAP_FIXTURE_GUI_PARAM_OUT_VALUE;

pub(crate) fn statics_fragment(
    plugin_type_id: &str,
    plugin_name: &str,
    instrument: bool,
    audio_output_count: u32,
) -> String {
    format!(
        r#"static FACTORY_ID: &[u8] = b"clap.plugin-factory\0";
static AUDIO_PORTS_ID: &[u8] = b"clap.audio-ports\0";
static NOTE_PORTS_ID: &[u8] = b"clap.note-ports\0";
static PARAMS_ID: &[u8] = b"clap.params\0";
static GUI_ID: &[u8] = b"clap.gui\0";
static STATE_ID: &[u8] = b"clap.state\0";
static LATENCY_ID: &[u8] = b"clap.latency\0";
static TAIL_ID: &[u8] = b"clap.tail\0";
static FEATURE_AUDIO_EFFECT: &[u8] = b"audio-effect\0";
static FEATURE_INSTRUMENT: &[u8] = b"instrument\0";
static FEATURE_UTILITY: &[u8] = b"utility\0";
static FEATURES: FeaturePtrs = FeaturePtrs([
    {primary_feature_symbol}.as_ptr() as *const c_char,
    FEATURE_UTILITY.as_ptr() as *const c_char,
    ptr::null(),
]);
static PLUGIN_ID: &[u8] = concat!("{plugin_type_id}", "\0").as_bytes();
static PLUGIN_NAME: &[u8] = concat!("{plugin_name}", "\0").as_bytes();
static VENDOR: &[u8] = b"Signal\0";
static URL: &[u8] = b"https://signal.dev\0";
static VERSION: &[u8] = b"0.1.0\0";
static DESCRIPTION: &[u8] = b"Signal CLAP Fixture\0";

static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {{
    clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }},
    id: PLUGIN_ID.as_ptr() as *const c_char,
    name: PLUGIN_NAME.as_ptr() as *const c_char,
    vendor: VENDOR.as_ptr() as *const c_char,
    url: URL.as_ptr() as *const c_char,
    manual_url: URL.as_ptr() as *const c_char,
    support_url: URL.as_ptr() as *const c_char,
    version: VERSION.as_ptr() as *const c_char,
    description: DESCRIPTION.as_ptr() as *const c_char,
    features: FEATURES.0.as_ptr(),
}};

static AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {{
    count: Some(audio_port_count),
    get: Some(audio_port_get),
}};
static LATENCY: clap_plugin_latency = clap_plugin_latency {{
    get: Some(latency_get),
}};

static NOTE_PORTS: clap_plugin_note_ports = clap_plugin_note_ports {{
    count: Some(note_port_count),
    get: Some(note_port_get),
}};

static PARAMS: clap_plugin_params = clap_plugin_params {{
    count: Some(param_count),
    get_info: Some(param_get_info),
    get_value: Some(param_get_value),
    value_to_text: None,
    text_to_value: None,
    flush: Some(param_flush),
}};

static STATE: clap_plugin_state = clap_plugin_state {{
    save: Some(state_save),
    load: Some(state_load),
}};

static PLUGIN: clap_plugin = clap_plugin {{
    desc: &DESCRIPTOR,
    plugin_data: ptr::null_mut(),
    init: Some(plugin_init),
    destroy: Some(plugin_destroy),
    activate: Some(plugin_activate),
    deactivate: Some(plugin_deactivate),
    start_processing: Some(plugin_start_processing),
    stop_processing: Some(plugin_stop_processing),
    reset: Some(plugin_reset),
    process: Some(plugin_process),
    get_extension: Some(plugin_get_extension),
    on_main_thread: None,
}};

static FACTORY: clap_plugin_factory = clap_plugin_factory {{
    get_plugin_count: Some(factory_get_plugin_count),
    get_plugin_descriptor: Some(factory_get_plugin_descriptor),
    create_plugin: Some(factory_create_plugin),
}};

#[unsafe(no_mangle)]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {{
    clap_version: clap_version {{ major: 1, minor: 0, revision: 0 }},
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
}};

unsafe extern "C" fn entry_init(_plugin_path: *const c_char) -> bool {{ true }}
unsafe extern "C" fn entry_deinit() {{}}

unsafe extern "C" fn entry_get_factory(factory_id: *const c_char) -> *const c_void {{
    let requested = CStr::from_ptr(factory_id).to_bytes_with_nul();
    if requested == FACTORY_ID {{
        (&FACTORY as *const clap_plugin_factory).cast()
    }} else {{
        ptr::null()
    }}
}}

unsafe extern "C" fn factory_get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {{ 1 }}
unsafe extern "C" fn factory_get_plugin_descriptor(
    _factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {{
    if index == 0 {{ &DESCRIPTOR }} else {{ ptr::null() }}
}}

unsafe extern "C" fn factory_create_plugin(
    _factory: *const clap_plugin_factory,
    host: *const c_void,
    _plugin_id: *const c_char,
) -> *const clap_plugin {{
    HOST.store(host as *mut clap_host, std::sync::atomic::Ordering::SeqCst);
    &PLUGIN
}}

unsafe extern "C" fn plugin_init(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_destroy(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_activate(_plugin: *const clap_plugin, _sample_rate: f64, _min: u32, _max: u32) -> bool {{ true }}
unsafe extern "C" fn plugin_deactivate(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_start_processing(_plugin: *const clap_plugin) -> bool {{ true }}
unsafe extern "C" fn plugin_stop_processing(_plugin: *const clap_plugin) {{}}
unsafe extern "C" fn plugin_reset(_plugin: *const clap_plugin) {{}}

/// Cap on per-block gain steps gathered from note/MIDI in-events (the
/// note/CC delivery proof; more than enough for the tests).
const GAIN_STEP_CAPACITY: usize = 64;

/// Apply pending in-events before the block renders. PARAM_VALUE events
/// for the Gain param (id 4096) keep their block-boundary semantics
/// (g12.023: stored immediately). Note and MIDI CC7 events become
/// SAMPLE-OFFSET voice-level steps for instruments (gain steps for effects)
/// so hosts can assert both the decoded bytes and
/// the intra-block offsets from the audio output alone:
///   NOTE_ON  → gain = velocity from the event's time offset
///   NOTE_OFF → gain = 0.0 from the event's time offset
///   MIDI 0xB0 cc=7 → gain = data2 / 127 from the event's time offset
/// Returns `(time, value, voice_level)` steps in delivery order.
unsafe fn apply_param_events(
    in_events: *const c_void,
    steps: &mut [(u32, f32, bool); GAIN_STEP_CAPACITY],
) -> usize {{
    if in_events.is_null() {{
        return 0;
    }}
    let list = &*(in_events as *const clap_input_events);
    let (Some(size), Some(get)) = (list.size, list.get) else {{
        return 0;
    }};
    let mut step_count = 0usize;
    let count = size(list as *const clap_input_events);
    for index in 0..count {{
        let header = get(list as *const clap_input_events, index);
        if header.is_null() {{
            continue;
        }}
        if (*header).space_id != CLAP_CORE_EVENT_SPACE_ID {{
            continue;
        }}
        match (*header).type_ {{
            CLAP_EVENT_PARAM_VALUE_TYPE => {{
                let event = &*(header as *const clap_event_param_value);
                if event.param_id == 4096 {{
                    GAIN_BITS.store(
                        (event.value as f32).to_bits(),
                        std::sync::atomic::Ordering::SeqCst,
                    );
                }}
            }}
            CLAP_EVENT_NOTE_ON_TYPE | CLAP_EVENT_NOTE_OFF_TYPE => {{
                let event = &*(header as *const clap_event_note);
                if step_count < GAIN_STEP_CAPACITY {{
                    let gain = if (*header).type_ == CLAP_EVENT_NOTE_ON_TYPE {{
                        event.velocity as f32
                    }} else {{
                        0.0
                    }};
                    steps[step_count] = ((*header).time, gain, {instrument});
                    step_count += 1;
                }}
            }}
            CLAP_EVENT_NOTE_EXPRESSION_TYPE => {{
                let event = &*(header as *const clap_event_note_expression);
                if step_count < GAIN_STEP_CAPACITY {{
                    steps[step_count] = ((*header).time, event.value as f32, {instrument});
                    step_count += 1;
                }}
            }}
            CLAP_EVENT_MIDI_TYPE => {{
                let event = &*(header as *const clap_event_midi);
                if event.data[0] & 0xF0 == 0xB0
                    && event.data[1] == 7
                    && step_count < GAIN_STEP_CAPACITY
                {{
                    steps[step_count] = ((*header).time, f32::from(event.data[2]) / 127.0, {instrument});
                    step_count += 1;
                }}
            }}
            _ => {{}}
        }}
    }}
    step_count
}}

/// CLAP control-thread parameter delivery used while audio processing is
/// stopped. State capture relies on this path to observe the latest queued
/// Gain value without requiring a synthetic audio block.
unsafe extern "C" fn param_flush(
    _plugin: *const clap_plugin,
    in_events: *const c_void,
    _out_events: *const c_void,
) {{
    let mut ignored_steps = [(0u32, 0f32, false); GAIN_STEP_CAPACITY];
    let _ = apply_param_events(in_events, &mut ignored_steps);
}}

/// Real audio processing: output = input × the LIVE Gain param on every
/// channel of the main port pair (in-events applied first, block-boundary).
/// Returns CLAP_PROCESS_CONTINUE (1) on success.
unsafe extern "C" fn plugin_process(
    _plugin: *const clap_plugin,
    process: *const clap_process,
) -> i32 {{
    if process.is_null() {{
        return 0;
    }}
    let process = &*process;
    // Exercise the host's compatibility contract: even though CLAP permits
    // this pointer to be null, real plugins such as Spire assume a transport
    // snapshot is always present.
    if process.transport.is_null() {{
        return 0;
    }}
    let mut gain_steps = [(0u32, 0f32, false); GAIN_STEP_CAPACITY];
    let step_count = apply_param_events(process.in_events, &mut gain_steps);
    if PENDING_PARAM_OUT.swap(false, std::sync::atomic::Ordering::SeqCst)
        && !process.out_events.is_null()
    {{
        let out_events = &*(process.out_events as *const clap_output_events);
        if let Some(try_push) = out_events.try_push {{
            let event = clap_event_param_value {{
                header: clap_event_header {{
                    size: std::mem::size_of::<clap_event_param_value>() as u32,
                    time: 0,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_VALUE_TYPE,
                    flags: 0,
                }},
                param_id: 4096,
                cookie: ptr::null_mut(),
                note_id: -1,
                port_index: -1,
                channel: -1,
                key: -1,
                value: {gui_param_out_value}f64,
            }};
            GAIN_BITS.store(
                ({gui_param_out_value}f32).to_bits(),
                std::sync::atomic::Ordering::SeqCst,
            );
            let _ = try_push(
                out_events as *const clap_output_events,
                &event.header as *const clap_event_header,
            );
        }}
    }}
    if process.audio_outputs_count < {audio_output_count} || process.audio_outputs.is_null() {{
        return 0;
    }}
    let output = &*process.audio_outputs;
    if output.data32.is_null() {{
        return 0;
    }}
    let input = if {instrument} {{
        None
    }} else {{
        if process.audio_inputs_count < 1 || process.audio_inputs.is_null() {{ return 0; }}
        let input = &*process.audio_inputs;
        if input.data32.is_null() {{ return 0; }}
        Some(input)
    }};
    let frames = process.frames_count as usize;
    let channels = input
        .map(|input| input.channel_count.min(output.channel_count))
        .unwrap_or(output.channel_count) as usize;
    for channel in 0..channels {{
        let source = input.map(|input| *input.data32.add(channel));
        let dest = *output.data32.add(channel);
        if source.is_some_and(|source| source.is_null()) || dest.is_null() {{
            return 0;
        }}
        // Gain and instrument voice level are independent: parameter writes
        // scale held notes instead of being overwritten by note events.
        let mut gain = f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst));
        let mut note_level = f32::from_bits(
            NOTE_LEVEL_BITS.load(std::sync::atomic::Ordering::SeqCst),
        );
        let mut next_step = 0usize;
        for frame in 0..frames {{
            while next_step < step_count && gain_steps[next_step].0 as usize <= frame {{
                if gain_steps[next_step].2 {{
                    note_level = gain_steps[next_step].1;
                }} else {{
                    gain = gain_steps[next_step].1;
                }}
                next_step += 1;
            }}
            *dest.add(frame) = match source {{
                Some(source) => *source.add(frame) * gain,
                None => note_level * gain,
            }};
        }}
    }}
    for step in &gain_steps[..step_count] {{
        if step.2 {{
            NOTE_LEVEL_BITS.store(step.1.to_bits(), std::sync::atomic::Ordering::SeqCst);
        }} else {{
            GAIN_BITS.store(step.1.to_bits(), std::sync::atomic::Ordering::SeqCst);
        }}
    }}
    1
}}

unsafe extern "C" fn plugin_get_extension(
    _plugin: *const clap_plugin,
    extension_id: *const c_char,
) -> *const c_void {{
    let requested = CStr::from_ptr(extension_id).to_bytes_with_nul();
    if requested == AUDIO_PORTS_ID {{
        (&AUDIO_PORTS as *const clap_plugin_audio_ports).cast()
    }} else if requested == NOTE_PORTS_ID {{
        (&NOTE_PORTS as *const clap_plugin_note_ports).cast()
    }} else if requested == PARAMS_ID {{
        (&PARAMS as *const clap_plugin_params).cast()
    }} else if requested == GUI_ID {{
        (&GUI as *const clap_plugin_gui).cast()
    }} else if requested == LATENCY_ID {{
        (&LATENCY as *const clap_plugin_latency).cast()
    }} else if requested == STATE_ID {{
        (&STATE as *const clap_plugin_state).cast()
    }} else if requested == TAIL_ID {{
        1usize as *const c_void
    }} else {{
        ptr::null()
    }}
}}"#,
        plugin_type_id = plugin_type_id,
        plugin_name = plugin_name,
        instrument = instrument,
        audio_output_count = audio_output_count,
        primary_feature_symbol = if instrument {
            "FEATURE_INSTRUMENT"
        } else {
            "FEATURE_AUDIO_EFFECT"
        },
        gui_param_out_value = CLAP_FIXTURE_GUI_PARAM_OUT_VALUE,
    )
}
