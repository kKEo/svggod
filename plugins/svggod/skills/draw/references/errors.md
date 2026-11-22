# Error codes and the next move

In text mode an error is printed on stderr as `error[code]: message`, with a suggestion on the
next line. In JSON mode it is the `error` object of the envelope (`code`, `message`, optional
`path` and `suggestion`). Read the suggestion first; it is usually the fix.

## Spec refused (exit 3)

| Code | Cause | Next move |
|---|---|---|
| `malformed-spec` | Not valid YAML or JSON | Often an unquoted edge with a label, or a colon in unquoted text. Quote it |
| `missing-field` | `version`, `diagram`, `title`, `members`, or `at` in a `free` diagram is absent | Add it |
| `unknown-field` | A key is not allowed there | Use the suggested key. There is no `color`, `style` or `size` on nodes |
| `invalid-value` | Outside the allowed set: a `kind`, an arrow, an id with capitals or underscores, a link scheme | The message lists the allowed values |
| `invalid-spec` | Wrong type, empty, too long, or not allowed in this diagram type (edges in a `timeline`, notes in a `compare`) | Check the limits in `spec.md` |
| `duplicate-id` | An id is used twice across nodes, groups and edges | Rename one; give parallel edges their own `id` |
| `unknown-reference` | An edge end, note target or group member is not defined, or is one letter off a declared id | Fix the typo or declare the node |
| `group-cycle`, `group-overlap` | Groups loop, or a node is in two groups that are not nested | Restructure the groups |
| `at-outside-free` | `at` on a node outside a `free` diagram | Remove `at` |
| `too-wide` | Wider than `max_width` even at the tightest spacing | `direction: down`, shorter labels, fewer nodes per rank, or a larger `max_width` (up to 2000) |
| `no-steps` | `--frames` on a spec without `steps` | Add steps or drop the flag |
| `no-match` | `preview --step N` on a drawing without that step, or a `--crop` selector that matches nothing | Steps are numbered from 1 and come from the spec |
| `diagram-too-large` | The drawing would be about 11,000 px on a side | Split it into several diagrams |

Spec errors come one per run. Fix and run again until the render passes.

## Other input (exit 3)

| Code | Cause | Next move |
|---|---|---|
| `file-not-found`, `not-utf8`, `input-too-large` | The input cannot be read, or is over 64 MB | Check the path |
| `unknown-theme`, `theme-not-found`, `invalid-theme`, `theme-no-dark` | A theme name or folder is wrong | `svggod theme list`; see `themes.md` |
| `invalid-config`, `unknown-rule`, `unknown-profile` | `svggod.toml` has a mistake | The message names the key; fix the config, or `--no-config` for one run |
| `not-svggod-output` | `--previous` was given an SVG that svggod did not draw | Drop `--previous` |
| `no-spec` | `describe` was given an SVG without an embedded spec | Give it the spec file |
| `memory-limit` | The run needed more than `--max-memory` (1 GB by default) | The input is too large or hostile. Do not raise the limit without asking |

## Output (exit 4)

| Code | Cause | Next move |
|---|---|---|
| `output-exists` | The output file is there | `--previous OUT` to keep the layout, `--force` to replace, or a new path |
| `layout-changed` | With `--previous` or `docs`: boxes cannot stay where they were | Make the change smaller, or `--force` for a fresh layout |
| `write-failed` | The file could not be written | Check the folder exists and is writable |

## Command line (exit 2)

`unknown-flag`, `unknown-command`, `missing-argument`, `argument-conflict`, `invalid-argument`.
The `missing-argument` message does not always say which argument: for `render` and `preview` it
is usually `-o OUT`; for `edit` and `fix` it is one of `--dry-run`, `-o OUT` or `--in-place`.
`svggod COMMAND --help` shows the usage.

## A bug (exit 5)

`internal-error`. Report the exact command and the input to the user; do not work around it
silently.
