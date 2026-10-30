use serde_json::Value;

// people are written "Last, First" (organizations as is, no comma) and
// displayed "First Last". only the first comma splits the name.
pub fn display_name(name: &str) -> String {
    match name.split_once(',') {
        Some((last, first)) if !first.trim().is_empty() => {
            format!("{} {}", first.trim(), last.trim())
        }
        Some((last, _)) => last.trim().to_string(),
        None => name.trim().to_string(),
    }
}

// the names in an `author` or `reader` value: one string, or a list of them
pub fn names(value: &Value) -> Vec<String> {
    match value {
        Value::String(name) => vec![name.clone()],
        Value::Array(list) => list
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

// display names joined as a sentence: "A", "A and B", "A, B, and C"
pub fn display_names(value: &Value) -> String {
    let display: Vec<String> = names(value).iter().map(|n| display_name(n)).collect();
    match display.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {}", rest.join(", "), last),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn display_name_flips_last_first() {
        assert_eq!(display_name("Vasquez, Maria"), "Maria Vasquez");
        assert_eq!(display_name("Smith, Mary Ann"), "Mary Ann Smith");
    }

    #[test]
    fn display_name_as_written_without_comma() {
        assert_eq!(display_name("Maria Vasquez"), "Maria Vasquez");
        assert_eq!(
            display_name("The Daystrom Institute"),
            "The Daystrom Institute"
        );
        assert_eq!(display_name("Nichelle N."), "Nichelle N.");
    }

    #[test]
    fn display_name_trailing_comma() {
        assert_eq!(display_name("Cher,"), "Cher");
    }

    #[test]
    fn display_names_joins_as_sentence() {
        assert_eq!(display_names(&json!("Smith, Mary Ann")), "Mary Ann Smith");
        assert_eq!(
            display_names(&json!(["Smith, Mary Ann", "Farmer, Jane"])),
            "Mary Ann Smith and Jane Farmer"
        );
        assert_eq!(
            display_names(&json!(["Smith, Mary Ann", "Farmer, Jane", "Lovelace, Ada"])),
            "Mary Ann Smith, Jane Farmer, and Ada Lovelace"
        );
    }

    #[test]
    fn display_names_empty_for_non_names() {
        assert_eq!(display_names(&json!(null)), "");
        assert_eq!(display_names(&json!([])), "");
        assert_eq!(display_names(&json!(42)), "");
    }
}
