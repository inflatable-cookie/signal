//! CoreMIDI-backed implementation of Signal's MIDI input contract.
//!
//! Owner-thread pattern lifted from `signal-hardware-cpal`: the MIDI client
//! and input port live and die on one dedicated thread per subscription.
//! That thread also owns the CFRunLoop CoreMIDI delivers device add/remove
//! notifications on (`MIDIClientCreate` binds notifications to the run loop
//! current at creation), so device loss is detected without any extra
//! machinery. The read callback is the real-time path: packet-list walk plus
//! the pure parser in [`crate::parse`], pushing into the caller-owned
//! [`MidiEventRing`] — no allocation, no locks, drop-and-count on overrun.

use std::cell::UnsafeCell;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use signal_hardware::{
    MidiEventRing, MidiInputBackend, MidiInputError, MidiInputEvent, MidiPortDescription,
    MidiSubscription, MidiSubscriptionState,
};

use crate::ffi;
use crate::parse::{parse_packet, MidiParseState};

/// Prefix of every CoreMIDI `port_id`; the suffix is the source endpoint's
/// `kMIDIPropertyUniqueID` in decimal — CoreMIDI's persistent machine-scoped
/// identity, stable across unplug/replug and reboots on the same machine.
const PORT_ID_PREFIX: &str = "coremidi:";

const STATE_ACTIVE: u8 = 0;
const STATE_CLOSED: u8 = 1;
const STATE_PORT_LOST: u8 = 2;

/// Idle poll interval for the owner thread's run loop, seconds.
const RUN_LOOP_SLICE_SECONDS: f64 = 0.2;

/// MIDI input backend over the machine's CoreMIDI sources.
#[derive(Debug, Default)]
pub struct CoreMidiInputBackend;

impl CoreMidiInputBackend {
    /// Construct a backend over the machine's CoreMIDI sources.
    pub fn new() -> Self {
        Self
    }
}

/// Build an owned CFString from a Rust string. Caller releases.
unsafe fn cf_string(text: &str) -> ffi::CFStringRef {
    unsafe {
        ffi::CFStringCreateWithBytes(
            ffi::kCFAllocatorDefault,
            text.as_ptr(),
            text.len() as ffi::CFIndex,
            ffi::kCFStringEncodingUTF8,
            0,
        )
    }
}

/// Copy a CFString into a Rust string. Does NOT release `string`.
unsafe fn string_from_cf(string: ffi::CFStringRef) -> Option<String> {
    if string.is_null() {
        return None;
    }
    unsafe {
        let length = ffi::CFStringGetLength(string);
        let max_bytes =
            ffi::CFStringGetMaximumSizeForEncoding(length, ffi::kCFStringEncodingUTF8) + 1;
        let mut buffer = vec![0u8; max_bytes.max(1) as usize];
        if ffi::CFStringGetCString(
            string,
            buffer.as_mut_ptr(),
            buffer.len() as ffi::CFIndex,
            ffi::kCFStringEncodingUTF8,
        ) == 0
        {
            return None;
        }
        let end = buffer.iter().position(|&byte| byte == 0)?;
        buffer.truncate(end);
        String::from_utf8(buffer).ok()
    }
}

/// Read a string property off a MIDI object, when present.
fn string_property(object: ffi::MIDIObjectRef, property: ffi::CFStringRef) -> Option<String> {
    unsafe {
        let mut value: ffi::CFStringRef = std::ptr::null();
        if ffi::MIDIObjectGetStringProperty(object, property, &mut value) != 0 || value.is_null() {
            return None;
        }
        let text = string_from_cf(value);
        ffi::CFRelease(value);
        text
    }
}

/// Read an integer property off a MIDI object, when present.
fn integer_property(object: ffi::MIDIObjectRef, property: ffi::CFStringRef) -> Option<i32> {
    unsafe {
        let mut value: i32 = 0;
        if ffi::MIDIObjectGetIntegerProperty(object, property, &mut value) != 0 {
            return None;
        }
        Some(value)
    }
}

/// Parse a `coremidi:<unique-id>` port id back to the CoreMIDI unique id.
fn parse_port_id(port_id: &str) -> Option<i32> {
    port_id.strip_prefix(PORT_ID_PREFIX)?.parse().ok()
}

impl MidiInputBackend for CoreMidiInputBackend {
    fn enumerate_ports(&self) -> Result<Vec<MidiPortDescription>, MidiInputError> {
        let mut ports = Vec::new();
        let source_count = unsafe { ffi::MIDIGetNumberOfSources() };
        for index in 0..source_count {
            let source = unsafe { ffi::MIDIGetSource(index) };
            if source == 0 {
                continue;
            }
            // Unique id is the identity anchor; a source without one cannot
            // be re-found later, so it is not offered.
            let Some(unique_id) = integer_property(source, unsafe { ffi::kMIDIPropertyUniqueID })
            else {
                continue;
            };
            let name = string_property(source, unsafe { ffi::kMIDIPropertyDisplayName })
                .or_else(|| string_property(source, unsafe { ffi::kMIDIPropertyName }))
                .unwrap_or_else(|| format!("MIDI Source {}", index + 1));
            let manufacturer = string_property(source, unsafe { ffi::kMIDIPropertyManufacturer })
                .unwrap_or_default();
            ports.push(MidiPortDescription {
                port_id: format!("{PORT_ID_PREFIX}{unique_id}"),
                name,
                manufacturer,
                // CoreMIDI has no default-source concept; the first
                // enumerated source is flagged, per the contract docs.
                is_default: ports.is_empty(),
            });
        }
        Ok(ports)
    }

    fn subscribe(
        &self,
        port_id: &str,
        ring: Arc<MidiEventRing>,
    ) -> Result<Box<dyn MidiSubscription>, MidiInputError> {
        let Some(unique_id) = parse_port_id(port_id) else {
            return Err(MidiInputError::port_not_found(port_id));
        };
        let shared = Arc::new(SubscriptionShared {
            state: AtomicU8::new(STATE_ACTIVE),
            endpoint: AtomicU32::new(0),
            run_loop: AtomicUsize::new(0),
            stop: AtomicBool::new(false),
            last_error: Mutex::new(None),
        });
        let owner_shared = Arc::clone(&shared);
        let owner_ring = Arc::clone(&ring);
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), MidiInputError>>();

        // Client, port, and notification run loop live and die on this
        // thread — the cpal stream-thread ownership pattern.
        let owner = std::thread::Builder::new()
            .name("signal-midi-input".to_string())
            .spawn(move || owner_thread(unique_id, owner_ring, owner_shared, ready_tx))
            .map_err(|error| MidiInputError::backend(format!("spawn midi thread: {error}")))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Box::new(CoreMidiSubscription {
                port_id: port_id.to_string(),
                ring,
                shared,
                owner: Some(owner),
            })),
            Ok(Err(error)) => {
                let _ = owner.join();
                Err(error)
            }
            Err(_) => {
                let _ = owner.join();
                Err(MidiInputError::backend(
                    "midi thread exited before reporting",
                ))
            }
        }
    }
}

/// State shared between the subscription handle, the owner thread, and the
/// CoreMIDI callbacks.
struct SubscriptionShared {
    state: AtomicU8,
    /// The connected source endpoint, so the notify proc can match removal
    /// notifications against it.
    endpoint: AtomicU32,
    /// The owner thread's CFRunLoop, so drop can wake it. 0 until running.
    run_loop: AtomicUsize,
    stop: AtomicBool,
    /// Detail behind a `PortLost` transition. Written by the notify proc,
    /// which runs on the owner thread's run loop — never the read (RT)
    /// callback — so a mutex is acceptable there.
    last_error: Mutex<Option<String>>,
}

/// Context handed to the CoreMIDI callbacks by raw pointer; owned by the
/// owner thread and freed only after `MIDIClientDispose` returns (CoreMIDI
/// stops invoking the procs by then).
struct CallbackContext {
    ring: Arc<MidiEventRing>,
    /// Parser state for the read proc. CoreMIDI invokes the read proc
    /// serially from its scheduling thread, so this single-thread mutation
    /// through `UnsafeCell` is sound.
    parse_state: UnsafeCell<MidiParseState>,
    timebase_numer: u32,
    timebase_denom: u32,
    shared: Arc<SubscriptionShared>,
}

// SAFETY: the read proc is the only mutator of `parse_state` and CoreMIDI
// serializes read-proc invocations for a port; every other field is
// internally synchronized.
unsafe impl Sync for CallbackContext {}

/// The RT path: walk the packet list and push resolved events into the ring.
extern "C" fn read_proc(
    packet_list: *const c_void,
    read_proc_ref_con: *mut c_void,
    _source_ref_con: *mut c_void,
) {
    if packet_list.is_null() || read_proc_ref_con.is_null() {
        return;
    }
    let context = unsafe { &*(read_proc_ref_con as *const CallbackContext) };
    // SAFETY: read procs are serialized (see `CallbackContext`); this is the
    // sole mutable reference to the parse state.
    let parse_state = unsafe { &mut *context.parse_state.get() };
    let base = packet_list as usize;
    let packet_count = unsafe { std::ptr::read_unaligned(packet_list as *const u32) };
    let mut packet = base + ffi::MIDI_PACKET_LIST_HEADER_BYTES;
    for _ in 0..packet_count {
        let (timestamp, length) = unsafe {
            (
                std::ptr::read_unaligned(packet as *const ffi::MIDITimeStamp),
                std::ptr::read_unaligned((packet + ffi::MIDI_PACKET_LENGTH_OFFSET) as *const u16),
            )
        };
        let data = unsafe {
            std::slice::from_raw_parts(
                (packet + ffi::MIDI_PACKET_DATA_OFFSET) as *const u8,
                usize::from(length),
            )
        };
        // Timestamp 0 means "now" per CoreMIDI; substitute the current host
        // time so consumers always get a real instant.
        let host_ticks = if timestamp == 0 {
            unsafe { ffi::mach_absolute_time() }
        } else {
            timestamp
        };
        let host_nanos = (u128::from(host_ticks) * u128::from(context.timebase_numer)
            / u128::from(context.timebase_denom.max(1))) as u64;
        parse_packet(
            parse_state,
            host_nanos,
            data,
            &mut |event: MidiInputEvent| {
                // Full ring drops and counts inside the ring; never blocks.
                context.ring.push(event);
            },
        );
        packet = ffi::midi_packet_next(packet + ffi::MIDI_PACKET_DATA_OFFSET + usize::from(length));
    }
}

/// Device add/remove notifications, delivered on the owner thread's run
/// loop (not the RT path). A removal of our connected source flips the
/// subscription to `PortLost` — the poll surface mirroring how cpal input
/// streams surface faults.
extern "C" fn notify_proc(message: *const ffi::MIDINotification, ref_con: *mut c_void) {
    if message.is_null() || ref_con.is_null() {
        return;
    }
    let context = unsafe { &*(ref_con as *const CallbackContext) };
    let header = unsafe { &*message };
    if header.messageID != ffi::kMIDIMsgObjectRemoved {
        return;
    }
    let removal = unsafe { &*(message as *const ffi::MIDIObjectAddRemoveNotification) };
    let endpoint = context.shared.endpoint.load(Ordering::Relaxed);
    if endpoint != 0 && removal.child == endpoint {
        if let Ok(mut detail) = context.shared.last_error.lock() {
            *detail = Some("midi source removed by the OS".to_string());
        }
        context
            .shared
            .state
            .store(STATE_PORT_LOST, Ordering::Relaxed);
    }
}

/// Body of the dedicated owner thread: create client + port, connect the
/// source, report readiness, then service the run loop until stopped.
fn owner_thread(
    unique_id: i32,
    ring: Arc<MidiEventRing>,
    shared: Arc<SubscriptionShared>,
    ready_tx: mpsc::Sender<Result<(), MidiInputError>>,
) {
    let mut timebase = ffi::MachTimebaseInfo { numer: 1, denom: 1 };
    unsafe { ffi::mach_timebase_info(&mut timebase) };
    let context = Box::new(CallbackContext {
        ring,
        parse_state: UnsafeCell::new(MidiParseState::new()),
        timebase_numer: timebase.numer.max(1),
        timebase_denom: timebase.denom.max(1),
        shared: Arc::clone(&shared),
    });
    let context_ptr = &*context as *const CallbackContext as *mut c_void;

    let open = (|| unsafe {
        let client_name = cf_string("signal-midi-input");
        let mut client: ffi::MIDIClientRef = 0;
        let status =
            ffi::MIDIClientCreate(client_name, Some(notify_proc), context_ptr, &mut client);
        ffi::CFRelease(client_name);
        if status != 0 {
            return Err(MidiInputError::backend(format!(
                "MIDIClientCreate failed: {status}"
            )));
        }

        let close_client = |client: ffi::MIDIClientRef| {
            ffi::MIDIClientDispose(client);
        };

        let port_name = cf_string("signal-midi-input-port");
        let mut port: ffi::MIDIPortRef = 0;
        let status = ffi::MIDIInputPortCreate(client, port_name, read_proc, context_ptr, &mut port);
        ffi::CFRelease(port_name);
        if status != 0 {
            close_client(client);
            return Err(MidiInputError::backend(format!(
                "MIDIInputPortCreate failed: {status}"
            )));
        }

        let mut object: ffi::MIDIObjectRef = 0;
        let mut object_type: i32 = 0;
        let status = ffi::MIDIObjectFindByUniqueID(unique_id, &mut object, &mut object_type);
        if status != 0 || object_type != ffi::kMIDIObjectType_Source {
            close_client(client);
            return Err(MidiInputError::port_not_found(&format!(
                "{PORT_ID_PREFIX}{unique_id}"
            )));
        }
        shared.endpoint.store(object, Ordering::Relaxed);

        let status = ffi::MIDIPortConnectSource(port, object, std::ptr::null_mut());
        if status != 0 {
            close_client(client);
            return Err(MidiInputError::backend(format!(
                "MIDIPortConnectSource failed: {status}"
            )));
        }
        Ok((client, port, object))
    })();

    let (client, port, source) = match open {
        Ok(handles) => handles,
        Err(error) => {
            let _ = ready_tx.send(Err(error));
            return;
        }
    };

    let run_loop = unsafe { ffi::CFRunLoopGetCurrent() };
    shared.run_loop.store(run_loop as usize, Ordering::Release);
    if ready_tx.send(Ok(())).is_err() {
        // Subscriber vanished before we reported: tear straight down.
        shared.stop.store(true, Ordering::Relaxed);
    }

    while !shared.stop.load(Ordering::Acquire) {
        // Notification delivery needs this thread's run loop serviced. A run
        // loop with no source yet (kCFRunLoopRunFinished == 1) would return
        // immediately; sleep the slice instead of spinning.
        let result = unsafe {
            ffi::CFRunLoopRunInMode(ffi::kCFRunLoopDefaultMode, RUN_LOOP_SLICE_SECONDS, 0)
        };
        if result == 1 {
            std::thread::sleep(std::time::Duration::from_millis(
                (RUN_LOOP_SLICE_SECONDS * 1_000.0) as u64,
            ));
        }
    }

    unsafe {
        ffi::MIDIPortDisconnectSource(port, source);
        // Disposing the client disposes its ports and stops both callbacks;
        // `context` may only drop after this returns.
        ffi::MIDIClientDispose(client);
    }
    drop(context);
}

/// Handle over a CoreMIDI source subscription; drop disconnects and joins
/// the owner thread.
struct CoreMidiSubscription {
    port_id: String,
    ring: Arc<MidiEventRing>,
    shared: Arc<SubscriptionShared>,
    owner: Option<std::thread::JoinHandle<()>>,
}

impl MidiSubscription for CoreMidiSubscription {
    fn state(&self) -> MidiSubscriptionState {
        match self.shared.state.load(Ordering::Relaxed) {
            STATE_ACTIVE => MidiSubscriptionState::Active,
            STATE_PORT_LOST => MidiSubscriptionState::PortLost,
            _ => MidiSubscriptionState::Closed,
        }
    }

    fn port_id(&self) -> &str {
        &self.port_id
    }

    fn overrun_events(&self) -> u64 {
        self.ring.overrun_events()
    }

    fn last_error(&self) -> Option<String> {
        self.shared
            .last_error
            .lock()
            .ok()
            .and_then(|detail| detail.clone())
    }
}

impl Drop for CoreMidiSubscription {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        let run_loop = self.shared.run_loop.load(Ordering::Acquire) as ffi::CFRunLoopRef;
        if !run_loop.is_null() {
            unsafe { ffi::CFRunLoopStop(run_loop) };
        }
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
        if self.shared.state.load(Ordering::Relaxed) == STATE_ACTIVE {
            self.shared.state.store(STATE_CLOSED, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test against the real CoreMIDI service; skips quietly when the
    /// machine has no MIDI sources (CI) — the cpal enumeration-test posture.
    #[test]
    fn enumerates_midi_sources_when_present() {
        let backend = CoreMidiInputBackend::new();
        let ports = backend.enumerate_ports().expect("enumerate midi sources");
        if ports.is_empty() {
            eprintln!("no midi sources; skipping");
            return;
        }
        assert!(ports[0].is_default);
        assert_eq!(ports.iter().filter(|port| port.is_default).count(), 1);
        for port in &ports {
            assert!(port.port_id.starts_with(PORT_ID_PREFIX), "{}", port.port_id);
            assert!(
                parse_port_id(&port.port_id).is_some(),
                "port id round-trips: {}",
                port.port_id
            );
            assert!(!port.name.is_empty());
        }
    }

    #[test]
    fn subscribing_a_malformed_port_id_is_port_not_found() {
        let backend = CoreMidiInputBackend::new();
        let ring = Arc::new(MidiEventRing::with_capacity(16));
        let error = backend
            .subscribe("not-a-coremidi-id", ring)
            .err()
            .expect("malformed id must not subscribe");
        assert_eq!(
            error.kind,
            signal_hardware::MidiInputErrorKind::PortNotFound
        );
    }

    #[test]
    fn subscribing_an_absent_unique_id_is_port_not_found() {
        // Unique ids are i32; this one is overwhelmingly unlikely to exist.
        let backend = CoreMidiInputBackend::new();
        let ring = Arc::new(MidiEventRing::with_capacity(16));
        let error = backend
            .subscribe("coremidi:2147480001", ring)
            .err()
            .expect("absent source must not subscribe");
        assert_eq!(
            error.kind,
            signal_hardware::MidiInputErrorKind::PortNotFound
        );
    }
}
