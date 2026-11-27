//! Text measurement with the bundled fonts, so results match on every machine.

use harfrust::font::glyph::outline::{DrawMemory, DrawOptions, PathSink};
use harfrust::{Buffer, Font, GlyphInfo, GlyphPosition, ShapeOptions, ShaperFont};

static INTER: &[u8] = include_bytes!("../../fonts/Inter-Regular.ttf");
static INTER_BOLD: &[u8] = include_bytes!("../../fonts/Inter-Bold.ttf");
static MONO: &[u8] = include_bytes!("../../fonts/JetBrainsMono-Regular.ttf");
static COMIC: &[u8] = include_bytes!("../../fonts/ComicNeue-Bold.ttf");
static ROUND: &[u8] = include_bytes!("../../fonts/VarelaRound-Regular.ttf");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Sans,
    /// Inter Bold, for a theme whose text is all bold.
    SansBold,
    Mono,
    /// Comic Neue, bundled in its bold weight only.
    Comic,
    /// Varela Round, which has one weight.
    Round,
}

struct Fonts {
    sans: Font,
    bold: Font,
    mono: Font,
    comic: Font,
    round: Font,
}

thread_local! {
    static FONTS: Fonts = Fonts {
        sans: Font::new(INTER, 0).expect("bundled Inter Regular"),
        bold: Font::new(INTER_BOLD, 0).expect("bundled Inter Bold"),
        mono: Font::new(MONO, 0).expect("bundled JetBrains Mono"),
        comic: Font::new(COMIC, 0).expect("bundled Comic Neue Bold"),
        round: Font::new(ROUND, 0).expect("bundled Varela Round"),
    };
}

/// The font files compiled into the binary.
pub fn bundled() -> [&'static [u8]; 5] {
    [INTER, INTER_BOLD, MONO, COMIC, ROUND]
}

impl Fonts {
    fn face(&self, class: Class, bold: bool) -> &Font {
        match (class, bold) {
            (Class::Mono, _) => &self.mono,
            (Class::SansBold, _) => &self.bold,
            (Class::Comic, _) => &self.comic,
            (Class::Round, _) => &self.round,
            (Class::Sans, true) => &self.bold,
            (Class::Sans, false) => &self.sans,
        }
    }
}

/// Shapes one line of text in font units, left to right.
fn shape(face: &Font, text: &str) -> Vec<(GlyphInfo, GlyphPosition)> {
    let mut buf = Buffer::new();
    buf.push_str(text);
    buf.guess_segment_properties();
    // Shaping fails only for a malformed font or an unset direction; the fonts are bundled and
    // the direction was just guessed.
    harfrust::shape(&ShaperFont::new(face), &mut buf, ShapeOptions::new()).expect("shaping a bundled font");
    buf.glyph_infos().iter().copied().zip(buf.glyph_positions().iter().copied()).collect()
}

pub struct Measured {
    /// Advance width in em.
    pub width: f64,
    /// True when a character had no glyph in the bundled face.
    pub missing: bool,
}

pub fn measure(text: &str, class: Class, bold: bool) -> Measured {
    if text.is_empty() {
        return Measured { width: 0.0, missing: false };
    }
    FONTS.with(|f| {
        let face = f.face(class, bold);
        let glyphs = shape(face, text);
        let upem = face.units_per_em() as f64;
        let adv: i64 = glyphs.iter().map(|(_, p)| p.x_advance as i64).sum();
        let missing = glyphs.iter().any(|(g, _)| g.glyph_id == 0);
        Measured { width: adv as f64 / upem, missing }
    })
}

/// Ascent and descent in em, both positive.
pub fn vmetrics(class: Class, bold: bool) -> (f64, f64) {
    FONTS.with(|f| {
        let face = f.face(class, bold);
        let upem = face.units_per_em() as f64;
        let line = face.metrics().h_line().unwrap_or_default();
        (line.ascender.to_f32() as f64 / upem, -(line.descender.to_f32() as f64) / upem)
    })
}

const MONO_NAMES: &[&str] = &[
    "monospace", "ui-monospace", "jetbrains mono", "courier", "courier new", "consolas", "menlo", "monaco",
    "fira code", "fira mono", "source code pro", "sf mono", "sfmono-regular", "dejavu sans mono",
    "liberation mono", "roboto mono", "ibm plex mono", "cascadia code", "andale mono", "lucida console",
];

pub const GENERIC: &[&str] = &[
    "serif", "sans-serif", "monospace", "cursive", "fantasy", "system-ui", "ui-serif", "ui-sans-serif",
    "ui-monospace", "ui-rounded", "math", "emoji", "fangsong",
];

pub fn families(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|f| f.trim().trim_matches(|c| c == '"' || c == '\'').trim().to_lowercase())
        .filter(|f| !f.is_empty())
        .collect()
}

/// Which bundled face stands in for a font-family list, and whether it is the named font itself.
pub fn classify(font_family: Option<&str>) -> (Class, bool) {
    let Some(value) = font_family else { return (Class::Sans, false) };
    let list = families(value);
    let Some(first) = list.first() else { return (Class::Sans, false) };
    let class = match first.as_str() {
        "comic neue" => Class::Comic,
        "varela round" => Class::Round,
        f if MONO_NAMES.contains(&f) => Class::Mono,
        _ => Class::Sans,
    };
    let exact = matches!(first.as_str(), "inter" | "jetbrains mono" | "comic neue" | "varela round");
    (class, exact)
}

struct Outline {
    d: String,
    x: f64,
    y: f64,
    scale: f64,
}

impl Outline {
    fn pt(&self, x: f32, y: f32) -> String {
        format!("{} {}", crate::util::fmt(self.x + x as f64 * self.scale), crate::util::fmt(self.y - y as f64 * self.scale))
    }
}

impl PathSink for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d.push_str(&format!("M{}", self.pt(x, y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.d.push_str(&format!("L{}", self.pt(x, y)));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.d.push_str(&format!("Q{} {}", self.pt(x1, y1), self.pt(x, y)));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push_str(&format!("C{} {} {}", self.pt(x1, y1), self.pt(x2, y2), self.pt(x, y)));
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}

/// Path data for a line of text set in a bundled face, starting at `x` on `baseline`.
pub fn outline(text: &str, class: Class, size: f64, x: f64, baseline: f64) -> String {
    FONTS.with(|f| {
        let face = f.face(class, false);
        let scale = size / face.units_per_em() as f64;
        let glyphs = face.outline_glyphs();
        let mut pen = 0.0;
        let mut d = String::new();
        for (info, pos) in shape(face, text) {
            let mut o = Outline { d: String::new(), x: x + (pen + pos.x_offset as f64) * scale, y: baseline - pos.y_offset as f64 * scale, scale };
            if let Some(glyph) = glyphs.get(harfrust::GlyphId::new(info.glyph_id)) {
                // Font units, unhinted. A glyph that fails to draw is left out, as before.
                let _ = glyph.draw(&DrawOptions::new(), DrawMemory::Scratch, &mut o);
            }
            d.push_str(&o.d);
            pen += pos.x_advance as f64;
        }
        d
    })
}
