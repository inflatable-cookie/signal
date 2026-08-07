pub(super) fn object_header_fragment() -> &'static str {
    r#"// ── The single-component plugin object (three facets, static) ──────────────

/// One static COM object: facet 0 = IComponent (also FUnknown/IPluginBase),
/// facet 1 = IAudioProcessor, facet 2 = IEditController. queryInterface
/// hands out facet addresses; refcounting is a no-op (static lifetime).
#[repr(C)]
struct FixtureObject {
    component_vtable: *const ComponentVTable,
    processor_vtable: *const AudioProcessorVTable,
    controller_vtable: *const EditControllerVTable,
    midi_mapping_vtable: *const MidiMappingVTable,
}

unsafe impl Sync for FixtureObject {}

"#
}

pub(super) fn object_footer_fragment() -> &'static str {
    r#"static FIXTURE_OBJECT: FixtureObject = FixtureObject {
    component_vtable: &COMPONENT_VTABLE,
    processor_vtable: &PROCESSOR_VTABLE,
    controller_vtable: &CONTROLLER_VTABLE,
    midi_mapping_vtable: &MIDI_MAPPING_VTABLE,
};

fn object_base() -> *mut c_void {
    &FIXTURE_OBJECT as *const FixtureObject as *mut c_void
}

fn processor_facet() -> *mut c_void {
    unsafe { &raw const FIXTURE_OBJECT.processor_vtable as *mut c_void }
}

fn controller_facet() -> *mut c_void {
    unsafe { &raw const FIXTURE_OBJECT.controller_vtable as *mut c_void }
}

fn midi_mapping_facet() -> *mut c_void {
    unsafe { &raw const FIXTURE_OBJECT.midi_mapping_vtable as *mut c_void }
}

unsafe fn facet_for(iid: *const Tuid) -> Option<*mut c_void> {
    if iid.is_null() {
        return None;
    }
    let iid = *iid;
    if iid == FUNKNOWN_IID || iid == IPLUGIN_BASE_IID || iid == ICOMPONENT_IID {
        Some(object_base())
    } else if iid == IAUDIO_PROCESSOR_IID {
        Some(processor_facet())
    } else if iid == IEDIT_CONTROLLER_IID {
        Some(controller_facet())
    } else if iid == IMIDI_MAPPING_IID {
        Some(midi_mapping_facet())
    } else {
        None
    }
}

unsafe extern "C" fn component_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    shared_query_interface(iid, out)
}

unsafe extern "C" fn processor_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    shared_query_interface(iid, out)
}

unsafe extern "C" fn controller_query_interface(
    _this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    shared_query_interface(iid, out)
}

unsafe fn shared_query_interface(iid: *const Tuid, out: *mut *mut c_void) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    match facet_for(iid) {
        Some(facet) => {
            *out = facet;
            K_RESULT_OK
        }
        None => {
            *out = ptr::null_mut();
            K_NO_INTERFACE
        }
    }
}

unsafe extern "C" fn no_op_add_ref(_this: *mut c_void) -> u32 { 1 }
unsafe extern "C" fn no_op_release(_this: *mut c_void) -> u32 { 1 }
unsafe extern "C" fn base_initialize(_this: *mut c_void, _context: *mut c_void) -> Tresult {
    K_RESULT_OK
}
unsafe extern "C" fn base_terminate(_this: *mut c_void) -> Tresult { K_RESULT_OK }
unsafe extern "C" fn state_noop(_this: *mut c_void, _stream: *mut c_void) -> Tresult {
    K_RESULT_OK
}

"#
}
