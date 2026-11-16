//! Counting where edges cross: the measure the router and the placement are judged by.
//!
//! Two routes cross at a point where one passes from one side of the other to the
//! other side. Meeting at an end (a shared port) or touching without passing through
//! is a touch, and running along the same line is an overlap; neither is a crossing.

use kurbo::Point;

const EPS: f64 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crossing {
    /// The two routes, by index, `a < b`.
    pub a: usize,
    pub b: usize,
    pub at: Point,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tally {
    pub crossings: Vec<Crossing>,
    /// Points where two routes meet without one passing through the other.
    pub touches: usize,
    /// Total length two routes run on top of each other.
    pub overlap: f64,
}

/// What a set of routes is judged by, most important first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub crossings: usize,
    pub overlap: u32,
    pub bends: u32,
    pub length: u32,
}

fn cross(a: Point, b: Point, c: Point) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// A route without repeated points.
fn clean(route: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(route.len());
    for p in route {
        if out.last().is_none_or(|q| q.distance(*p) > EPS) {
            out.push(*p);
        }
    }
    out
}

/// Where `p` lies on a route: the two points the route runs to on either side of it,
/// or `None` when `p` is an end of the route or not on it.
fn arms(route: &[Point], p: Point) -> Option<(Point, Point)> {
    let n = route.len();
    for k in 0..n {
        if route[k].distance(p) <= EPS {
            return (k > 0 && k + 1 < n).then(|| (route[k - 1], route[k + 1]));
        }
    }
    for w in route.windows(2) {
        let len = w[0].distance(w[1]);
        if cross(w[0], w[1], p).abs() <= EPS * len.max(1.0) && (p - w[0]).dot(w[1] - w[0]) > 0.0 && (p - w[1]).dot(w[0] - w[1]) > 0.0 {
            return Some((w[0], w[1]));
        }
    }
    None
}

/// True when route A, running `a0 -> p -> a1`, passes through route B, running `b0 -> p -> b1`:
/// going round `p`, the arms of the two routes alternate.
fn passes_through(p: Point, a: (Point, Point), b: (Point, Point)) -> bool {
    let angle = |q: Point| (q.y - p.y).atan2(q.x - p.x);
    let turn = |from: f64, to: f64| (to - from).rem_euclid(std::f64::consts::TAU);
    let (a0, a1) = (angle(a.0), angle(a.1));
    let span = turn(a0, a1);
    let mut within = 0;
    for q in [b.0, b.1] {
        let t = turn(a0, angle(q));
        // An arm of B along an arm of A is an overlap, not a crossing.
        if t < 1e-9 || (t - span).abs() < 1e-9 || (std::f64::consts::TAU - t) < 1e-9 {
            return false;
        }
        within += (t < span) as usize;
    }
    within == 1
}

/// Every crossing, touch and overlap between different routes.
pub fn tally(routes: &[Vec<Point>]) -> Tally {
    let routes: Vec<Vec<Point>> = routes.iter().map(|r| clean(r)).collect();
    let boxes: Vec<kurbo::Rect> = routes.iter().map(|r| r.iter().fold(None, |b: Option<kurbo::Rect>, p| Some(b.map_or(kurbo::Rect::from_points(*p, *p), |b| b.union_pt(*p)))).unwrap_or(kurbo::Rect::ZERO)).collect();
    let mut out = Tally::default();
    for a in 0..routes.len() {
        for b in a + 1..routes.len() {
            let (ra, rb) = (&routes[a], &routes[b]);
            if ra.len() < 2 || rb.len() < 2 || boxes[a].inflate(1.0, 1.0).intersect(boxes[b].inflate(1.0, 1.0)).area() <= 0.0 {
                continue;
            }
            // Points the two routes have in common, each once.
            let mut meets: Vec<Point> = Vec::new();
            for u in ra.windows(2) {
                for v in rb.windows(2) {
                    let (d, e) = (u[1] - u[0], v[1] - v[0]);
                    let det = d.cross(e);
                    if det.abs() <= EPS * d.hypot() * e.hypot() {
                        // Parallel: on the same line they may run on top of each other.
                        if cross(u[0], u[1], v[0]).abs() <= EPS * d.hypot().max(1.0) {
                            let along = |p: Point| (p - u[0]).dot(d) / d.hypot();
                            let (lo, hi) = (along(v[0]).min(along(v[1])).max(0.0), along(v[0]).max(along(v[1])).min(d.hypot()));
                            if hi - lo > EPS {
                                out.overlap += hi - lo;
                            }
                        }
                        continue;
                    }
                    let w = v[0] - u[0];
                    let (t, s) = (w.cross(e) / det, w.cross(d) / det);
                    let tol = |len: f64| EPS / len.max(EPS);
                    if t < -tol(d.hypot()) || t > 1.0 + tol(d.hypot()) || s < -tol(e.hypot()) || s > 1.0 + tol(e.hypot()) {
                        continue;
                    }
                    let p = u[0] + d * t.clamp(0.0, 1.0);
                    if !meets.iter().any(|q| q.distance(p) <= 1e-4) {
                        meets.push(p);
                    }
                }
            }
            for p in meets {
                match (arms(ra, p), arms(rb, p)) {
                    (Some(x), Some(y)) if passes_through(p, x, y) => out.crossings.push(Crossing { a, b, at: p }),
                    _ => out.touches += 1,
                }
            }
        }
    }
    out
}

/// A number of crossings that no drawing of this graph can go below, however it is laid out.
///
/// A graph with `n` points that can be drawn without crossings has at most `3n - 6` lines,
/// and at most `2n - 4` when no three points are all joined to each other. Every line beyond
/// that costs at least one crossing. The bound is often 0 for graphs that still need
/// crossings; it never claims more than is certain.
pub fn least(n: usize, edges: &[(usize, usize)]) -> usize {
    let mut lines: Vec<(usize, usize)> = edges.iter().filter(|(a, b)| a != b && *a < n && *b < n).map(|(a, b)| (*a.min(b), *a.max(b))).collect();
    lines.sort();
    lines.dedup();
    // The parts of the graph that hang together are counted one by one.
    let mut part: Vec<usize> = (0..n).collect();
    fn root(part: &mut [usize], mut v: usize) -> usize {
        while part[v] != v {
            part[v] = part[part[v]];
            v = part[v];
        }
        v
    }
    for (a, b) in &lines {
        let (ra, rb) = (root(&mut part, *a), root(&mut part, *b));
        part[ra] = rb;
    }
    let joined = |a: usize, b: usize| lines.binary_search(&(a.min(b), a.max(b))).is_ok();
    let mut total = 0;
    for r in 0..n {
        if root(&mut part, r) != r {
            continue;
        }
        let points: Vec<usize> = (0..n).filter(|v| root(&mut part, *v) == r).collect();
        let own: Vec<(usize, usize)> = lines.iter().copied().filter(|(a, _)| root(&mut part, *a) == r).collect();
        if points.len() < 3 {
            continue;
        }
        let triangle = own.iter().any(|(a, b)| points.iter().any(|c| c != a && c != b && joined(*a, *c) && joined(*b, *c)));
        let room = if triangle { 3 * points.len() - 6 } else { 2 * points.len() - 4 };
        total += own.len().saturating_sub(room);
    }
    total
}

/// The score of a set of routes: fewer crossings first, then less overlap, fewer bends, less line.
pub fn score(routes: &[Vec<Point>]) -> Score {
    let t = tally(routes);
    let mut s = Score { crossings: t.crossings.len(), overlap: t.overlap.round() as u32, bends: 0, length: 0 };
    let mut length = 0.0;
    for r in routes {
        let r = clean(r);
        s.bends += r.len().saturating_sub(2) as u32;
        length += r.windows(2).map(|w| w[0].distance(w[1])).sum::<f64>();
    }
    s.length = length.round() as u32;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn two_lines_through_each_other_cross_once() {
        let t = tally(&[vec![p(0.0, 50.0), p(100.0, 50.0)], vec![p(50.0, 0.0), p(50.0, 100.0)]]);
        assert_eq!(t.crossings, vec![Crossing { a: 0, b: 1, at: p(50.0, 50.0) }]);
        assert_eq!((t.touches, t.overlap), (0, 0.0));
    }

    #[test]
    fn a_shared_port_and_a_t_touch_are_not_crossings() {
        // Two edges leaving the same point, and one ending on the side of another.
        let t = tally(&[vec![p(0.0, 0.0), p(100.0, 0.0)], vec![p(0.0, 0.0), p(0.0, 100.0)], vec![p(50.0, 0.0), p(50.0, 80.0)]]);
        assert!(t.crossings.is_empty(), "{t:?}");
        assert_eq!(t.touches, 2);
    }

    #[test]
    fn a_crossing_at_a_bend_counts_once_and_a_bounce_does_not() {
        let line = vec![p(0.0, 50.0), p(100.0, 50.0)];
        // Bends exactly on the line and goes on to the other side.
        let through = vec![p(50.0, 0.0), p(50.0, 50.0), p(80.0, 90.0)];
        // Bends on the line and goes back to the side it came from.
        let bounce = vec![p(20.0, 0.0), p(30.0, 50.0), p(40.0, 0.0)];
        assert_eq!(tally(&[line.clone(), through]).crossings.len(), 1);
        let t = tally(&[line, bounce]);
        assert_eq!((t.crossings.len(), t.touches), (0, 1));
    }

    #[test]
    fn two_bends_on_one_point_cross_only_when_the_arms_alternate() {
        let a = vec![p(0.0, 50.0), p(50.0, 50.0), p(50.0, 0.0)];
        let kiss = vec![p(100.0, 50.0), p(50.0, 50.0), p(50.0, 100.0)];
        let through = vec![p(0.0, 0.0), p(50.0, 50.0), p(100.0, 100.0)];
        assert_eq!(tally(&[a.clone(), kiss]).crossings.len(), 0);
        assert_eq!(tally(&[a, through]).crossings.len(), 1);
    }

    #[test]
    fn running_on_the_same_line_is_overlap() {
        let t = tally(&[vec![p(0.0, 0.0), p(100.0, 0.0)], vec![p(40.0, 0.0), p(70.0, 0.0), p(70.0, 50.0)]]);
        assert!(t.crossings.is_empty());
        assert_eq!(t.overlap, 30.0);
    }

    #[test]
    fn some_graphs_cannot_be_drawn_without_a_crossing() {
        // Three points each joined to three others, and five points all joined to each other.
        let k33: Vec<(usize, usize)> = (0..3).flat_map(|a| (3..6).map(move |b| (a, b))).collect();
        let k5: Vec<(usize, usize)> = (0..5).flat_map(|a| (a + 1..5).map(move |b| (a, b))).collect();
        assert_eq!(least(6, &k33), 1);
        assert_eq!(least(5, &k5), 1);
        // A chain, a tree and a square with one diagonal can all be drawn flat.
        assert_eq!(least(4, &[(0, 1), (1, 2), (2, 3)]), 0);
        assert_eq!(least(4, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)]), 0);
        // Two copies side by side need one crossing each; a line drawn twice counts once.
        let two: Vec<(usize, usize)> = k33.iter().copied().chain(k33.iter().map(|(a, b)| (a + 6, b + 6))).chain([(0, 3)]).collect();
        assert_eq!(least(12, &two), 2);
    }

    #[test]
    fn score_orders_by_crossings_before_bends_and_length() {
        let crossed = score(&[vec![p(0.0, 50.0), p(100.0, 50.0)], vec![p(50.0, 0.0), p(50.0, 100.0)]]);
        let detour = score(&[vec![p(0.0, 50.0), p(100.0, 50.0)], vec![p(50.0, 0.0), p(120.0, 0.0), p(120.0, 100.0), p(50.0, 100.0)]]);
        assert_eq!((crossed.crossings, detour.crossings), (1, 0));
        assert!(detour < crossed);
    }
}
