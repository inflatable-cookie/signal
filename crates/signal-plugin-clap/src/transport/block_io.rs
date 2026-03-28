use std::io;

use signal_ipc::{SharedMemoryBroker, SharedMemoryTransportPayload};
use signal_plugin::{
    AudioBlock, BlockDispatch, BlockPayload, BlockProcessResult, EventPacket,
};

use crate::adapter::BrokeredBlockOutcome;
use crate::protocol::ClapBlockProtocol;

impl ClapBlockProtocol {
    pub fn write_block_dispatch(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
    ) -> io::Result<()> {
        self.write_block_payload(
            broker,
            transport,
            dispatch,
            &BlockPayload::new(
                AudioBlock::new(
                    dispatch.header.channel_count,
                    dispatch.header.frame_count,
                    vec![
                        0.0;
                        dispatch.header.channel_count as usize
                            * dispatch.header.frame_count as usize
                    ],
                )
                .expect("silence block"),
                EventPacket::new(Vec::new()),
            ),
        )
    }

    pub fn write_block_payload(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
        payload: &BlockPayload,
    ) -> io::Result<()> {
        let mut region = broker.attach_region(transport)?;
        dispatch
            .write_to_shared_memory(region.as_mut_slice())
            .map_err(io::Error::other)?;
        dispatch
            .write_input_payload(region.as_mut_slice(), payload)
            .map_err(io::Error::other)?;
        BlockProcessResult::ready_for(dispatch)
            .write_to_shared_memory(dispatch.layout, region.as_mut_slice())
            .map_err(io::Error::other)?;
        region.flush()
    }

    pub fn read_block_result(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        frame_count: u32,
    ) -> io::Result<BlockProcessResult> {
        let region = broker.attach_region(transport)?;
        let layout = self.shared_memory_layout(frame_count);
        BlockProcessResult::read_from_shared_memory(layout, region.as_slice())
            .map_err(io::Error::other)
    }

    pub fn read_block_outcome(
        &self,
        broker: &SharedMemoryBroker,
        transport: &SharedMemoryTransportPayload,
        dispatch: &BlockDispatch,
    ) -> io::Result<BrokeredBlockOutcome> {
        let region = broker.attach_region(transport)?;
        let input = dispatch
            .read_input_payload(region.as_slice())
            .map_err(io::Error::other)?;
        let output = dispatch
            .read_output_payload(region.as_slice())
            .map_err(io::Error::other)?;
        let result =
            BlockProcessResult::read_from_shared_memory(dispatch.layout, region.as_slice())
                .map_err(io::Error::other)?;

        Ok(BrokeredBlockOutcome {
            dispatch: dispatch.clone(),
            input,
            output,
            result,
        })
    }
}
