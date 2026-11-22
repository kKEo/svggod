# Quick start

This walk-through takes about ten minutes. You will build SVGGod, write a three-box diagram, turn it into an SVG, check it, and look at it as a picture.

You need a terminal and the Rust toolchain (the `cargo` command). SVGGod has no other requirements: fonts are built into the program.

> **Note.** SVGGod is built from source today. Ready-made downloads are planned for releases tagged `v1.0.0` and later; until one exists, use the steps below.

> **Using Claude Code?** You can skip the typing. Add the SVGGod skills with `claude plugin marketplace add kKEo/svggod` and `claude plugin install svggod@svggod`, then ask Claude to install SVGGod and draw a diagram. See [Claude Code skills](using-with-an-ai-agent.md#claude-code-skills). The steps below are still worth reading once, to know what Claude does on your behalf.

## 1. Build SVGGod

Get the source from the [repository](https://github.com/kKEo/svggod), then build it:

```bash
# from the folder that holds Cargo.toml
cargo build --release
# the program is now target/release/svggod
```

Put it somewhere on your `PATH`, or call it by its full path. The rest of this guide writes just `svggod`.

## 2. Write a spec

Create a file called `hello.yaml`:

```yaml
version: 1
diagram: flow
title: Hello pipeline
description: A web app calls an API, which queries a database.
nodes:
  - {id: web, label: Web app, kind: client}
  - {id: api, label: API, kind: service}
  - {id: db, label: Database, kind: datastore}
edges:
  - {from: web, to: api, label: request}
  - {from: api, to: db, label: query}
```

Every diagram needs `version: 1`, a `diagram` type, a `title` and at least one node. The `description` is optional but worth writing: it becomes the text a screen reader speaks for the picture.

## 3. Turn it into an SVG

```bash
svggod --format text render hello.yaml -o hello.svg
```

You should see:

```text
Wrote hello.svg (1790 bytes).

1 file checked: 0 errors, 0 warnings, 0 info, 0 fixable
```

The exit code is 0, which means success. `render` draws the diagram and checks the result in one go.

> **Tip.** Without `--format text`, SVGGod prints a machine-readable JSON report when its output is not a terminal. Add `--format text` for the human-readable form shown in this guide.

## 4. Look at it

```bash
svggod --format text preview hello.svg -o hello.png
```

```text
Wrote hello.png (1200 x 191 px, 15710 bytes).
```

Open `hello.png` in any image viewer. It shows the three boxes joined by two labelled arrows. `preview` writes a picture file; it does not open a window or update by itself, so run it again after each change. See [Looking at your diagram](looking-at-your-diagram.md).

## 5. Change something and repeat

Edit `hello.yaml`, for example change `label: API` to `label: Orders API`, then render again:

```bash
svggod --format text render hello.yaml -o hello.svg --force
```

`--force` lets SVGGod replace the file from the previous run. Without it SVGGod refuses and exits with code 4, so you never overwrite a file by accident.

## 6. Check an SVG that you did not make

SVGGod can also check pictures from other tools. The repository ships a deliberately broken one:

```bash
svggod --format text validate tests/fixtures/broken.svg
```

It lists 20 findings and ends with `1 file checked: 7 errors, 11 warnings, 2 info, 15 fixable` and exit code 1. Then ask SVGGod to repair what it safely can:

```bash
svggod --format text fix tests/fixtures/broken.svg --in-place
```

Learn more in [Checking and fixing SVGs](checking-and-fixing.md).

## What you learned

- A spec is a short text file; `render` turns it into an SVG and checks it.
- Exit code 0 means everything is fine; a non-zero code means something needs your attention.
- `preview` makes a PNG so you can see the result.
- `validate` and `fix` work on any SVG.

Next: [Everyday use](everyday-use.md) shows which command to reach for in each situation.
