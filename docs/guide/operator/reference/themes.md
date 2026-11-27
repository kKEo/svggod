# Themes

A theme is the look of a rendered diagram as data: nine colours per variant, a font-family list, two text sizes, two line widths and two corner radii. The built-in themes and themes of your own use one format, a `theme.toml` file. This page is the reference for that file and for how a theme is found. The machine-readable form is `svggod schema theme` (see [JSON Schemas](schemas.md)); the task-oriented page is [Making your own theme](../../user/making-a-theme.md) in the user guide.

## Built-in themes

| Name | Font | `size` / `small` | `width` / `thin` | `corner` / `box_corner` | Source |
|---|---|---|---|---|---|
| `docs` | Inter | 14 / 12 | 1.5 / 1 | 8 / 0 | `themes/docs/theme.toml` |
| `blueprint` | JetBrains Mono | 13 / 12 | 1 / 1 | 0 / 0 | `themes/blueprint/theme.toml` |
| `paper` | Inter | 14 / 12 | 1.5 / 1 | 12 / 6 | `themes/paper/theme.toml` |
| `bold` | Inter | 16 / 13 | 2.5 / 1.5 | 10 / 4 | `themes/bold/theme.toml` |
| `cards` | system-ui (measured with Inter) | 15 / 12.5 | 1 / 1 | 10 / 10 | `themes/cards/theme.toml` |
| `cartoon` | Comic Neue, bold | 16 / 13 | 3 / 2 | 12 / 6 | `themes/cartoon/theme.toml` |
| `sticker` | Varela Round | 15 / 12.5 | 2.5 / 1.5 | 16 / 16 | `themes/sticker/theme.toml` |

`cartoon` and `sticker` also set the `[style]` table: a hard shadow behind boxes, round line joins and larger arrowheads.

The seven files are compiled into the binary (`BUILTIN` in `src/render/theme.rs`) and parsed with the same code as a theme folder. Each has a light and a dark variant. `svggod theme list` prints them as data, and the [Theme gallery](../../user/theme-gallery.md) in the user guide shows one diagram in all of them.

`cards` has hairline box outlines by design: `node_stroke` on `background` is 1.29:1 in the light variant and 1.44:1 in the dark one, below the 3:1 that `theme check` asks of a custom theme. A copy made with `theme init --from cards` therefore fails `theme check` on that pair until the outline is darkened.

## Naming a theme

A theme value is read the same way wherever it is given (`resolve` in `src/render/theme.rs`):

| Value | Meaning |
|---|---|
| `NAME` or `NAME-light` | The light variant of a built-in theme |
| `NAME-dark` | Its dark variant |
| `NAME-auto` | The adaptive variant: the light drawing plus one `prefers-color-scheme: dark` rule with the dark colours |
| A value that starts with `.` or contains `/` or `\` | A theme folder (or the path of a theme file); `#light`, `#dark` or `#auto` after the path picks the variant |

Where the value comes from, in order of precedence, and what a relative folder is relative to:

| Source | Relative to | An unknown name is |
|---|---|---|
| `--theme` | The working directory | exit 2 |
| `theme` in the spec | The spec file; for `svggod docs`, the Markdown page; for a spec on stdin or passed as text, the working directory | exit 3, `path: theme` |
| `[style] theme` in `svggod.toml` | The config file | exit 3 |
| none of them | | `docs-light` is used |

Under `svggod mcp`, a theme folder that resolves outside the root is refused with `outside-root`, whichever of the three sources named it.

`[style] fonts` in `svggod.toml`, when set, replaces the theme's `font.family`.

## theme.toml

```toml
version = 1
name = "acme"
description = "House style"

[font]
family = ["Inter", "sans-serif"]
size = 14
small = 12
weight = 400

[line]
width = 1.5
thin = 1
corner = 8
box_corner = 0

[style]
shadow = [0, 0]
round = false
arrow = 1

[light]
background = "#ffffff"
text = "#1f2937"
muted = "#6b7280"
node_fill = "#dbeafe"
node_stroke = "#1f2937"
accent_fill = "#2563eb"
accent_text = "#ffffff"
note_fill = "#ffffff"
edge = "#1f2937"
# shadow = "#1f2937"   optional, used when [style] shadow draws one

[dark]
# the same keys
```

| Key | Type | Required | Default | Limits and meaning |
|---|---|---|---|---|
| `version` | integer | no | | If present, must be `1` |
| `name` | string | yes | | Lower-case letters, digits and hyphens, starting with a letter, at most 40 characters, not ending in `-light`, `-dark` or `-auto` |
| `description` | string | no | empty | Shown by `theme check` |
| `[font] family` | list of strings | no | `["Inter", "sans-serif"]` | Written to the root `font-family`. No quotes or commas inside a name |
| `[font] size` | number | no | 14 | 10 to 24. Labels in boxes; line distance is `size + 6` |
| `[font] small` | number | no | 12 | 8 to 20, not above `size`. Edge labels, notes, frame labels, legend, steps, caption, step badges; line distance is `small + 4` |
| `[font] weight` | integer | no | 400 | A hundred from 100 to 900. Written to the root `font-weight` unless it is 400. From 600 up, Inter is measured in bold |
| `[line] width` | number | no | 1.5 | 0.5 to 4. Shapes, edges, the timeline axis |
| `[line] thin` | number | no | 1 | 0.5 to 4. Frames, notes, leaders, lifelines |
| `[line] corner` | number | no | 8 | 0 to 16. `service` and `client` boxes and frames; `state` uses 1.5 times it; `external` boxes and notes use half |
| `[line] box_corner` | number | no | 0 | 0 to 16. The plain `box` |
| `[style] shadow` | two numbers | no | `[0, 0]` | Each 0 to 6. How far a hard shadow sits to the right of and below each box and note; `[0, 0]` draws none |
| `[style] round` | boolean | no | `false` | Round line joins and line ends (`stroke-linejoin` and `stroke-linecap` on the root) |
| `[style] arrow` | number | no | 1 | 1 to 2. Size of arrowheads, as a multiple of the plain 10 by 8 px one |
| `[light]` | table | yes | | All nine colours |
| `[dark]` | table | no | | All nine colours; without it `-dark`, `-auto`, `#dark` and `#auto` fail with `theme-no-dark` |

The nine colour keys are `background`, `text`, `muted`, `node_fill`, `node_stroke`, `accent_fill`, `accent_text`, `note_fill` and `edge`. Each is an opaque colour written `#rrggbb`. A tenth key, `shadow`, is optional in each table: the colour of the shadow that `[style] shadow` draws, the same as `node_stroke` when left out. Unknown keys are rejected with a suggestion.

## What a theme changes in the output

- **Colours** change `fill` and `stroke` values only.
- **`corner`, `box_corner`, `width`, `thin`** change `rx` and `stroke-width` attributes; an `rx` of 0 is left out. Box sizes and positions do not change, so a picture re-rendered with `--previous` keeps its layout.
- **`[style]`** adds markup without moving anything. A shadow is a copy of the box's outline drawn behind it, moved with `transform="translate(x y)"` and marked `data-shadow="1"`; a box out of focus has none. `--previous` and the `shape-overlap` rule skip marked shapes. The shadow fits inside the canvas margin, so the canvas size does not change. `round` adds two attributes to the root, and `arrow` changes the size of the `<marker>` elements.
- **`size`, `small`, `weight` and the first `family` name** change text measurement, and so box sizes and the layout. Text is measured with the bundled face the first family name asks for (`text::classify` in `src/svg/text.rs`): Inter, JetBrains Mono, Comic Neue (bundled in bold only) or Varela Round. Any other monospace name is measured with JetBrains Mono and anything else with Inter, in bold when `weight` is 600 or more. A theme cannot bring its own font file. With `--previous`, boxes that change size grow or shrink where they stand when there is room, and the render fails with `layout-changed` when there is not.

Rendering stays deterministic: the same spec, theme file and version give the same bytes. A change to a theme file makes the pictures drawn with it stale for `render --check` and `docs --check`.

## Errors

| Code | Exit | Raised when |
|---|---|---|
| `unknown-theme` | 2 (`--theme`) or 3 | The name is not a built-in theme, or the part after `#` is not `light`, `dark` or `auto` |
| `theme-not-found` | 3 | The folder does not exist or holds no `theme.toml` |
| `invalid-theme` | 3 | The file is not valid: unknown or missing key, value out of range, bad colour or name. `path` names the key |
| `theme-no-dark` | 3 | A dark or adaptive variant was asked for and the theme has no `[dark]` |
| `outside-root` | 3 | Under `svggod mcp`, the folder is outside the root |
