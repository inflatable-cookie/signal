//! Sandbox broker support unit tests.

use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::RuntimeErrorKind;

use super::types::{
    parse_broker_receipt_line, parse_parameter_inventory, split_broker_args,
    user_closed_editor_instance, SandboxBrokerClientSession, SandboxBrokerReceiptState,
    SandboxBrokerSpawnConfig,
};

#[test]
fn parses_legacy_five_field_parameter_inventory() {
    // Pre-g12.013 receipts carry no descriptor tokens: descriptor
    // fields fall back to None/automatable/not-bypass.
    let parameters = parse_parameter_inventory("4096:Gain:0:1:0.5;0:Bypass:0:1:0");
    assert_eq!(parameters.len(), 2);
    let gain = &parameters[0];
    assert_eq!(gain.parameter_id, 4096);
    assert_eq!(gain.name, "Gain");
    assert_eq!(gain.unit, None);
    assert_eq!(gain.step_count, None);
    assert!(gain.is_automatable, "legacy default: automatable");
    assert!(!gain.is_bypass, "legacy default: not bypass");
}

#[test]
fn parses_enriched_parameter_inventory_tokens() {
    let parameters = parse_parameter_inventory(
        "7:Dry%20/%20Wet:0:100:0.5:%25:::;0:Bypass:0:1:0::1:ab;9:Cutoff:20:20000:0.3:Hz::a",
    );
    assert_eq!(parameters.len(), 3);

    let mix = &parameters[0];
    assert_eq!(mix.name, "Dry / Wet");
    assert_eq!(mix.unit.as_deref(), Some("%"), "wire-encoded unit decodes");
    assert_eq!(mix.step_count, None, "empty steps token = continuous");
    assert!(
        !mix.is_automatable,
        "an explicit empty flags token means no flags",
    );

    let bypass = &parameters[1];
    assert_eq!(bypass.unit, None, "empty unit token = None");
    assert_eq!(bypass.step_count, Some(1));
    assert!(bypass.is_automatable);
    assert!(bypass.is_bypass);

    let cutoff = &parameters[2];
    assert_eq!(cutoff.unit.as_deref(), Some("Hz"));
    assert_eq!(cutoff.step_count, None);
    assert!(cutoff.is_automatable);
    assert!(!cutoff.is_bypass);
}

#[test]
fn parameter_inventory_ignores_unknown_trailing_tokens() {
    // Forward tolerance: a future encoder may append more tokens.
    let parameters = parse_parameter_inventory("1:Mode:0:2:0::2:a:future:tokens");
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].step_count, Some(2));
    assert!(parameters[0].is_automatable);
}

#[test]
fn splits_plain_whitespace_args() {
    assert_eq!(
        split_broker_args("run -q -p signal-plugin-sandbox"),
        vec!["run", "-q", "-p", "signal-plugin-sandbox"]
    );
    assert!(split_broker_args("   ").is_empty());
    assert!(split_broker_args("").is_empty());
}

#[test]
fn splits_quoted_paths_with_spaces() {
    assert_eq!(
        split_broker_args("--root \"/Library/Audio/Plug-Ins/My Plugins\" -v"),
        vec!["--root", "/Library/Audio/Plug-Ins/My Plugins", "-v"]
    );
    assert_eq!(
        split_broker_args("--name 'demo plugin'"),
        vec!["--name", "demo plugin"]
    );
    assert_eq!(
        split_broker_args(r"--path /tmp/with\ space"),
        vec!["--path", "/tmp/with space"]
    );
    assert_eq!(split_broker_args("''"), vec![""]);
    assert_eq!(
        split_broker_args(r#"--mix pre"fix mid"post"#),
        vec!["--mix", "prefix midpost"]
    );
}

#[test]
fn classifies_editor_receipts_and_user_close_notifications() {
    // Command receipt: editor_opened with size extras.
    let opened = parse_broker_receipt_line(
        "signal-plugin-sandbox state=editor_opened sandbox_id=plugin-sandbox-broker instance_id=- epoch=- lease_id=- region_id=- editor_instance=inst%3A1 width=400 height=300 detail=editor_opened|width=400|height=300\n",
    )
    .expect("editor_opened receipt should parse");
    assert_eq!(opened.state, SandboxBrokerReceiptState::EditorOpened);
    assert_eq!(opened.extra_value("width"), Some("400"));
    assert_eq!(user_closed_editor_instance(&opened), None);

    // Command receipt: host-requested close is NOT a notification.
    let closed = parse_broker_receipt_line(
        "signal-plugin-sandbox state=editor_closed sandbox_id=plugin-sandbox-broker instance_id=- epoch=- lease_id=- region_id=- editor_instance=inst%3A1 reason=host_requested detail=editor_closed|reason=host_requested\n",
    )
    .expect("editor_closed receipt should parse");
    assert_eq!(closed.state, SandboxBrokerReceiptState::EditorClosed);
    assert_eq!(user_closed_editor_instance(&closed), None);

    // Spontaneous notification: user_closed decodes the wire-encoded
    // instance token and never satisfies a command wait.
    let notification = parse_broker_receipt_line(
        "signal-plugin-sandbox state=editor_closed sandbox_id=plugin-sandbox-broker instance_id=- epoch=- lease_id=- region_id=- editor_instance=inst%3A1 reason=user_closed detail=editor_closed|reason=user_closed\n",
    )
    .expect("notification line should parse");
    assert_eq!(
        user_closed_editor_instance(&notification).as_deref(),
        Some("inst:1"),
    );
}

#[test]
fn parses_broker_receipt_lines() {
    let receipt = parse_broker_receipt_line(
        "signal-plugin-sandbox state=attached sandbox_id=plugin-sandbox-broker instance_id=instance:sandbox:default epoch=1 lease_id=lease:plugin-sandbox-broker region_id=region:plugin-sandbox-broker detail=lease_attached\n",
    )
    .expect("receipt should parse");

    assert_eq!(receipt.state, SandboxBrokerReceiptState::Attached);
    assert_eq!(receipt.sandbox_id, "plugin-sandbox-broker");
    assert_eq!(
        receipt.instance_id.as_deref(),
        Some("instance:sandbox:default")
    );
    assert_eq!(receipt.processing_epoch, Some(1));
    assert_eq!(
        receipt.lease_id.as_deref(),
        Some("lease:plugin-sandbox-broker")
    );
    assert_eq!(
        receipt.region_id.as_deref(),
        Some("region:plugin-sandbox-broker")
    );
    assert_eq!(receipt.detail, "lease_attached");
}

/// Spawns a long-lived stand-in child (`cat` waits on the piped stdin) and
#[test]
fn spawn_from_env_reports_actionable_missing_command() {
    let _guard = broker_command_env_lock();
    let previous = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND");
    unsafe {
        std::env::remove_var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND");
    }

    let error = SandboxBrokerClientSession::spawn_from_env(&SandboxBrokerSpawnConfig::default())
        .expect_err("spawn_from_env must fail without a prebuilt broker command");

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(
        error
            .message
            .contains("missing SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND"),
        "message should name the required env var, got {}",
        error.message
    );
    assert!(
        error.message.contains("prebuilt"),
        "message should require a prebuilt executable, got {}",
        error.message
    );
    assert!(
        error.message.contains("consuming-signal.md")
            && error.message.contains("broker:provision"),
        "message should point at the runbook and provisioner, got {}",
        error.message
    );

    unsafe {
        match previous {
            Some(value) => std::env::set_var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND", value),
            None => std::env::remove_var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND"),
        }
    }
}

fn broker_command_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// proves [`SandboxBrokerClientSession::child_pid`] is a direct read of the
/// owned [`std::process::Child`] identity — no `ps` or platform probe.
#[cfg(unix)]
#[test]
fn child_pid_reports_owned_child_id() {
    let mut session = SandboxBrokerClientSession::spawn_command(
        "cat",
        &[],
        &SandboxBrokerSpawnConfig {
            read_timeout_ms: Some(1_000),
            ..SandboxBrokerSpawnConfig::default()
        },
    )
    .expect("spawn stand-in broker child");
    assert_eq!(session.child_pid(), session.child.id());
    assert!(session.child_pid() > 0);
    session.kill();
}
