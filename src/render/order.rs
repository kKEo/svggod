//! The order of boxes within their ranks, chosen so that edges between ranks cross least.
//!
//! An edge that spans several ranks gets a stand-in on every rank it passes, so the count
//! knows which boxes it runs between. The stand-ins only exist here: nothing is placed for them.
//!
//! The search is the usual one for layered drawings: sweeps that sort each rank by the median
//! or mean position of its neighbours, swaps of neighbours while that helps, moving single
//! boxes to their best place, and fresh starts from shuffled orders. The best order seen is kept.

use super::effort::{Budget, Rng};

/// Something outside the ranks that a box has an edge to: it lies before the first rank
/// or after the last, at `at` across the flow, and keeps its place.
#[derive(Clone, Copy, Debug)]
pub struct Outside {
    pub kid: usize,
    pub after: bool,
    pub at: f64,
}

/// Which of the orders to produce.
#[derive(Clone, Copy)]
pub struct Search<'a> {
    /// 0 is the plain order; 1 improves on it only where that lowers the count;
    /// higher numbers start from shuffled orders and may differ from 1 at the same count.
    pub variant: usize,
    pub budget: &'a Budget,
}

impl Search<'static> {
    /// The plain order, with nothing searched.
    pub fn plain() -> Search<'static> {
        Search { variant: 0, budget: &super::effort::PLAIN }
    }
}

struct Ranked {
    /// Every rank in order, stand-ins included.
    layers: Vec<Vec<usize>>,
    /// Ranks that keep their order: the ones made of what lies outside.
    fixed: Vec<bool>,
    /// Neighbours on the rank before and on the rank after.
    up: Vec<Vec<usize>>,
    down: Vec<Vec<usize>>,
    layer_of: Vec<usize>,
    /// How many of the ids are boxes; the rest are stand-ins.
    real: usize,
    /// Where the first rank of boxes sits in `layers`.
    first: usize,
}

impl Ranked {
    fn new(ranks: &[Vec<usize>], flow: &[(usize, usize)], rank: &[usize], outside: &[Outside]) -> Ranked {
        let real = rank.len();
        let first = usize::from(outside.iter().any(|o| !o.after));
        let total = first + ranks.len() + usize::from(outside.iter().any(|o| o.after));
        let mut r = Ranked { layers: vec![Vec::new(); total], fixed: vec![false; total], up: vec![Vec::new(); real], down: vec![Vec::new(); real], layer_of: rank.iter().map(|k| k + first).collect(), real, first };
        // Every edge to lay, from an earlier rank to a later one, with where across the rank
        // (as a share of it) its stand-ins start out: beside the box the edge comes from.
        let mut pending: Vec<(usize, usize, f64)> = Vec::new();
        for rk in ranks {
            for (k, v) in rk.iter().enumerate() {
                let share = (k as f64 + 0.5) / rk.len() as f64;
                pending.extend(flow.iter().filter(|(a, _)| a == v).map(|(a, b)| (*a, *b, share)));
            }
        }
        // What lies outside: one fixed rank before and one after, ordered by where each thing is.
        for (after, layer) in [(false, 0), (true, total.saturating_sub(1))] {
            let mut list: Vec<&Outside> = outside.iter().filter(|o| o.after == after).collect();
            if list.is_empty() {
                continue;
            }
            list.sort_by(|x, y| x.at.total_cmp(&y.at).then(x.kid.cmp(&y.kid)));
            let (lo, hi) = (list[0].at, list[list.len() - 1].at);
            r.fixed[layer] = true;
            for o in list {
                let id = r.spawn(layer);
                r.layers[layer].push(id);
                let share = if hi > lo { (o.at - lo) / (hi - lo) } else { 0.5 };
                pending.push(if after { (o.kid, id, share) } else { (id, o.kid, share) });
            }
        }
        let mut waiting: Vec<Vec<(f64, usize)>> = vec![Vec::new(); total];
        for (a, b, share) in pending {
            if r.layer_of[a] >= r.layer_of[b] {
                continue;
            }
            let mut prev = a;
            for (layer, slot) in waiting.iter_mut().enumerate().take(r.layer_of[b]).skip(r.layer_of[a] + 1) {
                let id = r.spawn(layer);
                slot.push((share, id));
                r.down[prev].push(id);
                r.up[id].push(prev);
                prev = id;
            }
            r.down[prev].push(b);
            r.up[b].push(prev);
        }
        // Boxes keep their order; each stand-in goes where its edge comes from.
        for (k, rk) in ranks.iter().enumerate() {
            let layer = k + first;
            let mut keyed: Vec<(f64, usize, usize)> = rk.iter().enumerate().map(|(i, v)| ((i as f64 + 0.5) / rk.len() as f64, 0, *v)).collect();
            keyed.extend(waiting[layer].iter().map(|(at, id)| (*at, 1, *id)));
            keyed.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
            r.layers[layer] = keyed.into_iter().map(|(_, _, v)| v).collect();
        }
        r
    }

    fn spawn(&mut self, layer: usize) -> usize {
        self.up.push(Vec::new());
        self.down.push(Vec::new());
        self.layer_of.push(layer);
        self.up.len() - 1
    }

    fn positions(&self) -> Vec<usize> {
        let mut pos = vec![0; self.up.len()];
        for layer in &self.layers {
            for (k, v) in layer.iter().enumerate() {
                pos[*v] = k;
            }
        }
        pos
    }

    /// How many pairs of edges between neighbouring ranks cross.
    fn crossings(&self) -> usize {
        let pos = self.positions();
        let mut total = 0;
        for layer in &self.layers {
            let mut ends: Vec<(usize, usize)> = Vec::new();
            for v in layer {
                for w in &self.down[*v] {
                    ends.push((pos[*v], pos[*w]));
                }
            }
            for (i, a) in ends.iter().enumerate() {
                for b in &ends[i + 1..] {
                    total += usize::from((a.0 < b.0 && a.1 > b.1) || (a.0 > b.0 && a.1 < b.1));
                }
            }
        }
        total
    }

    /// Crossings between the edges of `u` and of `w` when `u` comes first in their rank.
    fn between(&self, pos: &[usize], u: usize, w: usize) -> usize {
        let mut total = 0;
        for side in [&self.up, &self.down] {
            for a in &side[u] {
                for b in &side[w] {
                    total += usize::from(pos[*a] > pos[*b]);
                }
            }
        }
        total
    }

    /// Sort one rank by where its neighbours on the rank before (or after) sit.
    fn sort_layer(&mut self, layer: usize, by_up: bool, median: bool) {
        if self.fixed[layer] {
            return;
        }
        let pos = self.positions();
        let share = |v: usize| (pos[v] as f64 + 0.5) / self.layers[self.layer_of[v]].len() as f64;
        let key = |v: usize| -> f64 {
            let mut ns: Vec<f64> = (if by_up { &self.up[v] } else { &self.down[v] }).iter().map(|u| share(*u)).collect();
            if ns.is_empty() {
                return share(v);
            }
            if median {
                ns.sort_by(f64::total_cmp);
                let m = ns.len() / 2;
                if ns.len() % 2 == 1 { ns[m] } else { (ns[m - 1] + ns[m]) / 2.0 }
            } else {
                ns.iter().sum::<f64>() / ns.len() as f64
            }
        };
        let mut keyed: Vec<(f64, usize, usize)> = self.layers[layer].iter().map(|v| (key(*v), pos[*v], *v)).collect();
        keyed.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        self.layers[layer] = keyed.into_iter().map(|(_, _, v)| v).collect();
    }

    /// Swap neighbours within a rank while that lowers the count.
    fn transpose(&mut self) {
        for _ in 0..24 {
            let mut gained = false;
            for layer in 0..self.layers.len() {
                if self.fixed[layer] {
                    continue;
                }
                for i in 0..self.layers[layer].len().saturating_sub(1) {
                    let pos = self.positions();
                    let (u, w) = (self.layers[layer][i], self.layers[layer][i + 1]);
                    if self.between(&pos, w, u) < self.between(&pos, u, w) {
                        self.layers[layer].swap(i, i + 1);
                        gained = true;
                    }
                }
            }
            if !gained {
                break;
            }
        }
    }

    /// Move each box or stand-in to the place in its rank where the count is lowest.
    fn sift(&mut self) {
        let mut best = self.crossings();
        for _ in 0..4 {
            let before = best;
            for layer in 0..self.layers.len() {
                if self.fixed[layer] || self.layers[layer].len() < 3 {
                    continue;
                }
                for v in self.layers[layer].clone() {
                    let home = self.layers[layer].iter().position(|x| *x == v).unwrap();
                    let mut at = home;
                    for slot in 0..self.layers[layer].len() {
                        let from = self.layers[layer].iter().position(|x| *x == v).unwrap();
                        let moved = self.layers[layer].remove(from);
                        self.layers[layer].insert(slot, moved);
                        let c = self.crossings();
                        if c < best {
                            (best, at) = (c, slot);
                        }
                    }
                    let from = self.layers[layer].iter().position(|x| *x == v).unwrap();
                    let moved = self.layers[layer].remove(from);
                    self.layers[layer].insert(at, moved);
                }
            }
            if best == before || best == 0 {
                break;
            }
        }
    }

    /// The boxes of every rank, in order, without the stand-ins.
    fn ranks(&self, count: usize) -> Vec<Vec<usize>> {
        (0..count).map(|k| self.layers[k + self.first].iter().copied().filter(|v| *v < self.real).collect()).collect()
    }
}

/// How many times the edges between ranks cross for this order of the boxes.
#[cfg(test)]
pub fn crossings(ranks: &[Vec<usize>], flow: &[(usize, usize)], rank: &[usize]) -> usize {
    Ranked::new(ranks, flow, rank, &[]).crossings()
}

/// A better order of the boxes within their ranks, or the order given when none is found.
///
/// `flow` holds the edges, each from a lower rank to a higher one, and `rank` the rank of
/// every box. With `strict` the order only changes when that lowers the count; without it,
/// a search that started elsewhere may return another order of the same count.
pub fn reorder(ranks: &[Vec<usize>], flow: &[(usize, usize)], rank: &[usize], outside: &[Outside], search: &Search, strict: bool, salt: u64) -> Vec<Vec<usize>> {
    if search.variant == 0 || ranks.iter().all(|r| r.len() < 2) {
        return ranks.to_vec();
    }
    let given = Ranked::new(ranks, flow, rank, outside);
    let was = given.crossings();
    let strict = strict || search.variant == 1;
    if strict && was == 0 {
        return ranks.to_vec();
    }
    let budget = search.budget;
    let mut rng = Rng::new((search.variant as u64) << 32 ^ salt);
    let mut best: Option<(usize, Vec<Vec<usize>>)> = None;
    for start in 0..budget.starts.max(1) {
        let mut r = Ranked { layers: given.layers.clone(), fixed: given.fixed.clone(), up: given.up.clone(), down: given.down.clone(), layer_of: given.layer_of.clone(), real: given.real, first: given.first };
        // The first start of the strict search is the order given; every other start is shuffled.
        if start > 0 || !strict {
            for layer in 0..r.layers.len() {
                if !r.fixed[layer] {
                    rng.shuffle(&mut r.layers[layer]);
                }
            }
        }
        let mut local = (r.crossings(), r.layers.clone());
        let mut idle = 0;
        for round in 0..budget.sweeps.0 {
            let median = round % 4 < 2;
            if round % 2 == 0 {
                for layer in 1..r.layers.len() {
                    r.sort_layer(layer, true, median);
                }
            } else {
                for layer in (0..r.layers.len().saturating_sub(1)).rev() {
                    r.sort_layer(layer, false, median);
                }
            }
            r.transpose();
            let c = r.crossings();
            if c < local.0 {
                local = (c, r.layers.clone());
                idle = 0;
            } else {
                idle += 1;
                if idle >= budget.sweeps.1 {
                    break;
                }
            }
            if local.0 == 0 {
                break;
            }
        }
        r.layers = local.1;
        if budget.sift && local.0 > 0 {
            r.sift();
            local.0 = r.crossings();
        }
        if best.as_ref().is_none_or(|(c, _)| local.0 < *c) {
            best = Some((local.0, r.ranks(ranks.len())));
        }
        if best.as_ref().is_some_and(|(c, _)| *c == 0) {
            break;
        }
    }
    match best {
        Some((c, order)) if c < was || (!strict && c == was) => order,
        _ => ranks.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::effort::Effort;

    fn search(variant: usize, budget: &Budget) -> Search<'_> {
        Search { variant, budget }
    }

    #[test]
    fn two_edges_that_cross_are_counted_and_untangled() {
        // 0 and 1 on the first rank, 2 and 3 on the second; 0 goes to 3 and 1 goes to 2.
        let (ranks, flow, rank) = (vec![vec![0, 1], vec![2, 3]], vec![(0, 3), (1, 2)], vec![0, 0, 1, 1]);
        assert_eq!(crossings(&ranks, &flow, &rank), 1);
        let budget = Effort::Normal.budget();
        let better = reorder(&ranks, &flow, &rank, &[], &search(1, &budget), true, 0);
        assert_eq!(crossings(&better, &flow, &rank), 0);
        assert_eq!(reorder(&ranks, &flow, &rank, &[], &search(0, &budget), true, 0), ranks, "the plain order is left alone");
    }

    #[test]
    fn an_order_without_crossings_is_kept_as_it_is() {
        let (ranks, flow, rank) = (vec![vec![0, 1], vec![2, 3]], vec![(0, 2), (1, 3)], vec![0, 0, 1, 1]);
        let budget = Effort::Max.budget();
        assert_eq!(reorder(&ranks, &flow, &rank, &[], &search(1, &budget), true, 0), ranks);
    }

    #[test]
    fn a_long_edge_is_counted_where_it_passes_a_rank() {
        // 0 and 2 on the first rank, 1 on the second, 4 and 3 on the third. The edge from 0 to 3
        // passes the second rank on the side of 0, so it crosses 1 -> 4 while 4 comes before 3.
        let (ranks, flow, rank) = (vec![vec![0, 2], vec![1], vec![4, 3]], vec![(0, 3), (2, 1), (1, 4)], vec![0, 1, 0, 2, 2]);
        assert_eq!(crossings(&ranks, &flow, &rank), 1);
        let budget = Effort::Normal.budget();
        let better = reorder(&ranks, &flow, &rank, &[], &search(1, &budget), true, 0);
        assert_eq!(better, vec![vec![0, 2], vec![1], vec![3, 4]]);
        assert_eq!(crossings(&better, &flow, &rank), 0);
    }

    #[test]
    fn what_lies_outside_pulls_boxes_to_its_side() {
        // Two boxes on one rank; the first has an edge to something far across, the second to something near.
        let (ranks, flow, rank) = (vec![vec![0, 1]], vec![], vec![0, 0]);
        let outside = [Outside { kid: 0, after: false, at: 500.0 }, Outside { kid: 1, after: false, at: 100.0 }];
        let budget = Effort::High.budget();
        assert_eq!(reorder(&ranks, &flow, &rank, &outside, &search(1, &budget), true, 0), vec![vec![1, 0]]);
        assert_eq!(reorder(&ranks, &flow, &rank, &[], &search(1, &budget), true, 0), ranks);
    }

    #[test]
    fn the_search_gives_the_same_order_every_time() {
        let (ranks, flow, rank) = (vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]], vec![(0, 5), (1, 3), (2, 4), (3, 8), (4, 6), (5, 7), (0, 7)], vec![0, 0, 0, 1, 1, 1, 2, 2, 2]);
        let budget = Effort::Max.budget();
        let once = reorder(&ranks, &flow, &rank, &[], &search(3, &budget), false, 9);
        assert_eq!(once, reorder(&ranks, &flow, &rank, &[], &search(3, &budget), false, 9));
        assert!(crossings(&once, &flow, &rank) <= crossings(&ranks, &flow, &rank));
    }
}
