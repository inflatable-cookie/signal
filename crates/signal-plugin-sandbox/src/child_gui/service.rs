use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use super::types::{write_user_closed_notification, GuiRequest, SharedLineWriter};

/// Service tick while editors are open (event pump + user-close poll).
const ACTIVE_TICK: Duration = Duration::from_millis(10);

/// Service tick while no editor is open (nothing to pump).
const IDLE_TICK: Duration = Duration::from_millis(250);

/// The main-thread GUI service loop: pumps AppKit events while editors are
/// open, serves marshaled editor requests, and reports user closes. Runs
/// until every [`ChildGuiHandle`](super::ChildGuiHandle) sender is dropped (the control thread
/// exiting after `serve` returns).
#[cfg(target_os = "macos")]
pub fn run_gui_service(
    requests: Receiver<GuiRequest>,
    mut writer: SharedLineWriter,
    sandbox_id: &str,
) {
    use super::editor::{open_editor, OpenEditor};
    use super::macos;

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
        editor.abandon_without_teardown();
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
