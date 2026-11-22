# Use as a library

The `svggod` crate is a Rust library as well as a program. Use the library when your own Rust program draws or checks diagrams and you do not want to start a process for each one. It does the same work as the commands and gives the same bytes, but it never prints, never reads stdin, never exits the process and never looks for an `svggod.toml` you did not name.

## Add the dependency

```toml
[dependencies]
svggod = { version = "1", default-features = false }
```

`default-features = false` leaves out the program. The crate has two features:

| Feature | Default | What it adds | Dependency it pulls in |
| --- | --- | --- | --- |
| `cli` | on | The `svggod` program, the MCP server and `svggod::app`. Turns on `preview`. | `clap` |
| `preview` | on | `svggod::rasterize`, which draws an SVG as a PNG. | `resvg` |

With both off, the library still draws, checks, fixes and describes. Add `features = ["preview"]` if you also want PNG output. The minimum Rust version is 1.85, the same as for the program.

## Draw a diagram

A spec is the same YAML or JSON text that `svggod render` reads; see the [Spec reference](../reference/spec.md).

```rust
use svggod::{Effort, Renderer};

fn main() -> Result<(), svggod::Error> {
    let spec = std::fs::read_to_string("orders.yaml").expect("spec file");

    // The defaults: the `docs` theme unless the spec names one.
    let svg = svggod::render(&spec)?;

    // Or choose the options. One renderer can draw many specs.
    let renderer = Renderer::new().theme("blueprint-dark").effort(Effort::High).embed_spec(true);
    let dark = renderer.render(&spec)?;

    std::fs::write("orders.svg", svg).expect("write");
    std::fs::write("orders-dark.svg", dark).expect("write");
    Ok(())
}
```

| `Renderer` method | Same as the flag | Notes |
| --- | --- | --- |
| `theme(name)` | `--theme` | A built-in name (`svggod::themes()` lists them) or a theme folder. Wins over `theme:` in the spec. |
| `base_dir(dir)` | none | Where a theme path written in the spec starts from. The command uses the spec file's folder. |
| `effort(level)` | `--effort` | Wins over `effort:` in the spec. |
| `text_as_paths(true)` | `--text-as-paths` | |
| `embed_spec(true)` | `--embed-spec` | Lets `svggod::describe` read the drawing back. |
| `interactive(true)` | `--interactive css` | |

`render` does not validate what it drew, unlike the command. Call `validate` on the result when you want the findings.

## Check and fix an SVG

```rust
use svggod::{Profile, Severity, Validator};

fn check(name: &str, svg: &str) -> Result<String, svggod::Error> {
    let validator = Validator::new().profile(Profile::Strict).skip(&["excess-margin"])?;

    for f in validator.validate(name, svg) {
        if f.severity >= Severity::Warning {
            eprintln!("{}:{} {} {}", f.file, f.line, f.rule, f.message);
        }
    }

    // Apply the safe fixes; `true` would also apply the ones that may change the look.
    let fixed = validator.fix(name, svg, false)?;
    Ok(fixed.svg)
}
```

`name` is only what the findings call the file; nothing is read from disk. A `Finding` has the same fields as a finding in the JSON envelope and serialises to the same shape ([Envelope and exit codes](../reference/envelope-exit-codes.md)); rule ids are in the [Rules catalogue](../reference/rules.md).

The library never searches for a config file. To use one, name it with `Validator::config_file("docs/svggod.toml")`, or pass its text to `Validator::config(text)`. The file's `[rules]` and `[[overrides]]` tables apply as described in [Configuration](../reference/configuration.md); overrides are matched against the `name` you pass, relative to the config file's folder.

## Describe and rasterize

```rust
let words = svggod::describe(&spec)?;          // title, short (alt text) and the full text
println!("{}", words.short);

let image = svggod::rasterize(&svg, 1200)?;    // needs the `preview` feature
std::fs::write("orders.png", image.png).expect("write");
```

## Errors

Every fallible call returns `svggod::Error`. It implements `std::error::Error`, so it works with `?`, `anyhow` and `Box<dyn Error>`. Its fields are the same as the `error` object in the envelope:

| Field | Meaning |
| --- | --- |
| `code` | A stable identifier to branch on, such as `malformed-spec`, `invalid-value` or `unknown-theme`. |
| `message` | A full sentence for a person. |
| `path` | The place in the spec or config the error is about, such as `nodes[1].kind`. |
| `suggestion` | What to try, when there is something to suggest. |
| `exit` | The exit code the program would give for this error. |

Findings are not errors: `validate` always returns its list, even for text that is not XML (one `xml-well-formed` finding).

## What is stable

The public API is what `src/lib.rs` exports: `render`, `Renderer`, `validate`, `Validator`, `Fixed`, `describe`, `Description`, `rasterize`, `Image`, `themes`, `Error`, `Finding`, `Fix`, `Severity`, `Profile`, `Effort` and `VERSION`. It follows semantic versioning together with the spec format, the rule ids and the error codes. Every other module is private and can change in any release. The result structs and the enums are marked non-exhaustive, so a later version can add a field or a variant without breaking your build.

`Renderer`, `Validator`, `Error` and `Finding` are `Send` and `Sync`: build one renderer or validator and share it between threads.

Not in the library yet: keeping the layout of an earlier drawing (`render --previous`), edit operations (`svggod edit`), `inspect`, and the Markdown `docs` command. Run the program for those.

## Run the program from your own binary

With the `cli` feature on, `svggod::app::run(&args)` runs one command line in process and returns the exit code, printing as the program does; `svggod::app::main()` is the whole program. Use them to ship `svggod` inside another tool.

```rust
let code = svggod::app::run(&["svggod".into(), "validate".into(), "orders.svg".into()]);
```

## Reference

The API reference is on [docs.rs/svggod](https://docs.rs/svggod). `cargo doc --no-deps --all-features --open` builds it from a checkout.
