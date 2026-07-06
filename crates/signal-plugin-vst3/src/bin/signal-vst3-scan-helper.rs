fn main() {
    let code = signal_plugin_vst3::vst3_scan_helper_main(std::env::args_os().skip(1));
    std::process::exit(code);
}
