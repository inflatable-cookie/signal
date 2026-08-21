//! VST3 hosting wire: module.

use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;
use std::ptr;

#[cfg(not(target_os = "macos"))]
use libloading::Library;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::introspection::resolve_module_binary_path;

#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::hosting::current_vst3_platform;
use crate::vst3_host_adapter::hosting::Vst3HostingError;
#[cfg(not(target_os = "macos"))]
use crate::vst3_host_adapter::Vst3HostPlatform;

use super::com::*;
use super::host_application::*;
use super::stream::*;

// ── Module loading ──────────────────────────────────────────────────────────

pub(crate) type EntryProc = unsafe extern "C" fn(*mut c_void) -> bool;
pub(crate) type ExitProc = unsafe extern "C" fn();
pub(crate) type GetPluginFactoryProc = unsafe extern "C" fn() -> *mut c_void;

#[cfg(target_os = "macos")]
pub(crate) mod macos_bundle {
    use super::*;

    type CFAllocatorRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFURLRef = *const c_void;
    type CFBundleRef = *mut c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;
        fn CFBundleCreate(allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef;
        fn CFBundleGetFunctionPointerForName(
            bundle: CFBundleRef,
            functionName: CFStringRef,
        ) -> *mut c_void;
        fn CFBundleLoadExecutable(bundle: CFBundleRef) -> u8;
        fn CFRelease(cf: *const c_void);
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            cStr: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFURLCreateWithFileSystemPath(
            allocator: CFAllocatorRef,
            filePath: CFStringRef,
            pathStyle: isize,
            isDirectory: u8,
        ) -> CFURLRef;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CF_URL_POSIX_PATH_STYLE: isize = 0;

    pub(super) struct MacVst3Bundle {
        pub(crate) bundle: CFBundleRef,
    }

    impl MacVst3Bundle {
        pub(super) fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
            let path = bundle_root
                .to_str()
                .ok_or_else(|| Vst3HostingError::new("module_bundle_path_invalid"))?;
            let path_c = CString::new(path)
                .map_err(|_| Vst3HostingError::new("module_bundle_path_invalid"))?;
            unsafe {
                let path_string = CFStringCreateWithCString(
                    kCFAllocatorDefault,
                    path_c.as_ptr(),
                    K_CF_STRING_ENCODING_UTF8,
                );
                if path_string.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_path_invalid"));
                }
                let bundle_url = CFURLCreateWithFileSystemPath(
                    kCFAllocatorDefault,
                    path_string,
                    K_CF_URL_POSIX_PATH_STYLE,
                    1,
                );
                CFRelease(path_string);
                if bundle_url.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_url_invalid"));
                }
                let bundle = CFBundleCreate(kCFAllocatorDefault, bundle_url);
                CFRelease(bundle_url);
                if bundle.is_null() {
                    return Err(Vst3HostingError::new("module_bundle_open_failed"));
                }
                if CFBundleLoadExecutable(bundle) == 0 {
                    CFRelease(bundle);
                    return Err(Vst3HostingError::new("module_open_failed"));
                }
                Ok(Self { bundle })
            }
        }

        pub(super) fn bundle_ref(&self) -> *mut c_void {
            self.bundle.cast()
        }

        pub(crate) unsafe fn function_ptr(&self, name: &str) -> Option<*mut c_void> {
            let name_c = CString::new(name).ok()?;
            let name_string = CFStringCreateWithCString(
                kCFAllocatorDefault,
                name_c.as_ptr(),
                K_CF_STRING_ENCODING_UTF8,
            );
            if name_string.is_null() {
                return None;
            }
            let pointer = CFBundleGetFunctionPointerForName(self.bundle, name_string);
            CFRelease(name_string);
            (!pointer.is_null()).then_some(pointer)
        }

        pub(super) unsafe fn entry(&self) -> Option<EntryProc> {
            self.function_ptr("bundleEntry")
                .map(|pointer| std::mem::transmute(pointer))
        }

        pub(super) unsafe fn exit(&self) -> Option<ExitProc> {
            self.function_ptr("bundleExit")
                .map(|pointer| std::mem::transmute(pointer))
        }

        pub(super) unsafe fn factory(&self) -> Option<GetPluginFactoryProc> {
            self.function_ptr("GetPluginFactory")
                .map(|pointer| std::mem::transmute(pointer))
        }
    }

    impl Drop for MacVst3Bundle {
        fn drop(&mut self) {
            unsafe {
                // Do not CFBundleUnloadExecutable: Objective-C classes cannot
                // be unregistered safely once a plugin bundle has registered
                // them with the process runtime.
                CFRelease(self.bundle);
            }
        }
    }
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

/// A dlopen'd VST3 module with its platform entry run and the factory
/// resolved. Runs the exit proc and closes the library on drop.
pub(crate) struct LoadedVst3Module {
    #[cfg(target_os = "macos")]
    bundle: macos_bundle::MacVst3Bundle,
    #[cfg(not(target_os = "macos"))]
    pub(crate) library: Library,
    pub(crate) factory: *mut c_void,
    pub(crate) factory_context_set: bool,
    pub(crate) exit: Option<ExitProc>,
}

impl LoadedVst3Module {
    #[cfg(target_os = "macos")]
    pub(crate) fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
        let bundle = macos_bundle::MacVst3Bundle::load(bundle_root)?;
        unsafe {
            if let Some(entry) = bundle.entry() {
                if !entry(bundle.bundle_ref()) {
                    return Err(Vst3HostingError::new("module_entry_failed"));
                }
            }
            let get_factory = bundle
                .factory()
                .ok_or_else(|| Vst3HostingError::new("get_plugin_factory_missing"))?;
            let factory = get_factory();
            if factory.is_null() {
                return Err(Vst3HostingError::new("plugin_factory_null"));
            }
            let factory_context_set =
                should_set_factory_host_context(bundle_root) && set_factory_host_context(factory);
            let exit = bundle.exit();
            Ok(Self {
                bundle,
                factory,
                factory_context_set,
                exit,
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn load(bundle_root: &Path) -> Result<Self, Vst3HostingError> {
        let platform = current_vst3_platform();
        let module_path = resolve_module_binary_path(bundle_root, platform)
            .map_err(|_| Vst3HostingError::new("module_binary_unresolved"))?;
        let library = unsafe { Library::new(&module_path) }
            .map_err(|_| Vst3HostingError::new("module_open_failed"))?;
        unsafe {
            if let Ok(entry) = library.get::<EntryProc>(entry_symbol(platform)) {
                if !entry(ptr::null_mut()) {
                    return Err(Vst3HostingError::new("module_entry_failed"));
                }
            }
            let get_factory = library
                .get::<GetPluginFactoryProc>(b"GetPluginFactory\0")
                .map_err(|_| Vst3HostingError::new("get_plugin_factory_missing"))?;
            let factory = get_factory();
            if factory.is_null() {
                return Err(Vst3HostingError::new("plugin_factory_null"));
            }
            let factory_context_set =
                should_set_factory_host_context(bundle_root) && set_factory_host_context(factory);
            let exit = library
                .get::<ExitProc>(exit_symbol(platform))
                .ok()
                .map(|symbol| *symbol);
            Ok(Self {
                library,
                factory,
                factory_context_set,
                exit,
            })
        }
    }

    /// Create an instance of `cid` exposing `iid` through the factory.
    pub(crate) unsafe fn create_instance(&self, cid: &Tuid, iid: &Tuid) -> Option<*mut c_void> {
        let vtable = vtable_of::<PluginFactoryVTable>(self.factory);
        let mut out: *mut c_void = ptr::null_mut();
        let mut result =
            ((*vtable).create_instance)(self.factory, cid.as_ptr(), iid.as_ptr(), &mut out);
        if self.factory_context_set && (result != K_RESULT_OK || out.is_null()) {
            clear_factory_host_context(self.factory);
            out = ptr::null_mut();
            result =
                ((*vtable).create_instance)(self.factory, cid.as_ptr(), iid.as_ptr(), &mut out);
        }
        (result == K_RESULT_OK && !out.is_null()).then_some(out)
    }

    /// Return the factory's sole audio-module class, if it has exactly one.
    pub(crate) unsafe fn unique_component_class_id(&self) -> Option<Tuid> {
        let vtable = vtable_of::<PluginFactoryVTable>(self.factory);
        let class_count = ((*vtable).count_classes)(self.factory);
        let mut component = None;
        for index in 0..class_count {
            let mut info = FactoryClassInfo {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
            };
            if ((*vtable).get_class_info)(
                self.factory,
                index,
                (&mut info as *mut FactoryClassInfo).cast(),
            ) != K_RESULT_OK
            {
                continue;
            }
            let category = CStr::from_ptr(info.category.as_ptr()).to_bytes();
            if category != b"Audio Module Class" {
                continue;
            }
            if component.is_some() {
                return None;
            }
            component = Some(info.cid);
        }
        component
    }
}

impl Drop for LoadedVst3Module {
    fn drop(&mut self) {
        if let Some(exit) = self.exit {
            unsafe { exit() };
        }
        #[cfg(not(target_os = "macos"))]
        // `library` drops after this body, unloading the module last.
        let _ = &self.library;
        #[cfg(target_os = "macos")]
        // `bundle` drops after this body, releasing the CFBundle object last.
        let _ = &self.bundle;
    }
}
