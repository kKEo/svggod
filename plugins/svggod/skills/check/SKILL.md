---
name: check
description: "Check, repair or edit SVG files that already exist with svggod, including SVGs drawn in other tools (Inkscape, Figma, draw.io, Mermaid exports, hand-written). Use when the user asks to validate, lint, review, audit, clean up or fix an SVG or a folder of SVGs; asks why an SVG is cut off, unreadable in dark mode, does not scale, has overlapping or tiny text, or is unsafe to publish; wants one attribute or element of an SVG changed; wants an SVG check in CI; or asks about svggod rules, profiles, svggod.toml or svggod-disable comments. To make a new diagram or change one that has a spec, use the companion `draw` skill."
argument-hint: "[SVG files or globs] [--fix]"
---

# Check and repair SVG files with svggod

`svggod validate` runs 39 rules in four groups on any SVG: structure (is it valid), geometry
(does everything fit and not collide), portability (will it look the same elsewhere, is it safe)
and accessibility (title, contrast, dark pages). `fix` repairs most findings; `edit` changes one
thing; `inspect` shows what is inside.

If `svggod --version` fails, use the `install` skill first.

**First ask where the SVG came from.** If a spec file sits beside it (same name, `.yaml`), or the
Markdown page holds a ```` ```svggod ```` block for it, the SVG is generated: change the spec and
render again with the `draw` skill. Edits to a generated SVG are lost on the next render.

## The loop

1. **Validate.** Quote globs so svggod expands them.

   ```sh
   svggod validate 'docs/**/*.svg' --format text
   ```

   One line per finding: line number, severity, rule, target, sentence. `[fix]` means a safe
   repair exists; `[fix --unsafe]` means the repair changes how the picture looks. The last line
   is the summary. Exit 1 means at least one error (or warning, with `--fail-on warning`).

   | Severity | Meaning | Fails the check |
   |---|---|---|
   | error | Wrong, unreadable or unsafe | Yes |
   | warning | Probably a problem | Only with `--fail-on warning` |
   | info | Tidiness | No |

   The list stops at 50 findings; the summary counts stay complete. Raise it with
   `--max-findings 200`, or narrow with `--rules ID,ID`.

2. **Preview the repair, then apply it.**

   ```sh
   svggod fix fig.svg --dry-run --format text      # the diff and what would remain
   svggod fix fig.svg -o fig.fixed.svg --format text   # safe repairs, original kept
   svggod fix fig.svg --in-place --format text         # safe repairs, file replaced
   ```

   Safe repairs keep the content, colours and positions of text, but they are not invisible:
   they can widen a box to fit its text, grow the viewBox, reattach a connector, thicken a
   hairline and add a background. `--unsafe` also renames duplicate ids, recolours low-contrast
   text, moves labels off lines and enlarges tiny text. So:

   - Use `--in-place` only when the file is under version control with no uncommitted changes,
     or the user said to replace it. Otherwise write with `-o` and keep the original.
   - Use `--unsafe` only when the user asked for it or agreed after seeing the dry run.
   - `fix` works in rounds until nothing more can be repaired, so it may report fixes for
     rules that `validate` did not list (they appear once an earlier repair is in).
   - One file per run, with exactly one of `--dry-run`, `-o OUT` or `--in-place`.

3. **Deal with what is left by hand.** Some findings need a decision only a person or the
   context can supply: `title-missing` and `desc-missing` (write the text), `dangling-reference`,
   `foreign-object`, `external-resource`, `text-overlap`. Use `inspect` to find the element and
   `edit` to change it; see `references/editing.md`. To learn what a rule wants:

   ```sh
   svggod rules --explain low-contrast --format text    # with a failing and a passing example
   ```

4. **Look at it.** Before and after a repair that changes the look:

   ```sh
   svggod preview fig.svg -o "$TMPDIR/fig.png" --annotate --force --format text
   ```

   `--annotate` outlines each finding and prints its rule id beside it. `--background dark`
   shows the picture on a dark page; `--crop '#id'` zooms in on one element. Read the PNG.

5. **Validate again** and report the summary line: what was fixed, what remains and why.

## Things to know

- **Always pass `--format text`** when you read the output yourself; without a terminal the
  default is JSON. Use JSON when you want to process findings: each has `rule`, `severity`,
  `file`, `target`, `line`, `message`, and, when fixable, a `fix` with the exact edit operations.
- **`fix` exits 1 while errors remain.** That is a report, not a failure of the command.
- **Files from other tools:** geometry findings rest on estimated positions, so they are never
  rated above warning, whatever the rule's normal severity and whatever `svggod.toml` says. The
  mark is `measured.approximate` in the JSON; text output does not show it. To make such
  findings fail a build, use `--fail-on warning`. Confirm them in the preview before acting.
- **Targets go stale.** A repair or edit can insert or remove elements, which renumbers
  `:nth-of-type(n)` targets. Validate again before reusing a target from an earlier report.
- **Config changes the result.** svggod uses the nearest `svggod.toml` above each file and logs
  `svggod: config <path>` on stderr. `--no-config` ignores it; `--profile minimal|docs|strict`
  sets the strictness for one run. For pictures you cannot change, `--profile minimal` checks
  only validity and safety. Details in `references/rules-and-config.md`.
- **Do not silence a finding to get a green result.** Suppress only when the finding is wrong
  for that element, and say so to the user. The comment needs a reason:
  `<!-- svggod-disable shape-overlap: badge sits on the corner by design -->`.
- **Unknown or untrusted files:** svggod has no time limit and a 1 GB memory cap. For a file of
  doubtful origin, lower the cap and add a timeout:
  `timeout 30 svggod --max-memory 64MB validate suspect.svg --format text`.
- **`validate` takes SVG only.** A YAML spec given to it is reported as `xml-well-formed`;
  render the spec instead.
- **Writes are atomic** and `-o` never overwrites silently, so a failed run leaves no half file.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | No finding at or above the fail level |
| 1 | Findings at or above the fail level (`preview` and `inspect` never give 1) |
| 2 | Wrong command line |
| 3 | Bad input: file missing, not UTF-8, over 64 MB, invalid config, a selector that matches nothing |
| 4 | The output file exists (`preview` needs `--force`), or writing failed |
| 5 | Bug in svggod; report the command and the input |

## In CI

```sh
svggod validate 'docs/**/*.svg' --fail-on warning      # hand-made and foreign SVGs
svggod render fig/orders.yaml -o fig/orders.svg --check   # a spec and its picture are in step
svggod docs docs/ --check                               # diagrams inside Markdown pages
```

All three write nothing and exit 1 on a problem. A glob that matches no file is an error
(`file-not-found`, exit 3), so add the `validate` line only where SVGs exist. In CI the JSON default is what you want; add
`--format text` only for human-readable logs.

## Read these when the task needs them

- `references/rules-and-config.md` — all 39 rules with severity and fix type, profiles,
  `svggod.toml` keys, per-file overrides and suppression comments. Read when deciding whether a
  finding matters, when setting project rules, or when a result differs from what you expect.
- `references/editing.md` — selectors, `inspect`, `edit --set` and `edit --ops` with every
  operation. Read before changing an SVG by hand.
