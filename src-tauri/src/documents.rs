//! Bounded, local-only extraction. Documents remain untrusted evidence.
use anyhow::{bail, ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path, time::UNIX_EPOCH};

pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractedDocument {
    pub title: String,
    pub source_path: String,
    pub content_hash: String,
    pub modified_at: i64,
    pub text: String,
}

pub fn extract_document(path: &Path) -> Result<ExtractedDocument> {
    let source = path
        .canonicalize()
        .context("Cannot resolve document path")?;
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    ensure!(
        matches!(extension.as_str(), "txt" | "md" | "json"),
        "Supported document types are .txt, .md and .json"
    );
    let title = source
        .file_name()
        .and_then(|value| value.to_str())
        .context("Document name must be valid Unicode")?;
    ensure!(
        title.chars().count() <= 200 && title.len() <= 800,
        "Document name exceeds 200 characters or 800 UTF-8 bytes"
    );
    validate_text(title)?;
    let source_path = source
        .to_str()
        .context("Document path must be valid Unicode")?
        .to_owned();
    ensure!(
        std::fs::metadata(&source)?.is_file(),
        "Document must be a regular file"
    );
    let mut file = File::open(&source).context("Cannot open document")?;
    let before = file.metadata().context("Cannot read document metadata")?;
    ensure!(before.is_file(), "Document must be a regular file");
    ensure!(
        before.len() <= MAX_SOURCE_BYTES as u64,
        "Document exceeds the 1 MiB source limit"
    );
    let modified = before
        .modified()
        .context("Cannot read document modification time")?;
    let modified_at = i64::try_from(
        modified
            .duration_since(UNIX_EPOCH)
            .context("Document modification time precedes the Unix epoch")?
            .as_millis(),
    )
    .context("Document modification time is out of range")?;
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.by_ref()
        .take((MAX_SOURCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .context("Cannot read document")?;
    ensure!(
        bytes.len() <= MAX_SOURCE_BYTES,
        "Document grew beyond the 1 MiB source limit"
    );
    let after = file.metadata().context("Cannot reread document metadata")?;
    ensure!(
        before.len() == after.len()
            && bytes.len() as u64 == after.len()
            && modified == after.modified()?,
        "Document changed while being read; import it again"
    );
    let raw = std::str::from_utf8(&bytes).context("Document must contain UTF-8 text")?;
    validate_text(raw)?;
    let text = if extension == "json" {
        // serde_json's default recursion limit rejects excessively nested input.
        let value: serde_json::Value =
            serde_json::from_str(raw).context("Invalid JSON document")?;
        let mut output = String::new();
        extract_json(&value, "$", &mut output)?;
        output
    } else {
        normalize(raw)?
    };
    ensure!(
        !text.trim().is_empty(),
        "Document contains no meaningful text"
    );
    Ok(ExtractedDocument {
        title: title.to_owned(),
        source_path,
        content_hash: format!("{:x}", Sha256::digest(&bytes)),
        modified_at,
        text,
    })
}

fn validate_text(text: &str) -> Result<()> {
    ensure!(
        !text
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t')),
        "Document contains binary data or unsupported control characters"
    );
    Ok(())
}

fn normalize(text: &str) -> Result<String> {
    validate_text(text)?;
    // Replacing CRLF/CR never increases the input, but enforce the output limit
    // before allocating the normalized copy. Outer whitespace is not evidence.
    let trimmed = text.trim();
    let normalized_length = trimmed.len()
        - trimmed
            .as_bytes()
            .windows(2)
            .filter(|pair| *pair == b"\r\n")
            .count();
    ensure!(
        normalized_length <= MAX_TEXT_BYTES,
        "Extracted text exceeds the 256 KiB limit"
    );
    Ok(trimmed.replace("\r\n", "\n").replace('\r', "\n"))
}

fn append(output: &mut String, value: &str) -> Result<()> {
    ensure!(
        output
            .len()
            .checked_add(value.len())
            .is_some_and(|length| length <= MAX_TEXT_BYTES),
        "Extracted text exceeds the 256 KiB limit"
    );
    output.push_str(value);
    Ok(())
}

fn child_path(parent: &str, key: &str) -> Result<String> {
    let mut path = String::new();
    append(&mut path, parent)?;
    append(&mut path, "/")?;
    // JSON Pointer escaping keeps literal slashes distinct from nested keys.
    for part in key.split_inclusive(['~', '/']) {
        if let Some(prefix) = part.strip_suffix('~') {
            append(&mut path, prefix)?;
            append(&mut path, "~0")?;
        } else if let Some(prefix) = part.strip_suffix('/') {
            append(&mut path, prefix)?;
            append(&mut path, "~1")?;
        } else {
            append(&mut path, part)?;
        }
    }
    Ok(path)
}

fn extract_json(value: &serde_json::Value, path: &str, output: &mut String) -> Result<()> {
    match value {
        serde_json::Value::Object(values) => {
            for (key, value) in values {
                validate_text(key)?;
                extract_json(value, &child_path(path, key)?, output)?;
            }
        }
        serde_json::Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                extract_json(value, &child_path(path, &index.to_string())?, output)?;
            }
        }
        serde_json::Value::Null => {}
        _ => {
            let text = match value {
                serde_json::Value::String(text) => normalize(text)?,
                serde_json::Value::Number(number) => number.to_string(),
                serde_json::Value::Bool(boolean) => boolean.to_string(),
                _ => bail!("Unsupported JSON value"),
            };
            if !text.is_empty() {
                if !output.is_empty() {
                    append(output, "\n")?;
                }
                append(output, path)?;
                append(output, ": ")?;
                append(output, &text)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("harness-documents-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, bytes).unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn text_provenance_normalization_and_json_leaves() {
        let fixture = Fixture::new();
        let path = fixture.write("売上.MD", "  # 売上\r\n**literal**\r\n\t€42  ".as_bytes());
        let first = extract_document(&path).unwrap();
        assert_eq!(first.title, "売上.MD");
        assert_eq!(first.text, "# 売上\n**literal**\n\t€42");
        assert!(Path::new(&first.source_path).is_absolute());
        assert_eq!(first.content_hash.len(), 64);
        assert!(first.modified_at > 0);
        fs::write(&path, "changed").unwrap();
        assert_ne!(
            extract_document(&path).unwrap().content_hash,
            first.content_hash
        );
        let json = fixture.write("stats.json", br#"{"revenue":42,"enabled":true,"missing":null,"team":["Ada",{"name":"Lin"}],"a/b":"value","empty":"  "}"#);
        let extracted = extract_document(&json).unwrap();
        assert!(extracted.text.contains("$/revenue: 42"));
        assert!(extracted.text.contains("$/enabled: true"));
        assert!(extracted.text.contains("$/team/0: Ada"));
        assert!(extracted.text.contains("$/team/1/name: Lin"));
        assert!(extracted.text.contains("$/a~1b: value"));
        assert!(!extracted.text.contains("missing"));
        assert!(!extracted.text.contains("empty"));
    }

    #[test]
    fn rejects_binary_empty_unsupported_and_oversized_documents() {
        let fixture = Fixture::new();
        for (name, bytes) in [
            ("binary.txt", b"text\0binary".as_slice()),
            ("invalid.txt", b"\xff".as_slice()),
            ("empty.txt", b" \n\t".as_slice()),
            ("unsupported.pdf", b"hello".as_slice()),
            ("empty.json", b"{\"unused\":null}".as_slice()),
            ("control.json", br#"{"value":"\u0001"}"#.as_slice()),
        ] {
            assert!(
                extract_document(&fixture.write(name, bytes)).is_err(),
                "{name}"
            );
        }
        let huge = fixture.write("huge.txt", &vec![b'a'; MAX_SOURCE_BYTES + 1]);
        assert!(extract_document(&huge).is_err());
        let output = fixture.write("output.txt", &vec![b'a'; MAX_TEXT_BYTES + 1]);
        assert!(extract_document(&output).is_err());
        let expanded = fixture.write(
            "expanded.json",
            format!("{{\"large\":\"{}\"}}", "a".repeat(MAX_TEXT_BYTES)).as_bytes(),
        );
        assert!(extract_document(&expanded).is_err());
        let deep = fixture.write(
            "deep.json",
            format!("{}0{}", "[".repeat(130), "]".repeat(130)).as_bytes(),
        );
        assert!(extract_document(&deep).is_err());
        assert!(extract_document(&fixture.0).is_err());
    }
}
