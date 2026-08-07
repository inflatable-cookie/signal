pub(crate) fn vtables_fragment(default_bus_channels: u16) -> String {
    format!(
        r#"// ── The single-component plugin object (three facets, static) ──────────────

/// One static COM object: facet 0 = IComponent (also FUnknown/IPluginBase),
/// facet 1 = IAudioProcessor, facet 2 = IEditController. queryInterface
/// hands out facet addresses; refcounting is a no-op (static lifetime).
#[repr(C)]
struct FixtureObject {{
    component_vtable: *const ComponentVTable,
    processor_vtable: *const AudioProcessorVTable,
    controller_vtable: *const EditControllerVTable,
    midi_mapping_vtable: *const MidiMappingVTable,
}}

unsafe impl Sync for FixtureObject {{}}

/// IMidiMapping (FUnknown + getMidiControllerAssignment).
#[repr(C)]
struct MidiMappingVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_midi_controller_assignment:
        unsafe extern "C" fn(*mut c_void, i32, i16, i16, *mut u32) -> Tresult,
}}

static MIDI_MAPPING_VTABLE: MidiMappingVTable = MidiMappingVTable {{
    query_interface: midi_mapping_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    get_midi_controller_assignment: midi_mapping_get_assignment,
}};

unsafe extern "C" fn midi_mapping_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

/// CC 7 plus VST3 pitch-bend (128) and aftertouch (129) on bus 0 / channel
/// 0 map to the Gain param (id 4096); everything else is unassigned.
unsafe extern "C" fn midi_mapping_get_assignment(
    _this: *mut c_void,
    bus_index: i32,
    channel: i16,
    controller_number: i16,
    parameter_id: *mut u32,
) -> Tresult {{
    if parameter_id.is_null() {{
        return K_RESULT_FALSE;
    }}
    if bus_index == 0 && channel == 0 && matches!(controller_number, 7 | 128 | 129) {{
        *parameter_id = 4096;
        K_RESULT_OK
    }} else {{
        K_RESULT_FALSE
    }}
}}

static COMPONENT_VTABLE: ComponentVTable = ComponentVTable {{
    query_interface: component_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    get_controller_class_id: component_get_controller_class_id,
    set_io_mode: component_set_io_mode,
    get_bus_count: component_get_bus_count,
    get_bus_info: component_get_bus_info,
    get_routing_info: component_get_routing_info,
    activate_bus: component_activate_bus,
    set_active: component_set_active,
    set_state: state_noop,
    get_state: state_noop,
}};

static PROCESSOR_VTABLE: AudioProcessorVTable = AudioProcessorVTable {{
    query_interface: processor_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    set_bus_arrangements: processor_set_bus_arrangements,
    get_bus_arrangement: processor_get_bus_arrangement,
    can_process_sample_size: processor_can_process_sample_size,
    get_latency_samples: processor_get_latency_samples,
    setup_processing: processor_setup_processing,
    set_processing: processor_set_processing,
    process: processor_process,
    get_tail_samples: processor_get_tail_samples,
}};

static CONTROLLER_VTABLE: EditControllerVTable = EditControllerVTable {{
    query_interface: controller_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    initialize: base_initialize,
    terminate: base_terminate,
    set_component_state: state_noop,
    set_state: state_noop,
    get_state: state_noop,
    get_parameter_count: controller_get_parameter_count,
    get_parameter_info: controller_get_parameter_info,
    get_param_string_by_value: controller_get_param_string_by_value,
    get_param_value_by_string: controller_get_param_value_by_string,
    normalized_param_to_plain: controller_normalized_param_to_plain,
    plain_param_to_normalized: controller_plain_param_to_normalized,
    get_param_normalized: controller_get_param_normalized,
    set_param_normalized: controller_set_param_normalized,
    set_component_handler: controller_set_component_handler,
    create_view: controller_create_view,
}};

static FIXTURE_OBJECT: FixtureObject = FixtureObject {{
    component_vtable: &COMPONENT_VTABLE,
    processor_vtable: &PROCESSOR_VTABLE,
    controller_vtable: &CONTROLLER_VTABLE,
    midi_mapping_vtable: &MIDI_MAPPING_VTABLE,
}};

fn object_base() -> *mut c_void {{
    &FIXTURE_OBJECT as *const FixtureObject as *mut c_void
}}

fn processor_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.processor_vtable as *mut c_void }}
}}

fn controller_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.controller_vtable as *mut c_void }}
}}

fn midi_mapping_facet() -> *mut c_void {{
    unsafe {{ &raw const FIXTURE_OBJECT.midi_mapping_vtable as *mut c_void }}
}}

unsafe fn facet_for(iid: *const Tuid) -> Option<*mut c_void> {{
    if iid.is_null() {{
        return None;
    }}
    let iid = *iid;
    if iid == FUNKNOWN_IID || iid == IPLUGIN_BASE_IID || iid == ICOMPONENT_IID {{
        Some(object_base())
    }} else if iid == IAUDIO_PROCESSOR_IID {{
        Some(processor_facet())
    }} else if iid == IEDIT_CONTROLLER_IID {{
        Some(controller_facet())
    }} else if iid == IMIDI_MAPPING_IID {{
        Some(midi_mapping_facet())
    }} else {{
        None
    }}
}}

unsafe extern "C" fn component_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe extern "C" fn processor_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe extern "C" fn controller_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {{
    shared_query_interface(iid, out)
}}

unsafe fn shared_query_interface(iid: *const Tuid, out: *mut *mut c_void) -> Tresult {{
    if out.is_null() {{
        return K_NO_INTERFACE;
    }}
    match facet_for(iid) {{
        Some(facet) => {{
            *out = facet;
            K_RESULT_OK
        }}
        None => {{
            *out = ptr::null_mut();
            K_NO_INTERFACE
        }}
    }}
}}

unsafe extern "C" fn no_op_add_ref(_this: *mut c_void) -> u32 {{ 1 }}
unsafe extern "C" fn no_op_release(_this: *mut c_void) -> u32 {{ 1 }}
unsafe extern "C" fn base_initialize(_this: *mut c_void, _context: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}
unsafe extern "C" fn base_terminate(_this: *mut c_void) -> Tresult {{ K_RESULT_OK }}
unsafe extern "C" fn state_noop(_this: *mut c_void, _stream: *mut c_void) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_controller_class_id(
    _this: *mut c_void,
    _class_id: *mut Tuid,
) -> Tresult {{
    // Single-component plugin: the controller is a facet of this object.
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_set_io_mode(_this: *mut c_void, _mode: i32) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_bus_count(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
) -> i32 {{
    if media_type == K_AUDIO {{ 1 }} else {{ 0 }}
}}

unsafe extern "C" fn component_get_bus_info(
    _this: *mut c_void,
    media_type: i32,
    direction: i32,
    index: i32,
    info: *mut BusInfo,
) -> Tresult {{
    if media_type != K_AUDIO || index != 0 || info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let info = &mut *info;
    info.media_type = K_AUDIO;
    info.direction = direction;
    info.channel_count = {default_bus_channels};
    info.bus_type = 0; // kMain
    info.flags = 1; // kDefaultActive
    let mut name = [0i16; 128];
    write_utf16(
        &mut name,
        if direction == K_INPUT {{ "Main Input" }} else {{ "Main Output" }},
    );
    info.name = name;
    K_RESULT_OK
}}

unsafe extern "C" fn component_get_routing_info(
    _this: *mut c_void,
    _input: *mut c_void,
    _output: *mut c_void,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn component_activate_bus(
    _this: *mut c_void,
    media_type: i32,
    _direction: i32,
    index: i32,
    _state: u8,
) -> Tresult {{
    if media_type == K_AUDIO && index == 0 {{ K_RESULT_OK }} else {{ K_RESULT_FALSE }}
}}

unsafe extern "C" fn component_set_active(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn processor_set_bus_arrangements(
    _this: *mut c_void,
    inputs: *mut u64,
    num_inputs: i32,
    outputs: *mut u64,
    num_outputs: i32,
) -> Tresult {{
    if num_inputs == 1
        && num_outputs == 1
        && !inputs.is_null()
        && !outputs.is_null()
        && *inputs == STEREO
        && *outputs == STEREO
    {{
        K_RESULT_OK
    }} else {{
        K_RESULT_FALSE
    }}
}}

unsafe extern "C" fn processor_get_bus_arrangement(
    _this: *mut c_void,
    _direction: i32,
    index: i32,
    arrangement: *mut u64,
) -> Tresult {{
    if index != 0 || arrangement.is_null() {{
        return K_RESULT_FALSE;
    }}
    *arrangement = STEREO;
    K_RESULT_OK
}}

unsafe extern "C" fn processor_can_process_sample_size(
    _this: *mut c_void,
    symbolic_sample_size: i32,
) -> Tresult {{
    if symbolic_sample_size == 0 {{ K_RESULT_OK }} else {{ K_RESULT_FALSE }}
}}

unsafe extern "C" fn processor_get_latency_samples(_this: *mut c_void) -> u32 {{ 0 }}

unsafe extern "C" fn processor_setup_processing(
    _this: *mut c_void,
    setup: *mut ProcessSetup,
) -> Tresult {{
    if setup.is_null() {{ K_RESULT_FALSE }} else {{ K_RESULT_OK }}
}}

unsafe extern "C" fn processor_set_processing(_this: *mut c_void, _state: u8) -> Tresult {{
    K_RESULT_OK
}}

// ── Input IParameterChanges consumption (g12.023) ──────────────────────────

#[repr(C)]
struct ParamValueQueueVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_id: unsafe extern "C" fn(*mut c_void) -> u32,
    get_point_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_point: unsafe extern "C" fn(*mut c_void, i32, *mut i32, *mut f64) -> Tresult,
    add_point: unsafe extern "C" fn(*mut c_void, i32, f64, *mut i32) -> Tresult,
}}

#[repr(C)]
struct ParameterChangesVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_parameter_data: unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void,
    add_parameter_data: unsafe extern "C" fn(*mut c_void, *const u32, *mut i32) -> *mut c_void,
}}

/// Per-block cap on gain steps gathered from param points and note events.
const GAIN_STEP_CAPACITY: usize = 64;

/// Gather every Gain (id 4096) point in the block's input parameter
/// changes as `(sample_offset, gain)` steps — the real host contract:
/// sample-offset points apply FROM their offset (wire writes arrive at
/// offset 0, IMidiMapping-routed CC at the CC event's offset).
unsafe fn gather_parameter_steps(
    changes: *mut c_void,
    steps: &mut [(i32, f32); GAIN_STEP_CAPACITY],
    step_count: &mut usize,
) {{
    if changes.is_null() {{
        return;
    }}
    let changes_vtable = *(changes as *mut *const ParameterChangesVTable);
    let count = ((*changes_vtable).get_parameter_count)(changes);
    for index in 0..count {{
        let queue = ((*changes_vtable).get_parameter_data)(changes, index);
        if queue.is_null() {{
            continue;
        }}
        let queue_vtable = *(queue as *mut *const ParamValueQueueVTable);
        if ((*queue_vtable).get_parameter_id)(queue) != 4096 {{
            continue;
        }}
        let points = ((*queue_vtable).get_point_count)(queue);
        for point in 0..points {{
            let mut sample_offset = 0i32;
            let mut value = 0f64;
            if ((*queue_vtable).get_point)(queue, point, &mut sample_offset, &mut value)
                == K_RESULT_OK
                && *step_count < GAIN_STEP_CAPACITY
            {{
                steps[*step_count] = (sample_offset, value as f32);
                *step_count += 1;
            }}
        }}
    }}
}}

// ── Input IEventList consumption (note delivery proof) ─────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct NoteOnEventPayload {{
    channel: i16,
    pitch: i16,
    tuning: f32,
    velocity: f32,
    length: i32,
    note_id: i32,
}}

#[repr(C)]
#[derive(Clone, Copy)]
union EventPayload {{
    note_on: NoteOnEventPayload,
    _size: [u64; 3],
}}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vst3Event {{
    bus_index: i32,
    sample_offset: i32,
    ppq_position: f64,
    flags: u16,
    type_: u16,
    payload: EventPayload,
}}

#[repr(C)]
struct EventListVTable {{
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_event_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_event: unsafe extern "C" fn(*mut c_void, i32, *mut Vst3Event) -> Tresult,
    add_event: unsafe extern "C" fn(*mut c_void, *mut Vst3Event) -> Tresult,
}}

/// Gather note events as gain steps: NOTE_ON (type 0) → gain = velocity at
/// its sample offset, NOTE_OFF (type 1) → gain = 0.0 at its sample offset —
/// making delivered notes AND their offsets audible in the output.
unsafe fn gather_note_steps(
    events: *mut c_void,
    steps: &mut [(i32, f32); GAIN_STEP_CAPACITY],
    step_count: &mut usize,
) {{
    if events.is_null() {{
        return;
    }}
    let list_vtable = *(events as *mut *const EventListVTable);
    let count = ((*list_vtable).get_event_count)(events);
    for index in 0..count {{
        let mut event = std::mem::MaybeUninit::<Vst3Event>::zeroed();
        if ((*list_vtable).get_event)(events, index, event.as_mut_ptr()) != K_RESULT_OK {{
            continue;
        }}
        let event = event.assume_init();
        if *step_count == GAIN_STEP_CAPACITY {{
            break;
        }}
        match event.type_ {{
            0 => {{
                steps[*step_count] = (event.sample_offset, event.payload.note_on.velocity);
                *step_count += 1;
            }}
            1 => {{
                steps[*step_count] = (event.sample_offset, 0.0);
                *step_count += 1;
            }}
            _ => {{}}
        }}
    }}
}}

/// Real audio processing: output = input × the LIVE Gain on every channel
/// of the main bus pair. The gain starts at the stored value and follows
/// the block's gathered `(offset, gain)` steps from their sample offsets
/// (param points, IMidiMapping CC points, and note events all land here);
/// the final step persists into later blocks.
unsafe extern "C" fn processor_process(_this: *mut c_void, data: *mut ProcessData) -> Tresult {{
    if data.is_null() {{
        return K_RESULT_FALSE;
    }}
    let data = &*data;
    // Real instruments such as Softube's assume the standard per-block
    // process context is present. Keep the fixture strict enough to catch a
    // host regression back to a null context.
    if data.process_context.is_null()
        || data.input_parameter_changes.is_null()
        || data.output_parameter_changes.is_null()
        || data.input_events.is_null()
        || data.output_events.is_null()
    {{
        return K_RESULT_FALSE;
    }}
    let mut gain_steps = [(0i32, 0f32); GAIN_STEP_CAPACITY];
    let mut step_count = 0usize;
    gather_parameter_steps(data.input_parameter_changes, &mut gain_steps, &mut step_count);
    gather_note_steps(data.input_events, &mut gain_steps, &mut step_count);
    gain_steps[..step_count].sort_by_key(|step| step.0);
    if data.num_inputs < 1
        || data.num_outputs < 1
        || data.inputs.is_null()
        || data.outputs.is_null()
    {{
        return K_RESULT_FALSE;
    }}
    let input = &*data.inputs;
    let output = &*data.outputs;
    if input.channel_buffers32.is_null() || output.channel_buffers32.is_null() {{
        return K_RESULT_FALSE;
    }}
    let frames = data.num_samples.max(0) as usize;
    let channels = input.num_channels.min(output.num_channels).max(0) as usize;
    for channel in 0..channels {{
        let source = *input.channel_buffers32.add(channel);
        let dest = *output.channel_buffers32.add(channel);
        if source.is_null() || dest.is_null() {{
            return K_RESULT_FALSE;
        }}
        let mut gain = f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst));
        let mut next_step = 0usize;
        for frame in 0..frames {{
            while next_step < step_count && gain_steps[next_step].0 as usize <= frame {{
                gain = gain_steps[next_step].1;
                next_step += 1;
            }}
            *dest.add(frame) = *source.add(frame) * gain;
        }}
    }}
    if step_count > 0 {{
        GAIN_BITS.store(
            gain_steps[step_count - 1].1.to_bits(),
            std::sync::atomic::Ordering::SeqCst,
        );
    }}
    K_RESULT_OK
}}

unsafe extern "C" fn processor_get_tail_samples(_this: *mut c_void) -> u32 {{ 0 }}

unsafe extern "C" fn controller_get_parameter_count(_this: *mut c_void) -> i32 {{ 2 }}

unsafe extern "C" fn controller_get_parameter_info(
    _this: *mut c_void,
    index: i32,
    info: *mut ParameterInfo,
) -> Tresult {{
    if info.is_null() {{
        return K_RESULT_FALSE;
    }}
    let (id, title, unit, flags, step_count, default_value) = match index {{
        0 => (4096u32, "Gain", "dB", PARAM_CAN_AUTOMATE, 0, 0.5f64),
        1 => (0u32, "Bypass", "", PARAM_CAN_AUTOMATE | PARAM_IS_BYPASS, 1, 0.0f64),
        _ => return K_RESULT_FALSE,
    }};
    let info = &mut *info;
    info.id = id;
    let mut buffer = [0i16; 128];
    write_utf16(&mut buffer, title);
    info.title = buffer;
    info.short_title = buffer;
    let mut unit_buffer = [0i16; 128];
    write_utf16(&mut unit_buffer, unit);
    info.units = unit_buffer;
    info.step_count = step_count;
    info.default_normalized_value = default_value;
    info.unit_id = 0;
    info.flags = flags;
    K_RESULT_OK
}}

unsafe extern "C" fn controller_get_param_string_by_value(
    _this: *mut c_void,
    _id: u32,
    value: f64,
    string: *mut i16,
) -> Tresult {{
    write_utf16_ptr(string, &format!("{{value:.2}}"));
    K_RESULT_OK
}}

unsafe extern "C" fn controller_get_param_value_by_string(
    _this: *mut c_void,
    _id: u32,
    _string: *mut i16,
    _value: *mut f64,
) -> Tresult {{
    K_RESULT_FALSE
}}

unsafe extern "C" fn controller_normalized_param_to_plain(
    _this: *mut c_void,
    _id: u32,
    normalized: f64,
) -> f64 {{
    normalized
}}

unsafe extern "C" fn controller_plain_param_to_normalized(
    _this: *mut c_void,
    _id: u32,
    plain: f64,
) -> f64 {{
    plain
}}

unsafe extern "C" fn controller_get_param_normalized(_this: *mut c_void, id: u32) -> f64 {{
    if id == 4096 {{ 0.5 }} else {{ 0.0 }}
}}

unsafe extern "C" fn controller_set_param_normalized(
    _this: *mut c_void,
    _id: u32,
    _value: f64,
) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn controller_set_component_handler(
    _this: *mut c_void,
    _handler: *mut c_void,
) -> Tresult {{
    K_RESULT_OK
}}

unsafe extern "C" fn controller_create_view(
    _this: *mut c_void,
    name: *const c_char,
) -> *mut c_void {{
    // Editor views only, per spec; the returned object is static so the
    // host's create/release probe and open/close cycles are all no-ops.
    if name.is_null() {{
        return ptr::null_mut();
    }}
    let mut len = 0usize;
    while *name.add(len) != 0 {{
        len += 1;
    }}
    let requested = std::slice::from_raw_parts(name as *const u8, len);
    if requested == b"editor" {{
        view_object()
    }} else {{
        ptr::null_mut()
    }}
}}"#,
        default_bus_channels = default_bus_channels,
    )
}
