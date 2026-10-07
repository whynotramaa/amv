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
        matches!(extension.as_str(), "txt" | "md" | "json" | "docx"),
        "Supported document types are .txt, .md, .json and .docx"
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
    let text = if extension == "docx" {
        extract_docx(&bytes)?
    } else {
        let raw = std::str::from_utf8(&bytes).context("Document must contain UTF-8 text")?;
        validate_text(raw)?;
        if extension == "json" {
            // serde_json's default recursion limit rejects excessively nested input.
            let value: serde_json::Value =
                serde_json::from_str(raw).context("Invalid JSON document")?;
            let mut output = String::new();
            extract_json(&value, "$", &mut output)?;
            output
        } else {
            normalize(raw)?
        }
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

// Main-body text only: no relationships, headers, notes, images or embedded objects.
fn extract_docx(bytes: &[u8]) -> Result<String> {
    validate_docx_zip_footer(bytes)?;
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).context("Invalid DOCX ZIP archive")?;
    ensure!(archive.len() <= 4096, "DOCX contains too many ZIP entries");
    // ponytail: zip canonicalizes duplicate names; no second ZIP-directory parser.
    let mut document = archive
        .by_name("word/document.xml")
        .context("DOCX must contain an unencrypted word/document.xml")?;
    ensure!(!document.encrypted(), "Encrypted DOCX is unsupported");
    ensure!(
        document.size() <= MAX_SOURCE_BYTES as u64,
        "DOCX main XML exceeds the 1 MiB decoded limit"
    );
    let mut xml = Vec::with_capacity(document.size() as usize);
    document
        .by_ref()
        .take((MAX_SOURCE_BYTES + 1) as u64)
        .read_to_end(&mut xml)
        .context("Cannot decode DOCX main XML")?;
    ensure!(
        xml.len() <= MAX_SOURCE_BYTES,
        "DOCX main XML exceeds the 1 MiB decoded limit"
    );
    let xml = std::str::from_utf8(&xml).context("DOCX main XML must use UTF-8")?;
    validate_xml_text(xml)?;
    extract_word_xml(xml)
}

// Check advertised entry counts before zip allocates its central-directory metadata.
// This intentionally accepts a single-footer ZIP32 subset, not ambiguous/ZIP64 files.
fn validate_docx_zip_footer(bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() <= MAX_SOURCE_BYTES,
        "DOCX exceeds the 1 MiB source limit"
    );
    let mut footer_position = None;
    for (position, signature) in bytes.windows(4).enumerate() {
        ensure!(
            !matches!(signature, b"PK\x06\x06" | b"PK\x06\x07"),
            "DOCX ZIP64 packages are unsupported"
        );
        if signature == b"PK\x05\x06" {
            ensure!(
                footer_position.replace(position).is_none(),
                "DOCX ZIP contains ambiguous end records"
            );
        }
    }
    let position = footer_position.context("DOCX ZIP end record is missing")?;
    let footer = &bytes[position..];
    ensure!(footer.len() >= 22, "DOCX ZIP end record is truncated");
    let short = |offset| u16::from_le_bytes([footer[offset], footer[offset + 1]]);
    let long = |offset| {
        u32::from_le_bytes([
            footer[offset],
            footer[offset + 1],
            footer[offset + 2],
            footer[offset + 3],
        ])
    };
    ensure!(
        footer.len() == 22 + usize::from(short(20)),
        "DOCX ZIP end record or comment does not end at EOF"
    );
    ensure!(
        short(4) == 0 && short(6) == 0,
        "DOCX multi-disk ZIP packages are unsupported"
    );
    let entries = short(10);
    ensure!(
        entries != u16::MAX && short(8) != u16::MAX && long(12) != u32::MAX && long(16) != u32::MAX,
        "DOCX ZIP64 packages are unsupported"
    );
    ensure!(
        entries > 0 && entries <= 4096 && short(8) == entries,
        "DOCX ZIP entry counts are invalid or exceed 4096"
    );
    let directory_end = u64::from(long(16)) + u64::from(long(12));
    ensure!(
        long(12) > 0 && directory_end <= position as u64,
        "DOCX ZIP central directory exceeds its end record"
    );
    Ok(())
}

fn validate_xml_text(text: &str) -> Result<()> {
    validate_text(text)?;
    ensure!(
        !text.chars().any(|c| matches!(c, '\u{fffe}' | '\u{ffff}')),
        "DOCX XML contains an illegal XML character"
    );
    Ok(())
}

fn extract_word_xml(xml: &str) -> Result<String> {
    use quick_xml::{events::Event, name::ResolveResult, reader::NsReader, XmlVersion};
    #[derive(Clone, Copy, PartialEq)]
    enum Kind {
        Document,
        Body,
        Text,
        Paragraph,
        Cell,
        Row,
        Other,
    }
    #[derive(Clone, Copy)]
    struct Frame {
        kind: Kind,
        excluded: bool,
        body: bool,
    }
    let mut reader = NsReader::from_str(xml);
    reader.config_mut().expand_empty_elements = true;
    let mut frames: Vec<Frame> = Vec::new();
    let mut output = String::new();
    let mut root_seen = false;
    let mut body_seen = false;
    let mut declarations = 0;
    for _ in 0..100_000 {
        let (namespace, event) = reader.read_resolved_event().context("Malformed DOCX XML")?;
        ensure!(
            !matches!(namespace, ResolveResult::Unknown(_)),
            "DOCX XML has an unbound namespace"
        );
        let markup_compatibility = matches!(&namespace, ResolveResult::Bound(ns)
            if ns.as_ref() == "http://schemas.openxmlformats.org/markup-compatibility/2006");
        let word = matches!(namespace, ResolveResult::Bound(ns) if matches!(ns.as_ref(),
            "http://schemas.openxmlformats.org/wordprocessingml/2006/main" |
            "http://purl.oclc.org/ooxml/wordprocessingml/main"));
        match event {
            Event::Start(element) => {
                ensure!(frames.len() < 128, "DOCX XML nesting exceeds 128 levels");
                for attribute in element.attributes() {
                    let attribute = attribute.context("Malformed DOCX XML attributes")?;
                    ensure!(
                        !attribute.value.contains('<'),
                        "DOCX XML attribute contains a literal <"
                    );
                    ensure!(
                        !matches!(
                            reader.resolver().resolve_attribute(attribute.key).0,
                            ResolveResult::Unknown(_)
                        ),
                        "DOCX XML attribute has an unbound namespace"
                    );
                    validate_xml_text(&attribute.normalized_value(XmlVersion::Explicit1_0)?)?;
                }
                let local = element.local_name();
                let name = local.as_ref();
                ensure!(
                    !(markup_compatibility && name == "AlternateContent"),
                    "DOCX alternate content is unsupported; export a plain-text copy"
                );
                let kind = if word {
                    match name {
                        "document" => Kind::Document,
                        "body" => Kind::Body,
                        "t" => Kind::Text,
                        "p" => Kind::Paragraph,
                        "tc" => Kind::Cell,
                        "tr" => Kind::Row,
                        _ => Kind::Other,
                    }
                } else {
                    Kind::Other
                };
                if frames.is_empty() {
                    ensure!(
                        !root_seen && kind == Kind::Document,
                        "DOCX XML needs one Word document root"
                    );
                    root_seen = true;
                }
                let parent = frames.last().copied();
                if kind == Kind::Body {
                    ensure!(
                        !body_seen && parent.is_some_and(|p| p.kind == Kind::Document),
                        "DOCX XML needs one main body"
                    );
                    body_seen = true;
                }
                let excluded = parent.is_some_and(|p| p.excluded)
                    || (word
                        && matches!(
                            name,
                            "del" | "moveFrom" | "instrText" | "drawing" | "pict" | "object"
                        ));
                let body = kind == Kind::Body || parent.is_some_and(|p| p.body);
                if body && !excluded && word {
                    match name {
                        "tab" => append(&mut output, "\t")?,
                        "br" | "cr" => append(&mut output, "\n")?,
                        _ => {}
                    }
                }
                frames.push(Frame {
                    kind,
                    excluded,
                    body,
                });
            }
            Event::End(_) => {
                let frame = frames.pop().context("Unexpected DOCX XML closing tag")?;
                if frame.body && !frame.excluded {
                    match frame.kind {
                        Kind::Paragraph | Kind::Row => append(&mut output, "\n")?,
                        Kind::Cell => {
                            if output.ends_with('\n') {
                                output.pop();
                            }
                            append(&mut output, "\t")?;
                        }
                        _ => {}
                    }
                }
            }
            Event::Text(text) => {
                let text = text.xml10_content();
                if frames
                    .last()
                    .is_some_and(|p| p.body && !p.excluded && p.kind == Kind::Text)
                {
                    append(&mut output, &text)?;
                } else if frames.is_empty() {
                    ensure!(text.trim().is_empty(), "Text outside DOCX XML root");
                }
            }
            Event::CData(text) => {
                ensure!(!frames.is_empty(), "CDATA outside DOCX XML root");
                if frames
                    .last()
                    .is_some_and(|p| p.body && !p.excluded && p.kind == Kind::Text)
                {
                    append(&mut output, &text.xml10_content())?;
                }
            }
            Event::GeneralRef(reference) => {
                // Only predefined XML entities and legal numeric references are accepted.
                let escaped = format!("&{};", reference.as_ref());
                let text =
                    quick_xml::escape::unescape(&escaped).context("Unsupported DOCX XML entity")?;
                validate_xml_text(&text)?;
                ensure!(!frames.is_empty(), "Entity outside DOCX XML root");
                if frames
                    .last()
                    .is_some_and(|p| p.body && !p.excluded && p.kind == Kind::Text)
                {
                    append(&mut output, &text)?;
                }
            }
            Event::Decl(declaration) => {
                declarations += 1;
                ensure!(
                    declarations == 1 && !root_seen,
                    "Misplaced DOCX XML declaration"
                );
                ensure!(
                    declaration.version()?.as_ref() == "1.0",
                    "DOCX supports XML 1.0 only"
                );
                if let Some(encoding) = declaration.encoding() {
                    ensure!(
                        encoding?.eq_ignore_ascii_case("UTF-8"),
                        "DOCX main XML must use UTF-8"
                    );
                }
            }
            Event::DocType(_) => bail!("DOCX XML DTDs are unsupported"),
            Event::PI(_) => bail!("DOCX XML processing instructions are unsupported"),
            Event::Eof => {
                ensure!(
                    frames.is_empty() && root_seen && body_seen,
                    "Incomplete DOCX XML document"
                );
                return normalize(&output);
            }
            _ => {}
        }
    }
    bail!("DOCX XML exceeds 100,000 events")
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

    fn docx(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(
                    *name,
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated),
                )
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn word(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
        )
    }

    #[test]
    fn docx_main_body_unicode_entities_tables_and_ignored_content() {
        let fixture = Fixture::new();
        let xml = word(
            r#"<w:p><w:r><w:t xml:space="preserve">売上 &amp; &#x20AC;42</w:t><w:tab/><w:t>Q1</w:t><w:br/><w:t><![CDATA[literal <text>]]></w:t></w:r></w:p><w:p><w:del><w:r><w:t>DELETED</w:t></w:r></w:del><w:r><w:instrText>SECRET FIELD</w:instrText><w:t>Result</w:t></w:r><w:drawing><w:t>IMAGE</w:t></w:drawing></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:hyperlink><w:r><w:t>Link label</w:t></w:r></w:hyperlink></w:p>"#,
        );
        let bytes = docx(&[
            ("word/document.xml", xml.as_bytes()),
            ("word/header1.xml", b"HEADER"),
            ("word/footnotes.xml", b"FOOTNOTE"),
            ("word/_rels/document.xml.rels", br#"<Relationships><Relationship Target="https://example.invalid" TargetMode="External"/></Relationships>"#),
            ("../escape.txt", b"DO NOT UNPACK"),
        ]);
        let path = fixture.write("sales.DOCX", &bytes);
        let extracted = extract_document(&path).unwrap();
        assert_eq!(
            extracted.text,
            "売上 & €42\tQ1\nliteral <text>\nResult\nA\tB\t\nLink label"
        );
        assert_eq!(
            extracted.content_hash,
            format!("{:x}", Sha256::digest(&bytes))
        );
        assert_eq!(extracted.title, "sales.DOCX");
        assert!(!fixture.0.join("escape.txt").exists());
        let strict = xml.replace(
            "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
            "http://purl.oclc.org/ooxml/wordprocessingml/main",
        );
        assert_eq!(
            extract_document(&fixture.write(
                "strict.docx",
                &docx(&[("word/document.xml", strict.as_bytes())])
            ))
            .unwrap()
            .text,
            extracted.text
        );
    }

    #[test]
    fn docx_zip_footer_bounds_metadata_before_archive_allocation() {
        let fixture = Fixture::new();
        let valid = docx(&[(
            "word/document.xml",
            word("<w:p><w:r><w:t>ok</w:t></w:r></w:p>").as_bytes(),
        )]);
        let footer = valid.windows(4).position(|w| w == b"PK\x05\x06").unwrap();
        assert!(validate_docx_zip_footer(&valid).is_ok());
        let mut invalid = Vec::new();
        let mut count = valid.clone();
        count[footer + 8..footer + 10].copy_from_slice(&4097u16.to_le_bytes());
        count[footer + 10..footer + 12].copy_from_slice(&4097u16.to_le_bytes());
        invalid.push(count);
        let mut inconsistent = valid.clone();
        inconsistent[footer + 8..footer + 10].copy_from_slice(&2u16.to_le_bytes());
        invalid.push(inconsistent);
        let mut disk = valid.clone();
        disk[footer + 4..footer + 6].copy_from_slice(&1u16.to_le_bytes());
        invalid.push(disk);
        let mut ambiguous = valid.clone();
        ambiguous.extend_from_slice(&valid[footer..]);
        invalid.push(ambiguous);
        invalid.push(valid[..valid.len() - 1].to_vec());
        invalid.push(valid[..footer].to_vec());
        let mut trailing = valid.clone();
        trailing.push(0);
        invalid.push(trailing);
        let mut directory = valid.clone();
        directory[footer + 16..footer + 20].copy_from_slice(&(valid.len() as u32).to_le_bytes());
        invalid.push(directory);
        let mut sentinel = valid.clone();
        sentinel[footer + 10..footer + 12].copy_from_slice(&u16::MAX.to_le_bytes());
        invalid.push(sentinel);
        // A padded package can advertise enormous ZIP64 metadata while its source
        // remains 1 MiB. Refuse it before ZipArchive::new can trust its count.
        let mut zip64 = vec![0; MAX_SOURCE_BYTES - 98];
        let zip64_position = zip64.len() as u64;
        zip64.extend_from_slice(b"PK\x06\x06");
        zip64.extend_from_slice(&44u64.to_le_bytes());
        zip64.extend_from_slice(&[45, 0, 45, 0]);
        zip64.extend_from_slice(&[0; 8]);
        zip64.extend_from_slice(&1_000_000u64.to_le_bytes());
        zip64.extend_from_slice(&1_000_000u64.to_le_bytes());
        zip64.extend_from_slice(&46u64.to_le_bytes());
        zip64.extend_from_slice(&0u64.to_le_bytes());
        zip64.extend_from_slice(b"PK\x06\x07");
        zip64.extend_from_slice(&0u32.to_le_bytes());
        zip64.extend_from_slice(&zip64_position.to_le_bytes());
        zip64.extend_from_slice(&1u32.to_le_bytes());
        zip64.extend_from_slice(&valid[footer..]);
        let zip64_footer = zip64.len() - 22;
        zip64[zip64_footer + 10..zip64_footer + 12].copy_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(zip64.len(), MAX_SOURCE_BYTES);
        invalid.push(zip64);
        for (index, bytes) in invalid.iter().enumerate() {
            assert!(
                validate_docx_zip_footer(bytes).is_err(),
                "footer case {index}"
            );
            assert!(
                extract_document(&fixture.write(&format!("footer-{index}.docx"), bytes)).is_err()
            );
        }
    }

    #[test]
    fn docx_rejects_hostile_malformed_empty_and_resource_excess() {
        let fixture = Fixture::new();
        let cases = [
            word(
                r#"<mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice Requires="w"><w:p><w:r><w:t>Choice</w:t></w:r></w:p></mc:Choice><mc:Fallback><w:p><w:r><w:t>Fallback</w:t></w:r></w:p></mc:Fallback></mc:AlternateContent>"#,
            ),
            word("<w:p><w:r><w:t>&custom;</w:t></w:r></w:p>"),
            word("<w:p><w:r><w:t>&#xFFFF;</w:t></w:r></w:p>"),
            word("<w:p><w:r><w:t>&#1;</w:t></w:r></w:p>"),
            word("<w:p><w:r><w:t>broken</w:p>"),
            word("<w:p unknown:attr=\"value\"/>"),
            word("<w:p attr=\"&custom;\"/>"),
            word("<w:p attr=\"<\"/>"),
            word("<w:p><w:r><w:t> \n </w:t></w:r></w:p>"),
            word("<w:p><w:del><w:t>deleted only</w:t></w:del></w:p>"),
            word("<w:p><w:r><w:instrText>field only</w:instrText></w:r></w:p>"),
            word("<w:p><w:r><fake:t xmlns:fake=\"urn:fake\">wrong namespace</fake:t></w:r></w:p>"),
            word("<w:p>text</w:p>").replace("UTF-8", "UTF-16"),
            word("<w:p>text</w:p>").replace("version=\"1.0\"", "version=\"1.1\""),
            word("<w:p/>") + "<another/>",
            word("<w:p/>") + "text",
            "<!DOCTYPE w:document [<!ENTITY secret SYSTEM 'file:///etc/passwd'>]>".to_owned()
                + &word("<w:p/>").replace("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", ""),
            word(&format!(
                "{}text{}",
                "<w:p>".repeat(129),
                "</w:p>".repeat(129)
            )),
            word(&format!(
                "<w:p><w:r><w:t>{}</w:t></w:r></w:p>",
                "a".repeat(MAX_TEXT_BYTES + 1)
            )),
            word(&format!("<!--{}--><w:p/>", "a".repeat(MAX_SOURCE_BYTES))),
            word(&"<w:p/>".repeat(50_001)),
        ];
        for (index, xml) in cases.iter().enumerate() {
            assert!(
                extract_document(&fixture.write(
                    &format!("bad-{index}.docx"),
                    &docx(&[("word/document.xml", xml.as_bytes())])
                ))
                .is_err(),
                "case {index}"
            );
        }
        for (name, bytes) in [
            ("corrupt.docx", b"not a ZIP".to_vec()),
            ("missing.docx", docx(&[("word/header.xml", b"hello")])),
            ("invalid-utf8.docx", docx(&[("word/document.xml", b"\xff")])),
            (
                "unclosed.docx",
                docx(&[("word/document.xml", word("<w:p>hello").as_bytes())]),
            ),
        ] {
            assert!(
                extract_document(&fixture.write(name, &bytes)).is_err(),
                "{name}"
            );
        }
        // Mark both local and central ZIP headers encrypted; no password guessing.
        let mut encrypted = docx(&[("word/document.xml", word("<w:p/>").as_bytes())]);
        for index in 0..encrypted.len().saturating_sub(10) {
            if encrypted[index..].starts_with(b"PK\x03\x04") {
                encrypted[index + 6] |= 1;
            }
            if encrypted[index..].starts_with(b"PK\x01\x02") {
                encrypted[index + 8] |= 1;
            }
        }
        assert!(extract_document(&fixture.write("encrypted.docx", &encrypted)).is_err());
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
