use crate::PluginEvent;

#[derive(Clone, Debug, PartialEq)]
pub struct AudioBlock {
    pub channel_count: u16,
    pub frame_count: u32,
    pub samples: Vec<f32>,
}

impl AudioBlock {
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

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn first_sample(&self) -> Option<f32> {
        self.samples.first().copied()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventPacket {
    pub events: Vec<PluginEvent>,
}

impl EventPacket {
    pub fn new(events: Vec<PluginEvent>) -> Self {
        Self { events }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn encoded_bytes(&self) -> u32 {
        (4 + self.events.len() * PluginEvent::ENCODED_BYTES) as u32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockPayload {
    pub audio: AudioBlock,
    pub events: EventPacket,
}

impl BlockPayload {
    pub fn new(audio: AudioBlock, events: EventPacket) -> Self {
        Self { audio, events }
    }
}
