//! Host-side IBStream for opaque component/controller state.

use std::ffi::c_void;
use std::ptr;

use super::super::com::*;

#[repr(C)]
pub(crate) struct MemoryStreamVTable {
    pub(crate) query_interface:
        unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    pub(crate) add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "C" fn(*mut c_void) -> u32,
    pub(crate) read: unsafe extern "C" fn(*mut c_void, *mut c_void, i32, *mut i32) -> Tresult,
    pub(crate) write: unsafe extern "C" fn(*mut c_void, *const c_void, i32, *mut i32) -> Tresult,
    pub(crate) seek: unsafe extern "C" fn(*mut c_void, i64, i32, *mut i64) -> Tresult,
    pub(crate) tell: unsafe extern "C" fn(*mut c_void, *mut i64) -> Tresult,
}

#[repr(C)]
pub(crate) struct MemoryStream {
    pub(crate) vtable: *const MemoryStreamVTable,
    pub(crate) bytes: Vec<u8>,
    pub(crate) position: usize,
    pub(crate) writable: bool,
}

impl MemoryStream {
    pub(crate) fn writer() -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: Vec::new(),
            position: 0,
            writable: true,
        }
    }

    pub(crate) fn reader(bytes: &[u8]) -> Self {
        Self {
            vtable: &MEMORY_STREAM_VTABLE,
            bytes: bytes.to_vec(),
            position: 0,
            writable: false,
        }
    }

    pub(crate) fn as_raw(&mut self) -> *mut c_void {
        (self as *mut Self).cast()
    }
}

unsafe extern "C" fn stream_query_interface(
    this: *mut c_void,
    iid: *const Tuid,
    out: *mut *mut c_void,
) -> Tresult {
    if out.is_null() {
        return K_RESULT_FALSE;
    }
    *out = ptr::null_mut();
    if !iid.is_null() && (*iid == FUNKNOWN_IID || *iid == IBSTREAM_IID) {
        *out = this;
        return K_RESULT_OK;
    }
    K_NO_INTERFACE
}

unsafe extern "C" fn stream_add_ref(_this: *mut c_void) -> u32 {
    1
}

unsafe extern "C" fn stream_release(_this: *mut c_void) -> u32 {
    1
}

pub(crate) unsafe extern "C" fn stream_read(
    this: *mut c_void,
    buffer: *mut c_void,
    requested: i32,
    read: *mut i32,
) -> Tresult {
    if this.is_null() || buffer.is_null() || requested < 0 {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    let available = stream.bytes.len().saturating_sub(stream.position);
    let count = available.min(requested as usize);
    ptr::copy_nonoverlapping(
        stream.bytes.as_ptr().add(stream.position),
        buffer.cast::<u8>(),
        count,
    );
    stream.position += count;
    if !read.is_null() {
        *read = count as i32;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_write(
    this: *mut c_void,
    buffer: *const c_void,
    requested: i32,
    written: *mut i32,
) -> Tresult {
    if this.is_null() || buffer.is_null() || requested < 0 {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    if !stream.writable {
        return K_RESULT_FALSE;
    }
    let count = requested as usize;
    let end = match stream.position.checked_add(count) {
        Some(end) => end,
        None => return K_RESULT_FALSE,
    };
    if end > stream.bytes.len() {
        stream.bytes.resize(end, 0);
    }
    ptr::copy_nonoverlapping(
        buffer.cast::<u8>(),
        stream.bytes.as_mut_ptr().add(stream.position),
        count,
    );
    stream.position = end;
    if !written.is_null() {
        *written = requested;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_seek(
    this: *mut c_void,
    offset: i64,
    mode: i32,
    result: *mut i64,
) -> Tresult {
    if this.is_null() {
        return K_RESULT_FALSE;
    }
    let stream = &mut *(this as *mut MemoryStream);
    let base = match mode {
        0 => 0i64,
        1 => stream.position as i64,
        2 => stream.bytes.len() as i64,
        _ => return K_RESULT_FALSE,
    };
    let Some(position) = base.checked_add(offset) else {
        return K_RESULT_FALSE;
    };
    if position < 0 {
        return K_RESULT_FALSE;
    }
    stream.position = position as usize;
    if !result.is_null() {
        *result = position;
    }
    K_RESULT_OK
}

pub(crate) unsafe extern "C" fn stream_tell(this: *mut c_void, position: *mut i64) -> Tresult {
    if this.is_null() || position.is_null() {
        return K_RESULT_FALSE;
    }
    *position = (*(this as *mut MemoryStream)).position as i64;
    K_RESULT_OK
}

pub(crate) static MEMORY_STREAM_VTABLE: MemoryStreamVTable = MemoryStreamVTable {
    query_interface: stream_query_interface,
    add_ref: stream_add_ref,
    release: stream_release,
    read: stream_read,
    write: stream_write,
    seek: stream_seek,
    tell: stream_tell,
};
