//! `svggod describe`: a diagram as plain text, from its spec or from a drawing that carries one.

use serde_json::{Value, json};

use super::read_input;
use crate::envelope::{AppError, EXIT_OK, Output, head};
use crate::{describe, spec};

pub fn run(file: &str, short: bool) -> Result<Output, AppError> {
    let text = read_input(file)?;
    let src = describe::source(file, &text)?;
    let doc = spec::read_document(&src, file)?;
    let parsed = spec::parse(&doc)?;
    let (brief, full) = (describe::short(&parsed), describe::full(&parsed));
    let mut m = head("describe", true);
    m.insert("title".into(), json!(parsed.title));
    m.insert("short".into(), json!(brief));
    m.insert("text".into(), json!(full));
    m.insert("spec".into(), doc);
    Ok(Output { json: Value::Object(m), text: if short { format!("{brief}\n") } else { full }, exit: EXIT_OK, raw: None })
}
