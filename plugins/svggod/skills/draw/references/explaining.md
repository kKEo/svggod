# Explaining a diagram: steps, focus, legend, caption

These spec keys tell the reader where to look. None of them moves a box.

## Steps: a numbered walk-through

```yaml
steps:
  - { text: The storefront sends the order to the API., show: [web, web--api] }
  - { text: "The API checks stock, then stores the order.", show: [api, api--db, db] }
  - A step with nothing to point at still gets a line.
```

- `text` is 1 to 200 characters; `show` lists node, group or edge ids.
- An edge without its own `id` is named `from--to`, such as `web--api`.
- Quote a sentence that contains a colon, or a comma inside `{ }`.
- At most 20 steps. Each shown item gets a numbered badge; the sentences are listed under the
  drawing.
- Steps work the same in every diagram type, sequences included.
- Start from `svggod examples steps --format text`.

**What to put in `show`.** A step highlights exactly what it lists; unlike `focus`, naming an
edge does not bring its two ends along. This pulls in two directions:

- For one combined picture, list mainly the edges, plus a node only where the step is about
  that node. Every listed item gets a badge for every step it is in, so nodes named in many
  steps collect stacks of badges ("1 2 3") that crowd their labels.
- For `--frames`, also list the nodes at both ends of each edge; otherwise the frame shows the
  highlighted arrow between two greyed-out boxes.

If you need both outputs, favour the frames and check the combined picture for badge stacks.

### One picture per step (slides, progressive pages)

```sh
svggod render orders.yaml -o orders.svg --frames --format text
```

Writes `orders.1.svg`, `orders.2.svg` and so on, one per step, all with the same layout and size.
It does not write `orders.svg` itself. To render the frames again, add `--force`; ignore the
`--previous` hint in the `output-exists` message here. Preview the frame files themselves
(`svggod preview orders.2.svg -o ...`) to see what will ship.

### Interactive versions

```sh
svggod render orders.yaml -o orders.html --interactive html --format text   # a page with Previous/Next
svggod render orders.yaml -o orders.svg  --interactive css  --format text   # hover highlights, no script
```

The `css` form only reacts when the page inlines the SVG. Shown through Markdown image syntax
(GitHub included) it looks like the plain picture.

### Checking a step of the interactive version

```sh
svggod preview orders.svg -o "$TMPDIR/step2.png" --step 2 --force --format text
```

This shows the combined picture as the interactive page shows it with step 2 chosen: the rest
faded, badges still visible. It is not the same picture as the frame `orders.2.svg`.

## Focus

```yaml
focus: [api, db]
```

Everything not listed is drawn muted. A group brings its members; an edge is in focus when
listed or when both ends are.

## Legend and caption

```yaml
legend: true          # explains the edge styles and emphasis the drawing uses
caption: Figure 1. The order path.
```

Or your own legend entries, each with `text` (up to 60 characters) and exactly one of `style`,
`kind` or `emphasis: true`:

```yaml
legend:
  - { style: async, text: event }
  - { kind: datastore, text: storage }
  - { emphasis: true, text: changed in this release }
```

`caption` (up to 300 characters) is drawn under everything. `title` and `description` are not
drawn; they are what screen readers announce.

## Tooltips and links

```yaml
nodes:
  - { id: api, label: Order API, detail: Owns the order lifecycle., link: "https://example.com/docs/orders" }
```

Both work only where the SVG is inlined or opened on its own, not through Markdown image syntax.

## The diagram in words

```sh
svggod describe orders.yaml --format text    # a full description
svggod describe orders.yaml --short --format text   # one or two sentences, for alt text
```

`describe` takes a spec, or an SVG rendered with `--embed-spec`.
