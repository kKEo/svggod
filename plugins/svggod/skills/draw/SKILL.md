---
name: draw
description: "Draw or change a diagram for documentation with svggod: write a short YAML spec, render it to SVG, check it and look at the result. Use whenever the user asks for a diagram, figure or chart of a system — architecture, request or data flow, layers or tiers, sequence of messages, state machine, timeline or roadmap, before/after comparison — to put in docs, a README or a Markdown page, even when svggod is not named, and whenever a task touches a svggod spec, a ```svggod block, `svggod render`, `svggod docs`, themes or step-by-step walk-throughs. For SVG files that already exist and have no spec, use the companion `check` skill."
argument-hint: "[what to draw] [where to save it]"
---

# Draw a diagram with svggod

svggod turns a small YAML spec into an SVG and checks the result against 39 rules in the same
run. You never write SVG by hand: you write the spec, render, read the report, look at the
picture, and repeat. The same spec always gives the same bytes.

If `svggod --version` fails, use the `install` skill first.

## The loop

1. **Pick a type and start from its example.** Do not write a spec from memory.

   ```sh
   svggod examples flow --format text > docs/fig/orders.yaml
   ```

   | `diagram` | Use for |
   |---|---|
   | `flow` | Components and what calls what. The default choice |
   | `layers` | Tiers: each top-level group is a band |
   | `sequence` | Messages over time between a few participants |
   | `state` | States and transitions, with `start` and `end` markers |
   | `timeline` | Milestones along an axis, grouped into phases (no edges) |
   | `compare` | Two groups side by side, such as before and after |
   | `free` | Every node placed by hand with `at: [x, y]`. Last resort |

   Examples: `minimal`, `flow`, `layers`, `sequence`, `free`, `state`, `timeline`, `compare`,
   `steps`.

2. **Edit the spec** with your file tools. The shape:

   ```yaml
   version: 1                      # required
   diagram: flow                   # required
   title: Order placement          # required; the accessible name, not drawn
   description: How an order moves from the storefront to fulfilment.
   direction: right                # or down
   nodes:
     - { id: web, label: Storefront, kind: client }
     - { id: api, label: Order API, kind: service, emphasis: true }
     - { id: db, label: Orders DB, kind: datastore }
   groups:
     - { id: backend, label: Backend, members: [api, db] }
   edges:
     - "web -> api: POST /orders"  # quote any edge that has a label
     - "api --> db: insert"        # --> async, ..> dependency, <-> both ways, -- no arrow
   notes:
     - { attach: api, text: Validates stock first }
   ```

   Every key, limit and per-type rule is in `references/spec.md`. Read it before writing anything
   beyond what the example already shows.

3. **Render.** Rendering also validates.

   ```sh
   svggod render docs/fig/orders.yaml -o docs/fig/orders.svg --format text
   ```

   Exit 0 with `0 errors` means the drawing passed. Exit 3 means nothing was written: either
   the spec was refused, or it is valid but cannot be laid out (`too-wide`). Fix what the
   message names and run again. Exit 1 means the drawing has error findings (rare for a
   rendered spec): shorten labels or change the structure in the spec; do not patch the SVG.

4. **Look at it.** A drawing can pass every rule and still be confusing.

   ```sh
   svggod preview docs/fig/orders.svg -o "$TMPDIR/orders.png" --force --format text
   ```

   Read the PNG. The rules do not judge clarity: a drawing with zero findings can still have
   an edge label that sits nearer to the wrong line, or a note wedged between arrows. Check
   that the reading order is clear, each label plainly belongs to its line, arrows do not
   cross needlessly and the main thing stands out. If not, change the spec and render again:
   the other `direction`, groups, fewer nodes, shorter edge labels, a note's `side`, or
   `--effort high`.

5. **Report** the spec path, the SVG path and the summary line (`1 file checked: 0 errors,
   ...`), plus the `Layout kept: ...` line when `--previous` was used. Keep the spec in the
   repository next to the SVG; it is the source.

## Things that cost time if you do not know them

- **Always pass `--format text`.** In an agent's shell stdout is not a terminal, so the default
  is JSON. `svggod examples flow > x.yaml` without the flag writes JSON into the file. Use JSON
  only when you want to parse findings. Global flags such as `--format` and `--max-memory 256MB`
  go before or after the subcommand.
- **Two edges between the same pair need their own `id`** (long form), or the render fails with
  `duplicate-id`. This is common in a `sequence`, where the same two participants exchange
  several messages.
- **Spec errors arrive one per run.** Three mistakes take three runs. Read `references/spec.md`
  first rather than discovering the format through errors.
- **Rendering onto an existing file fails** with `output-exists` (exit 4). When changing a
  diagram readers already know, add `--previous OUT.svg` (may be the output itself): boxes with
  the same `id` stay where they were. For a first draft you are still shaping, use `--force`.
  If `--previous` fails with `layout-changed`, shorten the change or add `--force`.
- **Keep ids stable** when rewording labels. A renamed `id` is a new box and moves the layout.
- **Quote edges with labels.** Unquoted `a -> b: text` is a YAML mapping. Spaces around the
  arrow are required.
- **There are no colours or sizes on nodes.** Looks come from `kind`, `emphasis` and the theme.
  `color:` and similar are `unknown-field`.
- **`too-wide`:** try `direction: down`, shorter labels, or a larger `max_width` (default 960,
  at most 2000).
- **`-o -` prints the SVG and drops the report.** Write to a file to see findings.
- **`validate` takes SVG only.** To check a spec, render it.
- **Undeclared ids become plain boxes** labelled with the id. Declare a node once it needs a
  label, a kind or emphasis.
- **`svggod edit` on a spec rewrites the whole YAML** (comments and compact rows are lost). Edit
  spec files directly instead.
- **No time limit.** `--effort max` on a large diagram can run long; start with the default and
  use `high` only when edges cross. Memory is capped at 1 GB (`--max-memory`).
- **Old names still work** in existing projects: `svgdog.toml`, ```` ```svgdog ```` blocks. Leave
  them unless asked to rename.

## Exit codes

| Code | Meaning | What to do |
|---|---|---|
| 0 | Passed | Look at the preview |
| 1 | Findings at error level | Change the spec; see the `check` skill for reading findings |
| 2 | Wrong command line | Check the flags with `svggod COMMAND --help` |
| 3 | Bad input: spec, file, theme, config | The message names the field and often the fix |
| 4 | Output exists, or layout cannot be kept | `--previous`, `--force` or a new path |
| 5 | Bug in svggod | Report the command and the input to the user |

## Read these when the task needs them

- `references/spec.md` — every spec key with limits, node kinds, arrows, and what each diagram
  type allows. **Read before writing a spec that goes beyond the example.**
- `references/explaining.md` — numbered steps, focus, legend, caption, tooltips and links;
  one picture per step (`--frames`); interactive pages. Read when the diagram must walk a reader
  through something or go on slides.
- `references/markdown-pages.md` — specs kept inside Markdown pages and `svggod docs`. Read when
  the diagram belongs in a `.md` file or the project already has ```` ```svggod ```` blocks.
- `references/themes.md` — built-in themes, light, dark and auto variants, custom themes. Read
  when asked about looks, dark mode or house style.
- `references/errors.md` — error codes and the next move for each. Read when a message is not
  enough.
