//! Damaged input must give findings or an error, never a panic.
//!
//! Each seed file is changed at random (a fixed sequence, so a failure repeats) and
//! handed to the library. `SVGGOD_MUTATIONS=N` sets how many changes each file gets;
//! the default keeps the test to a few seconds. `SVGGOD_MUTATE_FILE=PATH` tries one
//! saved input instead, under the same limits.
//!
//! Hostile input is tried only here, never with the `svggod` program itself: this
//! test cannot take more than `BUDGET` of memory or `DEADLINE` for one input, and
//! ends itself when either is passed. Nothing outside the process can be relied on
//! to stop a memory bomb in time.

//!
//! The test also holds the library to a memory budget: an allocation that would take
//! the test over `BUDGET` fails, which ends the test at once. The input being tried
//! is always on disk first (the path is printed), so the one to blame is not lost.

use std::alloc::{GlobalAlloc, Layout, System};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The most memory the whole test may hold at once.
const BUDGET: usize = 256 * 1024 * 1024;

/// The longest one input may take, all operations together.
const DEADLINE: std::time::Duration = std::time::Duration::from_secs(20);

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
    std::env::var("SVGGOD_MUTATIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(12)
}

/// When the input in hand was started, in milliseconds since the test began; 0 while none is.
static STARTED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// End the whole test when one input takes longer than `DEADLINE`: a loop that never
/// ends cannot be stopped from inside its own thread.
fn watch() -> std::time::Instant {
    static CLOCK: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    *CLOCK.get_or_init(|| {
        let began = std::time::Instant::now();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(250));
                let started = STARTED.load(Ordering::Relaxed);
                if started != 0 && (began.elapsed().as_millis() as u64).saturating_sub(started) > DEADLINE.as_millis() as u64 {
                    eprintln!("\nAn input took longer than {DEADLINE:?}; it is the one kept on disk (see above). Ending the test.");
                    std::process::abort();
                }
            }
        });
        began
    })
}

/// Try one input: a panic is given back as its message. The input is on disk first.
fn attempt(input: &str, kept: &std::path::Path, work: &dyn Fn(&str)) -> Option<String> {
    let began = watch();
    let _ = std::fs::write(kept, input);
    STARTED.store(began.elapsed().as_millis() as u64 + 1, Ordering::Relaxed);
    let outcome = catch_unwind(AssertUnwindSafe(|| work(input)));
    STARTED.store(0, Ordering::Relaxed);
    outcome.err().map(|panic| panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())
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
            if let Some(why) = attempt(&input, &current, &work) {
                let path = std::env::temp_dir().join(format!("svggod-{what}-{n}-{round}.txt"));
                let _ = std::fs::write(&path, &input);
                failed.push(format!("{name}, change {round}: {why}\n    input saved to {}", path.display()));
            }
        }
    }
    std::panic::set_hook(quiet);
    assert!(failed.is_empty(), "{} {what} input(s) made the library panic:\n{}", failed.len(), failed.join("\n"));
}

/// Which operation is running, written to stderr so that the last line names the one
/// that ended the test. Only when one saved input is tried.
fn stage(name: &str) {
    if std::env::var_os("SVGGOD_MUTATE_FILE").is_some() {
        eprintln!("  {name}");
    }
}

fn try_svg(svg: &str) {
    let v = svggod::Validator::new().profile(svggod::Profile::Strict);
    stage("validate");
    let _ = v.validate("x.svg", svg);
    stage("fix");
    let _ = v.fix("x.svg", svg, true);
    stage("describe");
    let _ = svggod::describe(svg);
    #[cfg(feature = "preview")]
    {
        stage("rasterize");
        let _ = svggod::rasterize(svg, 200);
    }
    stage("done");
}

fn try_spec(spec: &str) {
    let r = svggod::Renderer::new().max_effort(svggod::Effort::Normal).embed_spec(true).interactive(true);
    stage("render");
    if let Ok(svg) = r.render(spec) {
        // What we draw, we must be able to read back and check.
        stage("validate what was drawn");
        let _ = svggod::validate("x.svg", &svg);
        let _ = svggod::describe(&svg);
    }
    stage("describe");
    let _ = svggod::describe(spec);
    stage("done");
}

/// `SVGGOD_MUTATE_FILE=PATH cargo test --test mutate one_saved_input -- --nocapture`
/// tries one saved input, with the memory budget and the deadline in force.
#[test]
fn one_saved_input() {
    let Some(path) = std::env::var_os("SVGGOD_MUTATE_FILE") else { return };
    let input = std::fs::read_to_string(&path).unwrap();
    let kept = std::env::temp_dir().join("svggod-one-current.txt");
    let work: &dyn Fn(&str) = if input.trim_start().starts_with('<') { &try_svg } else { &try_spec };
    if let Some(why) = attempt(&input, &kept, work) {
        panic!("{} made the library panic: {why}", std::path::Path::new(&path).display());
    }
}

#[test]
fn damaged_svg_never_panics() {
    if std::env::var_os("SVGGOD_MUTATE_FILE").is_some() {
        return;
    }
    let mut all = seeds("tests/fixtures", "svg");
    all.extend(seeds("tests/golden", "svg"));
    hammer(all, "svg", try_svg);
}

#[test]
fn damaged_specs_never_panic() {
    if std::env::var_os("SVGGOD_MUTATE_FILE").is_some() {
        return;
    }
    let mut all = seeds("examples", "yaml");
    all.extend(seeds("tests/fixtures/gallery", "yaml"));
    hammer(all, "spec", try_spec);
}

/// Inputs that once took all the memory there was, or never finished. They live in
/// this file because here a relapse ends the test, not the machine.
#[test]
fn known_bombs_stay_defused() {
    if std::env::var_os("SVGGOD_MUTATE_FILE").is_some() {
        return;
    }
    let kept = std::env::temp_dir().join("svggod-bomb-current.txt");
    let svg = |body: &str| format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 600 400\"><title>t</title><desc>d</desc>{body}</svg>");
    let boxes = "<rect x=\"40\" y=\"40\" width=\"120\" height=\"60\" fill=\"#fff\" stroke=\"#333\"/><rect x=\"300\" y=\"200\" width=\"120\" height=\"60\" fill=\"#fff\" stroke=\"#333\"/>";
    let svgs = [
        // A routing grid as large as the path is long.
        svg(&format!("{boxes}<path d=\"M-1e308 280 L376 280 L376 328\" fill=\"none\" stroke=\"#333\"/>")),
        // A point every 2 px along a line of any length.
        svg(&format!("{boxes}<path d=\"M-1e308 70 L432 230\" fill=\"none\" stroke=\"#333\"/><path d=\"M1599999999999999999999 70 L360 230\" fill=\"none\" stroke=\"#333\"/>")),
        // Curves cut into pieces in proportion to their size.
        svg("<circle cx=\"33\" cy=\"99\" r=\"1e308\" fill=\"#2f5bd6\"/><ellipse cx=\"3\" cy=\"9\" rx=\"1e300\" ry=\"4\"/>"),
        svg("<path d=\"M176 52 A48 8 0 0 1 272 52 L272 116 A48 8 0 0 1 1799999999999999999999 116 Z\" fill=\"#e0e7ff\" stroke=\"#0a0a0a\"/>"),
        svg("<path d=\"M0 0 A1e308 1e308 0 0 1 50 50\" fill=\"none\" stroke=\"#333\"/>"),
        // What the rasterizer cannot survive.
        svg("<path d=\"M16 72 A56 8 0 0 1 128 72 L128 120 Z\" fill=\"#153d75\" stroke=\"#cfe0fa\" stroke-width=\"99999999999999999999\"/>"),
        svg("<rect x=\"1e308\" y=\"0\" width=\"608\" height=\"464\" fill=\"#fff\"/><path d=\"M0 0L10 4L0 99999999999999999999Z\" fill=\"#1f2937\"/>"),
    ];
    for input in &svgs {
        assert_eq!(attempt(input, &kept, &try_svg), None, "{input}");
    }
    // The far-out ones are refused by the rasterizer, not drawn.
    #[cfg(feature = "preview")]
    for input in &svgs[5..] {
        assert_eq!(svggod::rasterize(input, 200).unwrap_err().code, "not-renderable", "{input}");
    }

    let spec = |node: &str| format!("version: 1\ndiagram: free\ntitle: T\nnodes:\n  - {{ id: a, label: A, at: [40, 40] }}\n  - {node}\nedges:\n  - \"a -> b\"\n");
    for node in ["{ id: b, label: B, at: [-1e308, 440] }", "{ id: b, label: B, at: [40, 1e12] }", "{ id: b, label: B, at: [200, 40], width: 1e300 }"] {
        let input = spec(node);
        assert_eq!(attempt(&input, &kept, &try_spec), None, "{input}");
        assert_eq!(svggod::render(&input).unwrap_err().code, "invalid-value", "{input}");
    }
    // Inside the limits for one node, but together too far apart to route.
    let far = spec("{ id: b, label: B, at: [10000, 10000] }").replace("at: [40, 40]", "at: [-10000, -10000]");
    assert_eq!(attempt(&far, &kept, &try_spec), None);
    assert_eq!(svggod::render(&far).unwrap_err().code, "diagram-too-large");
}
