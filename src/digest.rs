// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) fn sha256(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    format!("{digest:x}")
}

/// Serialize JSON using the repository's digest contract: UTF-8, sorted object
/// keys, no insignificant whitespace, and unescaped non-ASCII text.
pub(crate) fn canonical_json(value: &Value) -> Vec<u8> {
    fn write(value: &Value, output: &mut Vec<u8>) {
        match value {
            Value::Null => output.extend_from_slice(b"null"),
            Value::Bool(value) => output.extend_from_slice(if *value { b"true" } else { b"false" }),
            Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
            Value::String(value) => output.extend_from_slice(
                serde_json::to_string(value)
                    .expect("serializing a JSON string cannot fail")
                    .as_bytes(),
            ),
            Value::Array(values) => {
                output.push(b'[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(b',');
                    }
                    write(value, output);
                }
                output.push(b']');
            }
            Value::Object(values) => {
                output.push(b'{');
                let mut keys = values.keys().collect::<Vec<_>>();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index != 0 {
                        output.push(b',');
                    }
                    output.extend_from_slice(
                        serde_json::to_string(key)
                            .expect("serializing a JSON key cannot fail")
                            .as_bytes(),
                    );
                    output.push(b':');
                    write(&values[key], output);
                }
                output.push(b'}');
            }
        }
    }

    let mut output = Vec::new();
    write(value, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::{canonical_json, sha256};
    use serde_json::json;

    #[test]
    fn sha256_matches_empty_vector() {
        assert_eq!(
            sha256([]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn canonical_json_sorts_keys_without_escaping_unicode() {
        assert_eq!(
            canonical_json(&json!({"z": "é", "a": [2, 1]})),
            "{\"a\":[2,1],\"z\":\"é\"}".as_bytes()
        );
    }
}
