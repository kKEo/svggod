//! SVGGod compiles declarative diagram descriptions to SVG and checks any SVG for
//! correctness, legibility and portability.
//!
//! The crate is both the `svggod` command line program and a library. The library
//! does the same work without touching stdout, stdin, the process arguments or the
//! exit code, and gives the same bytes for the same input on every machine.
//!
//! # Draw a diagram
//!
//! A spec is YAML or JSON text; [`render`] gives back the SVG.
//!
//! ```
//! let spec = "
//! version: 1
//! diagram: flow
//! title: Order flow
//! edges:
//!   - \"browser -> api: request\"
//!   - \"api -> database: query\"
//! ";
//! let svg = svggod::render(spec)?;
//! assert!(svg.starts_with("<svg"));
//! # Ok::<(), svggod::Error>(())
//! ```
//!
//! [`Renderer`] chooses the theme and the other options:
//!
//! ```
//! # let spec = "version: 1\ndiagram: flow\ntitle: T\nnodes:\n  - { id: a, label: A }\n";
//! use svggod::{Effort, Renderer};
//!
//! let svg = Renderer::new().theme("blueprint-dark").effort(Effort::High).render(spec)?;
//! # Ok::<(), svggod::Error>(())
//! ```
//!
//! # Check an SVG
//!
//! [`validate`] runs the rules over any SVG, not only one this crate drew, and
//! [`Validator`] chooses the profile, the rules and the config. [`Validator::fix`]
//! applies the fixes the rules offer.
//!
//! ```
//! use svggod::{Profile, Severity, Validator};
//!
//! let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
//! let findings = Validator::new().profile(Profile::Strict).validate("icon.svg", svg);
//! for f in findings.iter().filter(|f| f.severity == Severity::Error) {
//!     println!("{}:{} {} {}", f.file, f.line, f.rule, f.message);
//! }
//! ```
//!
//! # Features
//!
//! | Feature | Default | Adds |
//! |---|---|---|
//! | `cli` | yes | The `svggod` program, the MCP server and the [`app`] module. Pulls in `clap` and `preview`. |
//! | `preview` | yes | [`rasterize`], an SVG to PNG renderer. Pulls in `resvg`. |
//!
//! A library that only draws and checks SVG can leave both out:
//!
//! ```toml
//! svggod = { version = "1", default-features = false }
//! ```
//!
//! # Stability
//!
//! What this page lists follows semantic versioning. So do the spec format, the
//! rule ids and the error codes. The modules behind it are private.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Without the program, what only its commands call is unused; that is expected.
#![cfg_attr(not(feature = "cli"), allow(dead_code, unused_imports))]

mod api;
#[cfg(feature = "cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
pub mod app;
mod catalogue;
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod commands;
mod config;
mod describe;
mod diff;
mod envelope;
mod fix;
#[cfg(feature = "cli")]
mod mcp;
#[cfg(feature = "cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
pub mod memory;
mod ops;
#[cfg(feature = "preview")]
mod preview;
mod render;
mod rules;
mod spec;
mod spec_edit;
mod svg;
mod util;
mod yaml_patch;

pub use api::{Description, Fixed, Renderer, Validator, describe, render, themes, validate};
#[cfg(feature = "preview")]
#[cfg_attr(docsrs, doc(cfg(feature = "preview")))]
pub use api::{Image, rasterize};
pub use config::Profile;
pub use envelope::{AppError as Error, Finding, Fix, Severity};
pub use render::effort::Effort;

/// The version of this crate, as `svggod --version` prints it.
pub const VERSION: &str = envelope::VERSION;

/// Compiles the Rust examples in the README, so they cannot go stale.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;
