# Edge routing

After boxes are placed, SVGGod draws each connector as a polyline of horizontal and vertical segments. This chapter explains how it chooses which side of a box an edge uses, where on that side it attaches, and how the path between two attachments is found. Knowing this helps you read why an edge bends where it does and what to change in the spec when a route looks wrong. The code is in `src/render/mod.rs` (`connect`, `sides`), `src/render/route.rs` (the search) and `src/render/wire.rs` (routing all edges, and routing again where they cross).

## Step 1: choose the sides

For each edge (other than a self-loop) `sides` (`src/render/mod.rs:167`) picks a side of the source box and a side of the target box:

- If the two boxes are separated along only one axis, that axis is used.
- If they are separated along both, the diagram type decides: a **flow** keeps to its direction, **layers** connect across the flow (between bands), and **free** and **sequence** take the wider gap.
- If they overlap on both axes, the flow direction is used.

## Step 2: share each side among its edges

Several edges may leave or enter the same side of a box. Each side's length is shared among them (ports are spaced 16 px apart when they fit, 8 px if they must, and all at the middle if not). A side that carries a single edge is the exception: the arrow lands on the middle of the shape, on the line of the label for shapes that draw it off the middle (a data store's lid takes the top, so its arrows meet the body). It moves off the middle only when the box at the other end is within 16 px of straight ahead, so that nearly aligned boxes still get a straight edge instead of a small jog. On a side with several edges, each port aims straight at the other end and ports that want the same spot are spread along the side and ordered to avoid crossings. This is done in two passes: the second aims at the ports the first one chose. Ports are snapped to the grid. Two edges may later change places on a side when that removes a crossing (see step 4).

A connector that starts and ends on the same box is a self-loop. It is drawn round the top right corner, and the box keeps that corner free for it.

## Step 3: find the path

Between the two attachment points (each pushed out 16 px as a stub), `Grid::find` runs an A* search over a uniform 8 px grid (`src/render/route.rs`). The search moves only straight ahead or turns 90 degrees, and it minimises a cost made of:

| Cost | Value | Effect |
| --- | --- | --- |
| Step | 10 | Shorter paths win |
| Bend | 45 | Fewer turns |
| Cross another route | 240 | A route goes round when that takes about two more bends; it crosses only when the way round is longer. Higher [effort](crossings.md#the-effort-setting) raises this to 480 and 960 while crossings remain |
| Bend on top of another route | 200 | A turn on another line hides which line goes where |
| Share a track with another route | 400 | Edges avoid running on top of each other |
| Run along a group frame | 60 | Edges cross a frame at a right angle instead of hugging it |

Boxes (with an 8 px margin) and notes are obstacles the path cannot enter. Frames that hold neither end of the edge are treated as walls for that edge; if that leaves no path, the search is repeated without the walls. Each finished route is marked as taken so later routes keep clear. The stubs of all edges are marked before the first search, so a route that would cut between a box and the first turn of another edge pays for the crossing, and a route may not pass over its own stub at all. Routes are first found in spec order, so the order of `edges:` in the spec can change which of two competing edges gets the straighter path.

The search may reach the end of the route from the side and turn into the last stub there; that turn costs a bend and a little more, so a route that arrives straight ahead wins when both cost the same.

Two clean-ups follow: collinear points are removed, and a route with a single jog (four points) has the jog moved to the half-way point when that is free.

## Step 4: route again where edges cross

When the first pass leaves edges that cross, each crossing pair is taken up and laid again: one of the two alone, both in either order, the two changing places on a side they share, and one of them changing places with its neighbours on a side. A change is kept only when the whole drawing scores better, by crossings first, then overlap, bends and length (`improve`, `src/render/wire.rs`). How many such attempts are made is set by the effort; see [Crossings and effort](crossings.md).

## Step 5: land on the outline

For non-rectangular shapes (a decision diamond, a datastore cylinder, a queue) the endpoint is slid inward until it meets the shape's outline, so arrows touch the drawn edge rather than the bounding box (`land`, `src/render/mod.rs:133`).

## Edge labels

A label is placed near the middle of its route, in the gap it sits in, with room reserved by the tuning level (see [auto-fit](layout.md#auto-fit)). Labels that would sit on a connector are the subject of the `label-on-line` rule.

## What you can change from the spec

You cannot draw a route by hand; you can only change the inputs the router sees. The levers, from the description above, are:

- **Edge order.** Routes are first found in the order of `edges:`, and an earlier route is avoided by later ones, so reordering changes which edge gets the straighter path.
- **Effort.** `effort: high` (or `--effort high`) tries more routes and more placements where edges cross; see [Crossings and effort](crossings.md).
- **Direction.** `direction: down` changes which sides are used for a flow and which gap is the long one.
- **Box width.** A wider `width` on a box gives its sides more room for ports (see step 2).
- **Group membership.** Frames that hold neither end of an edge are walls for it, so moving a box into or out of a group changes what the edge must go round.

Try one change at a time and re-render; the result is deterministic, so a diff shows exactly what moved.
