# Crossings and effort

Two edges that cross make a reader stop and work out which line goes on. SVGDog therefore treats a crossing as the worst thing a route can do short of running through a box, and it keeps one only when it has found no drawing without it. This chapter says how crossings are counted, what is done to avoid them, what the `effort` setting changes, and what "unavoidable" can and cannot mean. The last part records the methods that were considered and where they come from.

The code is in `src/render/cross.rs` (counting), `src/render/wire.rs` (routing again), `src/render/order.rs` (the order of boxes) and `src/render/effort.rs` (the budgets).

## What counts as a crossing

Two edges cross at a point where one passes from one side of the other to the other side (`cross::tally`). Three things look similar and are not crossings:

- **A shared end.** Two edges that leave the same point of a box meet there and nothing more.
- **A touch.** An edge that ends on another, or bends on it and goes back the way it came, does not pass through it.
- **An overlap.** Two edges on the same line are a separate fault, counted by its length. The router charges far more for it than for a crossing.

A crossing exactly on a bend is counted once. The test looks at the four arms that leave the point, two per edge: the edges cross when the arms alternate round the point.

The rule `edge-crossing` reports every pair of connectors that cross, with the number of crossings and their points. It is a note (`info`), because some drawings cannot be made without a crossing, and it has no automatic fix. It runs on any SVG, not only on SVGDog's own.

## What is done to avoid one

A crossing can come from where the boxes are or from how the edges are routed between them. Both are worked on, the cheap way first.

### In the routing

1. **A crossing costs more than a detour.** Crossing another route costs 240 in the path search, against 45 for a bend and 10 for a step of 8 px (see [Edge routing](routing.md#step-3-find-the-path)). A route goes round when that takes about two more bends and up to some 120 px more line, and crosses when the way round is longer than that.
2. **The router sees the whole drawing.** Every edge leaves and reaches its box by a straight step of 16 px. Those steps are on the grid before the first edge is routed, so no route runs between a box and the first turn of another edge without paying for the crossing.
3. **The last leg may be turned into.** A route may reach the last step from the side and bend there, at the price of the bend. In the 48 px between two ranks this gives an edge three lanes to choose from where it had two.
4. **Crossing edges are routed again.** After the first pass, each pair of edges that cross is taken up and laid again in several ways: one of the two alone, both in either order, the two changing places on a side they share, and one of them changing places with its neighbours on a side, which sends it round the other side of the boxes it joins. A change is kept only when the drawing as a whole scores better.

The score that decides is, in this order: fewer crossings, less overlap, fewer bends, less line (`cross::Score`).

### In the placement

1. **The order within each rank.** A flow puts its boxes in ranks. The order within a rank decides which edges between ranks must cross. The plain order comes from a few sweeps that put each box at the average position of its neighbours. When the edges of that order cross, a search looks for a better one: sweeps that sort by the median and by the mean of the neighbours, swaps of neighbouring boxes while that helps, and at higher effort moving single boxes to their best place and fresh starts from shuffled orders (`order::reorder`). An edge that spans several ranks gets a stand-in on each rank it passes, so the count knows which boxes it runs between.
2. **The bands of a layers diagram.** The members of a band keep the order of the spec unless another order makes fewer edges between bands cross. Among orders that are equally good, the spec's wins.
3. **Groups and what lies outside them.** A group is laid out before what is around it, so on its own it cannot know on which side its outside neighbours are. At `high` and `max` everything is laid out twice, and the second pass tells each group where those neighbours ended up.
4. **Placements are judged by routing them.** The count between ranks is only an estimate of what the router will make of a placement. So the plain placement is always routed first. When its edges cross, other placements are routed as well and the drawing with the best score is kept.

A placement that is kept from an earlier drawing (`--previous`, `docs`) is not changed: there, only the routing is worked on. A `free` diagram has its boxes where the spec puts them, so the same holds.

## The effort setting

`effort` says how much of this to try. It has three levels.

| | `normal` | `high` | `max` |
| --- | --- | --- | --- |
| Searches spent on routing edges again | 64 | 512 | 4096 |
| Price of a crossing, raised while crossings remain | 240 | 240, 480 | 240, 480, 960 |
| Orders the edges are first routed in | spec | spec, shortest first, reversed | those and 6 shuffles |
| Starts of the search for an order within the ranks | 1 | 8 | 32 |
| Sweeps per start, and how many may pass without a gain | 6, 2 | 12, 4 | 24, 8 |
| Moving single boxes; groups see their outside neighbours | no | yes | yes |
| Other placements routed when the plain one has crossings | 1 | 3 | 6 |

Three properties make the setting safe to raise:

- **The levels build on each other.** A higher level first does everything the lower ones do. If that gives a drawing without crossings, it stops there, so a drawing that is clean at `normal` is byte for byte the same at `high` and `max`. Only when crossings remain does the level spend its own budget, and it keeps what it finds only when that scores better. More effort never gives more crossings.
- **Every budget is a count.** Nothing is limited by time, so the same spec, effort and version give the same bytes on a fast machine and on a slow one. Shuffles come from a small generator seeded by the attempt number (`effort::Rng`), not from the clock.
- **The work stops at a proven minimum.** See the next section.

Set it in one of three places. The first that says something wins:

1. `--effort high` on `svgdog render`, or `effort` on the `svgdog_render` tool.
2. `effort: high` at the top of the spec.
3. `[render] effort = "high"` in `svgdog.toml`.

`render --check` and `docs --check` compare bytes, so they must run with the effort the drawing was made with. Putting the effort in the spec or in `svgdog.toml` takes care of that; a flag has to be repeated. `svgdog docs` has no flag and reads the spec and the config.

### What it costs

`normal` adds little to a render, and nothing to a drawing whose first routing has no crossings. `high` and `max` cost time only where crossings remain. The numbers below were measured once with a release build on one laptop; the counts are exact and repeatable, the times are only an indication.

| Drawing | `normal` | `high` | `max` |
| --- | --- | --- | --- |
| Every fixture, gallery spec and example in the repository | 0 crossings | same bytes | same bytes |
| Three boxes each joined to three others (9 edges) | 5 crossings | 1, in about 0.1 s | 1, stops at once |
| A random graph of 10 boxes and 16 edges | 12 | 5, in about 0.4 s | 4, in about 3 s |
| A random graph of 14 boxes and 22 edges | 16 | 7, in about 0.8 s | 4, in about 15 s |

The random graphs are far denser than a diagram written for a reader, and they are not in the repository. They show the shape of the trade: `high` takes most of the gain for a fraction of a second, and `max` is for the drawing that is worth waiting several seconds for.

## What "unavoidable" means

SVGDog can say two different things about a crossing that is left, and they should not be confused.

- **Proven for the graph.** Some graphs cannot be drawn without a crossing, wherever the boxes go. A graph with `n` boxes that can be drawn flat has at most `3n - 6` edges, and at most `2n - 4` when no three boxes are all joined to each other; every edge beyond that costs at least one crossing (`cross::least`). Three boxes each joined to three others have 9 edges where 8 would fit, so one crossing is certain. When a drawing reaches this number the work stops, at any level, because nothing better exists.
- **None better found.** In every other case a crossing that is left means only that this effort found no drawing without it. The bound above is often 0 for graphs that still need crossings, and a drawing also has to respect things the bound knows nothing about: the direction of the flow, the sides edges use, the groups, the width. Raising the effort may remove the crossing, and so may one of the levers below.

SVGDog does not test a graph for planarity, so it cannot tell you that a drawing without crossings exists when it has not found one.

## What you can change from the spec

When a crossing remains after raising the effort:

- **Edge order.** The first routing follows the order of `edges:`; putting the edge that should run straight first often settles a pair.
- **Direction.** `direction: down` changes the sides edges use and which gap is the long one.
- **Groups.** A frame that holds neither end of an edge is a wall for it. Moving a box into or out of a group changes what the edge must go round.
- **Fewer edges.** An edge to a group, in place of one edge to each member, removes lines without removing meaning.

## The methods considered

This section records what was looked at when the crossing work was designed, so that the choices can be revisited. Claims marked **read** come from a source that was opened and read at the time. Claims marked **reported** come from search results or from memory and were not checked against the source; check them before relying on them.

### How much crossings matter

- Purchase, "Which aesthetic has the greatest effect on human understanding?" (Graph Drawing 1997): reducing crossings matters more to readers than reducing bends or adding symmetry. *Reported.*
- Huang, Eades and Hong, "Larger crossing angles make graphs easier to read" (Journal of Visual Languages and Computing, 2014), and their earlier eye-tracking work: crossings at close to a right angle disturb reading least. *Reported.* Every crossing SVGDog draws is at a right angle, which is why the real trade is between a crossing and a longer route with more bends.

### How hard the problem is

- Finding the least number of crossings of a graph is NP-complete (Garey and Johnson, 1983). So is ordering one rank against a fixed neighbour (Eades and Wormald, 1994), where sorting by the median is within a factor of 3 of the best. *Reported.*
- Whether a graph can be drawn with no crossings at all can be decided in linear time (Hopcroft and Tarjan, 1974; Boyer and Myrvold, 2004). *Reported.* That answers a different question from the one a layered drawing poses, where ranks and sides are fixed.
- Level planarity, the same question with the ranks fixed, is harder to implement correctly than its literature suggests: Fink, Pfretzschner, Rutter and Stumpf show three published simple algorithms to be wrong on a constructed instance (Graph Drawing 2024 poster, <https://arxiv.org/abs/2409.01727>). *Read.*

### Ordering the ranks

- The sweep with medians and swaps of neighbours is the method of Graphviz dot (Gansner, Koutsofios, North and Vo, "A technique for drawing directed graphs", IEEE Transactions on Software Engineering, 1993). In dot's source the loop runs at most 24 iterations and gives up after 8 without a real gain, both scaled by the `mclimit` attribute (<https://graphviz.org/docs/attrs/mclimit/>). *Read.*
- Sifting, moving one box through every place in its rank, is from Matuszewski, Schönfeld and Molitor (Graph Drawing 1999). *Reported.* It was the core of strong heuristic entries in the PACE 2024 challenge on one-sided crossing minimisation, for example Arcee (<https://arxiv.org/abs/2411.17596>), whose ablation also found that taking the better of the median and the mean beats either alone. *Read.*
- Restarts as the effort control: ELK Layered's `thoroughness` (default 7) is the number of randomised restarts, with a fixed seed (<https://eclipse.dev/elk/reference/options/org-eclipse-elk-layered-thoroughness.html>); OGDF's `SugiyamaLayout` has `runs` (default 15) and `fails` (default 4) (<https://ogdf.github.io/doc/ogdf/classogdf_1_1_sugiyama_layout.html>). *Read.* Both count attempts and neither limits time, which is the model SVGDog follows.
- Global sifting, which moves a long edge across all its ranks at once (Bachmaier, Brandenburg, Brunner and Hübner, Journal of Graph Algorithms and Applications, 2011), is reported to give 5 to 10 % fewer crossings than rank-by-rank methods. *Reported.* Not built: the stand-ins of a long edge are moved rank by rank.

### Routing

- Routing edges one after another depends on the order. The classic answers are to route short edges first (Graphviz's orthogonal router sorts by length, read in `lib/ortho/ortho.c`), to rip up and reroute, and negotiated congestion, where every net is rerouted each round against a rising price for shared resources (McMurchie and Ebeling, PathFinder, FPGA 1995). *Read* for Graphviz and PathFinder's outline. SVGDog rips up and reroutes pairs and tries other orders. Full negotiation was not built: a diagram has tens of edges, not thousands of nets.
- libavoid, the connector router of Adaptagrams, searches an orthogonal visibility graph with a cost of length and bends, has a `crossingPenalty` its documentation calls experimental, and separates parallel segments afterwards by "nudging" (Wybrow, Marriott and Stuckey, "Orthogonal Connector Routing", Graph Drawing 2009; <https://www.adaptagrams.org/documentation/namespaceAvoid.html>). *Read* for the options. SVGDog's grid search with a crossing cost is the same idea on a simpler graph.
- ELK and Graphviz order the segments inside the channel between two ranks so that they do not cross; ELK weights each choice by the crossings it would cause. *Read* in their sources. SVGDog has no channels; the port trades in the reroute step do part of that job.

### Considered and left out

- **Exact ordering.** One rank against a fixed neighbour can be solved exactly for some 20 boxes by dynamic programming over subsets, and the PACE 2024 exact track was won by branch and cut on a linear-ordering formulation (<https://arxiv.org/abs/2412.00292>). *Read* for the PACE result; the size limit is *reported*. Left out for now: it proves the best order for one pair of ranks, not for the drawing.
- **Planarization.** Drawing a largest planar part and inserting the remaining edges one by one is described as the strongest known heuristic for the crossing number (Chimani and Gutwenger, 2012). *Reported.* It gives up ranks and flow direction, which is what makes SVGDog's diagrams readable, and it is a large implementation.
- **Solvers.** Integer programming, SAT and semidefinite programming can prove optimal layered drawings of small graphs. They need a solver, and SVGDog stays a single binary with no heavy dependencies.
- **Learning.** A reinforcement-learning approach to the crossing number reported by its own authors as performing poorly against classical methods on total crossings (Brand and others, Graph Drawing 2025 poster, <https://arxiv.org/abs/2509.06108>). *Read.* Nothing found suggests a learned method beats sweeps and sifting at this size.
- **Line jumps.** Tools such as draw.io and JointJS draw a small arc or gap where one line passes over another. This makes a crossing that is left easier to read and may be added later; it removes none.
- **Time limits.** yFiles offers a stop duration. A limit on time makes the output depend on the machine, which SVGDog's promise of identical bytes rules out.
