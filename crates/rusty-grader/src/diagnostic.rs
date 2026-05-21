//! Rusty's own diagnostic model, decoupled from `cargo_metadata` so the rest of the
//! app (and the Sprint-4 UI) doesn't depend on that crate's types. We keep the
//! verbatim rustc-`rendered` text — the primary teaching surface (prompt §1.6).

use serde::{Deserialize, Serialize};

/// Severity of a compiler diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
    /// Any other / future `rustc` level (cargo's enum is `#[non_exhaustive]`).
    Other,
}

/// The primary source location of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub file_name: String,
    pub line_start: usize,
    pub column_start: usize,
}

/// One compiler diagnostic, normalized from `cargo_metadata`'s `Diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diag {
    /// The error code, e.g. `"E0382"` (the nested `DiagnosticCode.code`).
    pub code: Option<String>,
    pub level: Level,
    pub message: String,
    /// The full rustc-formatted text — rendered in the UI as-is.
    pub rendered: Option<String>,
    pub primary_span: Option<Span>,
}

/// Parse a `cargo --message-format=json` stream into our diagnostics.
pub fn parse_diagnostics(json: &str) -> Vec<Diag> {
    use cargo_metadata::Message;

    Message::parse_stream(json.as_bytes())
        .filter_map(Result::ok) // items are io::Result<Message>
        .filter_map(|message| match message {
            Message::CompilerMessage(cm) => Some(cm.message),
            _ => None,
        })
        .map(|d| {
            let primary_span = d.spans.iter().find(|s| s.is_primary).map(|s| Span {
                file_name: s.file_name.clone(),
                line_start: s.line_start,
                column_start: s.column_start,
            });
            Diag {
                code: d.code.map(|c| c.code),
                level: level_from(d.level),
                message: d.message,
                rendered: d.rendered,
                primary_span,
            }
        })
        .collect()
}

fn level_from(level: cargo_metadata::diagnostic::DiagnosticLevel) -> Level {
    use cargo_metadata::diagnostic::DiagnosticLevel as D;
    match level {
        D::Error => Level::Error,
        D::Warning => Level::Warning,
        D::Note => Level::Note,
        D::Help => Level::Help,
        _ => Level::Other, // `DiagnosticLevel` is #[non_exhaustive]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diagnostics_e0382() {
        let json = include_str!("../tests/fixtures/e0382.json");
        let diags = parse_diagnostics(json);
        let e0382 = diags
            .iter()
            .find(|d| d.code.as_deref() == Some("E0382"))
            .expect("an E0382 diagnostic");
        assert_eq!(e0382.level, Level::Error);
        assert!(e0382.rendered.as_deref().unwrap_or("").contains("E0382"));
        assert!(e0382.primary_span.is_some());
    }

    #[test]
    fn test_parse_diagnostics_empty() {
        // A `build-finished` line carries no compiler message.
        let json = include_str!("../tests/fixtures/build_finished.json");
        assert!(parse_diagnostics(json).is_empty());
    }
}
