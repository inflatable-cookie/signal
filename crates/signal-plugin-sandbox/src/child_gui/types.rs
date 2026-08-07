use std::io::Write;
use std::sync::{Arc, Mutex};

use signal_plugin_clap::ClapGuiRawParts;

use crate::broker::{encode_wire_token, SandboxBrokerReceipt, SandboxBrokerState};

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
        reply: std::sync::mpsc::Sender<Result<(u32, u32), String>>,
    },
    CloseEditor {
        instance: String,
        reply: std::sync::mpsc::Sender<Result<bool, String>>,
    },
    CloseAll {
        reply: std::sync::mpsc::Sender<()>,
    },
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
pub(super) fn write_user_closed_notification(
    writer: &mut SharedLineWriter,
    sandbox_id: &str,
    instance: &str,
) {
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
