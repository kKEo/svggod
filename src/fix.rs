//! The fix loop: apply the autofixes the rules offer until a pass changes nothing.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::config::Settings;
use crate::envelope::AppError;
use crate::{ops, rules};

/// Fixes can expose further fixable findings (a new viewBox makes margins measurable),
/// so the loop repeats until a pass changes nothing.
const MAX_ROUNDS: usize = 30;

/// Fixes whose value depends on the drawing's scale. They wait until the fixes that
/// change the viewBox have settled, or each of those would make them run again.
const SCALE_BOUND: &[&str] = &["stroke-too-thin", "font-too-small"];
const SCALE_CHANGING: &[&str] = &["viewbox-missing", "content-clipped", "excess-margin", "text-overflow"];

/// Fix `src`, the text of `file`. `only` limits the fixes to those rules when it is
/// not empty; fixes that may change how the drawing looks need `allow_unsafe`.
/// Gives back the new text and how many fixes each rule had applied.
pub fn fix_all(file: &str, src: &str, settings: &Settings, only: &[&'static str], allow_unsafe: bool) -> Result<(String, BTreeMap<&'static str, usize>), AppError> {
    let mut text = src.to_string();
    let mut seen = vec![src.to_string()];
    let mut fixed: BTreeMap<&'static str, usize> = BTreeMap::new();
    for _ in 0..MAX_ROUNDS {
        let findings = rules::analyze(file, &text, settings);
        let mut batch: Vec<(&'static str, Vec<ops::Op>)> = Vec::new();
        for f in &findings {
            let Some(fix) = &f.fix else { continue };
            if (!fix.safe && !allow_unsafe) || (!only.is_empty() && !only.contains(&f.rule)) {
                continue;
            }
            batch.push((f.rule, ops::parse_ops(&Value::Array(fix.ops.clone()))?));
        }
        if batch.iter().any(|(rule, _)| SCALE_CHANGING.contains(rule)) {
            batch.retain(|(rule, _)| !SCALE_BOUND.contains(rule));
        }
        if batch.is_empty() {
            break;
        }
        // A fix that collides with an earlier one in this pass waits for the next round.
        let groups: Vec<Vec<ops::Op>> = batch.iter().map(|(_, g)| g.clone()).collect();
        let result = ops::apply_groups(&text, &groups, false)?;
        if result.text == text {
            break;
        }
        for ((rule, _), ok) in batch.iter().zip(&result.accepted) {
            if *ok {
                *fixed.entry(rule).or_default() += 1;
            }
        }
        // A text seen before means two fixes undo each other; stop rather than loop.
        if seen.contains(&result.text) {
            break;
        }
        text = result.text;
        seen.push(text.clone());
    }

    Ok((text, fixed))
}
