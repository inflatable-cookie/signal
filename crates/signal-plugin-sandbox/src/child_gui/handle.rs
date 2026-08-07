use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use super::types::{ChildEditorSpec, GuiRequest};

/// How long the control thread waits for the main thread to answer an
/// editor request before failing the command with a typed token.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

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
