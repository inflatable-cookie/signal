use super::types::encode_parameter_inventory;
use super::*;
use std::io::Cursor;

fn serve_lines(commands: &str) -> Vec<String> {
    let mut broker = SandboxBrokerProcess::default();
    let mut output = Vec::new();
    broker
        .serve(Cursor::new(commands.to_string()), &mut output)
        .expect("broker serve should succeed");
    String::from_utf8(output)
        .expect("broker output should be utf-8")
        .lines()
        .map(|line| line.to_string())
        .collect()
}

#[test]
fn broker_reports_startup_and_shutdown() {
    let lines = serve_lines("shutdown\n");
    assert!(lines[0].contains("state=starting"));
    assert!(lines[1].contains("state=ready"));
    assert!(lines[2].contains("state=shutdown"));
    assert_eq!(lines.len(), 3);
}

#[test]
fn broker_attach_run_teardown_roundtrips_shared_memory() {
    let lines = serve_lines("attach\nrun\nteardown\nshutdown\n");
    assert!(lines[2].contains("state=attached"));
    assert!(lines[2].contains("lease_id=lease:plugin-sandbox-broker"));
    assert!(lines[2].contains("shm_bytes=65536"));
    let running = lines
        .iter()
        .filter(|line| line.contains("state=running"))
        .count();
    assert_eq!(running, 8);
    assert!(lines
        .iter()
        .any(|line| line.contains("execution_complete|processed_blocks=8")));
    assert!(lines
        .iter()
        .any(|line| line.contains("state=teardown_complete") && line.contains("lease_cleanup_ok")));
}

#[test]
fn broker_timeout_path_reports_recoverable_interrupt_and_reattaches() {
    let lines = serve_lines("attach\nrun-timeout\nteardown\nshutdown\n");
    assert!(lines
        .iter()
        .any(|line| line.contains("state=timed_out") && line.contains("timeout=recoverable")));
    assert!(lines
        .iter()
        .any(|line| line.contains("reattached_after_timeout")));
}

#[test]
fn broker_rejects_run_without_attach_and_unknown_commands() {
    let lines = serve_lines("run\nbogus\nshutdown\n");
    assert!(lines
        .iter()
        .any(|line| line.contains("state=crashed") && line.contains("missing_attached_session")));
    assert!(lines
        .iter()
        .any(|line| line.contains("unknown_command:bogus")));
}

#[test]
fn broker_rejects_plugin_commands_without_a_loaded_plugin() {
    let lines = serve_lines(
        "activate 48000 1 256\nstart-processing\nstop-processing\ndeactivate\nunload-plugin\nshutdown\n",
    );
    let crashed = lines
        .iter()
        .filter(|line| line.contains("state=crashed") && line.contains("missing_loaded_plugin"))
        .count();
    assert_eq!(crashed, 5);
}

#[test]
fn broker_rejects_malformed_plugin_commands() {
    let lines = serve_lines("load-plugin /tmp/only-path\nactivate 48000\nshutdown\n");
    assert!(lines
        .iter()
        .any(|line| line.contains("load_plugin_missing_plugin_id")));
    assert!(lines
        .iter()
        .any(|line| line.contains("activate_missing_min_frames")));
}

#[test]
fn broker_rejects_load_of_missing_library_with_typed_detail() {
    let lines = serve_lines("load-plugin /nonexistent/fixture.clap com.signal.missing\nshutdown\n");
    assert!(lines
        .iter()
        .any(|line| line.contains("state=crashed")
            && line.contains("load_plugin:library_open_failed")));
}

#[test]
fn broker_rejects_param_sets_without_a_loaded_plugin_and_malformed_commands() {
    let lines = serve_lines(
        "set-param 4096 0.25\nset-params 4096:0.25;0:1\nset-param 4096\nset-param nope 0.5\nset-params\nset-params 4096-0.25\nshutdown\n",
    );
    let missing = lines
        .iter()
        .filter(|line| line.contains("state=crashed") && line.contains("missing_loaded_plugin"))
        .count();
    assert_eq!(missing, 2, "well-formed sets fail on the missing plugin");
    assert!(lines
        .iter()
        .any(|line| line.contains("set_param_missing_value")));
    assert!(lines
        .iter()
        .any(|line| line.contains("set_param_missing_parameter_id")));
    assert!(lines
        .iter()
        .any(|line| line.contains("set_params_missing_changes")));
    assert!(lines
        .iter()
        .any(|line| line.contains("set_params_malformed_entry")));
}

#[test]
fn broker_rejects_editor_commands_without_plugin_or_gui_and_malformed_commands() {
    // No plugin loaded: open-editor fails on the missing plugin before
    // the gui check; close-editor is plugin-independent and fails on
    // the missing gui service (unit serves are GUI-less). Missing
    // instance tokens are typed parse failures.
    let lines = serve_lines(
        "open-editor instance:sandbox:a\nclose-editor instance:sandbox:a\nopen-editor\nclose-editor\nshutdown\n",
    );
    assert!(lines
        .iter()
        .any(|line| line.contains("state=crashed") && line.contains("missing_loaded_plugin")));
    assert!(lines.iter().any(
        |line| line.contains("state=crashed") && line.contains("close_editor:gui_unavailable")
    ));
    assert!(lines
        .iter()
        .any(|line| line.contains("open_editor_missing_instance")));
    assert!(lines
        .iter()
        .any(|line| line.contains("close_editor_missing_instance")));
}

#[test]
fn wire_token_encoding_escapes_separators() {
    assert_eq!(encode_wire_token("Gain"), "Gain");
    assert_eq!(encode_wire_token("Dry / Wet Mix"), "Dry%20/%20Wet%20Mix");
    assert_eq!(encode_wire_token("a:b;c=d|e%f"), "a%3Ab%3Bc%3Dd%7Ce%25f");
}

#[test]
fn parameter_inventory_encodes_descriptor_tokens() {
    use signal_plugin::{PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags};

    let parameters = vec![
        PluginParameterDescriptor {
            parameter_id: 4096,
            name: "Gain".into(),
            unit: Some("dB".into()),
            domain: PluginParameterDomain::GenericNormalized,
            default_normalized: 0.5,
            min_plain: 0.0,
            max_plain: 1.0,
            step_count: None,
            flags: PluginParameterFlags::automatable(),
        },
        PluginParameterDescriptor {
            parameter_id: 0,
            name: "Bypass".into(),
            unit: None,
            domain: PluginParameterDomain::Bypass,
            default_normalized: 0.0,
            min_plain: 0.0,
            max_plain: 1.0,
            step_count: Some(1),
            flags: PluginParameterFlags::bypass(),
        },
    ];
    assert_eq!(
        encode_parameter_inventory(&parameters),
        "4096:Gain:0:1:0.5:dB::a;0:Bypass:0:1:0::1:ab",
        "v1 prefix unchanged; unit/steps/flags ride as trailing tokens",
    );
}
