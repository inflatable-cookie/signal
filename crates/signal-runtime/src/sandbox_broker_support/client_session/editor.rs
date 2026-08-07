use super::super::types::*;

impl SandboxBrokerClientSession {
    /// Sends `open-editor <instance>` (g13.027): the child opens a
    /// child-owned floating editor window titled by `instance` on its main
    /// thread, hosting the plugin's editor via the format's gui adapter.
    /// The RT audio path is untouched. `instance` is an opaque parent
    /// token; the v1 wire format forbids whitespace in it.
    pub fn open_editor(&mut self, instance: &str) -> std::io::Result<SandboxEditorOpened> {
        if instance.is_empty() || instance.chars().any(char::is_whitespace) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "editor instance tokens must be non-empty and whitespace-free on the v1 wire",
            ));
        }
        self.write_command(&format!("open-editor {instance}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::EditorOpened {
            return Err(std::io::Error::other(format!(
                "unexpected broker open-editor state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        let width = receipt
            .extra_value("width")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let height = receipt
            .extra_value("height")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        Ok(SandboxEditorOpened {
            width,
            height,
            detail: receipt.detail,
        })
    }

    /// Sends `close-editor <instance>` (g13.027): the child destroys the
    /// editor window. Tolerant of an already-closed editor — see
    /// [`SandboxEditorClosed::closed`].
    pub fn close_editor(&mut self, instance: &str) -> std::io::Result<SandboxEditorClosed> {
        if instance.is_empty() || instance.chars().any(char::is_whitespace) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "editor instance tokens must be non-empty and whitespace-free on the v1 wire",
            ));
        }
        self.write_command(&format!("close-editor {instance}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::EditorClosed {
            return Err(std::io::Error::other(format!(
                "unexpected broker close-editor state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        Ok(SandboxEditorClosed {
            closed: receipt.extra_value("reason") == Some("host_requested"),
            detail: receipt.detail,
        })
    }

    /// Drain the editor instances the child reported closed on its own
    /// (the user clicked the window's close button — `reason=user_closed`
    /// notifications, g13.027). Polls pending receipt lines without
    /// blocking; lines that are not user-close notifications are kept for
    /// the next command read.
    pub fn take_editor_closed_notifications(&mut self) -> Vec<String> {
        while let Ok(Ok(line)) = self.receipts.try_recv() {
            match parse_broker_receipt_line(&line) {
                Ok(receipt) => match user_closed_editor_instance(&receipt) {
                    Some(instance) => self.editor_closed_notifications.push_back(instance),
                    None => self.pushback.push_back(line),
                },
                Err(_) => self.pushback.push_back(line),
            }
        }
        self.editor_closed_notifications.drain(..).collect()
    }
}
