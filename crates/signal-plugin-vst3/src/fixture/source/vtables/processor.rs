pub(super) fn processor_vtable_fragment() -> &'static str {
    r#"static PROCESSOR_VTABLE: AudioProcessorVTable = AudioProcessorVTable {
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
};

"#
}

pub(super) fn processor_impl_fragment() -> &'static str {
    r#"unsafe extern "C" fn processor_set_bus_arrangements(
    _this: *mut c_void,
    inputs: *mut u64,
    num_inputs: i32,
    outputs: *mut u64,
    num_outputs: i32,
) -> Tresult {
    if num_inputs == 1
        && num_outputs == 1
        && !inputs.is_null()
        && !outputs.is_null()
        && *inputs == STEREO
        && *outputs == STEREO
    {
        K_RESULT_OK
    } else {
        K_RESULT_FALSE
    }
}

unsafe extern "C" fn processor_get_bus_arrangement(
    _this: *mut c_void,
    _direction: i32,
    index: i32,
    arrangement: *mut u64,
) -> Tresult {
    if index != 0 || arrangement.is_null() {
        return K_RESULT_FALSE;
    }
    *arrangement = STEREO;
    K_RESULT_OK
}

unsafe extern "C" fn processor_can_process_sample_size(
    _this: *mut c_void,
    symbolic_sample_size: i32,
) -> Tresult {
    if symbolic_sample_size == 0 { K_RESULT_OK } else { K_RESULT_FALSE }
}

unsafe extern "C" fn processor_get_latency_samples(_this: *mut c_void) -> u32 { 0 }

unsafe extern "C" fn processor_setup_processing(
    _this: *mut c_void,
    setup: *mut ProcessSetup,
) -> Tresult {
    if setup.is_null() { K_RESULT_FALSE } else { K_RESULT_OK }
}

unsafe extern "C" fn processor_set_processing(_this: *mut c_void, _state: u8) -> Tresult {
    K_RESULT_OK
}

// ── Input IParameterChanges consumption (g12.023) ──────────────────────────

#[repr(C)]
struct ParamValueQueueVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_id: unsafe extern "C" fn(*mut c_void) -> u32,
    get_point_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_point: unsafe extern "C" fn(*mut c_void, i32, *mut i32, *mut f64) -> Tresult,
    add_point: unsafe extern "C" fn(*mut c_void, i32, f64, *mut i32) -> Tresult,
}

#[repr(C)]
struct ParameterChangesVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_parameter_data: unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void,
    add_parameter_data: unsafe extern "C" fn(*mut c_void, *const u32, *mut i32) -> *mut c_void,
}

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
) {
    if changes.is_null() {
        return;
    }
    let changes_vtable = *(changes as *mut *const ParameterChangesVTable);
    let count = ((*changes_vtable).get_parameter_count)(changes);
    for index in 0..count {
        let queue = ((*changes_vtable).get_parameter_data)(changes, index);
        if queue.is_null() {
            continue;
        }
        let queue_vtable = *(queue as *mut *const ParamValueQueueVTable);
        if ((*queue_vtable).get_parameter_id)(queue) != 4096 {
            continue;
        }
        let points = ((*queue_vtable).get_point_count)(queue);
        for point in 0..points {
            let mut sample_offset = 0i32;
            let mut value = 0f64;
            if ((*queue_vtable).get_point)(queue, point, &mut sample_offset, &mut value)
                == K_RESULT_OK
                && *step_count < GAIN_STEP_CAPACITY
            {
                steps[*step_count] = (sample_offset, value as f32);
                *step_count += 1;
            }
        }
    }
}

// ── Input IEventList consumption (note delivery proof) ─────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct NoteOnEventPayload {
    channel: i16,
    pitch: i16,
    tuning: f32,
    velocity: f32,
    length: i32,
    note_id: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
union EventPayload {
    note_on: NoteOnEventPayload,
    _size: [u64; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vst3Event {
    bus_index: i32,
    sample_offset: i32,
    ppq_position: f64,
    flags: u16,
    type_: u16,
    payload: EventPayload,
}

#[repr(C)]
struct EventListVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_event_count: unsafe extern "C" fn(*mut c_void) -> i32,
    get_event: unsafe extern "C" fn(*mut c_void, i32, *mut Vst3Event) -> Tresult,
    add_event: unsafe extern "C" fn(*mut c_void, *mut Vst3Event) -> Tresult,
}

/// Gather note events as gain steps: NOTE_ON (type 0) → gain = velocity at
/// its sample offset, NOTE_OFF (type 1) → gain = 0.0 at its sample offset —
/// making delivered notes AND their offsets audible in the output.
unsafe fn gather_note_steps(
    events: *mut c_void,
    steps: &mut [(i32, f32); GAIN_STEP_CAPACITY],
    step_count: &mut usize,
) {
    if events.is_null() {
        return;
    }
    let list_vtable = *(events as *mut *const EventListVTable);
    let count = ((*list_vtable).get_event_count)(events);
    for index in 0..count {
        let mut event = std::mem::MaybeUninit::<Vst3Event>::zeroed();
        if ((*list_vtable).get_event)(events, index, event.as_mut_ptr()) != K_RESULT_OK {
            continue;
        }
        let event = event.assume_init();
        if *step_count == GAIN_STEP_CAPACITY {
            break;
        }
        match event.type_ {
            0 => {
                steps[*step_count] = (event.sample_offset, event.payload.note_on.velocity);
                *step_count += 1;
            }
            1 => {
                steps[*step_count] = (event.sample_offset, 0.0);
                *step_count += 1;
            }
            _ => {}
        }
    }
}

/// Real audio processing: output = input × the LIVE Gain on every channel
/// of the main bus pair. The gain starts at the stored value and follows
/// the block's gathered `(offset, gain)` steps from their sample offsets
/// (param points, IMidiMapping CC points, and note events all land here);
/// the final step persists into later blocks.
unsafe extern "C" fn processor_process(_this: *mut c_void, data: *mut ProcessData) -> Tresult {
    if data.is_null() {
        return K_RESULT_FALSE;
    }
    let data = &*data;
    // Real instruments such as Softube's assume the standard per-block
    // process context is present. Keep the fixture strict enough to catch a
    // host regression back to a null context.
    if data.process_context.is_null()
        || data.input_parameter_changes.is_null()
        || data.output_parameter_changes.is_null()
        || data.input_events.is_null()
        || data.output_events.is_null()
    {
        return K_RESULT_FALSE;
    }
    let mut gain_steps = [(0i32, 0f32); GAIN_STEP_CAPACITY];
    let mut step_count = 0usize;
    gather_parameter_steps(data.input_parameter_changes, &mut gain_steps, &mut step_count);
    gather_note_steps(data.input_events, &mut gain_steps, &mut step_count);
    gain_steps[..step_count].sort_by_key(|step| step.0);
    if data.num_inputs < 1
        || data.num_outputs < 1
        || data.inputs.is_null()
        || data.outputs.is_null()
    {
        return K_RESULT_FALSE;
    }
    let input = &*data.inputs;
    let output = &*data.outputs;
    if input.channel_buffers32.is_null() || output.channel_buffers32.is_null() {
        return K_RESULT_FALSE;
    }
    let frames = data.num_samples.max(0) as usize;
    let channels = input.num_channels.min(output.num_channels).max(0) as usize;
    for channel in 0..channels {
        let source = *input.channel_buffers32.add(channel);
        let dest = *output.channel_buffers32.add(channel);
        if source.is_null() || dest.is_null() {
            return K_RESULT_FALSE;
        }
        let mut gain = f32::from_bits(GAIN_BITS.load(std::sync::atomic::Ordering::SeqCst));
        let mut next_step = 0usize;
        for frame in 0..frames {
            while next_step < step_count && gain_steps[next_step].0 as usize <= frame {
                gain = gain_steps[next_step].1;
                next_step += 1;
            }
            *dest.add(frame) = *source.add(frame) * gain;
        }
    }
    if step_count > 0 {
        GAIN_BITS.store(
            gain_steps[step_count - 1].1.to_bits(),
            std::sync::atomic::Ordering::SeqCst,
        );
    }
    K_RESULT_OK
}

unsafe extern "C" fn processor_get_tail_samples(_this: *mut c_void) -> u32 { 0 }

"#
}
