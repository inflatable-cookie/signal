use std::{
    ffi::{c_char, c_void},
    ptr,
};

use clap_sys::{host::clap_host, version::clap_version};

pub(super) fn discovery_host() -> clap_host {
    clap_host {
        clap_version: clap_version {
            major: 1,
            minor: 0,
            revision: 0,
        },
        host_data: ptr::null_mut(),
        name: c"Signal Discovery Host".as_ptr(),
        vendor: c"Signal".as_ptr(),
        url: c"https://signal.dev".as_ptr(),
        version: c"0.1.0".as_ptr(),
        get_extension: Some(discovery_host_get_extension),
        request_restart: Some(discovery_host_request_restart),
        request_process: Some(discovery_host_request_process),
        request_callback: Some(discovery_host_request_callback),
    }
}

unsafe extern "C" fn discovery_host_get_extension(
    _host: *const clap_host,
    _extension_id: *const c_char,
) -> *const c_void {
    ptr::null()
}

unsafe extern "C" fn discovery_host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn discovery_host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn discovery_host_request_callback(_host: *const clap_host) {}
