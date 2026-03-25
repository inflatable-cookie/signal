use super::types::{ConformanceMatrixEntry, ConformanceMatrixEntryKind};

pub(crate) fn conformance_matrix_entries() -> &'static [ConformanceMatrixEntry] {
    &[
        ConformanceMatrixEntry {
            id: "runtime-public-contract-boundary",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-runtime",
            surface: "SignalRuntime, RuntimeObservationReport, RuntimeSupervisorReport public reexports",
            command:
                "cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style consumer can capture runtime/export/plugin receipts without private internals.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-discovery-consumer",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-supervisor-tools",
            surface: "signal.supervisor.export JSON carrying runtime-owned plugin discovery catalog",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog",
            rationale:
                "Proves the versioned supervisor export carries the widened discovery boundary without host-local reconstruction.",
        },
        ConformanceMatrixEntry {
            id: "plugin-backend-breadth-coverage",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-runtime + signal-supervisor-tools",
            surface: "runtime reexports and supervisor export carrying backend-neutral plugin discovery coverage aggregates",
            command: "effigy acceptance:plugin-backend-breadth",
            rationale:
                "Proves widened multi-format discovery and capability coverage stays consumable through Signal-owned runtime and export surfaces.",
        },
        ConformanceMatrixEntry {
            id: "shared-host-edge-consumer",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-host-local + signal-host-server",
            surface: "shared-stable host constructors, RuntimeSupervisorApi, and supervisor_report()",
            command: "effigy acceptance:host-edge-consumer",
            rationale:
                "Proves the shared stable host edge remains consumable without private host internals or unstable summary helpers.",
        },
        ConformanceMatrixEntry {
            id: "runtime-supervisor-report-demo",
            kind: ConformanceMatrixEntryKind::Example,
            crate_name: "signal-runtime",
            surface: "supervisor_report_demo example",
            command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "Provides a host-free runnable example that emits the stabilized supervisor report surface.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-schema-description",
            kind: ConformanceMatrixEntryKind::Introspection,
            crate_name: "signal-supervisor-tools",
            surface: "signal-supervisor-tools export/conformance schema description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Lets consumers inspect the runnable conformance matrix without reading private implementation detail.",
        },
    ]
}
