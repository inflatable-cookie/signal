//! AUv2 Cocoa editor view hosting (g12.024, GUI phase 2).
//!
//! Embedded editor support for the IN-PROCESS tier: the unit's custom
//! Cocoa view is resolved through `kAudioUnitProperty_CocoaUI`
//! (`AudioUnitCocoaViewInfo`: view bundle URL + factory class name), the
//! bundle is loaded, the factory instantiated, and
//! `uiViewForAudioUnit:withSize:` produces an `NSView` the host attaches
//! as a child of its window's content view.
//!
//! Units WITHOUT a Cocoa UI report unsupported — the generic view is
//! deliberately not built (the parameter editor covers them; packet
//! posture). The Objective-C surface is the house-style handwritten FFI:
//! a handful of typed `objc_msgSend` casts, no objc crate.
//!
//! MAIN-THREAD CONTRACT: everything below `cocoa_view_info` touches AppKit
//! and must run on the application main thread (Tauri
//! `run_on_main_thread`) — this module can only document that, not enforce
//! it. The `cocoa_view_info` probe itself is a plain AudioUnit property
//! read and is safe from the lifecycle thread (load-time caching).

#![allow(unsafe_op_in_unsafe_fn)]

#[cfg_attr(not(target_os = "macos"), allow(unused_imports))]
use super::hosting::AuHostingError;

/// The unit's Cocoa editor description, probed at load: absolute view
/// bundle path plus the factory class name inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuCocoaViewInfo {
    /// Filesystem path of the view bundle (`CFURLRef` resolved to a path).
    pub bundle_path: String,
    /// The `AUCocoaUIBase`-conforming factory class name.
    pub view_class_name: String,
}

/// One live Cocoa editor view on a hosted AU instance: the retained
/// `NSView` child-attached into the host window's content view. Owned by
/// [`crate::AuHostedInstance`]; every method (and drop) runs on the
/// application MAIN THREAD.
#[derive(Debug)]
pub struct AuGuiSession {
    /// The retained plugin `NSView` (child of the host's content view).
    #[cfg(target_os = "macos")]
    view: *mut std::ffi::c_void,
    #[cfg(not(target_os = "macos"))]
    width: u32,
    #[cfg(not(target_os = "macos"))]
    height: u32,
}

impl AuGuiSession {
    /// Current content size. AU Cocoa views may resize themselves without a
    /// host callback, so macOS reads the live view frame on every call.
    /// MAIN THREAD ONLY while an editor is attached.
    pub fn size(&self) -> (u32, u32) {
        #[cfg(target_os = "macos")]
        unsafe {
            macos::view_size(self.view)
        }
        #[cfg(not(target_os = "macos"))]
        {
            (self.width, self.height)
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    #![allow(non_snake_case, non_upper_case_globals)]

    use std::ffi::{c_char, c_void, CString};

    use super::{AuCocoaViewInfo, AuGuiSession, AuHostingError};
    use crate::au_host_adapter::hosting::ffi;

    // ── CoreFoundation additions (bundle/url) ──────────────────────────

    type CFURLRef = *const c_void;
    type CFBundleRef = *mut c_void;
    type CFAllocatorRef = *const c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;
        fn CFURLCopyFileSystemPath(anURL: CFURLRef, pathStyle: isize) -> ffi::CFStringRef;
        fn CFURLCreateWithFileSystemPath(
            allocator: CFAllocatorRef,
            filePath: ffi::CFStringRef,
            pathStyle: isize,
            isDirectory: u8,
        ) -> CFURLRef;
        fn CFStringCreateWithCString(
            alloc: CFAllocatorRef,
            cStr: *const c_char,
            encoding: u32,
        ) -> ffi::CFStringRef;
        fn CFBundleCreate(allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef;
        fn CFBundleLoadExecutable(bundle: CFBundleRef) -> u8;
    }

    /// `kCFURLPOSIXPathStyle`.
    const CF_URL_POSIX_PATH_STYLE: isize = 0;

    /// `kAudioUnitProperty_CocoaUI` (AudioUnitProperties.h).
    const K_AUDIO_UNIT_PROPERTY_COCOA_UI: u32 = 31;

    /// `AudioUnitCocoaViewInfo` with ONE view class (the property is
    /// variable-length; class 0 is the factory hosts use).
    #[repr(C)]
    struct AudioUnitCocoaViewInfoOne {
        mCocoaAUViewBundleLocation: CFURLRef,
        mCocoaAUViewClass: [ffi::CFStringRef; 1],
    }

    // ── Objective-C runtime (typed objc_msgSend casts) ──────────────────

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_msgSend();
        #[cfg(target_arch = "x86_64")]
        fn objc_msgSend_stret();
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
    }

    // AppKit must be linked for NSView machinery to exist in-process.
    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}

    type MsgSendId = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
    type MsgSendVoid = unsafe extern "C" fn(*mut c_void, *mut c_void);
    type MsgSendVoidId = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
    type MsgSendVoidPoint = unsafe extern "C" fn(*mut c_void, *mut c_void, NSPoint);
    type MsgSendVoidUsize = unsafe extern "C" fn(*mut c_void, *mut c_void, usize);
    type MsgSendUiView =
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, NSSize) -> *mut c_void;
    #[cfg(target_arch = "aarch64")]
    type MsgSendRect = unsafe extern "C" fn(*mut c_void, *mut c_void) -> NSRect;
    #[cfg(target_arch = "x86_64")]
    type MsgSendRect = unsafe extern "C" fn(*mut NSRect, *mut c_void, *mut c_void);

    unsafe fn sel(name: &str) -> *mut c_void {
        let name = CString::new(name).expect("selector names never contain NUL");
        sel_registerName(name.as_ptr())
    }

    unsafe fn msg_id(receiver: *mut c_void, selector: *mut c_void) -> *mut c_void {
        let send: MsgSendId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        send(receiver, selector)
    }

    unsafe fn msg_void(receiver: *mut c_void, selector: *mut c_void) {
        let send: MsgSendVoid = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        send(receiver, selector)
    }

    unsafe fn msg_void_id(receiver: *mut c_void, selector: *mut c_void, argument: *mut c_void) {
        let send: MsgSendVoidId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        send(receiver, selector, argument)
    }

    unsafe fn msg_void_point(receiver: *mut c_void, selector: *mut c_void, argument: NSPoint) {
        let send: MsgSendVoidPoint = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        send(receiver, selector, argument)
    }

    unsafe fn msg_void_usize(receiver: *mut c_void, selector: *mut c_void, argument: usize) {
        let send: MsgSendVoidUsize = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        send(receiver, selector, argument)
    }

    /// `-[NSView frame]`: struct return differs per ABI — registers on
    /// arm64, hidden-pointer `objc_msgSend_stret` on x86_64.
    unsafe fn view_frame(view: *mut c_void) -> NSRect {
        let selector = sel("frame");
        #[cfg(target_arch = "aarch64")]
        {
            let send: MsgSendRect = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
            send(view, selector)
        }
        #[cfg(target_arch = "x86_64")]
        {
            let send: MsgSendRect =
                std::mem::transmute(objc_msgSend_stret as unsafe extern "C" fn());
            let mut rect = NSRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
            send(&mut rect, view, selector);
            rect
        }
    }

    pub(super) unsafe fn view_size(view: *mut c_void) -> (u32, u32) {
        let frame = view_frame(view);
        (
            frame.width.max(1.0).round() as u32,
            frame.height.max(1.0).round() as u32,
        )
    }

    /// Probe `kAudioUnitProperty_CocoaUI` on a live unit: the custom Cocoa
    /// view's bundle path + factory class, or `None` when the unit provides
    /// no editor. Plain property read — lifecycle-thread safe.
    ///
    /// # Safety
    /// `unit` must be a live `AudioUnit` handle.
    pub(crate) unsafe fn cocoa_view_info(unit: ffi::AudioUnit) -> Option<AuCocoaViewInfo> {
        let mut size = 0u32;
        let mut writable: ffi::Boolean = 0;
        if ffi::AudioUnitGetPropertyInfo(
            unit,
            K_AUDIO_UNIT_PROPERTY_COCOA_UI,
            ffi::kAudioUnitScope_Global,
            0,
            &mut size,
            &mut writable,
        ) != 0
            || (size as usize) < std::mem::size_of::<AudioUnitCocoaViewInfoOne>()
        {
            return None;
        }
        let mut info = AudioUnitCocoaViewInfoOne {
            mCocoaAUViewBundleLocation: std::ptr::null(),
            mCocoaAUViewClass: [std::ptr::null()],
        };
        let mut io_size = std::mem::size_of::<AudioUnitCocoaViewInfoOne>() as u32;
        if ffi::AudioUnitGetProperty(
            unit,
            K_AUDIO_UNIT_PROPERTY_COCOA_UI,
            ffi::kAudioUnitScope_Global,
            0,
            &mut info as *mut _ as *mut _,
            &mut io_size,
        ) != 0
        {
            return None;
        }
        // The property transfers ownership of the URL and class strings.
        let bundle_path = if info.mCocoaAUViewBundleLocation.is_null() {
            None
        } else {
            let path_string =
                CFURLCopyFileSystemPath(info.mCocoaAUViewBundleLocation, CF_URL_POSIX_PATH_STYLE);
            let path = ffi::cfstring_into_string(path_string);
            ffi::CFRelease(info.mCocoaAUViewBundleLocation);
            path
        };
        let view_class_name = ffi::cfstring_into_string(info.mCocoaAUViewClass[0]);
        match (bundle_path, view_class_name) {
            (Some(bundle_path), Some(view_class_name))
                if !bundle_path.is_empty() && !view_class_name.is_empty() =>
            {
                Some(AuCocoaViewInfo {
                    bundle_path,
                    view_class_name,
                })
            }
            _ => None,
        }
    }

    /// Load the view bundle, instantiate the factory class, and ask it for
    /// the unit's editor `NSView` (retained), child-attached into `parent`.
    /// MAIN THREAD ONLY.
    ///
    /// # Safety
    /// `unit` must be a live `AudioUnit`; `parent` must be a live `NSView*`.
    pub(crate) unsafe fn open_embedded(
        unit: ffi::AudioUnit,
        info: &AuCocoaViewInfo,
        parent: *mut c_void,
    ) -> Result<AuGuiSession, AuHostingError> {
        if parent.is_null() {
            return Err(AuHostingError::new("gui_parent_null"));
        }

        // Load the view bundle so its factory class is registered with the
        // objc runtime. CFBundleCreate returns the existing bundle object
        // for an already-loaded bundle, so reopen cycles are cheap.
        let path_c = CString::new(info.bundle_path.as_str())
            .map_err(|_| AuHostingError::new("gui_bundle_path_invalid"))?;
        let path_string = CFStringCreateWithCString(
            kCFAllocatorDefault,
            path_c.as_ptr(),
            ffi::kCFStringEncodingUTF8,
        );
        if path_string.is_null() {
            return Err(AuHostingError::new("gui_bundle_path_invalid"));
        }
        let bundle_url = CFURLCreateWithFileSystemPath(
            kCFAllocatorDefault,
            path_string,
            CF_URL_POSIX_PATH_STYLE,
            1,
        );
        ffi::CFRelease(path_string);
        if bundle_url.is_null() {
            return Err(AuHostingError::new("gui_bundle_url_invalid"));
        }
        let bundle = CFBundleCreate(kCFAllocatorDefault, bundle_url);
        ffi::CFRelease(bundle_url);
        if bundle.is_null() {
            return Err(AuHostingError::new("gui_bundle_open_failed"));
        }
        if CFBundleLoadExecutable(bundle) == 0 {
            // The bundle object stays alive intentionally (unloading code
            // with live objc classes is unsafe); only the load failed.
            return Err(AuHostingError::new("gui_bundle_load_failed"));
        }

        let class_c = CString::new(info.view_class_name.as_str())
            .map_err(|_| AuHostingError::new("gui_view_class_invalid"))?;
        let factory_class = objc_getClass(class_c.as_ptr());
        if factory_class.is_null() {
            return Err(AuHostingError::new("gui_view_class_missing"));
        }

        let pool = objc_autoreleasePoolPush();
        // [[Factory alloc] init]
        let factory = msg_id(msg_id(factory_class, sel("alloc")), sel("init"));
        if factory.is_null() {
            objc_autoreleasePoolPop(pool);
            return Err(AuHostingError::new("gui_factory_init_failed"));
        }
        // [factory uiViewForAudioUnit:unit withSize:preferred] — the view
        // arrives autoreleased; retain before the pool pops.
        let send_ui_view: MsgSendUiView =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let preferred = NSSize {
            width: 0.0,
            height: 0.0,
        };
        let view = send_ui_view(
            factory,
            sel("uiViewForAudioUnit:withSize:"),
            unit,
            preferred,
        );
        if !view.is_null() {
            msg_void(view, sel("retain"));
        }
        msg_void(factory, sel("release"));
        objc_autoreleasePoolPop(pool);
        let view = if view.is_null() {
            return Err(AuHostingError::new("gui_view_create_failed"));
        } else {
            view
        };

        // Factories are free to return a view with a non-zero frame origin.
        // That origin belongs to the factory's own test window, not this
        // host's content view; preserving it can push the editor underneath
        // the title bar or clip one edge. Anchor every embedded editor to the
        // host content origin while retaining the factory-provided size.
        msg_void_point(view, sel("setFrameOrigin:"), NSPoint { x: 0.0, y: 0.0 });

        // The plugin owns changes to its outer frame. Do not let AppKit grow
        // that frame merely because Soundcheck resizes the bootstrap parent
        // around it: the AU host polls the live frame as the plugin's desired
        // size, so a width/height-sizable mask creates an unbounded feedback
        // loop. This does not affect autoresizing inside the plugin view.
        msg_void_usize(view, sel("setAutoresizingMask:"), 0);

        // [parent addSubview:view]
        msg_void_id(parent, sel("addSubview:"), view);

        Ok(AuGuiSession { view })
    }

    impl Drop for AuGuiSession {
        fn drop(&mut self) {
            // Owner-driven destroy and instance teardown both land here;
            // the session existing implies the view is still attached.
            // MAIN THREAD (documented contract).
            unsafe {
                msg_void(self.view, sel("removeFromSuperview"));
                msg_void(self.view, sel("release"));
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) use macos::{cocoa_view_info, open_embedded};

// Off macOS the session can never be constructed (the hosting load path
// fails with `unsupported_platform` first); the type still exists so the
// hosting surface stays cfg-free.

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use crate::{AuHostedInstance, AU_REGISTRY_COMPONENT_PATH};
    use std::path::Path;

    /// The headless-safe half of AU editor hosting: the
    /// `kAudioUnitProperty_CocoaUI` probe (plain property read — no AppKit,
    /// no display). Stock Apple units answer differently across macOS
    /// releases, so both outcomes are legal; what the test pins is that the
    /// probe parses the variable-length property without corrupting memory
    /// and that the availability seam matches it. View INSTANTIATION
    /// (bundle load + factory + NSView) is operator-owed by definition.
    #[test]
    fn cocoa_ui_probe_parses_or_reports_no_editor() {
        let instance =
            AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), "aufx:dely:appl")
                .expect("stock AUDelay should load");
        match instance.cocoa_view_info() {
            Some(info) => {
                assert!(!info.bundle_path.is_empty());
                assert!(!info.view_class_name.is_empty());
                assert!(
                    Path::new(&info.bundle_path).exists(),
                    "view bundle path should exist: {}",
                    info.bundle_path,
                );
                assert!(instance.gui_supported());
            }
            None => {
                eprintln!(
                    "AUDelay reports no Cocoa UI on this macOS — generic \
                     view is deliberately not built; param editor covers it"
                );
                assert!(!instance.gui_supported());
            }
        }
        // No editor open in either case; unsupported units refuse with the
        // stable token.
        assert!(!instance.gui_is_open());
    }

    /// Units without a Cocoa UI refuse `gui_open_embedded` with the stable
    /// `gui_unsupported` token (never a crash, never a generic view).
    #[test]
    fn units_without_cocoa_ui_refuse_open_with_a_stable_token() {
        let mut instance =
            AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), "aufx:dely:appl")
                .expect("stock AUDelay should load");
        if instance.gui_supported() {
            eprintln!("AUDelay provides a Cocoa UI here; refusal path covered by fourcc units");
            return;
        }
        let mut fake_parent = 0u8;
        // SAFETY: the unit has no Cocoa UI, so this is refused before the
        // handle reaches any FFI call; `fake_parent` is a live local regardless.
        let refused =
            unsafe { instance.gui_open_embedded(&mut fake_parent as *mut u8 as *mut _, None) };
        assert_eq!(refused.unwrap_err().token, "gui_unsupported");
    }
}
