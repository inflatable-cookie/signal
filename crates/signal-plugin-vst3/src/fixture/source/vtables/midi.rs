pub(super) fn midi_fragment() -> &'static str {
    r#"/// IMidiMapping (FUnknown + getMidiControllerAssignment).
#[repr(C)]
struct MidiMappingVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_midi_controller_assignment:
        unsafe extern "C" fn(*mut c_void, i32, i16, i16, *mut u32) -> Tresult,
}

static MIDI_MAPPING_VTABLE: MidiMappingVTable = MidiMappingVTable {
    query_interface: midi_mapping_query_interface,
    add_ref: no_op_add_ref,
    release: no_op_release,
    get_midi_controller_assignment: midi_mapping_get_assignment,
};

unsafe extern "C" fn midi_mapping_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    shared_query_interface(iid, out)
}

/// CC 7 plus VST3 pitch-bend (128) and aftertouch (129) on bus 0 / channel
/// 0 map to the Gain param (id 4096); everything else is unassigned.
unsafe extern "C" fn midi_mapping_get_assignment(
    _this: *mut c_void,
    bus_index: i32,
    channel: i16,
    controller_number: i16,
    parameter_id: *mut u32,
) -> Tresult {
    if parameter_id.is_null() {
        return K_RESULT_FALSE;
    }
    if bus_index == 0 && channel == 0 && matches!(controller_number, 7 | 128 | 129) {
        *parameter_id = 4096;
        K_RESULT_OK
    } else {
        K_RESULT_FALSE
    }
}

"#
}
