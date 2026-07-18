//! Child-owned plugin editor windows for the sandboxed tier (g13.027
//! Batch 1).
//!
//! macOS requires AppKit on the process main thread, so the sandbox child
//! runs a GUI SERVICE LOOP on its main thread while the stdio protocol
//! (`broker::SandboxBrokerProcess::serve`) moves to a dedicated control
//! thread (`main.rs`). The RT audio thread is untouched: it still
//! spin/yield-waits on the shared-memory request stamp and never touches
//! AppKit — the isolation invariant of the packet.
//!
//! The control thread marshals editor lifecycle calls onto the main thread
//! through [`ChildGuiHandle`] (blocking request/reply channels — the
//! control thread waits, so instance access never overlaps). AppKit is
//! initialized LAZILY on the first `open-editor`: a child that never opens
//! an editor behaves exactly as before (no window-server connection).
//!
//! Editor windows are CHILD-OWNED floating `NSWindow`s titled by instance
//! (no cross-process view parenting — the packet's authority decision).
//! The user closing a window emits a spontaneous `editor_closed` receipt
//! line with `reason=user_closed` through the shared writer; child death
//! implies every window dies with the process.
//!
//! The Objective-C surface is the house-style handwritten FFI (typed
//! `objc_msgSend` casts, no objc crate — see `signal-plugin-au`'s gui).

use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use signal_plugin_clap::ClapGuiRawParts;

use crate::broker::{encode_wire_token, SandboxBrokerReceipt, SandboxBrokerState};

/// How long the control thread waits for the main thread to answer an
/// editor request before failing the command with a typed token.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// Service tick while editors are open (event pump + user-close poll).
const ACTIVE_TICK: Duration = Duration::from_millis(10);

/// Service tick while no editor is open (nothing to pump).
const IDLE_TICK: Duration = Duration::from_millis(250);

/// Format-selected editor open spec, extracted from the loaded instance on
/// the control thread and consumed on the main thread. CLAP is the
/// first-class child format (g13.027 Batch 1); VST3/AU child editors are
/// recorded follow-up state and never construct a spec.
pub enum ChildEditorSpec {
    /// CLAP `clap.gui` raw parts ([`ClapGuiRawParts`]).
    Clap(ClapGuiRawParts),
}

// Safety: the spec carries raw plugin pointers across the control→main
// channel exactly once, while the control thread blocks on the reply (no
// concurrent use), and the broker closes every editor before the instance
// that produced the parts is destroyed.
unsafe impl Send for ChildEditorSpec {}

/// One editor lifecycle request marshaled from the control thread.
pub enum GuiRequest {
    OpenEditor {
        instance: String,
        spec: ChildEditorSpec,
        reply: Sender<Result<(u32, u32), String>>,
    },
    CloseEditor {
        instance: String,
        reply: Sender<Result<bool, String>>,
    },
    CloseAll {
        reply: Sender<()>,
    },
}

/// Handle the broker's control thread uses to marshal editor lifecycle
/// onto the child's main thread. Every call blocks until the main thread
/// answers (bounded by [`REPLY_TIMEOUT`]); errors are stable tokens.
pub struct ChildGuiHandle {
    requests: Sender<GuiRequest>,
}

impl ChildGuiHandle {
    /// Open the editor window for `instance`. Returns the plugin's initial
    /// content size.
    pub fn open_editor(&self, instance: &str, spec: ChildEditorSpec) -> Result<(u32, u32), String> {
        let (reply, answer) = mpsc::channel();
        self.requests
            .send(GuiRequest::OpenEditor {
                instance: instance.to_string(),
                spec,
                reply,
            })
            .map_err(|_| "gui_service_gone".to_string())?;
        answer
            .recv_timeout(REPLY_TIMEOUT)
            .map_err(|_| "gui_service_timeout".to_string())?
    }

    /// Close the editor window for `instance`. `Ok(false)` when no editor
    /// with that instance is open (already user-closed, or never opened).
    pub fn close_editor(&self, instance: &str) -> Result<bool, String> {
        let (reply, answer) = mpsc::channel();
        self.requests
            .send(GuiRequest::CloseEditor {
                instance: instance.to_string(),
                reply,
            })
            .map_err(|_| "gui_service_gone".to_string())?;
        answer
            .recv_timeout(REPLY_TIMEOUT)
            .map_err(|_| "gui_service_timeout".to_string())?
    }

    /// Close every open editor (plugin unload / teardown ordering: editors
    /// must die before the instance their sessions point into). Best
    /// effort — a missing service just means no editors exist.
    pub fn close_all(&self) {
        let (reply, answer) = mpsc::channel();
        if self.requests.send(GuiRequest::CloseAll { reply }).is_ok() {
            let _ = answer.recv_timeout(REPLY_TIMEOUT);
        }
    }
}

/// Create the control↔main channel pair: the [`ChildGuiHandle`] goes to
/// the broker (control thread), the receiver to [`run_gui_service`] on the
/// main thread.
pub fn channel() -> (ChildGuiHandle, Receiver<GuiRequest>) {
    let (requests, service) = mpsc::channel();
    (ChildGuiHandle { requests }, service)
}

/// Line-atomic writer shared between the control thread (command receipts)
/// and the main thread (spontaneous `editor_closed` notifications).
#[derive(Clone)]
pub struct SharedLineWriter {
    inner: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl SharedLineWriter {
    pub fn new(writer: Box<dyn Write + Send>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(writer)),
        }
    }
}

impl Write for SharedLineWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = self.inner.lock().expect("shared writer poisoned");
        guard.write(buf)
    }

    // One lock per formatted write keeps receipt lines atomic across the
    // control and main threads (the default write_fmt issues one `write`
    // per fragment).
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        let mut guard = self.inner.lock().expect("shared writer poisoned");
        guard.write_fmt(args)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut guard = self.inner.lock().expect("shared writer poisoned");
        guard.flush()
    }
}

/// Render and write the spontaneous user-close notification line.
fn write_user_closed_notification(writer: &mut SharedLineWriter, sandbox_id: &str, instance: &str) {
    let receipt = SandboxBrokerReceipt {
        state: SandboxBrokerState::EditorClosed,
        sandbox_id: sandbox_id.to_string(),
        instance_id: None,
        processing_epoch: None,
        lease_id: None,
        region_id: None,
        extra: vec![
            ("editor_instance".into(), encode_wire_token(instance)),
            ("reason".into(), "user_closed".into()),
        ],
        detail: "editor_closed|reason=user_closed".into(),
    };
    let _ = writeln!(writer, "{}", receipt.render_line());
    let _ = writer.flush();
}

/// The main-thread GUI service loop: pumps AppKit events while editors are
/// open, serves marshaled editor requests, and reports user closes. Runs
/// until every [`ChildGuiHandle`] sender is dropped (the control thread
/// exiting after `serve` returns).
#[cfg(target_os = "macos")]
pub fn run_gui_service(
    requests: Receiver<GuiRequest>,
    mut writer: SharedLineWriter,
    sandbox_id: &str,
) {
    let mut editors: Vec<OpenEditor> = Vec::new();
    let mut app_ready = false;
    loop {
        let tick = if editors.is_empty() {
            IDLE_TICK
        } else {
            ACTIVE_TICK
        };
        match requests.recv_timeout(tick) {
            Ok(GuiRequest::OpenEditor {
                instance,
                spec,
                reply,
            }) => {
                let result = open_editor(&instance, spec, &mut editors, &mut app_ready);
                let _ = reply.send(result);
            }
            Ok(GuiRequest::CloseEditor { instance, reply }) => {
                let closed = match editors
                    .iter()
                    .position(|editor| editor.instance == instance)
                {
                    Some(index) => {
                        editors.remove(index).close();
                        true
                    }
                    None => false,
                };
                let _ = reply.send(Ok(closed));
            }
            Ok(GuiRequest::CloseAll { reply }) => {
                for editor in editors.drain(..) {
                    editor.close();
                }
                let _ = reply.send(());
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if app_ready {
            macos::pump_events();
            // User-close poll: a shown window that is no longer visible was
            // closed by the user — destroy the session, notify the parent.
            let mut index = 0;
            while index < editors.len() {
                if macos::window_is_visible(editors[index].window) {
                    index += 1;
                    continue;
                }
                let editor = editors.remove(index);
                let instance = editor.instance.clone();
                editor.close();
                write_user_closed_notification(&mut writer, sandbox_id, &instance);
            }
        }
    }
    // The control thread is gone; the plugin instance may already be
    // destroyed, so session Drop glue must not run — the process is
    // exiting and the OS reclaims the windows.
    for editor in editors.drain(..) {
        std::mem::forget(editor.session);
    }
}

/// Non-macOS: no child window system — answer every request with the
/// stable platform token so the wire behavior stays typed.
#[cfg(not(target_os = "macos"))]
pub fn run_gui_service(
    requests: Receiver<GuiRequest>,
    _writer: SharedLineWriter,
    _sandbox_id: &str,
) {
    while let Ok(request) = requests.recv() {
        match request {
            GuiRequest::OpenEditor { reply, .. } => {
                let _ = reply.send(Err("gui_platform_unsupported".to_string()));
            }
            GuiRequest::CloseEditor { reply, .. } => {
                let _ = reply.send(Ok(false));
            }
            GuiRequest::CloseAll { reply } => {
                let _ = reply.send(());
            }
        }
    }
}

/// One open format-selected editor session, owned by the main thread.
#[cfg(target_os = "macos")]
enum EditorSession {
    Clap(signal_plugin_clap::ClapGuiSession),
}

/// One open child-owned editor window.
#[cfg(target_os = "macos")]
struct OpenEditor {
    instance: String,
    window: *mut std::ffi::c_void,
    session: EditorSession,
}

#[cfg(target_os = "macos")]
impl OpenEditor {
    /// Destroy the plugin gui session first (CLAP ordering), then close
    /// and release the window.
    fn close(self) {
        drop(self.session);
        macos::close_window(self.window);
    }
}

#[cfg(target_os = "macos")]
fn open_editor(
    instance: &str,
    spec: ChildEditorSpec,
    editors: &mut Vec<OpenEditor>,
    app_ready: &mut bool,
) -> Result<(u32, u32), String> {
    if editors.iter().any(|editor| editor.instance == instance) {
        return Err("editor_already_open".to_string());
    }
    if !*app_ready {
        macos::init_app()?;
        *app_ready = true;
    }
    let window = macos::create_editor_window(instance)?;
    let parent = macos::content_view(window);
    if parent.is_null() {
        macos::close_window(window);
        return Err("gui_window_content_view_null".to_string());
    }
    let session = match spec {
        ChildEditorSpec::Clap(parts) => {
            // Safety: the broker only hands out parts for the live loaded
            // instance, closes editors before unload, and this call runs
            // on the main thread (the service loop's thread).
            match unsafe { parts.open_embedded(parent, None) } {
                Ok(session) => EditorSession::Clap(session),
                Err(error) => {
                    macos::close_window(window);
                    return Err(error.token);
                }
            }
        }
    };
    let (width, height) = match &session {
        EditorSession::Clap(clap) => clap.size(),
    };
    macos::set_content_size(window, width, height);
    macos::show_window(window);
    editors.push(OpenEditor {
        instance: instance.to_string(),
        window,
        session,
    });
    Ok((width, height))
}

/// Handwritten AppKit FFI (typed `objc_msgSend` casts — the house idiom;
/// see `signal-plugin-au`'s gui module).
#[cfg(target_os = "macos")]
mod macos {
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
    type MsgSendIdCStr =
        unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> *mut c_void;
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
            let init: MsgSendWindowInit =
                std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
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
            let set_size: MsgSendVoidSize =
                std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
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
            let is_visible: MsgSendBool =
                std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
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
}
