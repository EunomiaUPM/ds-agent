use ymir::errors::{Errors, Outcome};

/// Keys must follow an absolute path format: `/seg1/seg2/seg3`.
/// Rules:
///  - Must start with `/`
///  - No trailing `/`
///  - No empty segments (i.e. no `//`)
///  - Each segment: `[a-zA-Z0-9_.-]+`
pub(crate) fn validate_key(key: &str) -> Outcome<()> {
    if !key.starts_with('/') {
        return Err(Errors::validation(
            format!("invalid key '{}': must start with '/'", key),
            None,
        ));
    }
    if key.len() == 1 {
        return Err(Errors::validation(
            format!("invalid key '{}': must have at least one segment", key),
            None,
        ));
    }
    if key.ends_with('/') {
        return Err(Errors::validation(
            format!("invalid key '{}': must not end with '/'", key),
            None,
        ));
    }
    for segment in key[1..].split('/') {
        if segment.is_empty() {
            return Err(Errors::validation(
                format!("invalid key '{}': empty segment (double slash)", key),
                None,
            ));
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(Errors::validation(
                format!(
                    "invalid key '{}': segment '{}' contains invalid characters (allowed: a-z A-Z 0-9 _ - .)",
                    key, segment
                ),
                None,
            ));
        }
    }
    Ok(())
}
