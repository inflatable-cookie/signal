//! VST3 hosting wire: parameters.

use std::ffi::c_void;
use std::ptr;

use signal_plugin::{PluginParamChange, PLUGIN_PARAM_CHANGE_CAPACITY};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::introspection::resolve_module_binary_path;

use super::com::*;

// ── Host-side IParameterChanges (g12.023 param-set wire) ───────────────────
//
// The input parameter-change list handed to `IAudioProcessor::process`:
// one single-point queue per changed parameter, every point at sample
// offset 0 (block-boundary application, sample-accuracy posture v1).
// Everything is preallocated at the change-queue capacity and rebuilt in
// place per block — no allocation on the audio thread. Refcounting is a
// no-op: the session owns the objects and outlives every process call.

/// `IParamValueQueue` vtable (FUnknown + queue methods, declaration order).
#[repr(C)]
pub(crate) struct ParamValueQueueVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_parameter_id: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_point_count: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_point: unsafe extern "C" fn(*mut c_void, i32, *mut i32, *mut f64) -> Tresult,
    pub(crate) add_point: unsafe extern "C" fn(*mut c_void, i32, f64, *mut i32) -> Tresult,
}

/// `IParameterChanges` vtable (FUnknown + list methods, declaration order).
#[repr(C)]
pub(crate) struct ParameterChangesVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) get_parameter_count: unsafe extern "C" fn(*mut c_void) -> i32,
    pub(crate) get_parameter_data: unsafe extern "C" fn(*mut c_void, i32) -> *mut c_void,
    pub(crate) add_parameter_data:
        unsafe extern "C" fn(*mut c_void, *const u32, *mut i32) -> *mut c_void,
}

/// Sample-offset points one host param queue can carry per block (wire
/// writes use one point at offset 0; MIDI-mapped CC series use one point
/// per CC event at its intra-block offset).
pub(crate) const PARAM_QUEUE_POINT_CAPACITY: usize = 128;

/// One value queue: `(sample_offset, value)` points for one parameter,
/// ascending offsets. Preallocated; rebuilt in place per block.
#[repr(C)]
pub(crate) struct HostParamValueQueue {
    pub(crate) vtable: *const ParamValueQueueVTable,
    pub(crate) parameter_id: u32,
    pub(crate) points: Box<[(i32, f64)]>,
    pub(crate) point_count: usize,
}

/// The block's input parameter-change list: a fixed-length queue pool plus
/// the active count. Boxed by the session so every pointer handed to the
/// plugin stays stable.
#[repr(C)]
pub(crate) struct HostParameterChanges {
    pub(crate) vtable: *const ParameterChangesVTable,
    pub(crate) queues: Box<[HostParamValueQueue]>,
    pub(crate) active: usize,
}

pub(crate) static PARAM_VALUE_QUEUE_VTABLE: ParamValueQueueVTable = ParamValueQueueVTable {
    query_interface: param_queue_query_interface,
    add_ref: param_com_add_ref,
    release: param_com_release,
    get_parameter_id: param_queue_get_parameter_id,
    get_point_count: param_queue_get_point_count,
    get_point: param_queue_get_point,
    add_point: param_queue_add_point,
};

pub(crate) static PARAMETER_CHANGES_VTABLE: ParameterChangesVTable = ParameterChangesVTable {
    query_interface: param_changes_query_interface,
    add_ref: param_com_add_ref,
    release: param_com_release,
    get_parameter_count: param_changes_get_parameter_count,
    get_parameter_data: param_changes_get_parameter_data,
    add_parameter_data: param_changes_add_parameter_data,
};

pub(crate) unsafe extern "C" fn param_com_add_ref(_this: *mut c_void) -> u32 {
    1
}

pub(crate) unsafe extern "C" fn param_com_release(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn param_queue_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPARAM_VALUE_QUEUE_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn param_queue_get_parameter_id(this: *mut c_void) -> u32 {
    (*this.cast::<HostParamValueQueue>()).parameter_id
}

unsafe extern "C" fn param_queue_get_point_count(this: *mut c_void) -> i32 {
    (*this.cast::<HostParamValueQueue>()).point_count as i32
}

unsafe extern "C" fn param_queue_get_point(
    this: *mut c_void,
    index: i32,
    sample_offset: *mut i32,
    value: *mut f64,
) -> Tresult {
    if sample_offset.is_null() || value.is_null() {
        return K_NO_INTERFACE;
    }
    let queue = &*this.cast::<HostParamValueQueue>();
    if index < 0 || index as usize >= queue.point_count {
        return K_NO_INTERFACE;
    }
    let (offset, point_value) = queue.points[index as usize];
    *sample_offset = offset;
    *value = point_value;
    K_RESULT_OK
}

unsafe extern "C" fn param_queue_add_point(
    _this: *mut c_void,
    _sample_offset: i32,
    _value: f64,
    _index: *mut i32,
) -> Tresult {
    // Input list: the host writes it, the plugin only reads.
    K_NO_INTERFACE
}

unsafe extern "C" fn param_changes_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_NO_INTERFACE;
    }
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IPARAMETER_CHANGES_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    *out = ptr::null_mut();
    K_NO_INTERFACE
}

unsafe extern "C" fn param_changes_get_parameter_count(this: *mut c_void) -> i32 {
    (*this.cast::<HostParameterChanges>()).active as i32
}

unsafe extern "C" fn param_changes_get_parameter_data(
    this: *mut c_void,
    index: i32,
) -> *mut c_void {
    let changes = &mut *this.cast::<HostParameterChanges>();
    if index < 0 || index as usize >= changes.active {
        return ptr::null_mut();
    }
    (&mut changes.queues[index as usize] as *mut HostParamValueQueue).cast()
}

unsafe extern "C" fn param_changes_add_parameter_data(
    this: *mut c_void,
    id: *const u32,
    index: *mut i32,
) -> *mut c_void {
    if id.is_null() || index.is_null() {
        return ptr::null_mut();
    }
    let changes = &mut *this.cast::<HostParameterChanges>();
    if let Some((queue_index, queue)) = changes.queues[..changes.active]
        .iter_mut()
        .enumerate()
        .find(|(_, queue)| queue.parameter_id == *id)
    {
        *index = queue_index as i32;
        return (queue as *mut HostParamValueQueue).cast();
    }
    if changes.active == changes.queues.len() {
        return ptr::null_mut();
    }
    let queue_index = changes.active;
    changes.active += 1;
    let queue = &mut changes.queues[queue_index];
    queue.parameter_id = *id;
    queue.point_count = 0;
    *index = queue_index as i32;
    (queue as *mut HostParamValueQueue).cast()
}

impl HostParameterChanges {
    pub(crate) fn new() -> Box<Self> {
        let queues = (0..PLUGIN_PARAM_CHANGE_CAPACITY)
            .map(|_| HostParamValueQueue {
                vtable: &PARAM_VALUE_QUEUE_VTABLE,
                parameter_id: 0,
                points: vec![(0i32, 0f64); PARAM_QUEUE_POINT_CAPACITY].into_boxed_slice(),
                point_count: 0,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Box::new(Self {
            vtable: &PARAMETER_CHANGES_VTABLE,
            queues,
            active: 0,
        })
    }

    /// Reset to an empty list (start of block). Alloc-free.
    pub(crate) fn clear(&mut self) {
        self.active = 0;
    }

    /// Append one `(sample_offset, value)` point for `parameter_id`,
    /// reusing the parameter's queue when one is already active this block.
    /// Alloc-free; silently drops on pool/point capacity overflow.
    pub(crate) fn push_point(&mut self, parameter_id: u32, sample_offset: i32, value: f64) {
        let existing = self.queues[..self.active]
            .iter_mut()
            .find(|queue| queue.parameter_id == parameter_id);
        let queue = match existing {
            Some(queue) => queue,
            None => {
                if self.active == self.queues.len() {
                    return;
                }
                let queue = &mut self.queues[self.active];
                queue.parameter_id = parameter_id;
                queue.point_count = 0;
                self.active += 1;
                queue
            }
        };
        if queue.point_count == queue.points.len() {
            return;
        }
        queue.points[queue.point_count] = (sample_offset, value);
        queue.point_count += 1;
    }

    /// Rebuild the list in place from the drained wire changes (one point
    /// per parameter at offset 0 — block-boundary posture). Alloc-free.
    pub(crate) fn set_changes(&mut self, changes: &[PluginParamChange]) {
        self.clear();
        for change in changes {
            self.push_point(change.parameter_id, 0, change.value);
        }
    }
}
