pub mod change;
pub mod describe;
pub mod docs;
pub mod edit;
pub mod examples;
pub mod fix;
pub mod inspect;
pub mod preview;
pub mod render;
pub mod rules;
pub mod schema;
pub mod theme;
pub mod validate;

use std::io::Read;

use crate::envelope::AppError;

/// The largest input read, in bytes. A diagram is some kilobytes; this stops a
/// mistaken path or an endless stream from being read into memory whole.
pub const MAX_INPUT: u64 = 64 * 1024 * 1024;

fn too_large(arg: &str) -> AppError {
    AppError::input("input-too-large", format!("{} is larger than {} MB, the most that is read.", if arg == "-" { "stdin" } else { arg }, MAX_INPUT / (1024 * 1024))).path(arg)
}

/// Read one input: a path, or stdin for `-`.
pub fn read_input(arg: &str) -> Result<String, AppError> {
    let bytes = if arg == "-" {
        let mut buf = Vec::new();
        std::io::stdin().lock().take(MAX_INPUT + 1).read_to_end(&mut buf).map_err(|e| AppError::input("stdin-not-readable", format!("Cannot read stdin: {e}.")))?;
        if buf.len() as u64 > MAX_INPUT {
            return Err(too_large(arg));
        }
        buf
    } else {
        if std::fs::metadata(arg).is_ok_and(|m| m.is_file() && m.len() > MAX_INPUT) {
            return Err(too_large(arg));
        }
        std::fs::read(arg).map_err(|e| {
            let code = if e.kind() == std::io::ErrorKind::NotFound { "file-not-found" } else { "file-not-readable" };
            let what = if e.kind() == std::io::ErrorKind::NotFound { "File not found".to_string() } else { format!("Cannot read file ({e})") };
            AppError::input(code, format!("{what}: {arg}.")).path(arg)
        })?
    };
    String::from_utf8(bytes).map_err(|_| AppError::input("not-utf8", format!("{arg} is not UTF-8 text.")).path(arg))
}
