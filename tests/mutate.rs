//! Damaged input must give findings or an error, never a panic.
//!
//! Each seed file is changed at random (a fixed sequence, so a failure repeats) and
//! handed to the library. `SVGGOD_MUTATIONS=N` sets how many changes each file gets;
//! the default keeps the test to a few seconds.

//!
//! The test also holds the library to a memory budget: an allocation that would take
//! the test over `BUDGET` fails, which ends the test at once. The input being tried
//! is always on disk first (the path is printed), so the one to blame is not lost.

use std::alloc::{GlobalAlloc, Layout, System};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The most memory the whole test may hold at once.
const BUDGET: usize = 1024 * 1024 * 1024;

struct Counted;

static LIVE: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call goes to the system allocator unchanged; this only counts bytes
// and refuses, by returning null as an allocator may, what would pass the budget.
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size() > BUDGET {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            return std::ptr::null_mut();
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > layout.size() && LIVE.fetch_add(new_size - layout.size(), Ordering::Relaxed) + new_size - layout.size() > BUDGET {
            LIVE.fetch_sub(new_size - layout.size(), Ordering::Relaxed);
            return std::ptr::null_mut();
        }
        if new_size < layout.size() {
            LIVE.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counted = Counted;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const ODD: &[&str] = &["0", "-1", "1e308", "-1e308", "1e-320", "NaN", "inf", "99999999999999999999", "0.0000001", "", "\"", "<", ">", "&", "&#0;", "{", "}", "[", "]", ":", "- ", "\n", "\t", "#", "%", "url(#a)", "rotate(", "é", "\u{202e}", "\u{0}"];

/// A range that starts and ends between characters.
fn span(text: &str, rng: &mut Rng, most: usize) -> (usize, usize) {
    let mut a = rng.below(text.len() + 1);
    while !text.is_char_boundary(a) {
        a -= 1;
    }
    let mut b = (a + rng.below(most + 1)).min(text.len());
    while !text.is_char_boundary(b) {
        b -= 1;
    }
    (a, b.max(a))
}

fn mutate(seed: &str, rng: &mut Rng) -> String {
    let mut text = seed.to_string();
    for _ in 0..1 + rng.below(4) {
        let (a, b) = span(&text, rng, 40);
        match rng.below(6) {
            // Cut a piece out.
            0 => text.replace_range(a..b, ""),
            // Say a piece twice.
            1 => {
                let piece = text[a..b].to_string();
                text.insert_str(a, &piece);
            }
            // Put something odd in.
            2 => text.insert_str(a, ODD[rng.below(ODD.len())]),
            // Put something odd in place of a number.
            3 => {
                let digits = text[a..].find(|c: char| c.is_ascii_digit()).map(|i| a + i);
                if let Some(start) = digits {
                    let end = text[start..].find(|c: char| !c.is_ascii_digit() && c != '.').map_or(text.len(), |i| start + i);
                    text.replace_range(start..end, ODD[rng.below(8)]);
                }
            }
            // Move a piece somewhere else.
            4 => {
                let piece = text[a..b].to_string();
                text.replace_range(a..b, "");
                let (to, _) = span(&text, rng, 0);
                text.insert_str(to, &piece);
            }
            // Stop early.
            _ => text.truncate(a),
        }
    }
    text
}

fn seeds(dir: &str, ext: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == ext))
        .filter_map(|p| Some((p.to_string_lossy().into_owned(), std::fs::read_to_string(&p).ok()?)))
        // Small seeds: a change then lands on something that matters.
        .filter(|(_, text)| text.len() < 20_000)
        .collect();
    out.sort();
    assert!(!out.is_empty(), "no .{ext} seeds in {dir}");
    out
}

fn rounds() -> usize {
    std::env::var("SVGGOD_MUTATIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(40)
}

/// Run `work` on every mutation of every seed; a panic fails the test and saves the input.
fn hammer(seeds: Vec<(String, String)>, what: &str, work: impl Fn(&str)) {
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut failed = Vec::new();
    // The input in hand, kept on disk in case it ends the process.
    let current = std::env::temp_dir().join(format!("svggod-{what}-current.txt"));
    eprintln!("{what}: the input being tried is kept in {}", current.display());
    for (n, (name, seed)) in seeds.iter().enumerate() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (n as u64 + 1));
        for round in 0..rounds() {
            let input = mutate(seed, &mut rng);
            let _ = std::fs::write(&current, &input);
            if let Err(panic) = catch_unwind(AssertUnwindSafe(|| work(&input))) {
                let why = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                let path = std::env::temp_dir().join(format!("svggod-{what}-{n}-{round}.txt"));
                let _ = std::fs::write(&path, &input);
                failed.push(format!("{name}, change {round}: {why}\n    input saved to {}", path.display()));
            }
        }
    }
    std::panic::set_hook(quiet);
    assert!(failed.is_empty(), "{} {what} input(s) made the library panic:\n{}", failed.len(), failed.join("\n"));
}

#[test]
fn damaged_svg_never_panics() {
    let mut all = seeds("tests/fixtures", "svg");
    all.extend(seeds("tests/golden", "svg"));
    hammer(all, "svg", |svg| {
        let v = svggod::Validator::new().profile(svggod::Profile::Strict);
        let _ = v.validate("x.svg", svg);
        let _ = v.fix("x.svg", svg, true);
        let _ = svggod::describe(svg);
        #[cfg(feature = "preview")]
        let _ = svggod::rasterize(svg, 200);
    });
}

#[test]
fn damaged_specs_never_panic() {
    let mut all = seeds("examples", "yaml");
    all.extend(seeds("tests/fixtures/gallery", "yaml"));
    hammer(all, "spec", |spec| {
        let r = svggod::Renderer::new().max_effort(svggod::Effort::Normal).embed_spec(true).interactive(true);
        if let Ok(svg) = r.render(spec) {
            // What we draw, we must be able to read back and check.
            let _ = svggod::validate("x.svg", &svg);
            let _ = svggod::describe(&svg);
        }
        let _ = svggod::describe(spec);
    });
}
