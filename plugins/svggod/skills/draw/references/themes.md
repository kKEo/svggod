# Themes

A theme sets colours, font, text sizes, line widths and corner radii for the whole diagram.
There is no per-node styling: to make something stand out use `emphasis: true` or `focus`.

## Built-in themes

| Theme | Look | Good for |
|---|---|---|
| `docs` | Quiet blue and grey, 14 px text. The default | Documentation pages |
| `blueprint` | Fine blue lines, square corners, monospace | Technical drawings |
| `paper` | Warm cream page, terracotta accent, round corners | Handbooks, blog posts |
| `bold` | 16 px text, thick lines, strong contrast | Slides |
| `cards` | Soft grey cards, hairline borders, system font | Product pages |

`svggod theme list --format text` prints them.

Each has three variants: `paper` or `paper-light`, `paper-dark`, and `paper-auto`. The `-auto`
variant is one file that switches to the dark colours when the reader's system is in dark mode;
use it for a README or any page read in both modes. `svggod preview` always shows the light side
of an `-auto` picture. To look at the dark side, render once with `-dark` into a temporary file
and preview it with `--background dark`.

## Where a theme is named

The first that is set wins:

1. `svggod render x.yaml -o x.svg --theme paper-dark`
2. `theme: blueprint` in the spec
3. `[style] theme = "paper-auto"` in `svggod.toml`, for the whole project

Before choosing, look for a `svggod.toml` above the spec and for `theme:` in neighbouring specs,
and follow what the project already uses. Do not put `theme:` in a spec when the project sets
one in `svggod.toml`.

A theme with larger text makes larger boxes. If a diagram then fails with `too-wide`, shorten a
label, use `direction: down` or raise `max_width`. When re-theming a picture that exists, add
`--previous` to keep the boxes in place.

## A custom theme

A theme of your own is a folder with one `theme.toml`.

```sh
svggod theme init themes/acme --from paper --format text   # copy a built-in to start from
svggod theme check themes/acme --format text               # the file, colour contrast, and all examples drawn with it
svggod render x.yaml -o x.svg --theme ./themes/acme --format text
```

- The file has `[font]` (`family`, `size`, `small`), `[line]` (`width`, `thin`, `corner`,
  `box_corner`), `[light]` and optionally `[dark]`, each with nine colours: `background`, `text`,
  `muted`, `node_fill`, `node_stroke`, `accent_fill`, `accent_text`, `note_fill`, `edge`.
- For the dark or adaptive variant of a folder theme, add `#dark` or `#auto` to the path:
  `--theme './themes/acme#dark'`. Without a `[dark]` table this fails with `theme-no-dark`.
- In a spec or `svggod.toml` the folder is relative to that file.
- Run `theme check` after every change: it exits 1 when a colour pair has too little contrast.
- `svggod schema theme` prints the full schema.
