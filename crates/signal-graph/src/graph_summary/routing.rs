use super::*;
use std::collections::{BTreeMap, BTreeSet};

impl ExecutableGraph {
    pub fn routing_summary(&self) -> GraphRoutingSummary {
        let mut producer_counts = BTreeMap::<String, usize>::new();
        let mut consumer_counts = BTreeMap::<String, usize>::new();
        let mut bus_latency = BTreeMap::<String, u32>::new();
        let mut bus_tail = BTreeMap::<String, u32>::new();
        let mut max_bus_latency_samples = 0;
        let mut max_bus_tail_samples = 0;

        bus_latency.insert("main:in".into(), 0);
        bus_tail.insert("main:in".into(), 0);

        for node in &self.plan.nodes {
            *producer_counts
                .entry(node.buffer_contract.output.bus_id.clone())
                .or_default() += 1;
            *consumer_counts
                .entry(node.buffer_contract.input.bus_id.clone())
                .or_default() += 1;

            let input_latency = bus_latency
                .get(&node.buffer_contract.input.bus_id)
                .copied()
                .unwrap_or(0);
            let input_tail = bus_tail
                .get(&node.buffer_contract.input.bus_id)
                .copied()
                .unwrap_or(0);
            let output_latency = input_latency.saturating_add(node.latency_samples);
            let output_tail = input_tail.saturating_add(node.tail_samples);
            let entry = bus_latency
                .entry(node.buffer_contract.output.bus_id.clone())
                .or_default();
            *entry = (*entry).max(output_latency);
            max_bus_latency_samples = max_bus_latency_samples.max(*entry);
            let tail_entry = bus_tail
                .entry(node.buffer_contract.output.bus_id.clone())
                .or_default();
            *tail_entry = (*tail_entry).max(output_tail);
            max_bus_tail_samples = max_bus_tail_samples.max(*tail_entry);
        }

        let mut routed_bus_ids = BTreeSet::new();
        routed_bus_ids.extend(producer_counts.keys().cloned());
        routed_bus_ids.extend(consumer_counts.keys().cloned());

        let fan_in_bus_count = producer_counts.values().filter(|count| **count > 1).count();
        let fan_out_bus_count = consumer_counts
            .iter()
            .filter(|(bus_id, count)| bus_id.as_str() != "main:out" && **count > 1)
            .count();
        let direct_edge_count = routed_bus_ids
            .iter()
            .filter(|bus_id| {
                bus_id.as_str() != "main:in"
                    && producer_counts.get(*bus_id).copied().unwrap_or(0) == 1
                    && consumer_counts.get(*bus_id).copied().unwrap_or(0) == 1
            })
            .count();

        GraphRoutingSummary {
            routed_bus_count: routed_bus_ids.len(),
            direct_edge_count,
            fan_in_bus_count,
            fan_out_bus_count,
            mixed_bus_count: fan_in_bus_count,
            output_latency_samples: bus_latency.get("main:out").copied().unwrap_or(0),
            max_bus_latency_samples,
            output_tail_samples: bus_tail.get("main:out").copied().unwrap_or(0),
            max_bus_tail_samples,
        }
    }
}
