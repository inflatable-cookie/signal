pub(crate) fn json_escape_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

pub(crate) fn json_option_string(value: Option<&str>) -> String {
    match value {
        Some(value) => json_escape_string(value),
        None => "null".into(),
    }
}

pub(crate) fn json_option_u64(value: Option<u64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_option_u32(value: Option<u32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_option_usize(value: Option<usize>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_option_i64(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_option_f64(value: Option<f64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_u64_vec(values: &[u64]) -> String {
    let joined = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_option_f32(value: Option<f32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

pub(crate) fn json_string(value: &str) -> String {
    json_option_string(Some(value))
}

pub(crate) fn json_string_vec(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_option_string(Some(value.as_str())))
            .collect::<Vec<_>>()
            .join(",")
    )
}
