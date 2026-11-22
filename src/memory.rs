//! The program's memory limit: how much it may hold at once, and what it says when
//! an input would take it past that.
//!
//! Nothing outside a process can be relied on to stop a fast allocation in time, so
//! the program counts its own memory. The allocator itself is in `src/main.rs`, the
//! only place that needs `unsafe`; it asks [`charge`] before every allocation and
//! calls [`exceeded`] when the answer is no. The library alone sets no limit: a
//! program that uses it chooses its own allocator.

use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// What the program may hold when nothing else is said: 1 GB.
pub(crate) const DEFAULT: u64 = 1024 * 1024 * 1024;
/// Less than this and the program cannot start.
const LEAST: u64 = 16 * 1024 * 1024;
/// The environment variable read when `--max-memory` is not given.
pub(crate) const VARIABLE: &str = "SVGGOD_MAX_MEMORY";

static LIVE: AtomicUsize = AtomicUsize::new(0);
static LIMIT: AtomicUsize = AtomicUsize::new(DEFAULT as usize);
static FAILING: AtomicBool = AtomicBool::new(false);
/// What to print when the limit is passed, and whether it goes to stdout. Made
/// beforehand: putting a message together takes memory, which is what has run out.
static MESSAGE: Mutex<Option<(Vec<u8>, bool)>> = Mutex::new(None);

/// Count `bytes` more as held. `false` when that would pass the limit; nothing is
/// counted then.
pub fn charge(bytes: usize) -> bool {
    let held = LIVE.fetch_add(bytes, Ordering::Relaxed).saturating_add(bytes);
    if held > LIMIT.load(Ordering::Relaxed) {
        LIVE.fetch_sub(bytes, Ordering::Relaxed);
        return false;
    }
    true
}

/// Count `bytes` as given back.
pub fn release(bytes: usize) {
    LIVE.fetch_sub(bytes, Ordering::Relaxed);
}

/// The limit was passed: say so in the form the command line promised, and end the
/// program with exit code 3. The limit is lifted first, so that saying it can be done.
pub fn exceeded() -> ! {
    LIMIT.store(usize::MAX, Ordering::Relaxed);
    // Only the first thread to get here reports; the message is printed once.
    if !FAILING.swap(true, Ordering::SeqCst) {
        let armed = MESSAGE.try_lock().ok().and_then(|mut m| m.take());
        let (text, to_stdout) = armed.unwrap_or_else(|| (b"error[memory-limit]: svggod passed its memory limit. Raise --max-memory, or check the input.\n".to_vec(), false));
        // The streams' own files, not the program's handles: one of those may be in
        // the middle of a write, which is where the memory ran out.
        let device = if to_stdout { "/dev/stdout" } else { "/dev/stderr" };
        let written = std::fs::OpenOptions::new().write(true).open(device).and_then(|mut f| f.write_all(&text));
        if written.is_err() {
            let _ = if to_stdout { std::io::stdout().write_all(&text) } else { std::io::stderr().write_all(&text) };
        }
    }
    std::process::exit(crate::envelope::EXIT_INPUT)
}

/// Set how much may be held; `None` is no limit.
pub(crate) fn set_limit(bytes: Option<u64>) {
    LIMIT.store(bytes.map_or(usize::MAX, |b| usize::try_from(b).unwrap_or(usize::MAX)), Ordering::Relaxed);
}

/// Set what [`exceeded`] prints, and where.
pub(crate) fn arm(message: Vec<u8>, to_stdout: bool) {
    if let Ok(mut m) = MESSAGE.lock() {
        *m = Some((message, to_stdout));
    }
}

/// A limit as a person writes it, in megabytes: `512MB`, `2GB`, `1.5GB`, `200M`, `64000KB`.
/// `0` and `none` are no limit.
pub(crate) fn parse_size(text: &str) -> Result<Option<u64>, String> {
    let t = text.trim().to_ascii_lowercase();
    if t == "0" || t == "none" {
        return Ok(None);
    }
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(t.len());
    let unit: u64 = match t[split..].trim() {
        "k" | "kb" => 1024,
        "m" | "mb" => 1024 * 1024,
        "g" | "gb" => 1024 * 1024 * 1024,
        "" => return Err(format!("'{text}' has no unit; write it as 512MB or 2GB (0 is no limit).")),
        other => return Err(format!("'{other}' is not a unit; use KB, MB or GB, as in 512MB.")),
    };
    let number: f64 = t[..split].parse().map_err(|_| format!("'{text}' is not a size; write it as 512MB or 2GB (0 is no limit)."))?;
    let bytes = number * unit as f64;
    if !bytes.is_finite() || bytes < LEAST as f64 {
        return Err(format!("'{text}' is too little to run in; the least is {} MB.", LEAST / (1024 * 1024)));
    }
    Ok(Some(bytes.min(u64::MAX as f64) as u64))
}

/// A size for a message: `512 MB`, `1.5 GB`.
pub(crate) fn show(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 { format!("{} GB", crate::util::fmt(mb / 1024.0)) } else { format!("{} MB", crate::util::fmt(mb)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_read_as_written() {
        const MB: u64 = 1024 * 1024;
        assert_eq!(parse_size("512MB"), Ok(Some(512 * MB)));
        assert_eq!(parse_size("2gb"), Ok(Some(2048 * MB)));
        assert_eq!(parse_size(" 1.5 GB "), Ok(Some(1536 * MB)));
        assert_eq!(parse_size("200m"), Ok(Some(200 * MB)));
        assert_eq!(parse_size("64000KB"), Ok(Some(64000 * 1024)));
        assert_eq!(parse_size("0"), Ok(None));
        assert_eq!(parse_size("none"), Ok(None));
        for bad in ["", "lots", "512", "5MB", "1TB", "-1GB", "1e9GB", "MB"] {
            assert!(parse_size(bad).is_err(), "{bad}");
        }
        assert_eq!((show(512 * MB), show(1536 * MB), show(DEFAULT)), ("512 MB".into(), "1.5 GB".into(), "1 GB".into()));
    }

    #[test]
    fn a_charge_over_the_limit_is_refused_and_not_counted() {
        // The test program has no limit of its own (it does not use the allocator), so
        // the counter starts at nothing here.
        let before = LIVE.load(Ordering::Relaxed);
        assert!(charge(1000));
        assert!(!charge(usize::MAX - 10));
        assert!(!charge(DEFAULT as usize));
        assert_eq!(LIVE.load(Ordering::Relaxed), before + 1000);
        release(1000);
        assert_eq!(LIVE.load(Ordering::Relaxed), before);
    }
}
