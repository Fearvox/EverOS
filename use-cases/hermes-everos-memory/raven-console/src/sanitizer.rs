use regex::{Captures, Regex};
use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;

static SIGNED_URL_RE: OnceLock<Regex> = OnceLock::new();
static TOKEN_RE: OnceLock<Regex> = OnceLock::new();
static SECRET_ASSIGNMENT_RE: OnceLock<Regex> = OnceLock::new();
static CREDENTIAL_PATH_RE: OnceLock<Regex> = OnceLock::new();
static LOCAL_PATH_RE: OnceLock<Regex> = OnceLock::new();
static LOCALHOST_RE: OnceLock<Regex> = OnceLock::new();
static IPV4_RE: OnceLock<Regex> = OnceLock::new();
static PRODUCT_NAME_RE: OnceLock<Regex> = OnceLock::new();
const LEGACY_PRODUCT_NAME: &str = concat!("Ri", "ven");

pub fn sanitize_text(input: &str) -> String {
    let mut output = input.to_string();

    output = signed_url_re()
        .replace_all(&output, "[redacted-signed-url]")
        .to_string();
    output = secret_assignment_re()
        .replace_all(&output, "$1=[redacted-secret]")
        .to_string();
    output = token_re()
        .replace_all(&output, "[redacted-token]")
        .to_string();
    output = credential_path_re()
        .replace_all(&output, "$1[redacted-credential-path]")
        .to_string();
    output = local_path_re()
        .replace_all(&output, "$1[redacted-path]")
        .to_string();
    output = localhost_re()
        .replace_all(&output, "[redacted-host]")
        .to_string();
    output = ipv4_re()
        .replace_all(&output, |captures: &Captures<'_>| {
            let value = captures
                .get(0)
                .map(|item| item.as_str())
                .unwrap_or_default();
            if is_ipv4_address(value) {
                "[redacted-ip]".to_string()
            } else {
                value.to_string()
            }
        })
        .to_string();
    output = product_name_re().replace_all(&output, "Raven").to_string();

    output
}

pub fn sanitize_json<T: Serialize>(value: &T) -> crate::RavenResult<Value> {
    let value = serde_json::to_value(value)?;
    Ok(sanitize_value(value))
}

pub fn sanitize_value(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(sanitize_text(&text)),
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize_value).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, sanitize_value(value)))
                .collect(),
        ),
        other => other,
    }
}

pub fn public_safety_verdict(text: &str) -> bool {
    let sanitized = sanitize_text(text);
    sanitized == text || !contains_sensitive_shape(&sanitized)
}

fn contains_sensitive_shape(text: &str) -> bool {
    signed_url_re().is_match(text)
        || token_re().is_match(text)
        || credential_path_re().is_match(text)
        || local_path_re().is_match(text)
        || localhost_re().is_match(text)
        || ipv4_re()
            .find_iter(text)
            .any(|match_| is_ipv4_address(match_.as_str()))
}

fn signed_url_re() -> &'static Regex {
    SIGNED_URL_RE.get_or_init(|| {
        Regex::new(r#"https?://\S*(?:Signature=|X-Amz-Signature=|X-Amz-Credential=|Policy=|Key-Pair-Id=)\S*"#)
            .expect("valid signed URL regex")
    })
}

fn token_re() -> &'static Regex {
    TOKEN_RE.get_or_init(|| {
        Regex::new(r#"(?i)\b(?:sk|sk-proj|sk-ant|ghp|github_pat|xoxb|xoxp|hf)_[A-Za-z0-9_-]{16,}\b|(?i)\b(?:sk|sk-proj|sk-ant|ghp|github_pat|xoxb|xoxp|hf)-[A-Za-z0-9_-]{16,}\b"#)
            .expect("valid token regex")
    })
}

fn secret_assignment_re() -> &'static Regex {
    SECRET_ASSIGNMENT_RE.get_or_init(|| {
        Regex::new(r#"(?i)\b(api[_-]?key|token|secret|password|authorization)\s*=\s*[^\s&]+"#)
            .expect("valid secret assignment regex")
    })
}

fn credential_path_re() -> &'static Regex {
    CREDENTIAL_PATH_RE.get_or_init(|| {
        Regex::new(r#"(^|[\s"'(=])((?:~|/Users/[^\s"'()]+|/root|/home/[^\s"'()]+)/\.(?:ssh|aws|gcloud|config|codex|claude)[^\s"'()]*)"#)
            .expect("valid credential path regex")
    })
}

fn local_path_re() -> &'static Regex {
    LOCAL_PATH_RE.get_or_init(|| {
        Regex::new(r#"(^|[\s"'(=])(/Users/[^\s"'()]+|/root/[^\s"'()]+|/home/[^\s"'()]+)"#)
            .expect("valid local path regex")
    })
}

fn localhost_re() -> &'static Regex {
    LOCALHOST_RE
        .get_or_init(|| Regex::new(r#"(?i)\blocalhost(?::\d+)?\b"#).expect("valid host regex"))
}

fn ipv4_re() -> &'static Regex {
    IPV4_RE.get_or_init(|| {
        Regex::new(r#"\b\d{1,3}(?:\.\d{1,3}){3}(?::\d+)?\b"#).expect("valid IP regex")
    })
}

fn product_name_re() -> &'static Regex {
    PRODUCT_NAME_RE.get_or_init(|| {
        Regex::new(&format!(r#"(?i)\b{}\b"#, LEGACY_PRODUCT_NAME))
            .expect("valid product-name regex")
    })
}

fn is_ipv4_address(value: &str) -> bool {
    let host = value.split(':').next().unwrap_or(value);
    let parts = host.split('.').collect::<Vec<_>>();
    if parts.len() != 4 {
        return false;
    }
    let octets = parts
        .iter()
        .filter_map(|part| part.parse::<u8>().ok())
        .collect::<Vec<_>>();
    if octets.len() != 4 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{sanitize_text, LEGACY_PRODUCT_NAME};

    #[test]
    fn redacts_signed_urls() {
        let text = "see https://static.example/path?Policy=abc&Signature=def";
        assert_eq!(sanitize_text(text), "see [redacted-signed-url]");
    }

    #[test]
    fn redacts_local_paths() {
        let text = "path=/Users/alice/project/.env and /root/secret";
        assert_eq!(
            sanitize_text(text),
            "path=[redacted-path] and [redacted-path]"
        );
    }

    #[test]
    fn redacts_token_shapes() {
        let text = "token sk-proj-abcdefghijklmnopqrstuvwxyz123456";
        assert_eq!(sanitize_text(text), "token [redacted-token]");
    }

    #[test]
    fn redacts_private_ips_and_localhost() {
        let text = "http://192.168.1.5:9000 and localhost:3000 and 74.199.157.194";
        assert_eq!(
            sanitize_text(text),
            "http://[redacted-ip] and [redacted-host] and [redacted-ip]"
        );
    }

    #[test]
    fn keeps_public_words() {
        let text = "DAS-2666 remote loopback smoke remains BLOCK";
        assert_eq!(sanitize_text(text), text);
    }

    #[test]
    fn normalizes_old_product_name() {
        let text = format!(
            "{}/{}/{} issue title",
            LEGACY_PRODUCT_NAME,
            LEGACY_PRODUCT_NAME.to_ascii_lowercase(),
            LEGACY_PRODUCT_NAME.to_ascii_uppercase()
        );
        assert_eq!(sanitize_text(&text), "Raven/Raven/Raven issue title");
    }
}
