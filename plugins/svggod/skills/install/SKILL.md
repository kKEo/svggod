---
name: install
description: "Install the svggod command line tool on this machine and confirm it works. Use when the user asks to install, set up, update or reinstall svggod (also written SVGGod, formerly svgdog), when a command fails with 'svggod: command not found', or when another svggod skill reports that the binary is missing. Also covers registering svggod as an MCP server when the user asks for that."
argument-hint: "[--from-source] [--mcp <root-dir>]"
allowed-tools: ["Bash(command -v svggod)", "Bash(svggod --version)", "Bash(svggod examples:*)", "Bash(svggod render:*)", "Bash(cargo --version)", "Bash(rustc --version)", "Read"]
---

# Install svggod

svggod is one self-contained binary for macOS and Linux (no Windows build). It needs no system
libraries and never uses the network once installed. Installing changes the user's machine, so say
which route you are about to take before running it.

## 1. Check what is there

```sh
command -v svggod && svggod --version
```

If it prints a version, stop: report the path and version. Reinstall only when the user asked for
an update or the version is older than they need. An old `svgdog` binary is the same tool under
its former name; install `svggod` beside it rather than aliasing.

## 2. Install

Use the first route that applies.

**From crates.io (the normal route).** Builds from source; takes a few minutes.

```sh
cargo install svggod --locked
```

**From a checkout** (the user is working in the svggod repository, or asked for an unreleased
version):

```sh
cargo install --path . --locked
```

**Straight from GitHub**, without a checkout:

```sh
cargo install --git https://github.com/kKEo/svggod --locked
```

All three need Rust 1.88 or newer and put the binary in `~/.cargo/bin`.

If `cargo` is not found:
- Look for a Homebrew rustup that is not on the PATH: `ls /opt/homebrew/opt/rustup/bin/cargo`.
  If it exists, prefix the command: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo install ...`.
- Otherwise Rust is not installed. Ask the user before installing it (`brew install rustup` then
  `rustup default stable`, or the installer from https://rustup.rs). Do not install a toolchain
  unasked.

If `rustc --version` is below 1.88, run `rustup update stable` (ask first).

**Prebuilt binaries.** The project README also lists `cargo binstall svggod` and
`brew install kKEo/tap/svggod`. When this skill was written (svggod 1.1.2) no prebuilt binary or
Homebrew formula had been published for the `svggod` name, so do not rely on them. If the user
already has `cargo-binstall`, `cargo binstall svggod` is still safe: it falls back to building
from source.

## 3. Verify

```sh
command -v svggod && svggod --version
```

If the install succeeded but the command is not found, `~/.cargo/bin` is not on the PATH. Tell
the user to add `export PATH="$HOME/.cargo/bin:$PATH"` to their shell profile; until then call
the binary by its full path.

Then a smoke test in a temporary folder:

```sh
tmp=$(mktemp -d)
svggod examples minimal --format text > "$tmp/first.yaml"
svggod render "$tmp/first.yaml" -o "$tmp/first.svg" --format text
```

The last command must print `0 errors` and exit with 0. Report the version, the path and the
result of the smoke test. Remove the temporary folder afterwards.

## 4. MCP server (only when asked)

svggod can also serve its commands as MCP tools. Register it only if the user wants that; the
skills in this plugin work through the shell and do not need it.

```sh
claude mcp add svggod -- svggod mcp --root-path ./docs
```

`--root-path` is required: it is the one folder the server may read and write. Ask the user which
folder to use rather than guessing.
