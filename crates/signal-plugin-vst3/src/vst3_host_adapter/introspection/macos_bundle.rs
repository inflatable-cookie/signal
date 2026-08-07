//! macOS VST3 bundle loading for factory introspection.

use std::{
    ffi::{c_char, c_void, CString},
    io,
    path::Path,
};

use super::types::{EntryProc, ExitProc, GetPluginFactoryProc};

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
    bundle: CFBundleRef,
}

impl MacVst3Bundle {
    pub(super) fn load(bundle_root: &Path) -> io::Result<Self> {
        let path = bundle_root
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 path"))?;
        let path_c = CString::new(path)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 path"))?;
        unsafe {
            let path_string = CFStringCreateWithCString(
                kCFAllocatorDefault,
                path_c.as_ptr(),
                K_CF_STRING_ENCODING_UTF8,
            );
            if path_string.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid VST3 path",
                ));
            }
            let bundle_url = CFURLCreateWithFileSystemPath(
                kCFAllocatorDefault,
                path_string,
                K_CF_URL_POSIX_PATH_STYLE,
                1,
            );
            CFRelease(path_string);
            if bundle_url.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid VST3 bundle URL",
                ));
            }
            let bundle = CFBundleCreate(kCFAllocatorDefault, bundle_url);
            CFRelease(bundle_url);
            if bundle.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "failed to open VST3 bundle",
                ));
            }
            if CFBundleLoadExecutable(bundle) == 0 {
                CFRelease(bundle);
                return Err(io::Error::other("failed to load VST3 bundle executable"));
            }
            Ok(Self { bundle })
        }
    }

    pub(super) fn bundle_ref(&self) -> *mut c_void {
        self.bundle.cast()
    }

    fn function_ptr(&self, name: &str) -> Option<*mut c_void> {
        let name_c = CString::new(name).ok()?;
        unsafe {
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
    }

    pub(super) fn entry(&self) -> Option<EntryProc> {
        self.function_ptr("bundleEntry")
            .map(|pointer| unsafe { std::mem::transmute(pointer) })
    }

    pub(super) fn exit(&self) -> Option<ExitProc> {
        self.function_ptr("bundleExit")
            .map(|pointer| unsafe { std::mem::transmute(pointer) })
    }

    pub(super) fn factory(&self) -> Option<GetPluginFactoryProc> {
        self.function_ptr("GetPluginFactory")
            .map(|pointer| unsafe { std::mem::transmute(pointer) })
    }
}

impl Drop for MacVst3Bundle {
    fn drop(&mut self) {
        unsafe {
            // Objective-C classes registered by a plugin bundle cannot be
            // unregistered safely, so discovery releases the bundle object
            // without unloading executable code.
            CFRelease(self.bundle);
        }
    }
}
