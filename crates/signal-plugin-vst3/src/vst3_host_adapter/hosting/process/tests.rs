use super::super::instance::{select_main_bus, Vst3HostedPortLayout};
use super::super::wire::*;
use super::super::wire::{stream_read, stream_seek, stream_write};
use super::buffers::Vst3AudioBusBuffers;

#[test]
fn first_declared_main_bus_wins_for_multi_output_instruments() {
    let first = select_main_bus(None, K_MAIN, 0);
    let second = select_main_bus(first, K_MAIN, 1);

    assert_eq!(second, Some(0));
}

#[test]
fn multi_output_buffers_report_each_bus_declared_channel_count() {
    let buffers = Vst3AudioBusBuffers::new(&[2, 2, 1], Some(0), 64);

    assert_eq!(
        buffers
            .descriptors
            .iter()
            .map(|descriptor| descriptor.num_channels)
            .collect::<Vec<_>>(),
        vec![2, 2, 1]
    );
}

#[test]
fn stereo_processor_layout_accepts_effects_and_instruments_only() {
    let effect = Vst3HostedPortLayout {
        main_input_channels: 2,
        main_output_channels: 2,
    };
    let instrument = Vst3HostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 2,
    };
    let mono_output = Vst3HostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 1,
    };
    let surround = Vst3HostedPortLayout {
        main_input_channels: 2,
        main_output_channels: 6,
    };

    assert!(effect.is_stereo_effect());
    assert!(!effect.is_stereo_instrument());
    assert!(effect.is_supported_stereo_processor());
    assert!(!instrument.is_stereo_effect());
    assert!(instrument.is_stereo_instrument());
    assert!(instrument.is_supported_stereo_processor());
    assert!(!mono_output.is_supported_stereo_processor());
    assert!(!surround.is_supported_stereo_processor());
}

#[test]
fn tuid_layout_matches_platform_expectations() {
    let tuid = tuid_from_uid(0x11223344, 0x55667788, 0x99AABBCC, 0xDDEEFF00);
    if cfg!(target_os = "windows") {
        assert_eq!(
            tuid,
            [
                0x44, 0x33, 0x22, 0x11, 0x66, 0x55, 0x88, 0x77, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
                0xFF, 0x00
            ]
        );
    } else {
        assert_eq!(
            tuid,
            [
                0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
                0xFF, 0x00
            ]
        );
    }
}

#[test]
fn class_id_hex_round_trips_the_interface_encoding() {
    // The canonical hex of a component class must decode to the same
    // in-memory TUID that `tuid_from_uid` builds for those fields.
    let expected = tuid_from_uid(0x11223344, 0x55667788, 0x99AABBCC, 0xDDEEFF00);
    let decoded = tuid_from_class_id_hex("112233445566778899AABBCCDDEEFF00")
        .expect("canonical hex should decode");
    assert_eq!(decoded, expected);
    assert!(tuid_from_class_id_hex("nonsense").is_none());
    assert!(tuid_from_class_id_hex("1122").is_none());
}

#[test]
fn state_envelope_round_trips_component_and_controller_state() {
    let encoded = encode_state_envelope(b"component-state", b"controller-state");
    let (component, controller) = decode_state_envelope(&encoded).expect("valid state envelope");

    assert_eq!(component, b"component-state");
    assert_eq!(controller, b"controller-state");
    assert!(decode_state_envelope(b"not-a-state-envelope").is_none());

    let mut trailing_bytes = encoded;
    trailing_bytes.push(0);
    assert!(decode_state_envelope(&trailing_bytes).is_none());
}

#[test]
fn memory_stream_supports_plugin_write_seek_and_read_calls() {
    let mut stream = MemoryStream::writer();
    let source = b"plugin-state";
    let mut written = 0;
    let result = unsafe {
        stream_write(
            stream.as_raw(),
            source.as_ptr().cast(),
            source.len() as i32,
            &mut written,
        )
    };
    assert_eq!(result, K_RESULT_OK);
    assert_eq!(written, source.len() as i32);

    let mut position = -1;
    assert_eq!(
        unsafe { stream_seek(stream.as_raw(), 0, 0, &mut position) },
        K_RESULT_OK
    );
    assert_eq!(position, 0);

    let mut destination = [0u8; 12];
    let mut read = 0;
    assert_eq!(
        unsafe {
            stream_read(
                stream.as_raw(),
                destination.as_mut_ptr().cast(),
                destination.len() as i32,
                &mut read,
            )
        },
        K_RESULT_OK
    );
    assert_eq!(read, destination.len() as i32);
    assert_eq!(&destination, source);
}
