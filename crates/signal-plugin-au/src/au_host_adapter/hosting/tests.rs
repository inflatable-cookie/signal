use std::path::Path;

use super::instance::AuHostedInstance;
use super::types::{
    fourcc_from_str, fourcc_to_string, parse_load_key, AuHostingError, AU_REGISTRY_COMPONENT_PATH,
};

#[test]
fn fourcc_round_trips_printable_codes() {
    let code = fourcc_from_str("aufx").expect("aufx encodes");
    assert_eq!(code, u32::from_be_bytes(*b"aufx"));
    assert_eq!(fourcc_to_string(code), "aufx");
    assert!(fourcc_from_str("toolong").is_none());
    assert!(fourcc_from_str("ab").is_none());
}

#[test]
fn load_key_parses_the_fourcc_triple() {
    let (ty, sub, mfr) = parse_load_key("aufx:dely:appl").expect("triple parses");
    assert_eq!(fourcc_to_string(ty), "aufx");
    assert_eq!(fourcc_to_string(sub), "dely");
    assert_eq!(fourcc_to_string(mfr), "appl");
    assert!(parse_load_key("aufx:dely").is_none());
    assert!(parse_load_key("aufx:dely:appl:extra").is_none());
    assert!(parse_load_key("toolong:dely:appl").is_none());
}

fn error_token(result: Result<AuHostedInstance, AuHostingError>) -> String {
    result.err().expect("expected hosting error").token
}

#[test]
fn load_rejects_malformed_keys_with_a_stable_token() {
    assert_eq!(
        error_token(AuHostedInstance::load(
            Path::new(AU_REGISTRY_COMPONENT_PATH),
            "not-a-key"
        )),
        "load_key_invalid"
    );
}

#[test]
fn load_rejects_unknown_components_with_a_stable_token() {
    assert_eq!(
        error_token(AuHostedInstance::load(
            Path::new(AU_REGISTRY_COMPONENT_PATH),
            "aufx:zzzz:zzzz"
        )),
        "component_not_found"
    );
}
