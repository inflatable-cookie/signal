//! Pure MIDIPacket byte-stream parser: raw packet data in, complete
//! running-status-resolved [`MidiInputEvent`]s out.
//!
//! This is the real-time half of the read callback, isolated as a pure
//! function so it is unit-testable over byte fixtures without CoreMIDI. The
//! rules it implements:
//!
//! - Channel voice messages emit with their status byte resolved (running
//!   status expanded), 2 data bytes for note/poly-pressure/CC/pitch-bend and
//!   1 for program change/channel pressure.
//! - System real-time bytes (`0xF8..=0xFF`) emit immediately as one-byte
//!   events, even interleaved inside another message or a SysEx transfer.
//! - SysEx (`0xF0 .. 0xF7`) is skipped entirely, including transfers that
//!   span packets — [`MidiParseState`] carries the in-SysEx flag across
//!   calls. Recorded runway: nothing downstream consumes SysEx yet.
//! - System common messages parse with their own lengths and cancel running
//!   status, per the MIDI 1.0 specification.
//! - Malformed input never emits: stray data bytes without a status are
//!   dropped, and a message left incomplete at the end of a packet's data is
//!   dropped (non-SysEx messages do not span packets).
//!
//! No allocation anywhere on this path; events stream out through the
//! caller's `emit` closure.

use signal_hardware::MidiInputEvent;

/// Parser state that legitimately spans packets: the current running status
/// and whether a SysEx transfer is in progress. Partial non-SysEx messages
/// deliberately do NOT span packets (see module docs).
#[derive(Debug, Clone, Copy, Default)]
pub struct MidiParseState {
    running_status: Option<u8>,
    in_sysex: bool,
}

impl MidiParseState {
    /// Fresh state: no running status, not inside SysEx.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Data-byte count for a status byte, or `None` for statuses that carry no
/// parseable message (undefined system common `0xF4`/`0xF5`).
fn data_bytes_for_status(status: u8) -> Option<usize> {
    match status & 0xF0 {
        0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => Some(2),
        0xC0 | 0xD0 => Some(1),
        0xF0 => match status {
            0xF1 | 0xF3 => Some(1), // MTC quarter frame, song select
            0xF2 => Some(2),        // song position pointer
            0xF6 => Some(0),        // tune request
            _ => None,              // 0xF4/0xF5 undefined
        },
        _ => None,
    }
}

/// Parse one packet's raw MIDI bytes, emitting every complete resolved
/// message stamped with `timestamp_host_nanos` (CoreMIDI timestamps whole
/// packets, not individual messages). Alloc-free; safe on the read callback.
///
/// # Panics
///
/// Panics if the internal channel-message accumulator is completed without a
/// status byte having been latched. Running status is set before any data
/// byte is accepted, so this is a parser invariant, not a property of the
/// incoming packet: malformed, truncated, and interleaved real-time bytes are
/// all handled without panicking.
pub fn parse_packet(
    state: &mut MidiParseState,
    timestamp_host_nanos: u64,
    bytes: &[u8],
    emit: &mut dyn FnMut(MidiInputEvent),
) {
    let mut pending_status: Option<u8> = None;
    let mut pending = [0u8; 2];
    let mut pending_len = 0usize;
    let mut pending_needed = 0usize;

    for &byte in bytes {
        // System real-time: single byte, passes through anywhere — even
        // interleaved mid-message or mid-SysEx — disturbing nothing.
        if byte >= 0xF8 {
            emit(MidiInputEvent::new(timestamp_host_nanos, &[byte]));
            continue;
        }
        if byte == 0xF0 {
            // SysEx start: skip until EOX; SysEx cancels running status.
            state.in_sysex = true;
            state.running_status = None;
            pending_status = None;
            continue;
        }
        if byte == 0xF7 {
            // EOX: end of the (skipped) SysEx transfer.
            state.in_sysex = false;
            pending_status = None;
            continue;
        }
        if byte >= 0x80 {
            // A new status byte implicitly terminates SysEx and abandons any
            // incomplete message.
            state.in_sysex = false;
            pending_status = None;
            let Some(needed) = data_bytes_for_status(byte) else {
                // Undefined system common: drop, and per spec it still
                // cancels running status.
                state.running_status = None;
                continue;
            };
            if byte < 0xF0 {
                state.running_status = Some(byte);
            } else {
                state.running_status = None;
            }
            if needed == 0 {
                emit(MidiInputEvent::new(timestamp_host_nanos, &[byte]));
            } else {
                pending_status = Some(byte);
                pending_len = 0;
                pending_needed = needed;
            }
            continue;
        }
        // Data byte.
        if state.in_sysex {
            continue; // SysEx payload: skipped.
        }
        if pending_status.is_none() {
            match state.running_status {
                Some(running) => {
                    // Running status: this data byte starts a new message
                    // under the last channel status seen.
                    pending_status = Some(running);
                    pending_len = 0;
                    pending_needed = data_bytes_for_status(running).unwrap_or(2);
                }
                None => continue, // Stray data byte: malformed, dropped.
            }
        }
        pending[pending_len] = byte;
        pending_len += 1;
        if pending_len == pending_needed {
            let status = pending_status.take().expect("pending message has a status");
            let event = match pending_needed {
                1 => MidiInputEvent::new(timestamp_host_nanos, &[status, pending[0]]),
                _ => MidiInputEvent::new(timestamp_host_nanos, &[status, pending[0], pending[1]]),
            };
            emit(event);
            pending_len = 0;
        }
    }
    // Anything still pending here is a malformed tail: non-SysEx messages do
    // not span packets, so the partial message is dropped, not carried.
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the parser over a sequence of packets and collect what it emits.
    fn parse_packets(packets: &[(u64, &[u8])]) -> Vec<MidiInputEvent> {
        let mut state = MidiParseState::new();
        let mut events = Vec::new();
        for (timestamp, bytes) in packets {
            parse_packet(&mut state, *timestamp, bytes, &mut |event| {
                events.push(event);
            });
        }
        events
    }

    fn event(timestamp: u64, message: &[u8]) -> MidiInputEvent {
        MidiInputEvent::new(timestamp, message)
    }

    #[test]
    fn parses_plain_three_byte_channel_messages() {
        let events = parse_packets(&[(10, &[0x90, 60, 100, 0x80, 60, 0])]);
        assert_eq!(
            events,
            vec![event(10, &[0x90, 60, 100]), event(10, &[0x80, 60, 0])]
        );
    }

    #[test]
    fn resolves_running_status_within_and_across_packets() {
        // One explicit note-on status, then three messages under running
        // status — the last one arriving in a later packet.
        let events = parse_packets(&[(1, &[0x90, 60, 100, 62, 100, 64, 100]), (2, &[65, 100])]);
        assert_eq!(
            events,
            vec![
                event(1, &[0x90, 60, 100]),
                event(1, &[0x90, 62, 100]),
                event(1, &[0x90, 64, 100]),
                event(2, &[0x90, 65, 100]),
            ]
        );
    }

    #[test]
    fn parses_two_byte_messages_and_their_running_status() {
        // Program change and channel pressure carry one data byte each.
        let events = parse_packets(&[(5, &[0xC3, 12, 0xD0, 90, 91])]);
        assert_eq!(
            events,
            vec![
                event(5, &[0xC3, 12]),
                event(5, &[0xD0, 90]),
                event(5, &[0xD0, 91]), // running status on channel pressure
            ]
        );
    }

    #[test]
    fn passes_real_time_bytes_interleaved_inside_a_message() {
        // Clock (0xF8) lands between a note-on's data bytes; both survive.
        let events = parse_packets(&[(7, &[0x90, 60, 0xF8, 100])]);
        assert_eq!(events, vec![event(7, &[0xF8]), event(7, &[0x90, 60, 100])]);
    }

    #[test]
    fn skips_sysex_entirely() {
        let events = parse_packets(&[(3, &[0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7, 0x90, 60, 100])]);
        assert_eq!(events, vec![event(3, &[0x90, 60, 100])]);
    }

    #[test]
    fn skips_sysex_spanning_packets_and_passes_interleaved_real_time() {
        let events = parse_packets(&[
            (1, &[0xF0, 0x41, 0x10]),        // SysEx starts, no EOX yet
            (2, &[0x42, 0xF8, 0x43]),        // continues; clock passes through
            (3, &[0x44, 0xF7, 0xB0, 1, 64]), // EOX, then a normal CC
        ]);
        assert_eq!(events, vec![event(2, &[0xF8]), event(3, &[0xB0, 1, 64])]);
    }

    #[test]
    fn sysex_cancels_running_status() {
        let events = parse_packets(&[(1, &[0x90, 60, 100, 0xF0, 0x01, 0xF7, 61, 100])]);
        // The trailing data bytes have no status to resolve against.
        assert_eq!(events, vec![event(1, &[0x90, 60, 100])]);
    }

    #[test]
    fn parses_system_common_messages_and_cancels_running_status() {
        let events = parse_packets(&[
            (4, &[0x90, 60, 100, 0xF2, 0x10, 0x02, 0xF6]),
            (5, &[60, 0]), // running status was cancelled by system common
        ]);
        assert_eq!(
            events,
            vec![
                event(4, &[0x90, 60, 100]),
                event(4, &[0xF2, 0x10, 0x02]),
                event(4, &[0xF6]),
            ]
        );
    }

    #[test]
    fn drops_stray_data_bytes_without_any_status() {
        let events = parse_packets(&[(1, &[60, 100, 0x90, 61, 101])]);
        assert_eq!(events, vec![event(1, &[0x90, 61, 101])]);
    }

    #[test]
    fn drops_a_message_truncated_at_the_end_of_a_packet() {
        // The note-on never gets its velocity; a fresh packet starts with a
        // new message. The malformed tail must not merge across packets.
        let events = parse_packets(&[(1, &[0x90, 60]), (2, &[0xB0, 7, 99])]);
        assert_eq!(events, vec![event(2, &[0xB0, 7, 99])]);
    }

    #[test]
    fn undefined_system_common_bytes_are_dropped_and_cancel_running_status() {
        let events = parse_packets(&[(1, &[0x90, 60, 100, 0xF4, 61, 100])]);
        assert_eq!(events, vec![event(1, &[0x90, 60, 100])]);
    }
}
