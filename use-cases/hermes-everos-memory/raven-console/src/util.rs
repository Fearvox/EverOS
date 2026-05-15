use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn truncate(text: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for ch in text.chars().take(max_chars) {
        output.push(ch);
    }
    if text.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn run_id(prefix: &str) -> String {
    format!("{prefix}-{}", unix_timestamp())
}

pub fn path_for_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
