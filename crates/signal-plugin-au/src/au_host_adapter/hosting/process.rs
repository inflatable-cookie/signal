//! Audio-thread AU process session (pull-model render).

#[cfg(target_os = "macos")]
use super::ffi;
use super::types::AuHostingError;

// ── Raw process session (audio thread) ──────────────────────────────────────

/// The dry-input stash served by the pull-model render callback. Boxed so
/// its address stays stable while [`AuProcessSession`] moves between
/// threads (the callback's `inRefCon` points here).
#[cfg(target_os = "macos")]
struct RenderInputState {
    left: Vec<f32>,
    right: Vec<f32>,
    /// Frames stashed for the in-flight `AudioUnitRender` call.
    frames: usize,
}

/// The input-scope render callback trampoline: serves the stashed dry block
/// into `ioData`. Two callback buffer conventions exist and both are
/// handled alloc-free: a non-null `mData` is the unit's own buffer (copy
/// into it); a null `mData` asks the host to point the unit at host-owned
/// memory (swizzle to the stash). Frames beyond the stash are zero-filled.
#[cfg(target_os = "macos")]
unsafe extern "C" fn render_input_trampoline(
    in_ref_con: *mut std::ffi::c_void,
    _io_action_flags: *mut u32,
    _in_time_stamp: *const ffi::AudioTimeStamp,
    _in_bus_number: u32,
    in_number_frames: u32,
    io_data: *mut ffi::RawAudioBufferList,
) -> ffi::OSStatus {
    if in_ref_con.is_null() || io_data.is_null() {
        return -50; // paramErr
    }
    let state = &mut *(in_ref_con as *mut RenderInputState);
    let buffer_count = ((*io_data).mNumberBuffers as usize).min(2);
    let buffers = (*io_data).mBuffers.as_mut_ptr();
    for index in 0..buffer_count {
        let source = if index == 0 {
            &mut state.left
        } else {
            &mut state.right
        };
        let requested = (in_number_frames as usize).min(source.len());
        let served = requested.min(state.frames);
        // Zero the unstashed tail so a long pull never reads stale samples.
        for slot in source[served..requested].iter_mut() {
            *slot = 0.0;
        }
        let buffer = &mut *buffers.add(index);
        if buffer.mData.is_null() {
            buffer.mData = source.as_mut_ptr() as *mut _;
            buffer.mDataByteSize = (requested * std::mem::size_of::<f32>()) as u32;
        } else {
            let capacity = (buffer.mDataByteSize as usize) / std::mem::size_of::<f32>();
            let count = requested.min(capacity);
            std::ptr::copy_nonoverlapping(source.as_ptr(), buffer.mData as *mut f32, count);
        }
        buffer.mNumberChannels = 1;
    }
    0
}

/// Raw, movable process handle for one activated AU instance: the
/// `AudioUnit` handle, the boxed dry-input stash (callback `inRefCon`),
/// planar output buffers, and a preallocated fixed-stereo
/// `AudioBufferList` — per block only the buffer pointers/sizes are
/// swizzled. The render timestamp's `mSampleTime` advances monotonically by
/// the processed frame count (time-based units misbehave on static
/// timestamps). The sandbox moves this onto its audio thread; the owning
/// `AuHostedInstance` must outlive it and must not run lifecycle
/// transitions while the session is live. Zero allocation in the render
/// path.
pub struct AuProcessSession {
    #[cfg(target_os = "macos")]
    unit: ffi::AudioUnit,
    #[cfg(target_os = "macos")]
    input: Box<RenderInputState>,
    #[cfg(target_os = "macos")]
    output_left: Vec<f32>,
    #[cfg(target_os = "macos")]
    output_right: Vec<f32>,
    #[cfg(target_os = "macos")]
    render_list: ffi::StereoAudioBufferList,
    #[cfg(target_os = "macos")]
    sample_time: f64,
    processing: bool,
}

impl std::fmt::Debug for AuProcessSession {
    /// Reports the hosted unit and render shape. The render input state and
    /// buffer list are FFI scratch handed to `AudioUnitRender`.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuProcessSession")
            .field("unit", &self.unit)
            .field("max_frames", &self.output_left.len())
            .field("sample_time", &self.sample_time)
            .field("processing", &self.processing)
            .finish_non_exhaustive()
    }
}

// Safety: the session is handed to exactly one audio thread;
// `AudioUnitRender` is the AU render-thread entry point, the boxed input
// stash is only touched by the unit's synchronous mid-render pull on that
// same thread, and the owner serializes lifecycle against the session per
// the type contract (the vst3 session precedent).
unsafe impl Send for AuProcessSession {}

impl AuProcessSession {
    #[cfg(target_os = "macos")]
    pub(crate) fn new(
        unit: ffi::AudioUnit,
        max_frames: usize,
        has_audio_input: bool,
    ) -> Result<Self, AuHostingError> {
        let mut input = Box::new(RenderInputState {
            left: vec![0.0; max_frames],
            right: vec![0.0; max_frames],
            frames: 0,
        });
        let callback = ffi::AURenderCallbackStruct {
            inputProc: Some(render_input_trampoline),
            inputProcRefCon: &mut *input as *mut RenderInputState as *mut _,
        };
        if has_audio_input {
            let status = unsafe {
                ffi::AudioUnitSetProperty(
                    unit,
                    ffi::kAudioUnitProperty_SetRenderCallback,
                    ffi::kAudioUnitScope_Input,
                    0,
                    &callback as *const _ as *const _,
                    std::mem::size_of::<ffi::AURenderCallbackStruct>() as u32,
                )
            };
            if status != 0 {
                return Err(AuHostingError::new("render_callback_install_failed"));
            }
        }
        Ok(Self {
            unit,
            input,
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            render_list: ffi::StereoAudioBufferList {
                mNumberBuffers: 2,
                mBuffers: [
                    ffi::AudioBuffer {
                        mNumberChannels: 1,
                        mDataByteSize: 0,
                        mData: std::ptr::null_mut(),
                    },
                    ffi::AudioBuffer {
                        mNumberChannels: 1,
                        mDataByteSize: 0,
                        mData: std::ptr::null_mut(),
                    },
                ],
            },
            sample_time: 0.0,
            processing: false,
        })
    }

    /// Mark the session processing on the audio thread; must precede
    /// `process_*`. (AUv2 has no `setProcessing` equivalent — this is
    /// bookkeeping kept for surface parity with the CLAP/VST3 sessions.)
    pub fn start(&mut self) -> Result<(), AuHostingError> {
        self.processing = true;
        Ok(())
    }

    /// Mark the session stopped on the audio thread.
    pub fn stop(&mut self) {
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Pull one block from the unit via `AudioUnitRender` using the
    /// preallocated buffers. Returns `false` on unit error.
    ///
    /// # Safety
    /// `frames` must be within the preallocated buffer bounds (callers
    /// clamp).
    #[cfg(target_os = "macos")]
    unsafe fn render_block(&mut self, frames: usize) -> bool {
        self.input.frames = frames;
        // Per-block pointer swizzle only — the list itself is preallocated.
        let byte_size = (frames * std::mem::size_of::<f32>()) as u32;
        self.render_list.mBuffers[0].mData = self.output_left.as_mut_ptr() as *mut _;
        self.render_list.mBuffers[0].mDataByteSize = byte_size;
        self.render_list.mBuffers[0].mNumberChannels = 1;
        self.render_list.mBuffers[1].mData = self.output_right.as_mut_ptr() as *mut _;
        self.render_list.mBuffers[1].mDataByteSize = byte_size;
        self.render_list.mBuffers[1].mNumberChannels = 1;
        let timestamp = ffi::AudioTimeStamp {
            // Monotonically advancing sample time: delays keep their read
            // heads honest only when the timeline moves.
            mSampleTime: self.sample_time,
            mFlags: ffi::kAudioTimeStampSampleTimeValid,
            ..Default::default()
        };
        let mut action_flags: u32 = 0;
        let status = ffi::AudioUnitRender(
            self.unit,
            &mut action_flags,
            &timestamp,
            0,
            frames as u32,
            &mut self.render_list,
        );
        if status != 0 {
            return false;
        }
        self.sample_time += frames as f64;
        true
    }

    /// Read one rendered output sample; `AudioUnitRender` may point
    /// `mData` at the unit's own buffers instead of writing into ours, so
    /// output is always read back through the (possibly replaced) list
    /// pointers.
    ///
    /// # Safety
    /// Only valid after a successful `render_block(frames)` with
    /// `frame < frames`.
    #[cfg(target_os = "macos")]
    unsafe fn rendered_sample(&self, channel: usize, frame: usize) -> f32 {
        let data = self.render_list.mBuffers[channel].mData as *const f32;
        if data.is_null() {
            return 0.0;
        }
        *data.add(frame)
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (buffers preallocated at activate). On unit error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        #[cfg(target_os = "macos")]
        {
            let frames = frame_count
                .min(self.input.left.len())
                .min(input.len() / 2)
                .min(output.len() / 2);
            for frame in 0..frames {
                self.input.left[frame] = input[frame * 2];
                self.input.right[frame] = input[frame * 2 + 1];
            }
            if !unsafe { self.render_block(frames) } {
                output[..frames * 2].copy_from_slice(&input[..frames * 2]);
                return false;
            }
            for frame in 0..frames {
                output[frame * 2] = unsafe { self.rendered_sample(0, frame) };
                output[frame * 2 + 1] = unsafe { self.rendered_sample(1, frame) };
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = frame_count;
            let samples = input.len().min(output.len());
            output[..samples].copy_from_slice(&input[..samples]);
            false
        }
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on unit error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        self.process_in_place_with_events(io, frame_count, &[])
    }

    /// [`Self::process_in_place`] with a per-block plugin event slice
    /// (sorted by `offset_frames`): each note/CC event is scheduled onto
    /// the unit through `MusicDeviceMIDIEvent` with its intra-block offset
    /// frame BEFORE the render pull. This is the AU MIDI 1.0 downconversion
    /// boundary (velocity/value → `round(x * 127)`). Units that do not
    /// accept MIDI (plain effects) refuse per event; the audio path is
    /// unaffected — the honest fallback. Alloc-free.
    pub fn process_in_place_with_events(
        &mut self,
        io: &mut [f32],
        frame_count: usize,
        events: &[signal_plugin::PluginEvent],
    ) -> bool {
        #[cfg(target_os = "macos")]
        {
            use signal_plugin::{NoteEventKind, PluginEvent};
            for event in events {
                let (status, data1, data2, offset) = match event {
                    PluginEvent::Note(note) => {
                        let status = match note.kind {
                            NoteEventKind::NoteOn => 0x90,
                            NoteEventKind::NoteOff => 0x80,
                        };
                        (
                            status | u32::from(note.channel & 0x0F),
                            u32::from(note.key & 0x7F),
                            (note.velocity.clamp(0.0, 1.0) * 127.0).round() as u32,
                            note.offset_frames,
                        )
                    }
                    PluginEvent::ControlChange(change) => (
                        0xB0 | u32::from(change.channel & 0x0F),
                        u32::from(change.controller & 0x7F),
                        (change.value.clamp(0.0, 1.0) * 127.0).round() as u32,
                        change.offset_frames,
                    ),
                    PluginEvent::Midi(midi) => (
                        u32::from(midi.status),
                        u32::from(midi.data1),
                        u32::from(midi.data2),
                        midi.offset_frames,
                    ),
                    _ => continue,
                };
                // Per-event refusal (e.g. an effect unit without MIDI
                // input) is tolerated: audio must keep flowing.
                let _ =
                    unsafe { ffi::MusicDeviceMIDIEvent(self.unit, status, data1, data2, offset) };
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = events;
        }
        #[cfg(target_os = "macos")]
        {
            let frames = frame_count.min(self.input.left.len()).min(io.len() / 2);
            for frame in 0..frames {
                self.input.left[frame] = io[frame * 2];
                self.input.right[frame] = io[frame * 2 + 1];
            }
            if !unsafe { self.render_block(frames) } {
                return false;
            }
            for frame in 0..frames {
                io[frame * 2] = unsafe { self.rendered_sample(0, frame) };
                io[frame * 2 + 1] = unsafe { self.rendered_sample(1, frame) };
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (io, frame_count);
            false
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for AuProcessSession {
    fn drop(&mut self) {
        // Uninstall the render callback so the unit can never pull from the
        // freed input stash after the session is gone.
        let callback = ffi::AURenderCallbackStruct {
            inputProc: None,
            inputProcRefCon: std::ptr::null_mut(),
        };
        unsafe {
            let _ = ffi::AudioUnitSetProperty(
                self.unit,
                ffi::kAudioUnitProperty_SetRenderCallback,
                ffi::kAudioUnitScope_Input,
                0,
                &callback as *const _ as *const _,
                std::mem::size_of::<ffi::AURenderCallbackStruct>() as u32,
            );
        }
    }
}
