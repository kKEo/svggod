# 5. Let an agent and CI do the work

**Level: most advanced.** The first four workflows have a person running commands. Here you set the rules once, let your pipeline enforce them on every pull request, and let an AI assistant draw, check and repair diagrams on its own. Twenty minutes.

This is the end state: nobody has to remember anything, and a diagram that breaks the house rules cannot be merged.

## The workflow at a glance

```text
svggod.toml  ->  house rules for everyone (people, CI, agents)
CI           ->  validate + docs --check on every pull request
agent (MCP)  ->  render -> validate -> fix -> preview, inside one folder
```

## 1. Write the house rules once

Put a `svggod.toml` at the root of the repository. SVGGod looks for it next to the file it checks and then in each folder above, and uses the nearest one, so the same file governs your terminal, your pipeline and your agent.

```toml
version = 1
profile = "docs"                  # minimal, docs or strict

[target]
render_width = 720                # px the diagram is shown at
backgrounds = ["light", "dark"]   # page colours it must be readable on

[rules]
font-too-small = { severity = "error", min_px = 10 }

# Old pictures we cannot redraw yet: report these two as warnings, not errors.
[[overrides]]
files = ["docs/legacy/**"]
rules = { low-contrast = "warning", title-missing = "warning" }
```

What each part does:

- `profile` sets the starting strictness for every rule (see [workflow 2](2-check-any-svg.md#choosing-how-strict-to-be)).
- `[target]` says how big and on which backgrounds the pictures are shown. Rules about text size and contrast measure against it.
- `[rules]` switches a rule off, or changes its severity and limits.
- `[[overrides]]` loosens or tightens rules for the files that match. Here two rules stop counting as errors in the legacy folder, and the rest of the repository stays as strict as before. Every other error in that folder still fails the build. Later blocks win.

You can also set `[style]` (an allowed palette, stroke widths and grid) so that every diagram in the project looks like it belongs.

On a folder holding one good SVG and the broken legacy one, the same command gives different results with and without the file:

```text
$ svggod --format text validate 'docs/**/*.svg'
2 files checked: 4 errors, 14 warnings, 2 info, 15 fixable      <- with svggod.toml: 3 errors became warnings
$ svggod --format text validate 'docs/**/*.svg' --no-config
2 files checked: 7 errors, 11 warnings, 2 info, 15 fixable      <- defaults
```

## 2. Gate every pull request

Everything the CI needs is an exit code. Quote the glob so SVGGod expands it, which keeps `**` and the file list stable.

```yaml
# .github/workflows/diagrams.yml
name: diagrams
on: [pull_request]

jobs:
  diagrams:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install SVGGod
        run: cargo install svggod --version 1.0.4 --locked
      - name: Pictures in pages match their specs
        run: svggod docs docs/ README.md --check
      - name: Every SVG passes the house rules
        run: svggod validate 'docs/**/*.svg'
```

Three details matter:

- **Pin the version.** SVGGod output is deterministic, so a difference means the spec or the tool changed. Pin the version, or the `--check` step will fail on a tool upgrade.
- **Errors fail the job, warnings do not.** That is the default, and it is what lets the override above work: a legacy finding you downgraded to a warning is reported but does not block the merge. Add `--fail-on warning` only on a repository with no such exceptions, because it fails on downgraded findings too.
- **Failures as a list.** Output is JSON in CI, so you can turn it into annotations:

  ```bash
  svggod --format json validate 'docs/**/*.svg' \
    | jq -r '.findings[] | select(.severity=="error") | "\(.file):\(.line): \(.rule): \(.message)"'
  ```

  Each line has the form `file:line: rule: message`, which most log viewers turn into a clickable annotation.

Prefer a prebuilt binary to `cargo install`? Download the tarball for your platform from the [releases page](https://github.com/kKEo/svggod/releases) and check it against `SHA256SUMS`.

## 3. Hand the tools to an AI agent

There are two ways. The simpler one needs nothing: tell a coding assistant that `svggod` is installed. It reads the format with `svggod schema spec`, writes a spec, and repeats `svggod render` until the exit code is `0`.

The better one is the **Model Context Protocol (MCP)**: SVGGod serves its commands as tools over stdio.

```bash
svggod mcp --root-path ./docs
```

`--root-path` is the one folder the agent may read and write. Every path is relative to it and anything outside is refused with `outside-root`. That is the only access control, so point it at the documentation folder, never at a home directory.

Register it with your client. For Claude Code:

```bash
claude mcp add svggod -- svggod mcp --root-path ./docs
```

Clients that use a JSON file take an entry of this shape (check your client's documentation for the file name and keys):

```json
{
  "mcpServers": {
    "svggod": {
      "command": "svggod",
      "args": ["mcp", "--root-path", "/absolute/path/to/docs"]
    }
  }
}
```

The server offers a tool for every command: `svggod_render`, `svggod_validate`, `svggod_inspect`, `svggod_preview`, `svggod_edit`, `svggod_fix`, `svggod_schema`, `svggod_rules`, `svggod_docs`, `svggod_describe` and `svggod_examples`. Inputs mirror the flags and results are the same JSON as on the command line. `svggod_render` can also take the spec as text and return the SVG and a PNG directly, so an agent needs no files to draw.

You can try it without a client by piping the handshake in:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | svggod mcp --root-path ./docs --quiet
```

## 4. Ask for the loop, not for a picture

Good requests name the diagram, the kind and where it goes, and ask the agent to look at the result:

> Draw the checkout flow as a left-to-right flow diagram with SVGGod. Save the spec as `docs/checkout.yaml` and the SVG as `docs/checkout.svg`. Keep going until `svggod render` exits with 0, then preview it and tell me if anything is hard to read.

Why this works well:

- **A short loop.** Errors in a spec say what is wrong and suggest a fix (`Did you mean 'label'?`).
- **Safe repairs.** `fix` applies only repairs that do not change the look, unless you allow `--unsafe`.
- **Same result every time.** A spec that works today draws the same picture tomorrow, so the agent's output can be reviewed like code.
- **Contained.** The agent can only touch files under the root, and SVGGod never uses the network.

Ask the agent to **look at the picture**, not only the exit code: a diagram can pass every rule and still be confusing. And keep the specs in the repository next to the SVGs, so the agent, a reviewer or you can change a spec and redraw instead of editing SVG by hand.

## 5. Put it together

The whole system, in one paragraph. People and agents write **specs**, either as files or as blocks inside Markdown pages ([workflows 1 and 3](3-diagrams-in-pages.md)). `svggod.toml` holds the **house rules**. Edits keep the **layout** steady ([workflow 4](4-explain-and-change.md)). The agent drafts and repairs through **MCP**, inside one folder. Every pull request runs **`docs --check`** and **`validate`**, so a stale picture or a broken SVG turns the build red. The only thing a human has to do is look at the picture and decide whether it says the right thing.

## What you learned

| You did | With |
|---|---|
| Set rules for the whole project | `svggod.toml` with `[rules]` and `[[overrides]]` |
| Blocked bad diagrams in review | `svggod validate`, `svggod docs --check` |
| Let an assistant draw and repair | `svggod mcp --root-path` |

## Where to go next

- The [User guide](https://kkeo.github.io/svggod/user/) covers each diagram type and every spec field.
- The [Operator guide](https://kkeo.github.io/svggod/operator/) has the full configuration reference, the exit codes, the rule catalogue and more CI recipes.
