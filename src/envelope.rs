//! The output contract: envelope, findings, errors and exit codes.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub const TOOL: &str = "svggod";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const EXIT_OK: i32 = 0;
pub const EXIT_FINDINGS: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_INPUT: i32 = 3;
pub const EXIT_OUTPUT: i32 = 4;
pub const EXIT_INTERNAL: i32 = 5;

/// How serious a finding is. Ordered: `Info < Warning < Error`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Worth knowing; never fails a check.
    Info,
    /// Likely a problem for a reader.
    Warning,
    /// Wrong, unsafe or unreadable.
    Error,
}

impl Severity {
    /// The lowercase name: `info`, `warning` or `error`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
    /// The severity with this lowercase name.
    pub fn parse(s: &str) -> Option<Severity> {
        match s {
            "info" => Some(Severity::Info),
            "warning" => Some(Severity::Warning),
            "error" => Some(Severity::Error),
            _ => None,
        }
    }
}

/// One thing a rule found in an SVG. Serialises to the shape in `schemas/findings.json`.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Finding {
    /// Id of the rule that raised it, such as `text-overflow`.
    pub rule: &'static str,
    /// How serious it is, after the profile and the config.
    pub severity: Severity,
    /// The name the SVG was validated under.
    pub file: String,
    /// A selector for the element the finding is about.
    pub target: String,
    /// Line of that element in the source, from 1.
    pub line: u32,
    /// The element's box as `[x, y, width, height]` in viewBox units, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[Value; 4]>,
    /// What is wrong, as a sentence.
    pub message: String,
    /// The numbers the rule measured, by name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measured: Option<Map<String, Value>>,
    /// How to fix it, when the rule knows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

/// An autofix: operations in the `edit --ops` format.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Fix {
    /// True when `fix` applies it without --unsafe.
    pub safe: bool,
    /// The edit operations, as the `ops` schema describes them.
    pub ops: Vec<Value>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub files: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub fixable: usize,
    pub truncated: bool,
}

/// Why an operation failed.
///
/// `code` is a stable, kebab-case identifier to branch on (`malformed-spec`,
/// `unknown-theme`, ...); `message` is a full sentence for a person.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AppError {
    /// The exit code the command line gives for this failure: 2 usage, 3 input, 4 output, 5 internal.
    pub exit: i32,
    /// Stable identifier of the failure.
    pub code: &'static str,
    /// What went wrong, as a sentence.
    pub message: String,
    /// The file, or the place in the spec or config, the failure is about.
    pub path: Option<String>,
    /// What to try next, when there is something to suggest.
    pub suggestion: Option<String>,
    /// More about the failure, for a program to read.
    pub details: Option<Box<Value>>,
}

impl AppError {
    pub(crate) fn new(exit: i32, code: &'static str, message: impl Into<String>) -> Self {
        AppError { exit, code, message: message.into(), path: None, suggestion: None, details: None }
    }
    pub(crate) fn usage(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(EXIT_USAGE, code, message)
    }
    pub(crate) fn input(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(EXIT_INPUT, code, message)
    }
    pub(crate) fn path(mut self, p: impl Into<String>) -> Self {
        self.path = Some(p.into());
        self
    }
    pub(crate) fn suggest(mut self, s: Option<String>) -> Self {
        self.suggestion = s;
        self
    }
    pub(crate) fn details(mut self, d: Value) -> Self {
        self.details = Some(Box::new(d));
        self
    }

    pub(crate) fn to_json(&self, command: &str) -> Value {
        let mut err = Map::new();
        err.insert("code".into(), json!(self.code));
        err.insert("message".into(), json!(self.message));
        if let Some(p) = &self.path {
            err.insert("path".into(), json!(p));
        }
        if let Some(s) = &self.suggestion {
            err.insert("suggestion".into(), json!(s));
        }
        if let Some(d) = &self.details {
            err.insert("details".into(), (**d).clone());
        }
        json!({
            "tool": TOOL,
            "version": VERSION,
            "command": command,
            "ok": false,
            "error": Value::Object(err),
        })
    }

    pub(crate) fn to_text(&self) -> String {
        let mut s = format!("error[{}]: {}", self.code, self.message);
        if let Some(sug) = &self.suggestion {
            s.push_str(&format!("\n  {sug}"));
        }
        s
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.message, self.code)?;
        if let Some(s) = &self.suggestion {
            write!(f, " {s}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AppError {}

/// The opening fields every envelope shares.
pub fn head(command: &str, ok: bool) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("tool".into(), json!(TOOL));
    m.insert("version".into(), json!(VERSION));
    m.insert("command".into(), json!(command));
    m.insert("ok".into(), json!(ok));
    m
}

/// What a command hands back to `main`: one JSON object, its text rendering and an exit code.
pub struct Output {
    pub json: Value,
    pub text: String,
    pub exit: i32,
    /// When set, this goes to stdout in place of the envelope (`-o -`).
    pub raw: Option<Vec<u8>>,
}
