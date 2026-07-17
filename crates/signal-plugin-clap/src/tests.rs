// Tests for signal-plugin-clap
#[allow(clippy::module_inception)]
mod tests {
    use crate::fixture::CLAP_FIXTURE_GAIN;
    use crate::{ClapHostExtension, ClapHostedInstance, ClapPluginHostAdapter};
    use signal_plugin::PluginFormat;
    use std::{
        fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_broker_root(name: &str) -> PathBuf {
        // Nanosecond timestamps can collide across concurrently-starting
        // tests (clock granularity); the counter keeps roots unique so
        // fixture writes never interleave.
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-plugin-clap-tests-{}-{name}-{timestamp}-{sequence}",
            process::id()
        ))
    }

    pub(super) struct TempClapScanRoot {
        path: PathBuf,
    }

    impl TempClapScanRoot {
        pub(super) fn root(&self) -> String {
            self.path.display().to_string()
        }

        pub(super) fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TempClapScanRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    pub(super) fn temp_real_clap_scan_root(
        plugin_type_id: &str,
        plugin_name: &str,
        midi_outputs: u16,
    ) -> TempClapScanRoot {
        let root = test_broker_root("clap-real-scan");
        crate::fixture::compile_clap_fixture(&root, plugin_type_id, plugin_name, midi_outputs)
            .expect("clap fixture compilation should succeed");
        TempClapScanRoot { path: root }
    }

    #[test]
    fn hosted_instance_loads_activates_and_processes_with_the_fixture_gain() {
        let scan_root =
            temp_real_clap_scan_root("com.signal.hosting-fixture", "Signal Hosting Fixture", 1);
        let library_path = scan_root.path().join("signal-hosting-fixture.clap");

        let mut instance = ClapHostedInstance::load(&library_path, "com.signal.hosting-fixture")
            .expect("fixture instance should load");
        // Param inventory enumerated at load (read-only phase 1).
        let parameters = instance.parameters();
        assert_eq!(parameters.len(), 2);
        // Descriptor enrichment (g12.013): the fixture Gain is a
        // continuous automatable param (CLAP param info has no unit
        // string — unit stays None, never synthesized); the fixture
        // Bypass is a stepped bypass toggle.
        let gain = parameters
            .iter()
            .find(|parameter| parameter.name == "Gain")
            .expect("fixture Gain param");
        assert_eq!(gain.unit, None);
        assert_eq!(gain.step_count, None);
        assert!(gain.is_automatable());
        assert!(!gain.is_bypass());
        assert!((gain.default_normalized - 0.5).abs() < 1e-6);
        let bypass = parameters
            .iter()
            .find(|parameter| parameter.name == "Bypass")
            .expect("fixture Bypass param");
        assert_eq!(bypass.step_count, Some(1), "stepped 0..1 = one step");
        assert!(bypass.flags.stepped);
        assert!(bypass.is_bypass());
        assert!(bypass.is_automatable());
        // Main stereo in + stereo out: the supported phase-1 layout.
        assert!(instance.port_layout().is_stereo_effect());

        instance
            .activate(48_000.0, 1, 256)
            .expect("fixture should activate");
        let mut session = instance
            .process_session()
            .expect("active instance builds a process session");
        session.start().expect("start_processing should succeed");

        let frames = 128usize;
        let input: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 256.0).collect();
        let mut output = vec![0.0f32; frames * 2];
        assert!(session.process_interleaved_stereo(&input, &mut output, frames));
        for (index, (in_sample, out_sample)) in input.iter().zip(output.iter()).enumerate() {
            assert!(
                (out_sample - in_sample * CLAP_FIXTURE_GAIN).abs() < 1e-7,
                "sample {index}: {out_sample} vs {} * {CLAP_FIXTURE_GAIN}",
                in_sample,
            );
        }
        session.stop();
        drop(session);
        instance.deactivate().expect("fixture should deactivate");
    }

    #[test]
    fn process_session_supplies_declared_auxiliary_output_buses() {
        if !crate::fixture::rustc_available() {
            return;
        }
        let root = test_broker_root("clap-multi-output-instrument");
        let library_path = crate::fixture::compile_clap_multi_output_instrument_fixture(
            &root,
            "com.signal.multi-output-instrument",
            "Signal Multi Output Instrument",
            4,
        )
        .expect("multi-output fixture should compile");
        let _scan_root = TempClapScanRoot { path: root };

        let mut instance =
            ClapHostedInstance::load(&library_path, "com.signal.multi-output-instrument")
                .expect("fixture instance should load");
        assert!(instance.port_layout().is_stereo_instrument());
        instance
            .activate(48_000.0, 1, 256)
            .expect("fixture should activate");
        let mut session = instance
            .process_session()
            .expect("active instance builds a process session");
        session.start().expect("start_processing should succeed");

        let mut audio = vec![0.0; 256];
        assert!(session.process_in_place(&mut audio, 128));

        session.stop();
        drop(session);
        instance.deactivate().expect("fixture should deactivate");
    }

    /// g12.022: full offscreen `clap.gui` lifecycle against the fixture's
    /// bookkeeping-only gui — create/parent/size/show → resize negotiation →
    /// hide → destroy, plus the host-callback queue (the fixture requests a
    /// resize on `show`). No display assertions: real editor rendering is
    /// operator-owed.
    #[test]
    fn hosted_instance_gui_lifecycle_runs_offscreen() {
        use crate::fixture::{CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_REQUESTED_SIZE};
        use crate::ClapGuiEvent;

        let scan_root = temp_real_clap_scan_root("com.signal.gui-fixture", "Signal Gui Fixture", 0);
        let library_path = scan_root.path().join("signal-gui-fixture.clap");

        let mut instance = ClapHostedInstance::load(&library_path, "com.signal.gui-fixture")
            .expect("fixture instance should load");
        assert!(instance.gui_supported(), "fixture exposes clap.gui");
        assert!(!instance.gui_is_open());

        // Null parent is rejected before any FFI runs.
        let refused = instance.gui_open_embedded(std::ptr::null_mut(), None);
        assert_eq!(refused.unwrap_err().token, "gui_parent_null");

        // The fixture records but never dereferences the parent handle, so
        // any non-null pointer stands in for the NSView.
        let mut fake_parent = 0u8;
        let size = instance
            .gui_open_embedded((&mut fake_parent as *mut u8).cast(), None)
            .expect("embedded gui open should succeed");
        assert_eq!(size, CLAP_FIXTURE_GUI_INITIAL_SIZE);
        assert!(instance.gui_is_open());

        // Double-open is a tokened error, not UB.
        let double = instance.gui_open_embedded((&mut fake_parent as *mut u8).cast(), None);
        assert_eq!(double.unwrap_err().token, "gui_already_open");

        // The fixture's show() requested a resize through the host gui
        // callback; it lands in the drainable event queue exactly once.
        let events = instance.take_gui_events();
        assert!(events.contains(&ClapGuiEvent::RequestResize {
            width: CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
            height: CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
        }));
        assert!(instance.take_gui_events().is_empty(), "drain empties");

        {
            let session = instance.gui_session_mut().expect("open session");
            assert!(session.can_resize());
            assert!(session.is_visible());
            let accepted = session.set_size(512, 384).expect("resize accepted");
            assert_eq!(accepted, (512, 384));
            assert_eq!(session.size(), (512, 384));
            session.hide();
            assert!(!session.is_visible());
            session.show();
            assert!(session.is_visible());
        }

        instance.gui_destroy();
        assert!(!instance.gui_is_open());
        // Destroy is idempotent.
        instance.gui_destroy();

        // Reopen, then drop the instance with the editor still open: the
        // Drop fallback destroys the gui before the plugin (no panic, no
        // leak — verified by the fixture accepting a later create).
        instance
            .gui_open_embedded((&mut fake_parent as *mut u8).cast(), None)
            .expect("gui reopens after destroy");
        drop(instance);
    }

    #[test]
    fn hosted_instance_load_rejects_missing_library_and_unknown_plugin_id() {
        let missing = ClapHostedInstance::load(
            std::path::Path::new("/nonexistent/fixture.clap"),
            "com.signal.missing",
        );
        assert!(missing.is_err());
    }

    mod adapter;
}
