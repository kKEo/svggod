//! Routing all the edges of a drawing, and routing again where they cross.
//!
//! The first pass routes the edges one by one in the order of the spec. Each later
//! route sees the earlier ones, so the order decides which edge gets the clear path.
//! Where two edges cross, the pair is taken up and laid again in other ways; a change
//! is kept only when the drawing as a whole scores better.

use kurbo::{Point, Rect};

use super::cross::{self, Score};
use super::effort::{Budget, Rng};
use super::route::{Dir, Grid, Scratch, Seek};
use crate::spec::Ref;

/// How far a route runs straight out of a box before it may turn.
pub(crate) const STUB: f64 = 16.0;

/// One edge between two boxes, as the router sees it.
#[derive(Clone)]
pub(crate) struct Wire {
    pub edge: usize,
    /// The box and the side at each end: where the edge starts, then where it ends.
    pub ends: [(Ref, Dir); 2],
    /// The point on each of those sides.
    pub ports: [Point; 2],
    /// Frames that hold neither end: walls for this edge.
    pub walls: Vec<Rect>,
}

impl Wire {
    fn from(&self) -> Point {
        self.ports[0] + self.ends[0].1.vec() * STUB
    }
    fn to(&self) -> Point {
        self.ports[1] + self.ends[1].1.vec() * STUB
    }
    fn stubs(&self) -> [[Point; 2]; 2] {
        [[self.ports[0], self.from()], [self.to(), self.ports[1]]]
    }
}

/// Turns the route the search found (`None` when it found none) into the line that is drawn.
pub(crate) type Finish<'a> = &'a dyn Fn(&Wire, Option<&[Point]>) -> Vec<Point>;

struct Job<'a> {
    /// The line of every edge that is not routed: loops. Indexed by edge.
    fixed: &'a [Option<Vec<Point>>],
    finish: Finish<'a>,
}

#[derive(Clone)]
struct Laid {
    grid: Grid,
    wires: Vec<Wire>,
    /// What the search found for each wire.
    mids: Vec<Option<Vec<Point>>>,
}

/// Edges the search found no way for count before everything else.
type Mark = (usize, Score);

impl Job<'_> {
    fn routes(&self, laid: &Laid) -> Vec<Vec<Point>> {
        let mut out: Vec<Vec<Point>> = self.fixed.iter().map(|r| r.clone().unwrap_or_default()).collect();
        for (w, mid) in laid.wires.iter().zip(&laid.mids) {
            out[w.edge] = (self.finish)(w, mid.as_deref());
        }
        out
    }

    fn mark(&self, laid: &Laid) -> Mark {
        (laid.mids.iter().filter(|m| m.is_none()).count(), cross::score(&self.routes(laid)))
    }
}

/// Route one wire on the grid as it stands and record it there.
fn lay(laid: &mut Laid, k: usize, cross: u32, scratch: &mut Scratch) {
    let w = &laid.wires[k];
    // The wire's own first and last steps are not in its way.
    for stub in w.stubs() {
        laid.grid.unmark(&stub);
    }
    // A route that passed between a box and the first turn of its own edge would cross itself.
    let own: Vec<Rect> = (0..2).map(|e| Rect::from_points(w.ports[e], w.ports[e] + w.ends[e].1.vec() * (STUB / 2.0))).collect();
    let walled: Vec<Rect> = own.iter().chain(&w.walls).copied().collect();
    let seek = Seek { from: w.from(), out: w.ends[0].1, to: w.to(), arrive: w.ends[1].1.opposite(), avoid: &walled, strict: false, cross };
    let mid = laid.grid.find(&seek, scratch).or_else(|| laid.grid.find(&Seek { avoid: &own, ..seek }, scratch));
    for stub in w.stubs() {
        laid.grid.mark(&stub);
    }
    if let Some(m) = &mid {
        laid.grid.mark(m);
    }
    laid.mids[k] = mid;
}

fn lift(laid: &mut Laid, k: usize) {
    if let Some(m) = laid.mids[k].take() {
        laid.grid.unmark(&m);
    }
}

/// The order the wires are first routed in: the order of the spec, the shortest first,
/// the spec backwards, and after that shuffles that depend only on the number.
fn order(n: usize, wires: &[Wire]) -> Vec<usize> {
    let mut v: Vec<usize> = (0..wires.len()).collect();
    match n {
        0 => {}
        1 => {
            let span = |k: usize| {
                let d = wires[k].to() - wires[k].from();
                d.x.abs() + d.y.abs()
            };
            v.sort_by(|a, b| span(*a).total_cmp(&span(*b)).then(a.cmp(b)));
        }
        2 => v.reverse(),
        _ => Rng::new(n as u64).shuffle(&mut v),
    }
    v
}

/// One end of a wire: the wire, and 0 for where it starts or 1 for where it ends.
type End = (usize, usize);

/// A way to lay crossing wires again: exchange the points that pairs of wire ends have on
/// a side they share, then take the wires up and route them in the order given.
struct Move {
    swaps: Vec<(End, End)>,
    again: Vec<usize>,
}

impl Move {
    fn again(list: &[usize]) -> Move {
        Move { swaps: Vec::new(), again: list.to_vec() }
    }
}

fn apply(laid: &mut Laid, mv: &Move, cross: u32, scratch: &mut Scratch) {
    for &k in &mv.again {
        lift(laid, k);
        for stub in laid.wires[k].stubs() {
            laid.grid.unmark(&stub);
        }
    }
    for ((a, ea), (b, eb)) in &mv.swaps {
        let (pa, pb) = (laid.wires[*a].ports[*ea], laid.wires[*b].ports[*eb]);
        laid.wires[*a].ports[*ea] = pb;
        laid.wires[*b].ports[*eb] = pa;
    }
    for &k in &mv.again {
        for stub in laid.wires[k].stubs() {
            laid.grid.mark(&stub);
        }
    }
    for &k in &mv.again {
        lay(laid, k, cross, scratch);
    }
}

/// The ends of other wires that share a side with end `e` of wire `k`, at another point.
fn side_mates(laid: &Laid, k: usize, e: usize) -> Vec<End> {
    let mut out = Vec::new();
    for (j, w) in laid.wires.iter().enumerate() {
        for f in 0..2 {
            if j != k && w.ends[f] == laid.wires[k].ends[e] && w.ports[f] != laid.wires[k].ports[e] {
                out.push((j, f));
            }
        }
    }
    out
}

/// Every way to lay the crossing wires `a` and `b` again, the cheapest first.
fn moves(laid: &Laid, a: Option<usize>, b: Option<usize>) -> Vec<Move> {
    let mut out = Vec::new();
    match (a, b) {
        (Some(a), Some(b)) => {
            out.extend([Move::again(&[a]), Move::again(&[b]), Move::again(&[a, b]), Move::again(&[b, a])]);
            // The two change places on a side they share.
            for i in 0..2 {
                for j in 0..2 {
                    if laid.wires[a].ends[i] == laid.wires[b].ends[j] && laid.wires[a].ports[i] != laid.wires[b].ports[j] {
                        out.push(Move { swaps: vec![((a, i), (b, j))], again: vec![a, b] });
                    }
                }
            }
        }
        (Some(k), None) | (None, Some(k)) => out.push(Move::again(&[k])),
        (None, None) => {}
    }
    // One of the two changes places with its neighbours, at one end or at both: that sends
    // it round the other side of the boxes it joins.
    for k in [a, b].into_iter().flatten() {
        let mates = [side_mates(laid, k, 0), side_mates(laid, k, 1)];
        let options = |e: usize| std::iter::once(None).chain(mates[e].iter().copied().map(Some));
        for first in options(0) {
            for second in options(1) {
                let swaps: Vec<(End, End)> = [first.map(|m| ((k, 0), m)), second.map(|m| ((k, 1), m))].into_iter().flatten().collect();
                // A mate that is the same wire at both ends would be moved twice.
                if swaps.is_empty() || (swaps.len() == 2 && swaps[0].1.0 == swaps[1].1.0) {
                    continue;
                }
                let mut again = vec![k];
                again.extend(swaps.iter().map(|s| s.1.0));
                out.push(Move { swaps, again });
            }
        }
    }
    out
}

/// Take up crossing edges and lay them again, while that makes the drawing better
/// and searches are left.
fn improve(job: &Job, mut laid: Laid, cross: u32, floor: usize, left: &mut usize, scratch: &mut Scratch) -> Laid {
    let mut best = job.mark(&laid);
    let wire_of = |laid: &Laid, edge: usize| laid.wires.iter().position(|w| w.edge == edge);
    'rounds: while best.1.crossings > floor {
        let mut pairs: Vec<(usize, usize)> = cross::tally(&job.routes(&laid)).crossings.iter().map(|c| (c.a, c.b)).collect();
        pairs.dedup();
        for (ea, eb) in pairs {
            for mv in moves(&laid, wire_of(&laid, ea), wire_of(&laid, eb)) {
                if *left < mv.again.len() {
                    break 'rounds;
                }
                *left -= mv.again.len();
                let mut trial = laid.clone();
                apply(&mut trial, &mv, cross, scratch);
                let mark = job.mark(&trial);
                if mark < best {
                    (laid, best) = (trial, mark);
                    continue 'rounds;
                }
            }
        }
        break;
    }
    laid
}

/// Route every wire on `grid` and return the line of every edge, the fixed ones included.
/// `floor` is a number of crossings that is known to be unavoidable: there the work stops.
pub(crate) fn solve(grid: &Grid, wires: Vec<Wire>, fixed: &[Option<Vec<Point>>], budget: &Budget, floor: usize, finish: Finish) -> Vec<Vec<Point>> {
    let job = Job { fixed, finish };
    let mut scratch = Scratch::default();
    let mut base = Laid { grid: grid.clone(), mids: vec![None; wires.len()], wires };
    // Every edge leaves and reaches its box by a short straight step. Those are drawn
    // whatever the routes do, so they are on the grid before the first search.
    for w in &base.wires {
        for stub in w.stubs() {
            base.grid.mark(&stub);
        }
    }
    let mut left = budget.reroutes;
    let mut best: Option<(Mark, Laid)> = None;
    'orders: for n in 0..budget.orders.max(1) {
        // The first pass is what every level does; passes in other orders are paid for.
        if n > 0 {
            if left < base.wires.len() {
                break;
            }
            left -= base.wires.len();
        }
        let mut laid = base.clone();
        for k in order(n, &laid.wires) {
            lay(&mut laid, k, budget.cross[0], &mut scratch);
        }
        for &cross in budget.cross {
            laid = improve(&job, laid, cross, floor, &mut left, &mut scratch);
            let mark = job.mark(&laid);
            if best.as_ref().is_none_or(|(b, _)| mark < *b) {
                best = Some((mark, laid.clone()));
            }
            if best.as_ref().is_some_and(|(b, _)| b.1.crossings <= floor) || left == 0 {
                break 'orders;
            }
        }
    }
    match best {
        Some((_, laid)) => job.routes(&laid),
        None => job.routes(&base),
    }
}
