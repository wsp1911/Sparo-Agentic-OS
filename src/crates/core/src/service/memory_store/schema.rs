use std::collections::HashMap;

/// Layer classification for a memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    Identity,
    Persona,
    Project,
    Habit,
    Episodic,
    Narrative,
    Reference,
    Session,
    Pinned,
    Unknown,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Persona => "persona",
            Self::Project => "project",
            Self::Habit => "habit",
            Self::Episodic => "episodic",
            Self::Narrative => "narrative",
            Self::Reference => "reference",
            Self::Session => "session",
            Self::Pinned => "pinned",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "identity" | "assistant_identity" => Self::Identity,
            "persona" | "user" => Self::Persona,
            "project" => Self::Project,
            "habit" | "habits" | "feedback" | "collaboration" => Self::Habit,
            "episodic" | "episode" => Self::Episodic,
            "narrative" | "vision" => Self::Narrative,
            "reference" => Self::Reference,
            "session" => Self::Session,
            "pinned" => Self::Pinned,
            _ => Self::Unknown,
        }
    }
}

/// Lifecycle status of a memory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryStatus {
    Tentative,
    Confirmed,
    Consolidated,
    Archived,
}

impl MemoryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tentative => "tentative",
            Self::Confirmed => "confirmed",
            Self::Consolidated => "consolidated",
            Self::Archived => "archived",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "tentative" => Self::Tentative,
            "confirmed" => Self::Confirmed,
            "consolidated" => Self::Consolidated,
            "archived" => Self::Archived,
            _ => Self::Confirmed,
        }
    }
}

/// Sensitivity classification for a memory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySensitivity {
    Normal,
    Private,
    /// Secret entries must never be written to the filesystem.
    Secret,
}

impl MemorySensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Private => "private",
            Self::Secret => "secret",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "private" => Self::Private,
            "secret" => Self::Secret,
            _ => Self::Normal,
        }
    }
}

/// Parsed front matter for a memory entry file.
#[derive(Debug, Clone)]
pub struct MemoryEntryFrontMatter {
    pub id: Option<String>,
    pub layer: MemoryLayer,
    pub scope: Option<String>,
    pub created: Option<String>,
    pub last_seen: Option<String>,
    /// Activity strength in [0, 1]. None means unset (treated as 0.5 on hit).
    pub strength: Option<f32>,
    pub sensitivity: MemorySensitivity,
    pub status: MemoryStatus,
    pub tags: Vec<String>,
    pub entities: Vec<String>,
    pub links: Vec<String>,
    pub source_session: Option<String>,
    pub source_files: Vec<String>,
    /// Extra unknown fields preserved for round-trip fidelity.
    pub extra: HashMap<String, String>,
}

impl Default for MemoryEntryFrontMatter {
    fn default() -> Self {
        Self {
            id: None,
            layer: MemoryLayer::Unknown,
            scope: None,
            created: None,
            last_seen: None,
            strength: None,
            sensitivity: MemorySensitivity::Normal,
            status: MemoryStatus::Confirmed,
            tags: Vec::new(),
            entities: Vec::new(),
            links: Vec::new(),
            source_session: None,
            source_files: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

/// Result of parsing a memory entry file.
#[derive(Debug, Clone)]
pub struct ParsedMemoryEntry {
    pub front_matter: MemoryEntryFrontMatter,
    pub body: String,
}

/// Parse a memory entry file (YAML front matter + body).
///
/// Accepts files that have `---` front matter or plain body-only files.
/// The parser is intentionally lenient: unknown fields go into `extra`.
pub fn parse_entry(content: &str) -> ParsedMemoryEntry {
    if !content.starts_with("---") {
        return ParsedMemoryEntry {
            front_matter: MemoryEntryFrontMatter::default(),
            body: content.to_string(),
        };
    }

    let lines: Vec<&str> = content.lines().collect();
    let end_idx = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(idx, _)| idx);

    let Some(end_idx) = end_idx else {
        return ParsedMemoryEntry {
            front_matter: MemoryEntryFrontMatter::default(),
            body: content.to_string(),
        };
    };

    let yaml_lines = &lines[1..end_idx];
    let body_lines = &lines[end_idx + 1..];
    let body = body_lines.join("\n").trim_start().to_string();

    let mut fm = MemoryEntryFrontMatter::default();

    for &line in yaml_lines {
        let Some(colon_pos) = line.find(':') else {
            continue;
        };
        let key = line[..colon_pos].trim();
        let raw_value = line[colon_pos + 1..].trim();
        let value = raw_value.trim_matches('"').trim_matches('\'');

        match key {
            "id" => fm.id = Some(value.to_string()),
            "layer" | "type" => fm.layer = MemoryLayer::from_str(value),
            "scope" => fm.scope = Some(value.to_string()),
            "created" => fm.created = Some(value.to_string()),
            "last_seen" => fm.last_seen = Some(value.to_string()),
            "strength" => fm.strength = value.parse::<f32>().ok(),
            "sensitivity" => fm.sensitivity = MemorySensitivity::from_str(value),
            "status" => fm.status = MemoryStatus::from_str(value),
            "source_session" | "name" if key == "source_session" => {
                fm.source_session = Some(value.to_string())
            }
            "name" => {
                fm.extra.insert(key.to_string(), value.to_string());
            }
            "description" => {
                fm.extra.insert(key.to_string(), value.to_string());
            }
            "tags" | "entities" | "links" | "source_files" => {
                // Inline list: `tags: [a, b, c]`
                let list = parse_inline_list(raw_value);
                match key {
                    "tags" => fm.tags = list,
                    "entities" => fm.entities = list,
                    "links" => fm.links = list,
                    "source_files" => fm.source_files = list,
                    _ => {}
                }
            }
            _ => {
                fm.extra.insert(key.to_string(), value.to_string());
            }
        }
    }

    ParsedMemoryEntry {
        front_matter: fm,
        body,
    }
}

fn parse_inline_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![trimmed.trim_matches('"').trim_matches('\'').to_string()]
    }
}

/// Render a parsed memory entry back to its file representation.
pub fn render_entry(entry: &ParsedMemoryEntry) -> String {
    let fm = &entry.front_matter;
    let mut lines: Vec<String> = vec!["---".to_string()];

    if let Some(id) = &fm.id {
        lines.push(format!("id: {}", id));
    }
    lines.push(format!("layer: {}", fm.layer.as_str()));
    if let Some(scope) = &fm.scope {
        lines.push(format!("scope: {}", scope));
    }
    if let Some(created) = &fm.created {
        lines.push(format!("created: {}", created));
    }
    if let Some(last_seen) = &fm.last_seen {
        lines.push(format!("last_seen: {}", last_seen));
    }
    if let Some(strength) = fm.strength {
        lines.push(format!("strength: {:.2}", strength));
    }
    lines.push(format!("sensitivity: {}", fm.sensitivity.as_str()));
    lines.push(format!("status: {}", fm.status.as_str()));
    if !fm.tags.is_empty() {
        lines.push(format!("tags: [{}]", fm.tags.join(", ")));
    }
    if !fm.entities.is_empty() {
        lines.push(format!("entities: [{}]", fm.entities.join(", ")));
    }
    if !fm.links.is_empty() {
        lines.push(format!("links: [{}]", fm.links.join(", ")));
    }
    if let Some(src) = &fm.source_session {
        lines.push(format!("source_session: {}", src));
    }
    if !fm.source_files.is_empty() {
        lines.push(format!("source_files: [{}]", fm.source_files.join(", ")));
    }
    for (key, value) in &fm.extra {
        lines.push(format!("{}: {}", key, value));
    }

    lines.push("---".to_string());
    if !entry.body.is_empty() {
        lines.push(String::new());
        lines.push(entry.body.clone());
    }

    lines.join("\n")
}

/// Validate a front matter struct.
///
/// Returns a list of validation error messages. Empty means valid.
pub fn validate_entry(fm: &MemoryEntryFrontMatter) -> Vec<String> {
    let mut errors = Vec::new();

    if fm.sensitivity == MemorySensitivity::Secret {
        errors.push("secret sensitivity entries must not be written to the filesystem".to_string());
    }

    if let Some(strength) = fm.strength {
        if !(0.0..=1.0).contains(&strength) {
            errors.push(format!("strength {} is out of range [0, 1]", strength));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_body_without_front_matter() {
        let entry = parse_entry("Just plain text.");
        assert_eq!(entry.body, "Just plain text.");
        assert_eq!(entry.front_matter.layer, MemoryLayer::Unknown);
    }

    #[test]
    fn parse_entry_with_all_known_fields() {
        let content = r#"---
id: ep-2026-04-29-001
layer: episodic
scope: workspace
created: 2026-04-29T22:00:00+08:00
last_seen: 2026-04-29T22:00:00+08:00
strength: 0.80
sensitivity: normal
status: confirmed
tags: [streaming, openai]
entities: [stream_processor_openai]
links: [ep-2026-04-12-003]
source_session: abc123
---

Fixed OpenAI streaming event ordering."#;

        let entry = parse_entry(content);
        assert_eq!(entry.front_matter.id.as_deref(), Some("ep-2026-04-29-001"));
        assert_eq!(entry.front_matter.layer, MemoryLayer::Episodic);
        assert_eq!(entry.front_matter.strength, Some(0.80));
        assert_eq!(entry.front_matter.tags, vec!["streaming", "openai"]);
        assert_eq!(entry.front_matter.status, MemoryStatus::Confirmed);
        assert!(entry.body.contains("Fixed OpenAI streaming"));
    }

    #[test]
    fn layer_from_str_maps_legacy_types() {
        assert_eq!(MemoryLayer::from_str("user"), MemoryLayer::Persona);
        assert_eq!(MemoryLayer::from_str("feedback"), MemoryLayer::Habit);
        assert_eq!(MemoryLayer::from_str("collaboration"), MemoryLayer::Habit);
        assert_eq!(MemoryLayer::from_str("vision"), MemoryLayer::Narrative);
        assert_eq!(
            MemoryLayer::from_str("assistant_identity"),
            MemoryLayer::Identity
        );
    }

    #[test]
    fn validate_rejects_secret_entries() {
        let mut fm = MemoryEntryFrontMatter::default();
        fm.sensitivity = MemorySensitivity::Secret;
        let errors = validate_entry(&fm);
        assert!(!errors.is_empty());
    }

    #[test]
    fn render_round_trips() {
        let content = "---\nlayer: habit\nstatus: confirmed\n---\n\nDo not mock the database.\n";
        let parsed = parse_entry(content);
        let rendered = render_entry(&parsed);
        let reparsed = parse_entry(&rendered);
        assert_eq!(reparsed.front_matter.layer, MemoryLayer::Habit);
        assert!(reparsed.body.contains("Do not mock"));
    }
}
