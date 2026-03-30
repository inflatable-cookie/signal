#[path = "receipt_summary/complex_io_pin_matrix.rs"]
mod complex_io_pin_matrix;
#[path = "receipt_summary/recall_supervisor.rs"]
mod recall_supervisor;
#[path = "receipt_summary/setup.rs"]
mod setup;

#[test]
fn runtime_plugin_chain_snapshot_reports_compensation_and_recall() {
    let runtime = setup::build_plugin_chain_receipt_runtime();
    complex_io_pin_matrix::assert_complex_io_and_pin_matrix_receipts(&runtime);
    recall_supervisor::assert_compensation_recall_and_supervisor_receipts(&runtime);
}
