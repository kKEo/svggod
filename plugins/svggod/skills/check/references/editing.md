# Looking inside and changing an SVG: `inspect` and `edit`

Use these for SVGs that have no spec. For a diagram with a spec, change the spec and render
again; `edit` on a spec file works but rewrites the whole YAML and drops its comments.

## Selectors

| Form | Matches |
|---|---|
| `#node-api` | The element with that id |
| `rect` | Every element with that tag |
| `rect[fill="#ffffff"]` | A tag with an attribute value |
| `svg > rect:nth-of-type(2)` | Steps joined by ` > `, with `:nth-of-type(n)`; findings use this form for elements without an id |

Copy the `target` of a finding straight into `--select` or `--set`. In SVGs that svggod drew,
ids are `node-<id>`, `group-<id>`, `edge-<id>` and `note-<n>`.

## `inspect`: what is in the file

```sh
svggod inspect fig.svg --depth 2 --format text
svggod inspect fig.svg --select '#node-api' --fields id,tag,bbox,text,fill,stroke --format text
```

```text
#node-api <g> [280 120 112 48]
  #node-api > rect <rect> [280 120 112 48]
  #node-api > text <text> [303.98 135.53 64.05 16.94] "Order API"
```

The box is `[x y width height]` in viewBox units. `--fields` takes `id,tag,bbox,text` (the
default) plus `fill`, `stroke` and `refs`. Start with `--depth 2` on a large file. `inspect`
lists drawn SVG elements only: it does not show what is inside a `<foreignObject>` or a
`<style>` block. Read the file itself for those.

## `edit --set`: one attribute

```sh
svggod edit fig.svg --set '#node-api > rect.fill=#ffeeaa' --dry-run --format text
```

`SELECTOR.attr=value`; repeat `--set` for several changes. Every match is changed, so check the
selector with `inspect` first. Choose exactly one destination: `--dry-run` (shows the diff and
the findings after the change), `-o OUT` or `--in-place`. Run `--dry-run` first: the report
after the change shows whether the edit introduced a new finding, such as `low-contrast` after a
recolour.

## `edit --ops`: several operations at once

```sh
svggod edit fig.svg --ops ops.json --dry-run --format text      # or --ops - to read stdin
```

`ops.json` is an array. Targets refer to the file as it was before the batch.

```json
[
  { "op": "set", "target": "#node-api > rect", "attr": "fill", "value": "#ffeeaa" },
  { "op": "set", "target": "#a > text", "style": "font-size", "value": "12px" },
  { "op": "remove", "target": "svg", "attr": "width" },
  { "op": "remove", "target": "#stale" },
  { "op": "move", "target": "#note-1", "dx": 8, "dy": 0 },
  { "op": "rename-id", "target": "#b", "to": "b-old" },
  { "op": "add-node", "target": "svg", "tag": "title", "text": "Order placement", "at": "start" }
]
```

| Op | Required | Notes |
|---|---|---|
| `set` | `target`, `value`, and one of `attr` or `style` | Use `style` when an inline style, not an attribute, governs the value |
| `remove` | `target` | With `attr`, removes that attribute (a trailing `*` matches a prefix); without, the element. `comment(N)` addresses the N-th comment |
| `move` | `target`, `dx` and/or `dy` | In viewBox units |
| `rename-id` | `target`, `to` | The target must match exactly one element, so it cannot repair a duplicate id; use `fix --unsafe` for that |
| `add-node` | `target` (the parent), `tag` | Adds a child element with `attrs` (an object of strings) and `text`. Without `at` it is appended last; `"at": "start"` puts it before the first drawn child, which is after any `<style>` and `<defs>`. This is how a missing `<title>` or `<desc>` is added |

A sixth operation, `add-edge`, applies to diagram specs only and is refused on an SVG.

**Check the count.** The report ends with `N of M operations changed the file.` If N is less
than M, an operation was dropped without an error. This happens when one batch both adds a child
to a parent and removes another child of the same parent. Keep additions and removals in
separate runs, and validate between them, since removals renumber `:nth-of-type` targets.

`svggod schema ops` prints the full schema. The `fix` object inside a JSON finding is already in
this form, so one finding's repair can be applied alone by passing that object to `--ops`.

## Errors

| Code | Meaning |
|---|---|
| `no-match` | The selector matches nothing. Check it with `inspect` |
| `invalid-op`, `unknown-op` | An operation is malformed; the message names the field |
| `conflicting-ops` | Two operations in one batch clash; split them into two runs |
| `edit-breaks-xml` | The result would not be well-formed; nothing is written |
| `missing-argument` | No destination given: add `--dry-run`, `-o OUT` or `--in-place` |
