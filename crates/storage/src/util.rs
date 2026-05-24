pub(crate) fn clean_optional_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "-" {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_value_returns_none() {
        assert_eq!(clean_optional_value(""), None);
        assert_eq!(clean_optional_value("   "), None);
    }

    #[test]
    fn dash_value_returns_none() {
        assert_eq!(clean_optional_value("-"), None);
    }

    #[test]
    fn meaningful_value_is_returned_trimmed() {
        assert_eq!(clean_optional_value("  foo "), Some("foo".to_owned()));
    }
}
