//! Wire token decoding and broker receipt-line parsing.

use crate::{BrokerFailureStage, RuntimeError, RuntimeErrorKind, SignalRuntime};

use super::receipt::{SandboxBrokerReceiptLine, SandboxBrokerReceiptState};

/// Decode a wire-encoded token value (see the broker's `encode_wire_token`).
pub(crate) fn decode_wire_token(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 3 <= bytes.len() {
            let hex = value.get(index + 1..index + 3);
            if let Some(byte) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

pub(crate) fn split_broker_args(value: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_arg = false;
    let mut chars = value.chars();

    'outer: while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => {
                if in_arg {
                    args.push(std::mem::take(&mut current));
                    in_arg = false;
                }
            }
            '\\' => {
                in_arg = true;
                if let Some(escaped) = chars.next() {
                    current.push(escaped);
                }
            }
            '\'' => {
                in_arg = true;
                for inner in chars.by_ref() {
                    if inner == '\'' {
                        continue 'outer;
                    }
                    current.push(inner);
                }
                break;
            }
            '"' => {
                in_arg = true;
                while let Some(inner) = chars.next() {
                    match inner {
                        '"' => continue 'outer,
                        '\\' => {
                            if let Some(escaped) = chars.next() {
                                current.push(escaped);
                            }
                        }
                        other => current.push(other),
                    }
                }
                break;
            }
            other => {
                in_arg = true;
                current.push(other);
            }
        }
    }
    if in_arg {
        args.push(current);
    }
    args
}

pub(crate) fn parse_broker_receipt_line(line: &str) -> std::io::Result<SandboxBrokerReceiptLine> {
    let mut state = None;
    let mut sandbox_id = None;
    let mut instance_id = None;
    let mut processing_epoch = None;
    let mut lease_id = None;
    let mut region_id = None;
    let mut extra = Vec::new();
    let mut detail = None;

    for token in line.split_whitespace().skip(1) {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "state" => state = Some(SandboxBrokerReceiptState::parse(value)),
            "sandbox_id" => sandbox_id = Some(value.to_string()),
            "instance_id" if value != "-" => instance_id = Some(value.to_string()),
            "epoch" if value != "-" => processing_epoch = value.parse::<u64>().ok(),
            "lease_id" if value != "-" => lease_id = Some(value.to_string()),
            "region_id" if value != "-" => region_id = Some(value.to_string()),
            "detail" => detail = Some(value.to_string()),
            other => extra.push((other.to_string(), value.to_string())),
        }
    }

    Ok(SandboxBrokerReceiptLine {
        state: state.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing state",
            )
        })?,
        sandbox_id: sandbox_id.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing sandbox_id",
            )
        })?,
        instance_id,
        processing_epoch,
        lease_id,
        region_id,
        extra,
        detail: detail.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing detail",
            )
        })?,
    })
}

pub(crate) fn io_runtime_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, error.to_string())
}

pub(crate) fn record_broker_failure_and_convert(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<String>,
    processing_epoch: Option<u64>,
    block_sequence: Option<u64>,
    stage: BrokerFailureStage,
    error: std::io::Error,
) -> RuntimeError {
    let detail = error.to_string();
    runtime.record_broker_failure(
        sandbox_id,
        lease_id,
        processing_epoch,
        block_sequence,
        stage,
        detail.clone(),
    );
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, detail)
}
