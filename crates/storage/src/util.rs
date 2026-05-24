pub(crate) fn clean_optional_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "-" {
        None
    } else {
        Some(value.to_owned())
    }
}
