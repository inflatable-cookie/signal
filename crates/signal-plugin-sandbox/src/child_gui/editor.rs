use super::macos;
use super::types::ChildEditorSpec;

/// One open format-selected editor session, owned by the main thread.
enum EditorSession {
    Clap(signal_plugin_clap::ClapGuiSession),
}

/// One open child-owned editor window.
pub(super) struct OpenEditor {
    pub(super) instance: String,
    pub(super) window: *mut std::ffi::c_void,
    session: EditorSession,
}

impl OpenEditor {
    /// Destroy the plugin gui session first (CLAP ordering), then close
    /// and release the window.
    pub(super) fn close(self) {
        drop(self.session);
        macos::close_window(self.window);
    }

    /// Drop the window without running session teardown (process exit).
    pub(super) fn abandon_without_teardown(self) {
        std::mem::forget(self.session);
    }
}

pub(super) fn open_editor(
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
