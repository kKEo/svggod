//! The library's public face: draw, check, fix, describe and rasterize, as plain
//! functions over text. Nothing here prints, reads stdin or looks for a config
//! file that was not named.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{Config, Profile, RuleSel};
use crate::envelope::{AppError, Finding};
use crate::render::effort::Effort;
use crate::util::did_you_mean;
use crate::{catalogue, rules, spec};

/// Draws diagram specs as SVG.
///
/// One renderer can draw any number of specs and can be shared between threads.
/// The same spec and options always give the same bytes.
#[derive(Clone, Debug, Default)]
pub struct Renderer {
    theme: Option<String>,
    base: Option<PathBuf>,
    root: Option<PathBuf>,
    effort: Option<Effort>,
    max_effort: Option<Effort>,
    text_as_paths: bool,
    embed_spec: bool,
    interactive: bool,
}

impl Renderer {
    /// A renderer with the defaults: the `docs` theme unless the spec names one,
    /// text as text, nothing embedded.
    pub fn new() -> Renderer {
        Renderer::default()
    }

    /// The theme to draw with, whatever the spec says: a built-in name from
    /// [`themes`], with `-dark` or `-auto` for those variants, or the path of a
    /// theme folder (`./my-theme`, `./my-theme#dark`) relative to the working directory.
    ///
    /// The name is looked up when a spec is rendered; an unknown one fails there
    /// with the code `unknown-theme`.
    pub fn theme(mut self, theme: impl Into<String>) -> Renderer {
        self.theme = Some(theme.into());
        self
    }

    /// The folder a theme path written in a spec (`theme: ./brand`) starts from.
    /// Without it such a path starts from the working directory.
    pub fn base_dir(mut self, dir: impl Into<PathBuf>) -> Renderer {
        self.base = Some(dir.into());
        self
    }

    /// The only folder theme files may be read from. A theme path that leads out of
    /// it, from a spec or from [`theme`](Renderer::theme), fails with `outside-root`.
    /// Set this when the specs come from someone else; without it a spec can name
    /// any folder on the disk.
    pub fn root(mut self, dir: impl Into<PathBuf>) -> Renderer {
        self.root = Some(dir.into());
        self
    }

    /// How hard to work at edges that cross, whatever the spec says.
    pub fn effort(mut self, effort: Effort) -> Renderer {
        self.effort = Some(effort);
        self
    }

    /// The most effort a drawing may take, whatever the spec or [`effort`](Renderer::effort)
    /// asks for. A spec can say `effort: max`, which takes many times longer on a dense
    /// diagram; set this when the specs come from someone else.
    pub fn max_effort(mut self, most: Effort) -> Renderer {
        self.max_effort = Some(most);
        self
    }

    /// Draw every label as outlines, so the picture needs no font where it is shown.
    pub fn text_as_paths(mut self, yes: bool) -> Renderer {
        self.text_as_paths = yes;
        self
    }

    /// Store the spec inside the SVG, so [`describe`] can read the drawing back.
    pub fn embed_spec(mut self, yes: bool) -> Renderer {
        self.embed_spec = yes;
        self
    }

    /// Add the classes and the style block that let the drawing answer to the
    /// pointer where SVG is shown with CSS.
    pub fn interactive(mut self, yes: bool) -> Renderer {
        self.interactive = yes;
        self
    }

    /// Draw one spec, given as YAML or JSON text.
    ///
    /// # Errors
    ///
    /// When the text is not YAML or JSON (`malformed-spec`), when the spec is not a
    /// valid diagram (`missing-field`, `invalid-value`, `unknown-field`, ...), or
    /// when the theme cannot be found (`unknown-theme`, `theme-not-found`).
    /// [`Error::path`](crate::Error) names the place in the spec.
    pub fn render(&self, spec: &str) -> Result<String, AppError> {
        let doc = spec::read_document(spec, "spec")?;
        let parsed = spec::parse(&doc)?;
        let spec_dir = self.base.as_deref().unwrap_or(Path::new(""));
        let look = crate::render::theme::choose(&crate::render::theme::Choice { flag: self.theme.as_deref(), spec: parsed.theme.as_deref(), spec_dir, config: None, ceiling: self.root.as_deref() })?;
        let opt = crate::render::Options {
            theme: &look,
            text_as_paths: self.text_as_paths,
            embed: self.embed_spec.then_some(&doc),
            interactive: self.interactive,
            effort: {
                let asked = self.effort.or(parsed.effort).unwrap_or_default();
                self.max_effort.map_or(asked, |most| asked.min(most))
            },
        };
        crate::render::render(&parsed, &opt)
    }
}

/// Draw a diagram spec (YAML or JSON text) as SVG with the default options.
///
/// The same as `Renderer::new().render(spec)`; see [`Renderer::render`] for the errors.
pub fn render(spec: &str) -> Result<String, AppError> {
    Renderer::new().render(spec)
}

/// The names of the built-in themes, each also with `-light`, `-dark` and `-auto` added.
pub fn themes() -> Vec<String> {
    crate::render::theme::names()
}

/// Checks SVG text against the rules.
///
/// One validator can check any number of files and can be shared between threads.
/// It never looks for an `svggod.toml` by itself; name one with
/// [`config_file`](Validator::config_file) or give its text to [`config`](Validator::config).
#[derive(Default)]
pub struct Validator {
    profile: Option<Profile>,
    sel: RuleSel,
    config: Option<Config>,
}

fn rule_ids(ids: &[&str]) -> Result<Vec<&'static str>, AppError> {
    ids.iter()
        .map(|id| catalogue::rule(id).map(|r| r.id).ok_or_else(|| AppError::usage("unknown-rule", format!("Unknown rule '{id}'.")).suggest(did_you_mean(id, catalogue::ids()))))
        .collect()
}

impl Validator {
    /// A validator with the `docs` profile, every rule at its default, no config.
    pub fn new() -> Validator {
        Validator::default()
    }

    /// The rule profile to use, whatever the config says.
    pub fn profile(mut self, profile: Profile) -> Validator {
        self.profile = Some(profile);
        self
    }

    /// Run these rules and no others.
    ///
    /// # Errors
    ///
    /// `unknown-rule` when an id is not in the catalogue.
    pub fn only(mut self, rules: &[&str]) -> Result<Validator, AppError> {
        self.sel.only = Some(rule_ids(rules)?);
        Ok(self)
    }

    /// Leave these rules out.
    ///
    /// # Errors
    ///
    /// `unknown-rule` when an id is not in the catalogue.
    pub fn skip(mut self, rules: &[&str]) -> Result<Validator, AppError> {
        self.sel.skip = rule_ids(rules)?;
        Ok(self)
    }

    /// Read rule settings from an `svggod.toml`. Its `[[overrides]]` globs are
    /// matched against the names given to [`validate`](Validator::validate),
    /// relative to the folder the file is in.
    ///
    /// # Errors
    ///
    /// `file-not-found` or `invalid-config`, with the key in [`Error::path`](crate::Error).
    pub fn config_file(mut self, path: impl AsRef<Path>) -> Result<Validator, AppError> {
        self.config = Some(Config::load(path.as_ref())?);
        Ok(self)
    }

    /// Read rule settings from the text of an `svggod.toml`.
    ///
    /// # Errors
    ///
    /// `invalid-config`, with the key in [`Error::path`](crate::Error).
    pub fn config(mut self, toml: &str) -> Result<Validator, AppError> {
        self.config = Some(Config::parse(toml, Path::new("svggod.toml"), PathBuf::new())?);
        Ok(self)
    }

    fn settings(&self, name: &str) -> crate::config::Settings {
        let empty;
        let config = match &self.config {
            Some(c) => c,
            None => {
                empty = Config::empty(PathBuf::new());
                &empty
            }
        };
        config.settings(Some(Path::new(name)), self.profile, &self.sel)
    }

    /// Check one SVG. `name` is what the findings call the file; nothing is read
    /// from disk. Text that is not XML gives one `xml-well-formed` finding.
    /// Findings come back sorted by line, then rule id.
    pub fn validate(&self, name: &str, svg: &str) -> Vec<Finding> {
        rules::analyze(name, svg, &self.settings(name))
    }

    /// Apply the fixes the rules offer, again and again until nothing changes.
    /// Fixes that may change how the drawing looks are applied only with `allow_unsafe`.
    ///
    /// # Errors
    ///
    /// When a fix cannot be applied to the text, which is a bug in the rule.
    pub fn fix(&self, name: &str, svg: &str, allow_unsafe: bool) -> Result<Fixed, AppError> {
        let settings = self.settings(name);
        let (svg, applied) = crate::fix::fix_all(name, svg, &settings, &[], allow_unsafe)?;
        let remaining = rules::analyze(name, &svg, &settings);
        Ok(Fixed { svg, applied, remaining })
    }
}

/// What [`Validator::fix`] did.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Fixed {
    /// The SVG after the fixes; the input unchanged when there was nothing to fix.
    pub svg: String,
    /// How many fixes were applied, by rule id.
    pub applied: BTreeMap<&'static str, usize>,
    /// What the rules still find in the fixed SVG.
    pub remaining: Vec<Finding>,
}

/// Check one SVG with the default profile and no config.
///
/// The same as `Validator::new().validate(name, svg)`.
pub fn validate(name: &str, svg: &str) -> Vec<Finding> {
    Validator::new().validate(name, svg)
}

/// A diagram in words.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Description {
    /// The diagram's title.
    pub title: String,
    /// One or two sentences, fit for alt text.
    pub short: String,
    /// The whole diagram, section by section.
    pub text: String,
}

/// Put a diagram into words. `source` is a spec (YAML or JSON), or an SVG drawn
/// with [`Renderer::embed_spec`].
///
/// # Errors
///
/// `no-spec` for an SVG without an embedded spec; otherwise as [`Renderer::render`].
pub fn describe(source: &str) -> Result<Description, AppError> {
    let text = crate::describe::source("source", source)?;
    let parsed = spec::parse(&spec::read_document(&text, "spec")?)?;
    Ok(Description { short: crate::describe::short(&parsed), text: crate::describe::full(&parsed), title: parsed.title })
}

/// A rasterized SVG.
#[cfg(feature = "preview")]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Image {
    /// The PNG file.
    pub png: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Draw an SVG as a PNG `width` pixels wide (at most 2000) on a white page.
///
/// Only the fonts bundled with the crate are used, so the same SVG gives the same
/// PNG on every machine; text in other fonts falls back to Inter.
///
/// # Errors
///
/// `malformed-xml` when the text is not XML, `not-renderable` when it has no size
/// or is not something the renderer can draw.
#[cfg(feature = "preview")]
pub fn rasterize(svg: &str, width: u32) -> Result<Image, AppError> {
    let doc = crate::svg::parse(svg).map_err(|e| AppError::input("malformed-xml", format!("The SVG is not well-formed XML: {e}.")))?;
    let canvas = crate::svg::Scene::build(&doc).canvas;
    let shot = crate::preview::rasterize(svg, canvas, None, width.max(1), crate::util::Rgba::WHITE, None)?;
    Ok(Image { png: shot.png, width: shot.width, height: shot.height })
}
