//! In-process backend unit tests.

use super::prelude::*;

/// g12.022: gui lifecycle through the in-process backend's delegates —
/// the exact surface the Tauri host calls (open/size/resize/events/
/// close), offscreen against the fixture's bookkeeping gui, while the
/// audio path keeps processing (gui takes the instance lock, never the
/// session lock).
#[test]
fn in_process_backend_hosts_the_fixture_gui_offscreen() {
    use signal_plugin_clap::fixture::{
        CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_REQUESTED_SIZE,
    };

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-gui-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc-gui",
        "Signal Bridge InProc Gui",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc-gui",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    assert!(backend.gui_supported());
    assert!(!backend.gui_is_open());
    assert_eq!(backend.gui_size(), None);
    assert_eq!(backend.state_dirty_request_count(), 0);

    let mut fake_parent = 0u8;
    let size = backend
        .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
        .expect("gui opens");
    assert_eq!(size, CLAP_FIXTURE_GUI_INITIAL_SIZE);
    assert!(backend.gui_is_open());
    assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_INITIAL_SIZE));
    assert!(backend.gui_can_resize());
    assert_eq!(backend.state_dirty_request_count(), 1);

    // Audio still processes with the editor open.
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    assert!(handle.process(&mut scratch, 128, 2));

    // Fixture show() queued a host resize request.
    let events = backend.gui_take_events();
    assert!(events.contains(&PluginGuiEvent::RequestResize {
        width: CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
        height: CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
    }));

    // Granting the request through set_size sticks.
    assert_eq!(
        backend.gui_set_size(
            CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
            CLAP_FIXTURE_GUI_REQUESTED_SIZE.1
        ),
        Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE)
    );
    assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE));

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
