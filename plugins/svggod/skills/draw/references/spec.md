# The svggod spec

A spec is YAML (or JSON, when the text starts with `{`). Parsing is strict: an unknown key is an
error with a did-you-mean hint. The machine-readable form is `svggod schema spec`.

## Contents

- Root keys
- Nodes
- Edges
- Groups and notes
- What each diagram type allows
- Ids in the rendered SVG

## Root keys

| Key | Required | Default | Values and limits |
|---|---|---|---|
| `version` | yes | | `1` |
| `diagram` | yes | | `flow`, `layers`, `sequence`, `free`, `state`, `timeline`, `compare` |
| `title` | yes | | 1 to 120 characters. Not drawn; it is the SVG `<title>` |
| `description` | no | generated | Up to 500 characters; the SVG `<desc>` and the alt text. Write one |
| `direction` | no | `right` | `right`, `down` |
| `theme` | no | config, else `docs-light` | Built-in name or a theme folder relative to the spec |
| `effort` | no | `normal` | `normal`, `high`, `max`: how hard to work at crossing edges |
| `max_width` | no | 960 | 200 to 2000. A drawing that cannot fit fails with `too-wide` |
| `nodes`, `edges`, `groups`, `notes` | no | empty | A spec needs at least one node, declared or implied |
| `steps`, `focus`, `legend`, `caption` | no | | See `explaining.md` |

## Nodes

| Key | Required | Values and limits |
|---|---|---|
| `id` | yes | Lowercase letters, digits, hyphens; starts with a letter; up to 63 characters |
| `label` | yes | 1 to 80 characters. Not used for `kind: start` and `kind: end` |
| `kind` | no | `box` (default), `service`, `client`, `datastore`, `queue`, `actor`, `external`, `decision`, `document`, `state`; `start` and `end` in a state diagram only |
| `emphasis` | no | `true` highlights the node. Use it for one or two nodes, not many |
| `width` | no | 40 to 2000; normally leave it out |
| `at` | `free` only | `[x, y]`, each from -10000 to 10000; required in `free`, refused elsewhere |
| `when` | no | `timeline` only; up to 30 characters, printed on the axis |
| `match` | no | `compare` only; nodes on both sides with the same key share a row. Defaults to the label |
| `detail`, `link` | no | Tooltip text (up to 300) and a link (`http:`, `https:`, `mailto:` or a relative path) |

`nodes` may be left out: an id that only an edge or group names becomes a `box` labelled with the
id (hyphens become spaces). An undeclared id that is one letter away from a declared one is
refused as `unknown-reference`, so typos do not create boxes silently.

## Edges

Short form, a string: `"from ARROW to: label"`. Spaces around the arrow are required. Quote the
string whenever it has a label.

| Arrow | Meaning |
|---|---|
| `->` | Solid, a synchronous call |
| `-->` | Dashed, asynchronous |
| `..>` | Dotted, a dependency |
| `<->`, `<-->` | Solid or dashed with arrows at both ends |
| `--` | A line with no arrowhead |

Long form, a mapping, needed for an explicit `id` (parallel edges, or to name the edge in a note
or a step):

```yaml
edges:
  - { id: reply, from: auth, to: app, label: token, style: async }
```

| Key | Default | Values and limits |
|---|---|---|
| `from`, `to` | | A node or group id (nodes only in a `sequence`) |
| `id` | `<from>--<to>` | A second edge from the same node to the same node needs its own id, or it is `duplicate-id`. Repeated messages in a sequence count |
| `label` | | Up to 40 characters. Keep it to two or three words |
| `style` | `sync` | `sync`, `async`, `dependency` |
| `arrow` | `end` | `end`, `both`, `none` |
| `detail`, `link` | | As for nodes |

## Groups and notes

```yaml
groups:
  - { id: backend, label: Backend, members: [api, db] }
notes:
  - { attach: api, text: Validates stock first, side: right }
```

- A group needs `id` and a non-empty `members` list; `label` is up to 60 characters. Groups may
  contain groups. A node in two groups is allowed only when one group is inside the other
  (`group-overlap` otherwise).
- A note needs `attach` (a node, group or edge id) and `text` (1 to 120 characters). `side` is
  `top`, `right` (default), `bottom` or `left`.
- Ids are unique across nodes, groups and edges together.

## What each diagram type allows

| Type | Layout | Rules |
|---|---|---|
| `flow` | Layered along `direction`; edges routed around boxes | Everything allowed |
| `layers` | Each top-level group is a band; bands stack across the flow | Ungrouped nodes share an unframed first band |
| `sequence` | Nodes are lifelines in spec order; edges are messages top to bottom in spec order | Edge ends must be nodes. A node may message itself |
| `state` | As `flow` | Default kind is `state`; `start` and `end` nodes take no label |
| `timeline` | Nodes in spec order along an axis, each with `when` | Groups are phases and may not nest. No edges, no notes |
| `compare` | Exactly two groups side by side | Every node belongs to one of the two groups; no other groups; no notes |
| `free` | Each node at its `at`, snapped to an 8 px grid | `at` required on every node |

Order matters in `sequence`, `timeline` and `compare`: the drawing follows the spec.

## Ids in the rendered SVG

Rendered elements are `node-<id>`, `group-<id>`, `edge-<id>` and `note-<n>`. These are what
`svggod inspect --select '#node-api'` and `svggod preview --crop '#group-backend'` take.
