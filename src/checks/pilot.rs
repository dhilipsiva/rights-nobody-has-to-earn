// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic HTML/EPUB builder and structural pilot-reader check.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use percent_encoding::percent_decode_str;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::{canonical_json, sha256};

const OUTPUT_BASENAME: &str = "book-1-pilot-snapshot";
const MANIFEST_SCHEMA: &str = "book-1-pilot-snapshot-manifest/v1";
const ARTIFACT_FORMAT: &str = "book-1-pilot-reader-artifacts/v1";
const GENERATOR_PATH: &str = "src/checks/pilot.rs";

const REQUIRED_OPENING_SECTIONS: [&str; 6] = [
    "Reader's Map",
    "Annotated contents",
    "Concise glossary",
    "Roles, bodies, and cases",
    "Domains and chapters",
    "Accessible diagrams",
];

const CSS: &str = r#":root { color-scheme: light; font-family: Georgia, 'Times New Roman', serif; }
body { color: #171717; background: #fff; line-height: 1.62; margin: 0 auto;
       max-width: 48rem; padding: 1.25rem; }
a { color: #0645ad; text-decoration-thickness: .08em; }
a:focus { outline: .2rem solid #b45309; outline-offset: .15rem; }
.skip-link { position: absolute; left: -10000px; top: auto; }
.skip-link:focus { left: 1rem; top: 1rem; background: #fff; padding: .6rem;
                   z-index: 2; }
header[role=banner] { border-bottom: .12rem solid #555; margin-bottom: 2rem; }
.status { border: .12rem solid #7c2d12; padding: .8rem 1rem; font-weight: 700; }
nav ol { padding-left: 1.4rem; }
main article { break-before: page; page-break-before: always; }
main article:first-child { break-before: auto; page-break-before: auto; }
h1, h2, h3, h4 { line-height: 1.22; break-after: avoid; }
blockquote { border-left: .25rem solid #777; margin-left: 0; padding-left: 1rem; }
pre { overflow-x: auto; border: .08rem solid #777; padding: .8rem; white-space: pre-wrap; }
code { font-family: ui-monospace, Consolas, monospace; }
table { border-collapse: collapse; width: 100%; }
th, td { border: .08rem solid #777; padding: .45rem; text-align: left; vertical-align: top; }
caption { font-weight: 700; text-align: left; margin-bottom: .4rem; }
@media print {
  body { max-width: none; padding: 0; font-size: 11.5pt; }
  nav a { color: #171717; text-decoration: none; }
  a[href^='http']::after { content: ' (' attr(href) ')'; font-size: 85%; }
  @page { size: A4; margin: 20mm 18mm 22mm; }
}"#;

static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+?)\s*$").expect("heading regex"));
static IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^]]*)\]\([^)]+\)").expect("image regex"));
static INLINE_IMAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^]]+)\]\(([^)]+)\)").expect("image regex"));
static INLINE_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^]]+)\]\(([^)]+)\)").expect("link regex"));
static HREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a href="([^"]+)">"#).expect("href regex"));

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct FileDigest {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotIdentity {
    manifest_schema: String,
    artifact_format: String,
    generator: FileDigest,
    ordered_inputs: Vec<FileDigest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceRevision {
    vcs: String,
    commit: Option<String>,
    bound_paths_match_commit: bool,
    mismatched_paths: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ArtifactKind {
    Html,
    Epub,
    Pdf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactOutput {
    format: ArtifactKind,
    path: String,
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_html_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AccessibilityContract {
    semantic_navigation: bool,
    text_alternatives_required: bool,
    layout_or_colour_only_meaning_prohibited: bool,
    human_screen_reader_attestation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SnapshotManifest {
    spdx: String,
    manifest_schema: String,
    artifact_format: String,
    artifact_status: String,
    pub(crate) snapshot_id: String,
    snapshot_identity: SnapshotIdentity,
    generator: FileDigest,
    source_revision: SourceRevision,
    ordered_inputs: Vec<FileDigest>,
    outputs: Vec<ArtifactOutput>,
    pdf_source: String,
    accessibility_contract: AccessibilityContract,
}

#[derive(Clone, Debug)]
struct SourceDocument {
    path: PathBuf,
    title: String,
    body_html: String,
    heading_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Report {
    pub(crate) link_controls: usize,
    pub(crate) pdf_controls: usize,
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "15-pilot-reader-artifacts: ordered inputs and accessible HTML/EPUB are structurally valid and deterministic; {} missing-link and {} stale/invalid-PDF mutations watched failing; PDF rendering and human screen-reader attestation remain external",
            self.link_controls, self.pdf_controls
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PilotError(String);

impl PilotError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for PilotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

type PilotResult<T> = Result<T, PilotError>;

fn io_error(action: &str, error: impl fmt::Display) -> PilotError {
    PilotError::new(format!("{action}: {error}"))
}

fn relative(context: &Context, path: &Path) -> String {
    path.strip_prefix(context.root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn ordered_inputs(context: &Context) -> PilotResult<Vec<PathBuf>> {
    let book = context.path("book-1");
    let mut numbered = fs::read_dir(&book)
        .map_err(|error| io_error("cannot read book-1", error))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("cannot read book-1 entry", error))?;
    numbered.retain(|path| {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            return false;
        };
        let bytes = name.as_bytes();
        bytes.len() >= 6
            && bytes[0].is_ascii_digit()
            && bytes[1].is_ascii_digit()
            && bytes[2] == b'-'
            && name.ends_with(".md")
    });
    numbered.sort();
    let mut paths = Vec::with_capacity(numbered.len() + 2);
    paths.push(book.join("epigraph.md"));
    paths.extend(numbered);
    paths.push(book.join("method.md"));
    let missing = paths
        .iter()
        .filter(|path| !path.is_file())
        .map(|path| relative(context, path))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(PilotError::new(format!(
            "missing ordered input(s): {}",
            missing.join(", ")
        )));
    }
    if !paths
        .iter()
        .any(|path| path == &book.join("00-opening-note.md"))
    {
        return Err(PilotError::new(
            "ordered inputs omit book-1/00-opening-note.md",
        ));
    }
    Ok(paths)
}

fn html_escape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(character),
        }
    }
    output
}

fn slug(value: &str) -> String {
    let plain = value
        .chars()
        .filter(|character| !matches!(character, '`' | '*' | '_' | '[' | ']' | '(' | ')'))
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let mut result = String::new();
    let mut separator = false;
    for character in plain.chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            if separator && !result.is_empty() {
                result.push('-');
            }
            separator = false;
            result.push(character);
        } else {
            separator = true;
        }
    }
    if result.is_empty() {
        "section".to_owned()
    } else {
        result
    }
}

fn strip_inline_markdown(value: &str) -> String {
    let without_images = Regex::new(r"!\[([^]]*)\]\([^)]+\)")
        .expect("image regex")
        .replace_all(value, "$1");
    let without_links = Regex::new(r"\[([^]]+)\]\([^)]+\)")
        .expect("link regex")
        .replace_all(&without_images, "$1");
    without_links
        .chars()
        .filter(|character| !matches!(character, '`' | '*' | '_'))
        .collect::<String>()
        .trim()
        .to_owned()
}

fn render_inline(value: &str) -> String {
    let code_re = Regex::new(r"`([^`]+)`").expect("code regex");
    let mut tokens = Vec::new();
    let protected = code_re
        .replace_all(value, |captures: &Captures<'_>| {
            tokens.push(format!("<code>{}</code>", html_escape(&captures[1])));
            format!("\0{}\0", tokens.len() - 1)
        })
        .into_owned();
    let mut escaped = html_escape(&protected);
    escaped = INLINE_IMAGE_RE
        .replace_all(&escaped, |captures: &Captures<'_>| {
            format!(
                "<img src=\"{}\" alt=\"{}\"/>",
                html_escape(&captures[2]),
                html_escape(&captures[1])
            )
        })
        .into_owned();
    escaped = INLINE_LINK_RE
        .replace_all(&escaped, |captures: &Captures<'_>| {
            format!(
                "<a href=\"{}\">{}</a>",
                html_escape(&captures[2]),
                &captures[1]
            )
        })
        .into_owned();
    escaped = Regex::new(r"\*\*([^*]+)\*\*")
        .expect("strong regex")
        .replace_all(&escaped, "<strong>$1</strong>")
        .into_owned();
    escaped = Regex::new(r"\*([^*]+)\*")
        .expect("emphasis regex")
        .replace_all(&escaped, "<em>$1</em>")
        .into_owned();
    escaped = Regex::new(r"_([^_]+)_")
        .expect("emphasis regex")
        .replace_all(&escaped, "<em>$1</em>")
        .into_owned();
    for (index, token) in tokens.into_iter().enumerate() {
        escaped = escaped.replace(&format!("\0{index}\0"), &token);
    }
    escaped
}

fn flush_paragraph(output: &mut Vec<String>, paragraph: &mut Vec<String>, source_stem: &str) {
    if paragraph.is_empty() {
        return;
    }
    let content = if source_stem == "epigraph" {
        paragraph
            .iter()
            .map(|part| render_inline(part.trim()))
            .collect::<Vec<_>>()
            .join("<br/>\n")
    } else {
        render_inline(
            &paragraph
                .iter()
                .map(|part| part.trim())
                .collect::<Vec<_>>()
                .join(" "),
        )
    };
    output.push(format!("<p>{content}</p>"));
    paragraph.clear();
}

fn flush_quote(output: &mut Vec<String>, quote: &mut Vec<String>) {
    if quote.is_empty() {
        return;
    }
    output.push(format!(
        "<blockquote><p>{}</p></blockquote>",
        render_inline(&quote.join(" "))
    ));
    quote.clear();
}

fn flush_list(output: &mut Vec<String>, list_kind: &mut Option<&'static str>) {
    if let Some(kind) = list_kind.take() {
        output.push(format!("</{kind}>"));
    }
}

fn flush_table(output: &mut Vec<String>, table_lines: &mut Vec<String>) {
    if table_lines.is_empty() {
        return;
    }
    let rows = table_lines
        .iter()
        .map(|line| {
            line.trim()
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let separator = Regex::new(r"^:?-{3,}:?$").expect("table separator regex");
    if rows.len() >= 2 && rows[1].iter().all(|cell| separator.is_match(cell)) {
        output.push("<div role=\"region\" aria-label=\"Table\" tabindex=\"0\"><table>".to_owned());
        output.push(format!(
            "<thead><tr>{}</tr></thead>",
            rows[0]
                .iter()
                .map(|cell| format!("<th scope=\"col\">{}</th>", render_inline(cell)))
                .collect::<String>()
        ));
        output.push("<tbody>".to_owned());
        for row in &rows[2..] {
            output.push(format!(
                "<tr>{}</tr>",
                row.iter()
                    .map(|cell| format!("<td>{}</td>", render_inline(cell)))
                    .collect::<String>()
            ));
        }
        output.push("</tbody></table></div>".to_owned());
    } else {
        for line in table_lines.iter() {
            output.push(format!("<p>{}</p>", render_inline(line)));
        }
    }
    table_lines.clear();
}

fn flush_all(
    output: &mut Vec<String>,
    paragraph: &mut Vec<String>,
    quote: &mut Vec<String>,
    table_lines: &mut Vec<String>,
    list_kind: &mut Option<&'static str>,
    source_stem: &str,
) {
    flush_paragraph(output, paragraph, source_stem);
    flush_quote(output, quote);
    flush_table(output, table_lines);
    flush_list(output, list_kind);
}

fn markdown_to_html(text: &str, source_stem: &str) -> PilotResult<(String, String, Vec<String>)> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let unordered_re = Regex::new(r"^[-*]\s+(.+)$").expect("unordered list regex");
    let ordered_re = Regex::new(r"^\d+[.)]\s+(.+)$").expect("ordered list regex");
    let rule_re = Regex::new(r"^-{3,}$").expect("rule regex");
    let mut output = Vec::new();
    let mut paragraph = Vec::new();
    let mut quote = Vec::new();
    let mut code_lines = Vec::new();
    let mut table_lines = Vec::new();
    let mut list_kind = None;
    let mut in_code = false;
    let mut in_comment = false;
    let mut title = String::new();
    let mut heading_ids = Vec::new();
    let mut used_ids = BTreeSet::new();

    for raw in normalized.split('\n') {
        let mut line = raw.trim_end().to_owned();
        if line.starts_with("```") {
            flush_all(
                &mut output,
                &mut paragraph,
                &mut quote,
                &mut table_lines,
                &mut list_kind,
                source_stem,
            );
            if in_code {
                output.push(format!(
                    "<pre><code>{}</code></pre>",
                    html_escape(&code_lines.join("\n"))
                ));
                code_lines.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            code_lines.push(line);
            continue;
        }
        if in_comment {
            let Some((_, tail)) = line.split_once("-->") else {
                continue;
            };
            line = tail.to_owned();
            in_comment = false;
        }
        while let Some((prefix, remainder)) = line.split_once("<!--") {
            if let Some((_, tail)) = remainder.split_once("-->") {
                line = format!("{prefix}{tail}");
            } else {
                line = prefix.to_owned();
                in_comment = true;
                break;
            }
        }
        if line.trim().is_empty() && in_comment {
            continue;
        }
        if line.starts_with('|') && line.ends_with('|') {
            flush_paragraph(&mut output, &mut paragraph, source_stem);
            flush_quote(&mut output, &mut quote);
            flush_list(&mut output, &mut list_kind);
            table_lines.push(line);
            continue;
        }
        flush_table(&mut output, &mut table_lines);
        if let Some(heading) = HEADING_RE.captures(&line) {
            flush_all(
                &mut output,
                &mut paragraph,
                &mut quote,
                &mut table_lines,
                &mut list_kind,
                source_stem,
            );
            let level = heading[1].len();
            let visible = strip_inline_markdown(&heading[2]);
            if title.is_empty() && level == 1 {
                title.clone_from(&visible);
            }
            let base = format!("{source_stem}-{}", slug(&visible));
            let mut heading_id = base.clone();
            let mut suffix = 2;
            while used_ids.contains(&heading_id) {
                heading_id = format!("{base}-{suffix}");
                suffix += 1;
            }
            used_ids.insert(heading_id.clone());
            heading_ids.push(heading_id.clone());
            output.push(format!(
                "<h{level} id=\"{heading_id}\">{}</h{level}>",
                render_inline(&heading[2])
            ));
            continue;
        }
        let unordered = unordered_re.captures(&line);
        let ordered = ordered_re.captures(&line);
        if unordered.is_some() || ordered.is_some() {
            flush_paragraph(&mut output, &mut paragraph, source_stem);
            flush_quote(&mut output, &mut quote);
            let (kind, item) = if let Some(captures) = unordered {
                ("ul", captures[1].to_owned())
            } else {
                let captures = ordered.expect("one list shape matched");
                ("ol", captures[1].to_owned())
            };
            if list_kind != Some(kind) {
                flush_list(&mut output, &mut list_kind);
                output.push(format!("<{kind}>"));
                list_kind = Some(kind);
            }
            output.push(format!("<li>{}</li>", render_inline(&item)));
            continue;
        }
        if let Some(value) = line.strip_prefix("> ") {
            flush_paragraph(&mut output, &mut paragraph, source_stem);
            flush_list(&mut output, &mut list_kind);
            quote.push(value.trim().to_owned());
            continue;
        }
        if rule_re.is_match(line.trim()) {
            flush_all(
                &mut output,
                &mut paragraph,
                &mut quote,
                &mut table_lines,
                &mut list_kind,
                source_stem,
            );
            output.push("<hr/>".to_owned());
            continue;
        }
        if line.trim().is_empty() {
            flush_all(
                &mut output,
                &mut paragraph,
                &mut quote,
                &mut table_lines,
                &mut list_kind,
                source_stem,
            );
            continue;
        }
        flush_quote(&mut output, &mut quote);
        flush_list(&mut output, &mut list_kind);
        paragraph.push(line);
    }

    if in_code {
        return Err(PilotError::new(format!(
            "{source_stem}: unclosed fenced code block"
        )));
    }
    if in_comment {
        return Err(PilotError::new(format!(
            "{source_stem}: unclosed HTML comment"
        )));
    }
    flush_all(
        &mut output,
        &mut paragraph,
        &mut quote,
        &mut table_lines,
        &mut list_kind,
        source_stem,
    );
    if title.is_empty() {
        if source_stem != "epigraph" {
            return Err(PilotError::new(format!(
                "{source_stem}: missing level-one title"
            )));
        }
        title = "Epigraph".to_owned();
        heading_ids.insert(0, "epigraph-epigraph".to_owned());
        output.insert(0, "<h1 id=\"epigraph-epigraph\">Epigraph</h1>".to_owned());
    }
    Ok((title, output.join("\n"), heading_ids))
}

fn read_documents(context: &Context) -> PilotResult<Vec<SourceDocument>> {
    ordered_inputs(context)?
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).map_err(|error| {
                io_error(&format!("cannot read {}", relative(context, &path)), error)
            })?;
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| PilotError::new("ordered input has no UTF-8 stem"))?;
            let (title, body_html, heading_ids) = markdown_to_html(&text, stem)?;
            Ok(SourceDocument {
                path,
                title,
                body_html,
                heading_ids,
            })
        })
        .collect()
}

fn source_fragment_targets(
    context: &Context,
    document: &SourceDocument,
) -> PilotResult<BTreeMap<String, String>> {
    let text = fs::read_to_string(&document.path).map_err(|error| {
        io_error(
            &format!("cannot read {}", relative(context, &document.path)),
            error,
        )
    })?;
    let mut targets = BTreeMap::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut heading_index = 0usize;
    for line in text.lines() {
        let Some(heading) = HEADING_RE.captures(line) else {
            continue;
        };
        let base = slug(&strip_inline_markdown(&heading[2]));
        let count = counts.entry(base.clone()).or_default();
        *count += 1;
        let source_fragment = if *count == 1 {
            base
        } else {
            format!("{base}-{}", *count - 1)
        };
        let Some(generated_id) = document.heading_ids.get(heading_index) else {
            return Err(PilotError::new(format!(
                "{}: heading map is incomplete",
                relative(context, &document.path)
            )));
        };
        targets.insert(source_fragment, generated_id.clone());
        heading_index += 1;
    }
    let expected = document.heading_ids.len()
        - usize::from(
            document.path.file_name().and_then(|value| value.to_str()) == Some("epigraph.md"),
        );
    if heading_index != expected {
        return Err(PilotError::new(format!(
            "{}: heading map does not match rendered IDs",
            relative(context, &document.path)
        )));
    }
    Ok(targets)
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn external_href(value: &str) -> bool {
    if value.starts_with("//") {
        return true;
    }
    let Some(colon) = value.find(':') else {
        return false;
    };
    let scheme = &value[..colon];
    !scheme.is_empty()
        && scheme.as_bytes()[0].is_ascii_alphabetic()
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
}

fn normalized_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

fn decode_url_part(value: &str, source: &SourceDocument, context: &Context) -> PilotResult<String> {
    percent_decode_str(value)
        .decode_utf8()
        .map(|value| value.into_owned())
        .map_err(|error| {
            PilotError::new(format!(
                "{}: local link is not UTF-8: {value}: {error}",
                relative(context, &source.path)
            ))
        })
}

fn resolve_body_href(
    context: &Context,
    value: &str,
    source: &SourceDocument,
    documents: &[SourceDocument],
    output_context: &str,
) -> PilotResult<String> {
    if external_href(value) {
        return Ok(value.to_owned());
    }
    let (before_fragment, encoded_fragment) = value
        .split_once('#')
        .map_or((value, None), |(before, fragment)| (before, Some(fragment)));
    let (encoded_path, query) = before_fragment
        .split_once('?')
        .map_or((before_fragment, None), |(path, query)| (path, Some(query)));
    if query.is_some_and(|value| !value.is_empty()) {
        return Err(PilotError::new(format!(
            "{}: local link queries are unsupported: {value}",
            relative(context, &source.path)
        )));
    }
    let raw_path = decode_url_part(encoded_path, source, context)?;
    let target_path = if raw_path.is_empty() {
        normalized_path(&source.path)
    } else {
        normalized_path(
            &source
                .path
                .parent()
                .unwrap_or(context.root())
                .join(raw_path),
        )
    };
    let Some((target_index, target)) = documents
        .iter()
        .enumerate()
        .find(|(_, document)| normalized_path(&document.path) == target_path)
    else {
        return Err(PilotError::new(format!(
            "{}: local link is not an ordered input: {value}",
            relative(context, &source.path)
        )));
    };
    let generated_id = if let Some(fragment) = encoded_fragment {
        let fragment = decode_url_part(fragment, source, context)?;
        source_fragment_targets(context, target)?
            .get(&fragment)
            .cloned()
            .ok_or_else(|| {
                PilotError::new(format!(
                    "{}: local link fragment does not exist: {value}",
                    relative(context, &source.path)
                ))
            })?
    } else {
        target
            .heading_ids
            .first()
            .cloned()
            .ok_or_else(|| PilotError::new("ordered input has no rendered heading"))?
    };
    match output_context {
        "html" => Ok(format!("#{generated_id}")),
        "epub" => Ok(format!(
            "chapter-{:02}.xhtml#{generated_id}",
            target_index + 1
        )),
        _ => Err(PilotError::new(format!(
            "unknown link output context: {output_context}"
        ))),
    }
}

fn rewrite_document_links(
    context: &Context,
    documents: &[SourceDocument],
    output_context: &str,
) -> PilotResult<Vec<SourceDocument>> {
    let mut rewritten = Vec::with_capacity(documents.len());
    for source in documents {
        let mut body = String::with_capacity(source.body_html.len());
        let mut cursor = 0usize;
        for captures in HREF_RE.captures_iter(&source.body_html) {
            let whole = captures.get(0).expect("href match has a whole span");
            body.push_str(&source.body_html[cursor..whole.start()]);
            let original = html_unescape(&captures[1]);
            let resolved =
                resolve_body_href(context, &original, source, documents, output_context)?;
            let _ = write!(body, "<a href=\"{}\">", html_escape(&resolved));
            cursor = whole.end();
        }
        body.push_str(&source.body_html[cursor..]);
        rewritten.push(SourceDocument {
            path: source.path.clone(),
            title: source.title.clone(),
            body_html: body,
            heading_ids: source.heading_ids.clone(),
        });
    }

    let known_ids = rewritten
        .iter()
        .flat_map(|document| document.heading_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let epub_href = Regex::new(r"^chapter-(\d{2})\.xhtml#([a-z0-9-]+)$").expect("EPUB href regex");
    for document in &rewritten {
        for captures in HREF_RE.captures_iter(&document.body_html) {
            let href = html_unescape(&captures[1]);
            if external_href(&href) {
                continue;
            }
            if output_context == "html" {
                if !href
                    .strip_prefix('#')
                    .is_some_and(|fragment| known_ids.contains(fragment))
                {
                    return Err(PilotError::new(format!(
                        "{}: invalid combined-HTML href: {href}",
                        relative(context, &document.path)
                    )));
                }
                continue;
            }
            let Some(target) = epub_href.captures(&href) else {
                return Err(PilotError::new(format!(
                    "{}: invalid EPUB href: {href}",
                    relative(context, &document.path)
                )));
            };
            let index = target[1]
                .parse::<usize>()
                .map_err(|_| PilotError::new("invalid EPUB chapter index"))?;
            if index == 0 || index > rewritten.len() {
                return Err(PilotError::new(format!(
                    "{}: EPUB chapter target is absent: {href}",
                    relative(context, &document.path)
                )));
            }
            if !rewritten[index - 1]
                .heading_ids
                .iter()
                .any(|value| value == &target[2])
            {
                return Err(PilotError::new(format!(
                    "{}: EPUB fragment target is absent: {href}",
                    relative(context, &document.path)
                )));
            }
        }
    }
    Ok(rewritten)
}

fn html_document(
    context: &Context,
    documents: &[SourceDocument],
    snapshot_id: &str,
) -> PilotResult<Vec<u8>> {
    let documents = rewrite_document_links(context, documents, "html")?;
    let nav = documents
        .iter()
        .map(|document| {
            format!(
                "<li><a href=\"#{}\">{}</a></li>",
                document.heading_ids[0],
                html_escape(&document.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let articles = documents
        .iter()
        .map(|document| {
            format!(
                "<article aria-labelledby=\"{}\">{}</article>",
                document.heading_ids[0], document.body_html
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>The Rights Nobody Has to Earn - pilot study snapshot</title>\n\
         <style>{CSS}</style>\n\
         </head>\n\
         <body>\n\
         <a class=\"skip-link\" href=\"#main-content\">Skip to the book</a>\n\
         <header role=\"banner\">\n\
         <p class=\"status\">Pilot study snapshot. Not an edition, release candidate, or Gate C artifact.</p>\n\
         <p>Snapshot identifier: <code>{snapshot_id}</code></p>\n\
         <nav aria-label=\"Book contents\"><h2>Contents</h2><ol>{nav}</ol></nav>\n\
         </header>\n\
         <main id=\"main-content\">{articles}</main>\n\
         </body>\n\
         </html>\n"
    )
    .into_bytes())
}

fn xhtml_document(document: &SourceDocument) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <!DOCTYPE html>\n\
         <html xmlns=\"http://www.w3.org/1999/xhtml\" lang=\"en\" xml:lang=\"en\">\n\
         <head><meta charset=\"utf-8\"/><title>{}</title>\n\
         <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\"/></head>\n\
         <body><main><article aria-labelledby=\"{}\">\n\
         {}\n\
         </article></main></body></html>\n",
        html_escape(&document.title),
        document.heading_ids[0],
        document.body_html
    )
    .into_bytes()
}

fn zip_write<W: Write + std::io::Seek>(
    archive: &mut ZipWriter<W>,
    name: &str,
    value: &[u8],
    compress: bool,
) -> PilotResult<()> {
    let options = FileOptions::default()
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644)
        .compression_method(if compress {
            CompressionMethod::Deflated
        } else {
            CompressionMethod::Stored
        });
    archive
        .start_file(name, options)
        .map_err(|error| io_error(&format!("cannot add EPUB member {name}"), error))?;
    archive
        .write_all(value)
        .map_err(|error| io_error(&format!("cannot write EPUB member {name}"), error))
}

fn epub_document(
    context: &Context,
    documents: &[SourceDocument],
    snapshot_id: &str,
) -> PilotResult<Vec<u8>> {
    let documents = rewrite_document_links(context, documents, "epub")?;
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    zip_write(&mut archive, "mimetype", b"application/epub+zip", false)?;
    zip_write(
        &mut archive,
        "META-INF/container.xml",
        b"<?xml version=\"1.0\"?>\n<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\"><rootfiles><rootfile full-path=\"EPUB/package.opf\" media-type=\"application/oebps-package+xml\"/></rootfiles></container>\n",
        true,
    )?;
    let mut manifest_items = Vec::new();
    let mut spine_items = Vec::new();
    let mut nav_items = Vec::new();
    for (offset, document) in documents.iter().enumerate() {
        let index = offset + 1;
        let item_id = format!("chapter-{index:02}");
        let filename = format!("{item_id}.xhtml");
        zip_write(
            &mut archive,
            &format!("EPUB/{filename}"),
            &xhtml_document(document),
            true,
        )?;
        manifest_items.push(format!(
            "<item id=\"{item_id}\" href=\"{filename}\" media-type=\"application/xhtml+xml\"/>"
        ));
        spine_items.push(format!("<itemref idref=\"{item_id}\"/>"));
        nav_items.push(format!(
            "<li><a href=\"{filename}#{}\">{}</a></li>",
            document.heading_ids[0],
            html_escape(&document.title)
        ));
    }
    let nav = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <!DOCTYPE html><html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"en\" xml:lang=\"en\"><head><title>Contents</title></head><body><nav epub:type=\"toc\" aria-label=\"Book contents\"><h1>Contents</h1><ol>{}</ol></nav></body></html>",
        nav_items.join("")
    );
    zip_write(&mut archive, "EPUB/nav.xhtml", nav.as_bytes(), true)?;
    zip_write(&mut archive, "EPUB/style.css", CSS.as_bytes(), true)?;
    let package = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"book-id\" xml:lang=\"en\">\n\
         <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:identifier id=\"book-id\">urn:sha256:{snapshot_id}</dc:identifier><dc:title>The Rights Nobody Has to Earn - pilot study snapshot</dc:title><dc:language>en</dc:language><dc:rights>CC-BY-4.0</dc:rights><meta property=\"dcterms:modified\">1980-01-01T00:00:00Z</meta></metadata>\n\
         <manifest><item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/><item id=\"css\" href=\"style.css\" media-type=\"text/css\"/>{}</manifest>\n\
         <spine>{}</spine></package>",
        manifest_items.join(""),
        spine_items.join("")
    );
    zip_write(&mut archive, "EPUB/package.opf", package.as_bytes(), true)?;
    archive
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| io_error("cannot finish EPUB", error))
}

fn validate_xml(name: &str, value: &[u8]) -> PilotResult<()> {
    let mut reader = Reader::from_reader(value);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Eof) => return Ok(()),
            Ok(_) => buffer.clear(),
            Err(error) => {
                return Err(PilotError::new(format!(
                    "EPUB XML is malformed: {name}: {error}"
                )));
            }
        }
    }
}

fn validate_epub(value: &[u8], expected_documents: usize) -> PilotResult<()> {
    let mut archive = ZipArchive::new(Cursor::new(value))
        .map_err(|error| io_error("EPUB archive is invalid", error))?;
    let names = archive.file_names().map(str::to_owned).collect::<Vec<_>>();
    for name in names
        .iter()
        .filter(|name| name.ends_with(".xml") || name.ends_with(".xhtml") || name.ends_with(".opf"))
    {
        let mut entry = archive
            .by_name(name)
            .map_err(|error| io_error(&format!("cannot read EPUB member {name}"), error))?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| io_error(&format!("cannot read EPUB member {name}"), error))?;
        validate_xml(name, &bytes)?;
    }
    let first_name = archive
        .by_index(0)
        .map(|entry| entry.name().to_owned())
        .map_err(|error| io_error("cannot read first EPUB member", error))?;
    if first_name != "mimetype" {
        return Err(PilotError::new(
            "EPUB mimetype must be the first archive member",
        ));
    }
    {
        let mut mimetype = archive
            .by_name("mimetype")
            .map_err(|error| io_error("cannot read EPUB mimetype", error))?;
        if mimetype.compression() != CompressionMethod::Stored {
            return Err(PilotError::new("EPUB mimetype must be uncompressed"));
        }
        let mut bytes = Vec::new();
        mimetype
            .read_to_end(&mut bytes)
            .map_err(|error| io_error("cannot read EPUB mimetype", error))?;
        if bytes != b"application/epub+zip" {
            return Err(PilotError::new("EPUB mimetype is invalid"));
        }
    }
    let chapter_re = Regex::new(r"^EPUB/chapter-\d{2}\.xhtml$").expect("chapter regex");
    if names
        .iter()
        .filter(|name| chapter_re.is_match(name))
        .count()
        != expected_documents
    {
        return Err(PilotError::new(
            "EPUB spine does not contain every ordered input",
        ));
    }
    let mut nav = Vec::new();
    archive
        .by_name("EPUB/nav.xhtml")
        .map_err(|error| io_error("cannot read EPUB navigation", error))?
        .read_to_end(&mut nav)
        .map_err(|error| io_error("cannot read EPUB navigation", error))?;
    if !nav
        .windows(b"epub:type=\"toc\"".len())
        .any(|window| window == b"epub:type=\"toc\"")
    {
        return Err(PilotError::new(
            "EPUB navigation document lacks a table of contents",
        ));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> PilotResult<String> {
    fs::read(path)
        .map(sha256)
        .map_err(|error| io_error(&format!("cannot read {}", path.display()), error))
}

fn source_manifest(context: &Context, paths: &[PathBuf]) -> PilotResult<Vec<FileDigest>> {
    paths
        .iter()
        .map(|path| {
            Ok(FileDigest {
                path: relative(context, path),
                sha256: sha256_file(path)?,
            })
        })
        .collect()
}

fn generator_manifest(context: &Context) -> PilotResult<FileDigest> {
    let path = context.path(GENERATOR_PATH);
    Ok(FileDigest {
        path: GENERATOR_PATH.to_owned(),
        sha256: sha256_file(&path)?,
    })
}

fn snapshot_identity(inputs: &[FileDigest], generator: &FileDigest) -> SnapshotIdentity {
    SnapshotIdentity {
        manifest_schema: MANIFEST_SCHEMA.to_owned(),
        artifact_format: ARTIFACT_FORMAT.to_owned(),
        generator: generator.clone(),
        ordered_inputs: inputs.to_vec(),
    }
}

fn snapshot_identifier(identity: &SnapshotIdentity) -> PilotResult<String> {
    let boundary = serde_json::to_value(identity)
        .map_err(|error| PilotError::new(format!("cannot serialize snapshot identity: {error}")))?;
    Ok(sha256(canonical_json(&boundary)))
}

fn git_source_revision(
    context: &Context,
    paths: &[PathBuf],
    require_match: bool,
) -> PilotResult<SourceRevision> {
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(context.root())
        .output();
    let commit = match revision {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .map_err(|error| io_error("Git revision is not UTF-8", error))?
            .trim()
            .to_owned(),
        _ if require_match => {
            return Err(PilotError::new(
                "cannot resolve the committed source revision",
            ));
        }
        _ => {
            return Ok(SourceRevision {
                vcs: "git".to_owned(),
                commit: None,
                bound_paths_match_commit: false,
                mismatched_paths: paths.iter().map(|path| relative(context, path)).collect(),
            });
        }
    };

    let mut mismatched = Vec::new();
    for path in paths {
        let relative_path = relative(context, path);
        let committed = Command::new("git")
            .args(["show", &format!("{commit}:{relative_path}")])
            .current_dir(context.root())
            .output();
        let live = fs::read(path).map_err(|error| {
            io_error(&format!("cannot read bound source {relative_path}"), error)
        })?;
        match committed {
            Ok(output) if output.status.success() && output.stdout == live => {}
            _ => mismatched.push(relative_path),
        }
    }
    if require_match && !mismatched.is_empty() {
        return Err(PilotError::new(format!(
            "freeze-grade output requires committed bound paths; mismatch: {}",
            mismatched.join(", ")
        )));
    }
    Ok(SourceRevision {
        vcs: "git".to_owned(),
        commit: Some(commit),
        bound_paths_match_commit: mismatched.is_empty(),
        mismatched_paths: mismatched,
    })
}

fn prior_pdf_output(
    manifest_path: &Path,
    pdf_path: &Path,
    snapshot_id: &str,
    html_sha256: &str,
) -> PilotResult<Option<ArtifactOutput>> {
    if !pdf_path.is_file() {
        return Ok(None);
    }
    if !manifest_path.is_file() {
        return Err(PilotError::new(
            "pre-existing PDF lacks the prior manifest required for binding",
        ));
    }
    let prior_text = fs::read_to_string(manifest_path)
        .map_err(|error| io_error("cannot read prior snapshot manifest", error))?;
    let prior: SnapshotManifest = serde_json::from_str(&prior_text).map_err(|error| {
        PilotError::new(format!("prior snapshot manifest is invalid JSON: {error}"))
    })?;
    if prior.snapshot_id != snapshot_id {
        return Err(PilotError::new(
            "pre-existing PDF belongs to a different snapshot identifier",
        ));
    }
    let html = prior
        .outputs
        .iter()
        .filter(|item| item.format == ArtifactKind::Html)
        .collect::<Vec<_>>();
    if html.len() != 1 || html[0].sha256 != html_sha256 {
        return Err(PilotError::new(
            "pre-existing PDF source HTML digest does not match",
        ));
    }
    let pdf =
        fs::read(pdf_path).map_err(|error| io_error("cannot read pre-existing PDF", error))?;
    if pdf.is_empty() {
        return Err(PilotError::new("pre-existing PDF must be nonempty"));
    }
    if !pdf.starts_with(b"%PDF-") {
        return Err(PilotError::new("pre-existing PDF lacks the %PDF- header"));
    }
    Ok(Some(ArtifactOutput {
        format: ArtifactKind::Pdf,
        path: pdf_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned(),
        sha256: sha256(pdf),
        source_html_sha256: Some(html_sha256.to_owned()),
    }))
}

fn validate_sources(context: &Context, documents: &[SourceDocument]) -> PilotResult<()> {
    let opening_path = context.path("book-1/00-opening-note.md");
    let opening = fs::read_to_string(&opening_path)
        .map_err(|error| io_error("cannot read book-1/00-opening-note.md", error))?;
    for section in REQUIRED_OPENING_SECTIONS {
        if !opening.contains(&format!("## {section}")) {
            return Err(PilotError::new(format!(
                "opening note missing required section: {section}"
            )));
        }
    }
    for document in documents {
        let text = fs::read_to_string(&document.path).map_err(|error| {
            io_error(
                &format!("cannot read {}", relative(context, &document.path)),
                error,
            )
        })?;
        for captures in IMAGE_RE.captures_iter(&text) {
            if captures[1].trim().is_empty() {
                return Err(PilotError::new(format!(
                    "{}: image lacks text alternative",
                    relative(context, &document.path)
                )));
            }
        }
        let levels = text
            .lines()
            .filter_map(|line| HEADING_RE.captures(line).map(|captures| captures[1].len()))
            .collect::<Vec<_>>();
        if document.path.file_name().and_then(|value| value.to_str()) == Some("epigraph.md") {
            if !levels.is_empty() {
                return Err(PilotError::new(format!(
                    "{}: epigraph must remain unheaded",
                    relative(context, &document.path)
                )));
            }
            continue;
        }
        if levels.first() != Some(&1) {
            return Err(PilotError::new(format!(
                "{}: first heading must be H1",
                relative(context, &document.path)
            )));
        }
        for levels in levels.windows(2) {
            if levels[1] > levels[0] + 1 {
                return Err(PilotError::new(format!(
                    "{}: heading level skips H{} to H{}",
                    relative(context, &document.path),
                    levels[0],
                    levels[1]
                )));
            }
        }
    }
    Ok(())
}

struct PreparedArtifacts {
    documents: Vec<SourceDocument>,
    inputs: Vec<FileDigest>,
    generator: FileDigest,
    identity: SnapshotIdentity,
    snapshot_id: String,
    html: Vec<u8>,
    epub: Vec<u8>,
    source_revision: SourceRevision,
}

fn prepare_artifacts(context: &Context, require_committed: bool) -> PilotResult<PreparedArtifacts> {
    let documents = read_documents(context)?;
    validate_sources(context, &documents)?;
    let source_paths = documents
        .iter()
        .map(|document| document.path.clone())
        .collect::<Vec<_>>();
    let inputs = source_manifest(context, &source_paths)?;
    let generator = generator_manifest(context)?;
    let identity = snapshot_identity(&inputs, &generator);
    let snapshot_id = snapshot_identifier(&identity)?;
    let html = html_document(context, &documents, &snapshot_id)?;
    let epub = epub_document(context, &documents, &snapshot_id)?;
    validate_epub(&epub, documents.len())?;
    let mut bound_paths = source_paths.clone();
    bound_paths.push(context.path(GENERATOR_PATH));
    let source_revision = git_source_revision(context, &bound_paths, require_committed)?;

    Ok(PreparedArtifacts {
        documents,
        inputs,
        generator,
        identity,
        snapshot_id,
        html,
        epub,
        source_revision,
    })
}

fn install_artifacts(
    prepared: &PreparedArtifacts,
    output_dir: &Path,
) -> PilotResult<SnapshotManifest> {
    let html_path = output_dir.join(format!("{OUTPUT_BASENAME}.html"));
    let epub_path = output_dir.join(format!("{OUTPUT_BASENAME}.epub"));
    let pdf_path = output_dir.join(format!("{OUTPUT_BASENAME}.pdf"));
    let manifest_path = output_dir.join(format!("{OUTPUT_BASENAME}-manifest.json"));
    let html_sha256 = sha256(&prepared.html);
    let pdf_output = prior_pdf_output(
        &manifest_path,
        &pdf_path,
        &prepared.snapshot_id,
        &html_sha256,
    )?;

    fs::create_dir_all(output_dir)
        .map_err(|error| io_error("cannot create output directory", error))?;
    fs::write(&html_path, &prepared.html)
        .map_err(|error| io_error("cannot write pilot HTML", error))?;
    fs::write(&epub_path, &prepared.epub)
        .map_err(|error| io_error("cannot write pilot EPUB", error))?;
    let mut outputs = vec![
        ArtifactOutput {
            format: ArtifactKind::Html,
            path: html_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned(),
            sha256: html_sha256,
            source_html_sha256: None,
        },
        ArtifactOutput {
            format: ArtifactKind::Epub,
            path: epub_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_owned(),
            sha256: sha256(&prepared.epub),
            source_html_sha256: None,
        },
    ];
    if let Some(pdf) = pdf_output {
        outputs.push(pdf);
    }
    let manifest = SnapshotManifest {
        spdx: "CC-BY-4.0".to_owned(),
        manifest_schema: MANIFEST_SCHEMA.to_owned(),
        artifact_format: ARTIFACT_FORMAT.to_owned(),
        artifact_status: "pilot-study-snapshot-not-an-edition".to_owned(),
        snapshot_id: prepared.snapshot_id.clone(),
        snapshot_identity: prepared.identity.clone(),
        generator: prepared.generator.clone(),
        source_revision: prepared.source_revision.clone(),
        ordered_inputs: prepared.inputs.clone(),
        outputs,
        pdf_source: html_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned(),
        accessibility_contract: AccessibilityContract {
            semantic_navigation: true,
            text_alternatives_required: true,
            layout_or_colour_only_meaning_prohibited: true,
            human_screen_reader_attestation: "external-pending".to_owned(),
        },
    };
    let rendered = serde_json::to_string_pretty(&manifest)
        .map_err(|error| PilotError::new(format!("cannot render snapshot manifest: {error}")))?
        + "\n";
    fs::write(&manifest_path, rendered)
        .map_err(|error| io_error("cannot write snapshot manifest", error))?;
    Ok(manifest)
}

fn build_inner(
    context: &Context,
    output_dir: &Path,
    require_committed: bool,
) -> PilotResult<SnapshotManifest> {
    let prepared = prepare_artifacts(context, require_committed)?;
    install_artifacts(&prepared, output_dir)
}

fn watched_link_controls(context: &Context, documents: &[SourceDocument]) -> PilotResult<usize> {
    let source_index = documents
        .iter()
        .position(|document| {
            document.path.file_name().and_then(|value| value.to_str()) == Some("00-opening-note.md")
        })
        .ok_or_else(|| PilotError::new("opening note is absent from ordered documents"))?;
    let controls = [
        (
            "missing local file",
            "does-not-exist.md",
            "not an ordered input",
        ),
        (
            "missing local fragment",
            "15-the-five-joints.md#does-not-exist",
            "fragment does not exist",
        ),
    ];
    let mut watched = 0usize;
    for output_context in ["html", "epub"] {
        for (label, href, expected) in controls {
            let mut mutated = documents.to_vec();
            mutated[source_index]
                .body_html
                .push_str(&format!("<p><a href=\"{href}\">link control</a></p>"));
            match rewrite_document_links(context, &mutated, output_context) {
                Err(error) if error.0.contains(expected) => watched += 1,
                Err(error) => {
                    return Err(PilotError::new(format!(
                        "{label} failed for the wrong reason in {output_context}: {error}"
                    )));
                }
                Ok(_) => {
                    return Err(PilotError::new(format!(
                        "{label} did not fail in {output_context}"
                    )));
                }
            }
        }
    }
    Ok(watched)
}

fn manifest_outputs(manifest: &SnapshotManifest, format: ArtifactKind) -> Vec<&ArtifactOutput> {
    manifest
        .outputs
        .iter()
        .filter(|item| item.format == format)
        .collect()
}

fn write_manifest(path: &Path, manifest: &SnapshotManifest) -> PilotResult<()> {
    let rendered = serde_json::to_string_pretty(manifest)
        .map_err(|error| PilotError::new(format!("cannot render control manifest: {error}")))?
        + "\n";
    fs::write(path, rendered).map_err(|error| io_error("cannot write control manifest", error))
}

fn watched_pdf_controls(prepared: &PreparedArtifacts, base: &Path) -> PilotResult<usize> {
    let valid_dir = base.join("valid-pdf-binding");
    let initial = install_artifacts(prepared, &valid_dir)?;
    let pdf_path = valid_dir.join(format!("{OUTPUT_BASENAME}.pdf"));
    fs::write(&pdf_path, b"%PDF-1.7\n%%EOF\n")
        .map_err(|error| io_error("cannot write PDF positive control", error))?;
    let rebound = install_artifacts(prepared, &valid_dir)?;
    let pdf_outputs = manifest_outputs(&rebound, ArtifactKind::Pdf);
    let html_outputs = manifest_outputs(&rebound, ArtifactKind::Html);
    if pdf_outputs.len() != 1 || html_outputs.len() != 1 {
        return Err(PilotError::new(
            "valid PDF binding did not produce one HTML and one PDF",
        ));
    }
    if pdf_outputs[0].source_html_sha256.as_ref() != Some(&html_outputs[0].sha256) {
        return Err(PilotError::new(
            "valid PDF binding did not bind its source HTML digest",
        ));
    }
    if initial.snapshot_id != rebound.snapshot_id {
        return Err(PilotError::new(
            "adding a valid PDF changed the snapshot identifier",
        ));
    }

    let controls: [(&str, &[u8], Option<&str>, &str); 4] = [
        (
            "stale snapshot",
            b"%PDF-1.7\n%%EOF\n",
            Some("snapshot"),
            "different snapshot",
        ),
        (
            "stale HTML",
            b"%PDF-1.7\n%%EOF\n",
            Some("html"),
            "HTML digest",
        ),
        ("empty PDF", b"", None, "nonempty"),
        ("wrong PDF header", b"not-a-pdf", None, "%PDF-"),
    ];
    let mut watched = 0usize;
    for (label, pdf_bytes, mutation, expected) in controls {
        let control_dir = base.join(slug(label));
        let mut manifest = install_artifacts(prepared, &control_dir)?;
        let control_pdf = control_dir.join(format!("{OUTPUT_BASENAME}.pdf"));
        let control_manifest = control_dir.join(format!("{OUTPUT_BASENAME}-manifest.json"));
        fs::write(&control_pdf, pdf_bytes)
            .map_err(|error| io_error("cannot write invalid PDF control", error))?;
        match mutation {
            Some("snapshot") => {
                manifest.snapshot_id = "0".repeat(64);
            }
            Some("html") => {
                let html = manifest
                    .outputs
                    .iter_mut()
                    .find(|item| item.format == ArtifactKind::Html)
                    .expect("generated manifest has one HTML output");
                html.sha256 = "0".repeat(64);
            }
            None => {}
            Some(other) => return Err(PilotError::new(format!("unknown PDF mutation {other}"))),
        }
        if mutation.is_some() {
            write_manifest(&control_manifest, &manifest)?;
        }
        match install_artifacts(prepared, &control_dir) {
            Err(error) if error.0.contains(expected) => watched += 1,
            Err(error) => {
                return Err(PilotError::new(format!(
                    "{label} failed for the wrong reason: {error}"
                )));
            }
            Ok(_) => {
                return Err(PilotError::new(format!("{label} PDF control did not fail")));
            }
        }
    }
    Ok(watched)
}

fn check_inner(context: &Context) -> PilotResult<Report> {
    let temporary = TempDir::with_prefix("book-1-pilot-artifacts-")
        .map_err(|error| io_error("cannot create pilot-artifact check directory", error))?;
    let prepared = prepare_artifacts(context, false)?;
    let first = install_artifacts(&prepared, &temporary.path().join("first"))?;

    // Regenerate the output-bearing functions once to catch accidental entropy
    // without re-running Git provenance lookups for every watched PDF case.
    let regenerated_html = html_document(context, &prepared.documents, &prepared.snapshot_id)?;
    let regenerated_epub = epub_document(context, &prepared.documents, &prepared.snapshot_id)?;
    if regenerated_html != prepared.html || regenerated_epub != prepared.epub {
        return Err(PilotError::new(
            "generated HTML/EPUB bytes are not deterministic",
        ));
    }
    let second = install_artifacts(&prepared, &temporary.path().join("second"))?;
    if first.snapshot_id != second.snapshot_id {
        return Err(PilotError::new("snapshot identifier is not deterministic"));
    }
    let output_digests = |manifest: &SnapshotManifest| {
        manifest
            .outputs
            .iter()
            .map(|item| (item.format, item.sha256.clone()))
            .collect::<Vec<_>>()
    };
    if output_digests(&first) != output_digests(&second) {
        return Err(PilotError::new(
            "generated HTML/EPUB bytes are not deterministic",
        ));
    }
    if snapshot_identifier(&first.snapshot_identity)? != first.snapshot_id {
        return Err(PilotError::new(
            "manifest snapshot identity does not reproduce its identifier",
        ));
    }
    if first.source_revision.commit.is_none() {
        return Err(PilotError::new(
            "manifest lacks Git source-revision provenance",
        ));
    }
    Ok(Report {
        link_controls: watched_link_controls(context, &prepared.documents)?,
        pdf_controls: watched_pdf_controls(&prepared, &temporary.path().join("pdf-controls"))?,
    })
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_inner(context).map_err(|error| Error::new(format!("15-pilot-reader-artifacts: {error}")))
}

pub(crate) fn build(context: &Context, output_dir: &Path) -> Result<SnapshotManifest, Error> {
    build_inner(context, output_dir, true)
        .map_err(|error| Error::new(format!("15-pilot-reader-artifacts: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renderer_handles_headings_links_lists_tables_and_code() {
        let source =
            "# Title\n\nA [link](other.md) and `code`.\n\n- one\n- two\n\n| A |\n| --- |\n| B |\n";
        let (title, body, headings) = markdown_to_html(source, "sample").unwrap();
        assert_eq!(title, "Title");
        assert_eq!(headings, ["sample-title"]);
        assert!(body.contains("<a href=\"other.md\">link</a>"));
        assert!(body.contains("<code>code</code>"));
        assert!(body.contains("<ul>\n<li>one</li>\n<li>two</li>\n</ul>"));
        assert!(body.contains("<thead>"));
    }

    #[test]
    fn epub_is_deterministic_and_structurally_valid() {
        let temporary = TempDir::new().unwrap();
        let context = Context::from_test_root(temporary.path().to_path_buf());
        let document = SourceDocument {
            path: temporary.path().join("one.md"),
            title: "One".to_owned(),
            body_html: "<h1 id=\"one-one\">One</h1>".to_owned(),
            heading_ids: vec!["one-one".to_owned()],
        };
        fs::write(&document.path, "# One\n").unwrap();
        let first =
            epub_document(&context, std::slice::from_ref(&document), &"a".repeat(64)).unwrap();
        let second = epub_document(&context, &[document], &"a".repeat(64)).unwrap();
        assert_eq!(first, second);
        validate_epub(&first, 1).unwrap();
    }

    #[test]
    fn typed_manifest_rejects_unknown_nested_fields() {
        let source = r#"{
            "manifest_schema":"book-1-pilot-snapshot-manifest/v1",
            "artifact_format":"book-1-pilot-reader-artifacts/v1",
            "generator":{"path":"generator.rs","sha256":"00","extra":true},
            "ordered_inputs":[]
        }"#;
        let error = serde_json::from_str::<SnapshotIdentity>(source)
            .expect_err("unknown nested field must fail");
        assert!(error.to_string().contains("unknown field `extra`"));
    }

    #[test]
    #[ignore = "live repository generation parity"]
    fn live_check_passes() {
        let context = Context::discover().unwrap();
        let report = check(&context).unwrap();
        assert_eq!(report.link_controls, 4);
        assert_eq!(report.pdf_controls, 4);
    }

    #[test]
    #[ignore = "live repository artifact generation"]
    fn live_native_artifact_generation_is_deterministic() {
        let context = Context::discover().unwrap();
        let temporary = TempDir::new().unwrap();
        let first = temporary.path().join("first");
        let second = temporary.path().join("second");
        let prepared = prepare_artifacts(&context, false).unwrap();
        let first_manifest = install_artifacts(&prepared, &first).unwrap();
        let second_manifest = install_artifacts(&prepared, &second).unwrap();
        assert_eq!(first_manifest, second_manifest);
        for name in [
            format!("{OUTPUT_BASENAME}.html"),
            format!("{OUTPUT_BASENAME}.epub"),
            format!("{OUTPUT_BASENAME}-manifest.json"),
        ] {
            assert_eq!(
                fs::read(first.join(&name)).unwrap(),
                fs::read(second.join(name)).unwrap()
            );
        }
    }
}
