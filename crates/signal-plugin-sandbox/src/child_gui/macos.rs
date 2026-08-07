#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_void, CString};

#[repr(C)]
#[derive(Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
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
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

// AppKit must be linked for NSApplication/NSWindow to exist in-process.
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

type MsgSendId = unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void;
type MsgSendVoid = unsafe extern "C" fn(*mut c_void, *mut c_void);
type MsgSendVoidId = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
type MsgSendBool = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i8;
type MsgSendBoolIsize = unsafe extern "C" fn(*mut c_void, *mut c_void, isize) -> i8;
type MsgSendVoidBool = unsafe extern "C" fn(*mut c_void, *mut c_void, i8);
type MsgSendVoidIsize = unsafe extern "C" fn(*mut c_void, *mut c_void, isize);
type MsgSendVoidSize = unsafe extern "C" fn(*mut c_void, *mut c_void, NSSize);
type MsgSendIdCStr = unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> *mut c_void;
type MsgSendWindowInit =
    unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect, usize, usize, i8) -> *mut c_void;
type MsgSendNextEvent = unsafe extern "C" fn(
    *mut c_void,
    *mut c_void,
    u64,
    *mut c_void,
    *mut c_void,
    i8,
) -> *mut c_void;

/// `NSApplicationActivationPolicyAccessory`: no Dock icon, windows
/// allowed — the helper-process posture.
const ACTIVATION_POLICY_ACCESSORY: isize = 1;
/// Titled | closable | miniaturizable | resizable.
const STYLE_MASK: usize = 1 | 2 | 4 | 8;
/// `NSBackingStoreBuffered`.
const BACKING_STORE_BUFFERED: usize = 2;
/// `NSFloatingWindowLevel` — the packet's floating-window posture.
const FLOATING_WINDOW_LEVEL: isize = 3;

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

unsafe fn nsstring(value: &str) -> *mut c_void {
    let Ok(value) = CString::new(value) else {
        return std::ptr::null_mut();
    };
    let send: MsgSendIdCStr = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
    send(
        objc_getClass(c"NSString".as_ptr()),
        sel("stringWithUTF8String:"),
        value.as_ptr(),
    )
}

unsafe fn shared_application() -> *mut c_void {
    msg_id(
        objc_getClass(c"NSApplication".as_ptr()),
        sel("sharedApplication"),
    )
}

/// Initialize AppKit for the child: shared application, accessory
/// activation policy (no Dock icon), finish launching. Called lazily
/// on the first editor open — a child that never opens an editor never
/// touches the window server.
pub(super) fn init_app() -> Result<(), String> {
    unsafe {
        let app = shared_application();
        if app.is_null() {
            return Err("gui_appkit_unavailable".to_string());
        }
        let set_policy: MsgSendBoolIsize =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let _ = set_policy(
            app,
            sel("setActivationPolicy:"),
            ACTIVATION_POLICY_ACCESSORY,
        );
        msg_void(app, sel("finishLaunching"));
    }
    Ok(())
}

/// Create the child-owned floating editor window titled by `instance`
/// (hidden until [`show_window`]). Owned by the caller (`releasedWhenClosed`
/// disabled; balanced by [`close_window`]).
pub(super) fn create_editor_window(instance: &str) -> Result<*mut c_void, String> {
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let window_class = objc_getClass(c"NSWindow".as_ptr());
        let allocated = msg_id(window_class, sel("alloc"));
        if allocated.is_null() {
            objc_autoreleasePoolPop(pool);
            return Err("gui_window_alloc_failed".to_string());
        }
        let init: MsgSendWindowInit = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let frame = NSRect {
            x: 120.0,
            y: 120.0,
            width: 400.0,
            height: 300.0,
        };
        let window = init(
            allocated,
            sel("initWithContentRect:styleMask:backing:defer:"),
            frame,
            STYLE_MASK,
            BACKING_STORE_BUFFERED,
            0,
        );
        if window.is_null() {
            objc_autoreleasePoolPop(pool);
            return Err("gui_window_init_failed".to_string());
        }
        let set_released: MsgSendVoidBool =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        set_released(window, sel("setReleasedWhenClosed:"), 0);
        let set_level: MsgSendVoidIsize =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        set_level(window, sel("setLevel:"), FLOATING_WINDOW_LEVEL);
        let title = nsstring(instance);
        if !title.is_null() {
            msg_void_id(window, sel("setTitle:"), title);
        }
        objc_autoreleasePoolPop(pool);
        Ok(window)
    }
}

/// The window's content view (the `NSView*` parent handed to the
/// per-format gui adapter).
pub(super) fn content_view(window: *mut c_void) -> *mut c_void {
    unsafe { msg_id(window, sel("contentView")) }
}

/// Resize the window content to the plugin's reported editor size.
pub(super) fn set_content_size(window: *mut c_void, width: u32, height: u32) {
    unsafe {
        let set_size: MsgSendVoidSize = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        set_size(
            window,
            sel("setContentSize:"),
            NSSize {
                width: f64::from(width.max(1)),
                height: f64::from(height.max(1)),
            },
        );
    }
}

/// Center and order the window front.
pub(super) fn show_window(window: *mut c_void) {
    unsafe {
        msg_void(window, sel("center"));
        msg_void_id(window, sel("makeKeyAndOrderFront:"), std::ptr::null_mut());
    }
}

/// Whether the window is still on screen (`false` after the user
/// clicks close — the user-close poll signal).
pub(super) fn window_is_visible(window: *mut c_void) -> bool {
    unsafe {
        let is_visible: MsgSendBool = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        is_visible(window, sel("isVisible")) != 0
    }
}

/// Close and release a window created by [`create_editor_window`].
pub(super) fn close_window(window: *mut c_void) {
    unsafe {
        msg_void(window, sel("close"));
        msg_void(window, sel("release"));
    }
}

/// Drain and dispatch every pending AppKit event (non-blocking), then
/// refresh windows. One autorelease pool per pump.
pub(super) fn pump_events() {
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let app = shared_application();
        if app.is_null() {
            objc_autoreleasePoolPop(pool);
            return;
        }
        let distant_past = msg_id(objc_getClass(c"NSDate".as_ptr()), sel("distantPast"));
        let mode = nsstring("kCFRunLoopDefaultMode");
        let next_event: MsgSendNextEvent =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        loop {
            let event = next_event(
                app,
                sel("nextEventMatchingMask:untilDate:inMode:dequeue:"),
                u64::MAX,
                distant_past,
                mode,
                1,
            );
            if event.is_null() {
                break;
            }
            msg_void_id(app, sel("sendEvent:"), event);
        }
        msg_void(app, sel("updateWindows"));
        objc_autoreleasePoolPop(pool);
    }
}
