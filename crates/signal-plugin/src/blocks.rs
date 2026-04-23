use crate::PluginEvent;

/// An interleaved buffer of audio samples for a fixed number of channels and frames.
///
/// Samples are stored in interleaved channel order: frame 0 channel 0, frame 0 channel 1, …,
/// frame N channel C. The total sample count must equal `channel_count * frame_count`.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioBlock {
    /// Number of audio channels.
    pub channel_count: u16,
    /// Number of audio frames in this block.
    pub frame_count: u32,
    /// Interleaved PCM samples; length must equal `channel_count * frame_count`.
    pub samples: Vec<f32>,
}

impl AudioBlock {
    /// Creates a new `AudioBlock`, returning an error if `samples.len() != channel_count * frame_count`.
    pub fn new(
        channel_count: u16,
        frame_count: u32,
        samples: Vec<f32>,
    ) -> Result<Self, &'static str> {
        if samples.len() != channel_count as usize * frame_count as usize {
            return Err("audio block samples do not match channel_count * frame_count");
        }

        Ok(Self {
            channel_count,
            frame_count,
            samples,
        })
    }

    /// Returns the total number of samples (channels × frames).
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the first sample, or `None` if the block is empty.
    pub fn first_sample(&self) -> Option<f32> {
        self.samples.first().copied()
    }
}

/// An ordered list of plugin events to be delivered or consumed within a single block.
#[derive(Clone, Debug, PartialEq)]
pub struct EventPacket {
    /// The events in delivery order.
    pub events: Vec<PluginEvent>,
}

impl EventPacket {
    /// Creates a new `EventPacket` from the given event list.
    pub fn new(events: Vec<PluginEvent>) -> Self {
        Self { events }
    }

    /// Returns the number of events in this packet.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the number of bytes required to encode this packet in the shared-memory wire format.
    pub fn encoded_bytes(&self) -> u32 {
        (4 + self.events.len() * PluginEvent::ENCODED_BYTES) as u32
    }
}

/// Combined audio and event payload for a single processing block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockPayload {
    /// Audio samples for this block.
    pub audio: AudioBlock,
    /// Events to be delivered alongside the audio.
    pub events: EventPacket,
}

impl BlockPayload {
    /// Creates a new `BlockPayload` from an audio block and an event packet.
    pub fn new(audio: AudioBlock, events: EventPacket) -> Self {
        Self { audio, events }
    }
}
