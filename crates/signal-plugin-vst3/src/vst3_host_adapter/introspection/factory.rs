//! In-process VST3 factory class enumeration.

use crate::vst3_host_adapter::Vst3HostPlatform;
use std::{ffi::c_void, io, path::Path};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use crate::vst3_host_adapter::hosting::{
    clear_factory_host_context, set_factory_host_context, should_set_factory_host_context,
};

#[cfg(not(target_os = "macos"))]
use super::derive::libloading_to_io;
use super::derive::{bytes_to_upper_hex, c_char_array_to_string, role_from_category};
#[cfg(target_os = "macos")]
use super::macos_bundle;
#[cfg(not(target_os = "macos"))]
use super::paths::resolve_module_binary_path;
use super::types::*;
#[cfg(target_os = "macos")]
pub(crate) fn load_vst3_factory_classes_from_module(
    bundle_root: &Path,
    _platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let bundle = macos_bundle::MacVst3Bundle::load(bundle_root)?;
    unsafe {
        if let Some(entry) = bundle.entry() {
            if !entry(bundle.bundle_ref()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VST3 bundleEntry returned false",
                ));
            }
        }
        let get_plugin_factory = bundle.factory().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "VST3 GetPluginFactory missing")
        })?;
        let snapshot = read_factory_classes(
            get_plugin_factory(),
            should_set_factory_host_context(bundle_root),
        );
        if let Some(exit) = bundle.exit() {
            exit();
        }
        snapshot
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn load_vst3_factory_classes_from_module(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let module_path = resolve_module_binary_path(bundle_root, platform)?;
    let library = unsafe { Library::new(&module_path) }.map_err(libloading_to_io)?;
    unsafe {
        if let Ok(entry) = library.get::<EntryProc>(entry_symbol(platform)) {
            if !entry(std::ptr::null_mut()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VST3 module entry returned false",
                ));
            }
        }
        let get_plugin_factory = library
            .get::<GetPluginFactoryProc>(b"GetPluginFactory\0")
            .map_err(libloading_to_io)?;
        let snapshot = read_factory_classes(
            get_plugin_factory(),
            should_set_factory_host_context(bundle_root),
        );
        if let Ok(exit) = library.get::<ExitProc>(exit_symbol(platform)) {
            exit();
        }
        snapshot
    }
}

pub(crate) fn read_factory_classes(
    factory_ptr: *mut c_void,
    set_host_context: bool,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    if factory_ptr.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 GetPluginFactory returned null",
        ));
    }

    let factory = factory_ptr as *mut RawPluginFactory;
    let vtable = unsafe { (*factory).vtable };
    if vtable.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 factory vtable was null",
        ));
    }
    let context_set = set_host_context && unsafe { set_factory_host_context(factory_ptr) };

    let mut factory_info = PFactoryInfo {
        vendor: [0; 64],
        url: [0; 256],
        email: [0; 128],
        flags: 0,
    };
    let vendor = if unsafe { ((*vtable).get_factory_info)(factory_ptr, &mut factory_info) } == 0 {
        Some(c_char_array_to_string(&factory_info.vendor))
    } else {
        None
    };

    let mut class_count = unsafe { ((*vtable).count_classes)(factory_ptr) };
    if class_count <= 0 && context_set {
        unsafe {
            clear_factory_host_context(factory_ptr);
        }
        class_count = unsafe { ((*vtable).count_classes)(factory_ptr) };
    }
    if class_count <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 factory exposed no classes",
        ));
    }
    let mut factory_2_ptr = std::ptr::null_mut();
    let factory_2 = if unsafe {
        ((*vtable).query_interface)(
            factory_ptr,
            IPLUGIN_FACTORY_2_IID.as_ptr().cast(),
            &mut factory_2_ptr,
        )
    } == 0
        && !factory_2_ptr.is_null()
    {
        Some(factory_2_ptr)
    } else {
        None
    };
    let mut classes = Vec::new();
    for index in 0..class_count {
        let mut class_info = PClassInfo {
            cid: [0; 16],
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
        };
        if unsafe { ((*vtable).get_class_info)(factory_ptr, index, &mut class_info) } != 0 {
            continue;
        }
        let category = c_char_array_to_string(&class_info.category);
        let class_info_2 = factory_2.and_then(|factory_2_ptr| {
            let factory_2 = factory_2_ptr as *mut RawPluginFactory;
            let factory_2_vtable = unsafe { (*factory_2).vtable as *const PluginFactory2VTable };
            let mut info = PClassInfo2 {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
                class_flags: 0,
                subcategories: [0; 128],
                vendor: [0; 64],
                version: [0; 64],
                sdk_version: [0; 64],
            };
            if !factory_2_vtable.is_null()
                && unsafe {
                    ((*factory_2_vtable).get_class_info_2)(factory_2_ptr, index, &mut info)
                } == 0
            {
                Some(info)
            } else {
                None
            }
        });
        let subcategories = class_info_2
            .as_ref()
            .map(|info| c_char_array_to_string(&info.subcategories))
            .unwrap_or_default()
            .split('|')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
        classes.push(Vst3FactoryClass {
            role: role_from_category(&category),
            class_id: bytes_to_upper_hex(&class_info.cid),
            category,
            name: c_char_array_to_string(&class_info.name),
            vendor: class_info_2
                .as_ref()
                .map(|info| c_char_array_to_string(&info.vendor))
                .filter(|value| !value.is_empty()),
            version: class_info_2
                .as_ref()
                .map(|info| c_char_array_to_string(&info.version))
                .filter(|value| !value.is_empty()),
            subcategories,
        });
    }

    if let Some(factory_2_ptr) = factory_2 {
        let factory_2 = factory_2_ptr as *mut RawPluginFactory;
        let factory_2_vtable = unsafe { (*factory_2).vtable };
        if !factory_2_vtable.is_null() {
            unsafe { ((*factory_2_vtable).release)(factory_2_ptr) };
        }
    }

    if classes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 factory exposed no readable classes",
        ));
    }
    Ok((vendor.filter(|value| !value.is_empty()), classes))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn entry_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleEntry\0",
        Vst3HostPlatform::Linux => b"ModuleEntry\0",
        Vst3HostPlatform::Windows => b"InitDll\0",
    }
}
#[cfg(not(target_os = "macos"))]
pub(crate) fn exit_symbol(platform: Vst3HostPlatform) -> &'static [u8] {
    match platform {
        Vst3HostPlatform::MacOs => b"bundleExit\0",
        Vst3HostPlatform::Linux => b"ModuleExit\0",
        Vst3HostPlatform::Windows => b"ExitDll\0",
    }
}
