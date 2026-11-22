//! Damaged input must give findings or an error, never a panic.
//!
//! Each seed file is changed at random (a fixed sequence, so a failure repeats) and
//! handed to the library. `SVGGOD_MUTATIONS=N` sets how many changes each file gets;
//! the default keeps the test to a few seconds.

use std::panic::{AssertUnwindSafe, catch_unwind};

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
    for (n, (name, seed)) in seeds.iter().enumerate() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (n as u64 + 1));
        for round in 0..rounds() {
            let input = mutate(seed, &mut rng);
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
