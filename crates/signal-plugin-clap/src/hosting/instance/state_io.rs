use std::{ffi::c_void, ptr};

use clap_sys::stream::{clap_istream, clap_ostream};

pub(crate) struct ClapStateReadCursor<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) offset: usize,
}

pub(crate) unsafe extern "C" fn clap_state_write(
    stream: *const clap_ostream,
    buffer: *const c_void,
    size: u64,
) -> i64 {
    if stream.is_null() || buffer.is_null() || size > i64::MAX as u64 {
        return -1;
    }
    let bytes = &mut *((*stream).ctx as *mut Vec<u8>);
    let input = std::slice::from_raw_parts(buffer.cast::<u8>(), size as usize);
    bytes.extend_from_slice(input);
    size as i64
}

pub(crate) unsafe extern "C" fn clap_state_read(
    stream: *const clap_istream,
    buffer: *mut c_void,
    size: u64,
) -> i64 {
    if stream.is_null() || buffer.is_null() || size > i64::MAX as u64 {
        return -1;
    }
    let source = &mut *((*stream).ctx as *mut ClapStateReadCursor<'_>);
    let remaining = source.bytes.len().saturating_sub(source.offset);
    let count = remaining.min(size as usize);
    if count > 0 {
        ptr::copy_nonoverlapping(
            source.bytes.as_ptr().add(source.offset),
            buffer.cast(),
            count,
        );
        source.offset += count;
    }
    count as i64
}
