//! In-process backend unit tests.

use super::prelude::*;

/// g12.024: the VST3 IPlugView mirror of the CLAP gui lifecycle test —
/// the exact surface the Tauri host calls (open/size/resize/events/
/// close), offscreen against the fixture's bookkeeping view, while the
/// audio path keeps processing (gui takes the instance lock, never the
/// session lock).
#[test]
fn in_process_vst3_backend_hosts_the_fixture_view_offscreen() {
    use signal_plugin_vst3::fixture::{
        VST3_FIXTURE_VIEW_INITIAL_SIZE, VST3_FIXTURE_VIEW_REQUESTED_SIZE,
    };

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-vst3-gui-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:bridge-inproc-gui",
        "Signal Bridge InProc VST3 Gui",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(&bundle, VST3_FIXTURE_CLASS_ID_HEX, 48_000, 256)
            .expect("backend should load and activate"),
    );
    assert!(backend.gui_supported(), "edit controller is available");
    assert!(!backend.gui_is_open());
    assert_eq!(backend.gui_size(), None);

    let mut fake_parent = 0u8;
    let size = backend
        .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
        .expect("view opens");
    assert_eq!(size, VST3_FIXTURE_VIEW_INITIAL_SIZE);
    assert!(backend.gui_is_open());
    assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_INITIAL_SIZE));
    assert!(backend.gui_can_resize());

    // Audio still processes with the editor open.
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    assert!(handle.process(&mut scratch, 128, 2));

    // Fixture attached() asked the host IPlugFrame for a resize.
    let events = backend.gui_take_events();
    assert!(events.contains(&PluginGuiEvent::RequestResize {
        width: VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
        height: VST3_FIXTURE_VIEW_REQUESTED_SIZE.1,
    }));

    // Plugin-requested sizes bypass the host/user constraint pass and are
    // granted directly through onSize.
    assert_eq!(
        backend.gui_accept_plugin_resize(
            VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
            VST3_FIXTURE_VIEW_REQUESTED_SIZE.1
        ),
        Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE)
    );
    assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE));

    backend.gui_close();
    assert!(!backend.gui_is_open());
    backend.gui_close(); // idempotent

    // Dead backends refuse to open editors.
    backend.shutdown();
    let refused = backend.gui_open_embedded(&mut fake_parent as *mut u8 as usize, None);
    assert_eq!(refused.unwrap_err(), "backend_dead");

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}
