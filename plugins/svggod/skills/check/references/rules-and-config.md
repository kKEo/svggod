# Rules, profiles and configuration

## Contents

- The 39 rules
- Profiles
- `svggod.toml`
- Suppressing one finding
- Precedence

## The 39 rules

Default severity under the `docs` profile; fix is `safe` (keeps content and colours, but may
resize a box or the viewBox), `unsafe` (moves or recolours content, needs `--unsafe`) or none
(decide by hand). `svggod rules --explain ID --format text` prints the rule's threshold keys
(such as `min_px` for `font-too-small`) and a failing and a passing example; that is the only
place the threshold names are listed.

Two limits of this table:
- On an SVG that svggod did not draw, geometry rules report at most a warning, even where the
  table says error and even when `svggod.toml` raises the severity.
- A listed fix is not available for every case of a rule. A finding can be repaired
  automatically only when its line carries `[fix]` or `[fix --unsafe]`. For example
  `font-not-portable` inside a `<style>` block has no fix, and `edit` cannot reach CSS text:
  change the stylesheet with your file tools.

**Structure**

| Rule | Severity | Fix | Fails when |
|---|---|---|---|
| `xml-well-formed` | error | | The file does not parse as XML |
| `svg-root` | error | safe | The root is not `<svg>` in the SVG namespace |
| `unknown-element` | error | | An element is not part of static SVG |
| `unknown-attribute` | warning | | An attribute is not valid on its element |
| `duplicate-id` | error | unsafe | Two elements share an id |
| `dangling-reference` | error | | A `url(#...)`, `href` or marker names no element |
| `viewbox-missing` | error | safe | The root has no `viewBox` |
| `invalid-path-data` | error | | A path's `d` does not parse |
| `invalid-value` | error | | A length, colour or transform does not parse |
| `unused-def` | info | safe | An item in `<defs>` is never referenced |

**Geometry**

| Rule | Severity | Fix | Fails when |
|---|---|---|---|
| `text-overflow` | error | safe | Text is larger than its container less padding |
| `text-overlap` | error | | Two text runs intersect |
| `shape-overlap` | warning | | Two sibling shapes overlap without one containing the other |
| `content-clipped` | error | safe | Content extends outside the viewBox |
| `excess-margin` | info | safe | Empty margin exceeds 15% of either dimension |
| `edge-detached` | error | safe | A connector ends more than 2 px from any shape |
| `edge-through-node` | warning | unsafe | A connector crosses a shape that is not its endpoint |
| `edge-crossing` | info | | Two connectors cross |
| `label-on-line` | warning | unsafe | Text intersects a connector stroke |
| `font-too-small` | error | unsafe | Text renders under 9 px at the target width |
| `stroke-too-thin` | warning | safe | A stroke renders under 0.75 px at the target width |
| `tight-spacing` | info | | The gap between sibling shapes is under 8 px |
| `off-grid` | info | safe | A coordinate is off the configured grid |

**Portability**

| Rule | Severity | Fix | Fails when |
|---|---|---|---|
| `script-content` | error | safe | `<script>`, event handlers or `javascript:` URLs |
| `external-resource` | error | | An image, font, stylesheet or `<use>` points outside the file |
| `foreign-object` | error | | `<foreignObject>`, which many PDF and image pipelines drop |
| `animation` | warning | safe | SMIL or CSS animation |
| `font-not-portable` | warning | safe | A font-family has no generic fallback |
| `css-unsupported` | warning | | Custom properties, media queries or blend modes |
| `fixed-size` | warning | safe | Absolute `width` and `height` on the root, so it cannot scale |
| `file-too-large` | warning | | Over 200 kB |
| `embedded-raster` | info | | A base64 raster image is embedded |
| `editor-metadata` | info | safe | Editor namespaces, generator comments or timestamps |

**Accessibility**

| Rule | Severity | Fix | Fails when |
|---|---|---|---|
| `title-missing` | error | | The root has no `<title>` |
| `desc-missing` | warning | | The root has no `<desc>` |
| `low-contrast` | error | unsafe | Text contrast under 4.5:1, or 3:1 for large text |
| `background-missing` | warning | safe | Dark content on a transparent background |
| `palette` | warning | unsafe | A colour is not in the configured palette |
| `stroke-width-style` | info | safe | A stroke width is not in the configured set |

`palette`, `stroke-width-style` and `off-grid` report nothing until `svggod.toml` sets the
palette, widths or grid. Size rules (`font-too-small`, `stroke-too-thin`) measure at
`[target] render_width`, 720 px by default.

## Profiles

| Profile | What it checks | Use for |
|---|---|---|
| `minimal` | Structure rules plus `script-content` and `external-resource` | Pictures from other tools that cannot be changed |
| `docs` (default) | Every rule at its normal severity | Documentation |
| `strict` | Everything raised one step: info to warning, warning to error | Releases, shared libraries |

Per run: `svggod validate x.svg --profile strict`. For the project: `profile = "strict"` in
`svggod.toml`.

## `svggod.toml`

For each input, svggod uses the nearest `svggod.toml` in the file's folder or above it. Unknown
keys are refused. `--config PATH` names one file for every input; `--no-config` ignores all.
`svggod schema config` prints the schema.

```toml
version = 1
profile = "docs"                       # minimal, docs or strict

[target]
render_width = 720                     # px the diagram is shown at
backgrounds = ["light", "dark"]        # pages it must be legible on; the first is used for contrast

[style]
theme = "docs-auto"                    # for specs that name no theme
palette = ["#2563eb", "#1f2937"]       # allowed fills and strokes (rule: palette)
stroke_widths = [1, 1.5]               # rule: stroke-width-style
grid = 8                               # rule: off-grid

[render]
effort = "high"                        # for specs that name no effort

[rules]
excess-margin = "off"                  # off, info, warning or error
text-overflow = { severity = "warning", padding = 6 }   # severity plus a threshold

[[overrides]]
files = ["legacy/*.svg"]               # globs relative to this file; * does not cross /
rules = { palette = "off" }
```

Change project rules in `svggod.toml` only when the user asks for a project-wide decision.
Turning a rule off there hides it for every file.

## Suppressing one finding

A comment directly before the element silences the named rules for it and everything inside:

```xml
<!-- svggod-disable text-overlap, shape-overlap: badge sits on the corner by design -->
<g id="badge">...</g>
```

The reason after the colon is required. A missing reason, an unknown rule or no element after
the comment is itself reported as `invalid-suppression`, which cannot be turned off.

## Precedence

For each rule on each file, highest first:

1. `--rules` (run only these) and `--skip` (turn these off) on the command line
2. Matching `[[overrides]]`, later blocks winning
3. `[rules]`
4. The profile: `--profile`, else the config's, else `docs`
