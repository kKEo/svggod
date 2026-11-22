# SVGGod

A single-binary command line tool, and a Rust library, that compiles a declarative
diagram description to SVG and checks any SVG for correctness, legibility and
portability. It is built
for diagrams in technical documentation, and for two kinds of caller: AI agents
that use it as a tool, and people or CI pipelines that lint diagrams in a
documentation repository.

> **Renamed from svgdog.** Version 1.1.0 renamed the tool from `svgdog` to `svggod`. Files made
> for the old name keep working: `svgdog.toml` is read when no `svggod.toml` is beside it,
> `<!-- svgdog-disable ... -->` comments and ` ```svgdog ` blocks are still recognised, SVGs
> marked `data-svgdog` or carrying `svgdog-spec` metadata count as our own output, and the MCP
> server answers the old `svgdog_*` tool names. New output uses only the new names.

## Documentation

Full documentation lives in [`docs/guide/`](docs/guide/index.html): a landing page, a
[TL;DR deck](docs/guide/tldr.html) and three books, the
[user guide](docs/guide/user/README.md), the
[operator guide](docs/guide/operator/README.md) and the
[tester guide](docs/guide/tester/README.md). `make docs` builds them with mdBook.

## Install

```sh
cargo install svggod                 # from crates.io, builds from source
cargo binstall svggod                # prebuilt binary, needs cargo-binstall
brew install kKEo/tap/svggod         # Homebrew (macOS and Linux)
```

Or download a tarball for Linux (x86_64 and arm64, static) or macOS (Apple silicon and
Intel) from the [releases page](https://github.com/kKEo/svggod/releases) and check it
against `SHA256SUMS`.

## Use as a library

The same crate is a library. It draws and checks without touching stdout, the process
arguments or the exit code, and never looks for a config file you did not name.

```toml
[dependencies]
svggod = { version = "1", default-features = false }
```

```rust
use svggod::{Profile, Renderer, Severity, Validator};

fn main() -> Result<(), svggod::Error> {
    let spec = "version: 1\ndiagram: flow\ntitle: Request path\nedges:\n  - \"browser -> api: request\"\n";

    let svg = Renderer::new().theme("blueprint").render(spec)?;       // spec text in, SVG out
    let findings = Validator::new().profile(Profile::Strict).validate("request.svg", &svg);
    assert!(findings.iter().all(|f| f.severity < Severity::Error));

    println!("{}", svggod::describe(spec)?.short);                    // the diagram in words
    Ok(())
}
```

| Feature | Default | Adds |
|---|---|---|
| `cli` | yes | The `svggod` program and the MCP server (`clap`); turns on `preview` |
| `preview` | yes | `svggod::rasterize`, SVG to PNG (`resvg`) |

With `default-features = false` neither `clap` nor `resvg` is built. The API is small on
purpose: `render` and `Renderer`, `validate` and `Validator` (with `fix`), `describe`,
`rasterize`, `themes`, and the `Error`, `Finding` and `Severity` types. It follows
semantic versioning; everything else in the crate is private. The reference is on
[docs.rs](https://docs.rs/svggod), and `cargo doc --open` builds it locally.

## Build

```sh
cargo build --release      # the binary is target/release/svggod
cargo test                 # unit, library, command line and MCP tests
```

No network, browser or language runtime is needed at run time. Fonts (Inter and
JetBrains Mono, both under the SIL Open Font License, see `fonts/`) are compiled in.

## Diagram types

`flow`, `layers`, `sequence` and `free` for structure; `state` for state machines,
`timeline` for things in time order, and `compare` for two versions side by side.
`svggod examples` lists a small spec for each.

## Explaining, not only drawing

A spec can carry the story as well as the structure: `steps` number a walk-through and
list it under the drawing, `focus` dims everything else, `legend` and `caption` add the
words a figure needs, and `detail` and `link` on any item give tooltips and links.
`render --frames` writes one picture per step. `svggod examples steps` shows all of it.

For readers on a web page there are two interactive versions, both the same drawing
with something added: `--interactive css` is an SVG that highlights what a hovered box
is connected to, with no script; `--interactive html` is a self-contained page with
Previous and Next buttons for the steps. The static SVG stays the default.

## In documentation

`svggod docs docs/` renders every `svggod` code block in your Markdown pages to an SVG
next to the page and keeps an image line under the block pointing at it; `--check`
fails when a picture is out of date. `--theme docs-auto` gives one file that follows
the reader's light or dark mode. `svggod describe` turns a diagram into words, and
`render --embed-spec` stores the spec in the SVG so it can be read back.

## Themes

A theme sets colours, font, text sizes, line widths and corner radii. Five are built
in: `docs` (the default), `blueprint`, `paper`, `bold` and `cards`, each with a light, a dark
(`-dark`) and an adaptive (`-auto`) variant. Name one with `--theme`, with `theme:` in
a spec, or with `[style] theme` in `svggod.toml`.

```sh
svggod theme list                               # the built-in themes
svggod render fig/orders.yaml -o fig/orders.svg --theme paper-dark
svggod theme init themes/acme --from paper      # a theme of your own: one theme.toml
svggod theme check themes/acme                  # the file, contrast, and every example drawn with it
svggod render fig/orders.yaml -o fig/orders.svg --theme ./themes/acme
```

Themes are read from disk only; nothing is downloaded.

## Changing a diagram

A diagram that readers know should not rearrange itself when one box is added.
`svggod render orders.yaml -o orders.svg --previous orders.svg` keeps every box that
is in both versions where it was and puts new boxes into free room. When that cannot
be done, the command stops with `layout-changed` and says why; `--force` lays the
diagram out afresh. `svggod docs` does the same for the pictures it already wrote.

## The loop

```sh
svggod examples flow --format text      # a spec to start from (schema spec has every field)
svggod render orders.yaml -o orders.svg  # compile and check in one call
svggod fix orders.svg --in-place         # apply the safe fixes findings carry
svggod preview orders.svg -o orders.png --annotate   # look at it
```

Repeat render (or fix) until the exit code is 0.

## Commands

| Command | Purpose |
| --- | --- |
| `render <spec> -o <out.svg>` | Compile a YAML or JSON spec to SVG and validate the result; `--previous` keeps an earlier layout |
| `validate <file.svg>...` | Run the 39 rules on SVG files, including ones SVGGod did not make |
| `inspect <file.svg>` | List visible elements with their bounding boxes, as data |
| `preview <file.svg> -o <out.png>` | Rasterize to PNG; `--annotate` outlines findings, `--crop` zooms in |
| `edit <file> --set ... \| --ops ...` | Targeted changes to an SVG or a spec |
| `fix <file.svg>` | Apply autofixes; `--unsafe` adds those that move or recolour |
| `schema spec\|config\|findings\|scene\|ops\|theme` | Print a JSON Schema |
| `rules [--explain ID]` | List rules, or explain one with a failing and a passing example |
| `examples [NAME]` | List the bundled example specs, or print one to start from |
| `docs <path>... [--check]` | Render the `svggod` blocks inside Markdown pages |
| `describe <spec or svg>` | Say in words what a diagram shows |
| `theme list \| init DIR \| check DIR` | List the built-in themes, start one of your own, check it |
| `mcp --root-path DIR` | Serve the commands as Model Context Protocol tools over stdio |

Every command has `--help` with one example. With `--format json` (the default
when stdout is not a terminal) each command prints exactly one JSON object.

Exit codes: `0` success, `1` findings at or above the failure level (`validate` takes `--fail-on`; other
commands fail on errors), `2` usage error,
`3` input error, `4` output error, `5` internal error.

## A spec

```yaml
version: 1
diagram: flow            # flow, layers, sequence or free
title: Order placement
direction: right

nodes:
  - {id: web, label: Storefront, kind: client}
  - {id: api, label: Order API, kind: service, emphasis: true}
  - {id: db, label: Orders DB, kind: datastore}

groups:
  - {id: backend, label: Backend, members: [api, db]}

edges:
  - {from: web, to: api, label: POST /orders}
  - {from: api, to: db, label: insert}

notes:
  - {attach: db, text: Nightly backup}
```

The author states structure; SVGGod chooses every coordinate. More examples are in
`tests/fixtures/`.

## Configuration

One `svggod.toml`, found by walking up from the input file:

```toml
version = 1
profile = "docs"                 # minimal, docs or strict

[target]
render_width = 720               # px the diagram is shown at
backgrounds = ["light", "dark"]

[style]
palette = ["#1f2937", "#ffffff", "#2563eb", "#dbeafe", "#6b7280"]
stroke_widths = [1, 1.5, 2]
grid = 8

[rules]
excess-margin = "off"
font-too-small = { severity = "error", min_px = 10 }
```

A rule is silenced for one element with a comment directly before it; the reason
is required:

```xml
<!-- svggod-disable shape-overlap: badge sits on the corner by design -->
```

## MCP

```sh
svggod mcp --root-path ./docs
```

- MCP Registry name: `mcp-name: io.github.kKEo/svggod`

The server exposes `svggod_render`, `svggod_validate`, `svggod_inspect`,
`svggod_preview`, `svggod_edit`, `svggod_fix`, `svggod_schema`, `svggod_rules`,
`svggod_docs`, `svggod_describe`, `svggod_examples` and three `svggod_theme_*` tools. `svggod_render` also takes the spec as text and can return the SVG
and a PNG directly, so an agent needs no files to draw.
Inputs mirror the flags and results are the same JSON envelope. Every path is
relative to the root and paths outside it are refused. It serves both the
`initialize` handshake and the per-request protocol style of MCP 2026-07-28.

## Claude Code skills

This repository is also a Claude Code plugin marketplace. The `svggod` plugin gives
Claude three skills: `svggod:install` (put the binary on the machine), `svggod:draw`
(write a spec, render it, look at the result) and `svggod:check` (check, repair or
edit SVG files that already exist).

```sh
claude plugin marketplace add kKEo/svggod      # or the path of a checkout
claude plugin install svggod@svggod
```

The skills live in [`plugins/svggod/`](plugins/svggod/README.md).

## Development notes

- `tests/golden/` holds expected output, compared byte for byte. After an intended
  change, regenerate with `SVGGOD_UPDATE_GOLDEN=1 cargo test r8_output_matches`.
- `.github/workflows/ci.yml` runs the tests on Linux and macOS; passing on
  both is the cross-platform determinism check. Windows is not built or tested.
- `.github/workflows/release.yml` publishes a release when a version tag is pushed.
  Run `make bump` (`BUMP=minor` or `BUMP=major` for larger steps; it commits and tags
  but does not push), then `git push origin master vX.Y.Z`. The workflow reruns the
  checks, refuses a tag that does not match `Cargo.toml`, and then:
  - creates the GitHub release with binaries for Linux (x86_64 and arm64, static) and
    macOS (Apple silicon and Intel), plus `SHA256SUMS`;
  - publishes the crate to crates.io (Trusted Publishing, no stored token);
  - updates the formula in the `kKEo/homebrew-tap` repository;
  - lists the new version in the MCP Registry (`server.json`).

  A tag with a suffix such as `v1.1.0-rc.1` only creates a GitHub pre-release.
  Running the workflow by hand from the Actions tab builds the binaries without
  publishing. `make release-check` dry-runs the crates.io package locally.
- One-time setup for the channels above:
  1. crates.io: publish the first version by hand (`cargo login`, `cargo publish`),
     then add this repository and `release.yml` under the crate's Trusted Publishing
     settings.
  2. Homebrew: create the public repository `kKEo/homebrew-tap` and add a
     `TAP_GITHUB_TOKEN` secret here (fine-grained token, Contents read and write on
     that repository only).
  3. MCP Registry: nothing to set up. GitHub OIDC proves the `io.github.kKEo`
     namespace, and the registry reads the `mcp-name:` line from the crates.io README,
     so the crate has to be published first.
  Other MCP directories (Smithery, Glama and similar) are listed by hand.
- `svggod rules --explain ID` examples are executed by the tests: each failing
  example must raise its rule, each passing one must not, and each fix must clear
  its finding.

## Known limits

- The program holds at most 1 GB of memory; an input that needs more ends it with the
  error `memory-limit` (exit 3). `--max-memory 512MB` or `SVGGOD_MAX_MEMORY` sets another
  limit, `0` none. Input limits are listed in the operator guide.
- Geometry findings on SVG files SVGGod did not render rest on heuristics; they are
  marked `approximate` and capped at warning.
- Editing a YAML spec keeps comments and layout for field changes, renames and
  additions. Removing a node, group or edge rewrites the spec in canonical form;
  the result says `"formatting": "rewritten"`.
- A drawing wider than `max_width` is first set more tightly; if it still does not
  fit, render fails with advice instead of producing an unreadable figure.
