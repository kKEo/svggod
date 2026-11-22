# Using SVGGod with an AI agent

SVGGod was designed so that an AI assistant can make and check diagrams on its own: the format is small, every message says how to fix the problem, and every result has an exit code. This page explains how to connect an assistant and what to ask. It takes about ten minutes.

## Three ways to connect

**1. Claude Code skills.** If you use Claude Code, this is the shortest route. The SVGGod repository is also a plugin marketplace with three skills that teach Claude the tool, so it does not have to learn it from the help text each time. See [Claude Code skills](#claude-code-skills) below.

**2. Let the assistant run commands.** If your assistant can run terminal commands (as coding assistants can), tell it that `svggod` is available. It reads the format with `svggod schema spec`, writes a spec, and repeats `svggod render` until the exit code is 0. This needs no setup beyond having SVGGod installed.

**3. Connect it as a tool.** SVGGod can talk the Model Context Protocol (MCP), the standard way for assistants to call tools. You start it with:

```bash
svggod mcp --root-path ./docs
```

`--root-path` is the one folder the assistant may read and write. Every path it uses is relative to that folder, and anything outside is refused. Most assistants that support MCP ask for a command and arguments to start the server; give them `svggod` and `mcp --root-path ./docs`. Check your assistant's own documentation for where to enter this.

The server offers fourteen tools, named after the commands you already know:

| Tool | Same as |
|---|---|
| `svggod_render` | `svggod render` |
| `svggod_validate` | `svggod validate` |
| `svggod_inspect` | `svggod inspect` |
| `svggod_preview` | `svggod preview` |
| `svggod_edit` | `svggod edit` |
| `svggod_fix` | `svggod fix` |
| `svggod_schema` | `svggod schema` |
| `svggod_rules` | `svggod rules` |
| `svggod_examples` | `svggod examples` |
| `svggod_docs` | `svggod docs` |
| `svggod_describe` | `svggod describe` |
| `svggod_theme_list`, `svggod_theme_init`, `svggod_theme_check` | `svggod theme list`, `init` and `check` |

The assistant does not need files to draw. `svggod_render` accepts the spec as text and, with `preview: true`, answers with the checked drawing and a picture of it in one step. Ask for files only when you want to keep the result.

`svggod_preview` can return the picture straight to the assistant, so that it can look at its own work, when you do not give an output file. The details of each tool are in the [Operator guide](../operator/).

## Claude Code skills

A skill is a short set of instructions that Claude Code loads when a request matches it. Add the marketplace and install the plugin once:

```bash
claude plugin marketplace add kKEo/svggod      # or the path of a checkout of the repository
claude plugin install svggod@svggod
```

Inside Claude Code the same two steps are `/plugin marketplace add kKEo/svggod` and `/plugin install svggod@svggod`. The skills are available from the next session.

| Skill | What it does | When it starts |
|---|---|---|
| `svggod:install` | Puts the `svggod` program on your machine and confirms that it works | When you ask for it, or when `svggod` is missing |
| `svggod:draw` | Writes a spec, renders it, reads the report and looks at the picture. Covers all diagram types, diagrams inside Markdown pages, themes and step-by-step walk-throughs | When you ask for a diagram, even without naming SVGGod |
| `svggod:check` | Checks, repairs and edits SVG files that already exist, including ones from other tools | When you ask to check, clean up or fix an SVG |

You do not call the skills yourself. Ask in plain words:

> Draw the checkout flow for `docs/checkout.md`, top to bottom.

> Check the SVGs under `docs/` and fix what is safe to fix.

To run one by name, type it as a command, for example `/svggod:install`.

Two things to know:

- `svggod:install` builds SVGGod with `cargo`, so it needs the Rust toolchain. If that is missing, Claude asks before installing it.
- `svggod:check` keeps your original file unless it is safely under version control or you say to replace it, and it applies repairs that change how a picture looks only when you agree.

## What to ask for

Good requests name the diagram, the kind and where to save it:

> Draw the checkout flow as a top-to-bottom flow diagram with SVGGod. Save the spec as `docs/checkout.yaml` and the SVG as `docs/checkout.svg`. Keep going until `svggod render` exits with 0, then look at a preview and tell me if anything is hard to read.

Ask the assistant to **look at the picture**, not only to check the exit code. A diagram can pass every rule and still be confusing.

## Why this works well

- **Short feedback loop.** Errors in the spec say what is wrong and suggest a fix (`Did you mean 'label'?`).
- **Safe repairs.** `fix` applies only repairs that do not change the look, unless you allow `--unsafe`.
- **Same result every time.** A spec that works today will draw the same picture tomorrow.
- **Contained.** The assistant can only touch files under the root you gave it, and SVGGod never uses the network.

> **Tip.** Keep the spec files in your repository next to the SVGs. The assistant (or you) can then change a spec and redraw the picture, which is much easier than editing SVG by hand.
