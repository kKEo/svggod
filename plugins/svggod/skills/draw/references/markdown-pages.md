# Diagrams inside Markdown pages: `svggod docs`

The spec lives in the page; svggod draws the picture and keeps the page pointing at it.

## Write the block

````markdown
```svggod request-path
version: 1
diagram: flow
title: Request path
edges:
  - "browser -> api: request"
  - "api -> database: query"
```
````

- The word after `svggod` names the picture. Without it diagrams are numbered in page order, so
  inserting one renames the rest. Always give a name.
- To show only the picture and hide the YAML, put the same spec in a comment. The opening
  `<!-- svggod NAME` and the closing `-->` must each be on their own line.

## Draw

```sh
svggod docs docs/ --format text          # folders or single files: svggod docs README.md docs/
```

**This rewrites the Markdown file.** Under each block it adds or updates one image line:

```markdown
![Request path. Flow diagram: browser to api (request); api to database (query).](diagrams/page-request-path.svg)
```

Do not write that line yourself and do not edit it; change the spec's `title` or `description`
and run the command again. Read the file again after running `docs` before making further edits,
because its contents have changed.

- Pictures go to `diagrams/` next to each page, named `<page>-<name>.svg`. `--assets DIR` picks
  another folder. Keep that folder for svggod alone.
- Running again when nothing changed writes nothing.
- Boxes already in a picture stay in place when the spec changes. If they cannot, the run stops
  with `layout-changed`; pass `--force` to accept a fresh layout.
- A mistake in one spec stops the whole run; the message names the page and the line.
- Deleting or renaming a block leaves the old picture on disk. Remove it yourself.
- `--theme docs-auto` gives one picture that follows the reader's light or dark mode; a good
  default for a README.
- `--interactive html` also writes a page per diagram and links it after the image line.

## Keep pictures from going stale (CI)

```sh
svggod docs docs/ --check --format text
```

Writes nothing; exits 1 and lists the files when a picture is missing, a spec changed, or an
image line is wrong. Use the same `--theme`, `--assets` and `--interactive` flags as when the
pictures were drawn. For a spec in its own file the matching check is
`svggod render orders.yaml -o orders.svg --check`.
