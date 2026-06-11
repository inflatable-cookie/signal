use super::*;

impl SignalRuntime {
    pub(crate) fn emit(&mut self, event: RuntimeEvent) {
        for sink in &mut self.sinks {
            sink.push(event.clone());
        }
    }
}
