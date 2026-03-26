use super::*;

impl RuntimeEventSink for RuntimeEventRecorder {
    fn push(&mut self, event: RuntimeEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}
