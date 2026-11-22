use std::path::Path;

use serde_json::json;

use super::change::{dest, finish};
use super::read_input;
use super::validate::{Paint, rule_ids};
use crate::config::{Loader, Profile, RuleSel};
use crate::envelope::{AppError, Output};

#[allow(clippy::too_many_arguments)]
pub fn run(file: &str, only: &[String], allow_unsafe: bool, dry_run: bool, output: Option<&str>, in_place: bool, loader: &mut Loader, profile: Option<Profile>, paint: &Paint) -> Result<Output, AppError> {
    let only = rule_ids(only, "--rules")?;
    let dest = dest(file, dry_run, output, in_place)?;
    let src = read_input(file)?;
    let path = (file != "-").then(|| Path::new(file));
    let settings = loader.for_input(path)?.settings(path, profile, &RuleSel::default());

    let (text, fixed) = crate::fix::fix_all(file, &src, &settings, &only, allow_unsafe)?;

    let total: usize = fixed.values().sum();
    let list: Vec<String> = fixed.iter().map(|(r, n)| format!("{r} x{n}")).collect();
    let note = if total == 0 { String::new() } else { format!("Applied {total} fix{}: {}.\n", if total == 1 { "" } else { "es" }, list.join(", ")) };
    let extra = vec![("fixed", json!(fixed.iter().map(|(r, n)| json!({ "rule": r, "count": n })).collect::<Vec<_>>()))];
    finish("fix", file, &src, text, dest, &settings, extra, note, paint, None)
}
