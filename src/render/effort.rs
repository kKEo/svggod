//! How hard to work at a drawing. Every budget is a count, never a time, so the same
//! effort gives the same drawing on every machine.

/// The levels of the `effort` setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effort {
    #[default]
    Normal,
    High,
    Max,
}

/// What one level may spend. A higher level first does all that the lower ones do and
/// stops there when the drawing has no crossings; only then does it spend its own budget,
/// and it keeps what it finds only when that scores better.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    /// Searches the router may spend on routing edges again, beyond the first pass.
    pub reroutes: usize,
    /// The price of a crossing, raised step by step while crossings remain.
    pub cross: &'static [u32],
    /// Orders the edges are routed in: the order of the spec, then others.
    pub orders: usize,
    /// Orders of the boxes within their ranks that the placement starts from.
    pub starts: usize,
    /// Sweeps over the ranks per start, and how many may pass without a gain.
    pub sweeps: (usize, usize),
    /// Move single boxes to their best place in the rank, and let groups see their outside neighbours.
    pub sift: bool,
    /// Other placements that are routed in full, when the plain one has crossings, before the best is kept.
    pub routed: usize,
}

impl Effort {
    pub fn parse(s: &str) -> Option<Effort> {
        match s {
            "normal" => Some(Effort::Normal),
            "high" => Some(Effort::High),
            "max" => Some(Effort::Max),
            _ => None,
        }
    }

    /// The budgets to work through for this level, the lower levels first.
    pub fn ladder(self) -> Vec<Budget> {
        let levels: &[Effort] = match self {
            Effort::Normal => &[Effort::Normal],
            Effort::High => &[Effort::Normal, Effort::High],
            Effort::Max => &[Effort::Normal, Effort::High, Effort::Max],
        };
        levels.iter().map(|e| e.budget()).collect()
    }

    pub const fn budget(self) -> Budget {
        match self {
            Effort::Normal => Budget { reroutes: 64, cross: &[240], orders: 1, starts: 1, sweeps: (6, 2), sift: false, routed: 1 },
            Effort::High => Budget { reroutes: 512, cross: &[240, 480], orders: 3, starts: 8, sweeps: (12, 4), sift: true, routed: 3 },
            Effort::Max => Budget { reroutes: 4096, cross: &[240, 480, 960], orders: 9, starts: 32, sweeps: (24, 8), sift: true, routed: 6 },
        }
    }
}

/// The budget of a plain layout, for callers that only want to know where things go.
pub static PLAIN: Budget = Effort::Normal.budget();

/// A small generator of numbers that only depend on the seed: splitmix64.
pub(crate) struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x5356_4744_4F47)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `n`.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            v.swap(i, self.below(i + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_shuffle() {
        let shuffled = |seed| {
            let mut v: Vec<usize> = (0..20).collect();
            Rng::new(seed).shuffle(&mut v);
            v
        };
        assert_eq!(shuffled(7), shuffled(7));
        assert_ne!(shuffled(7), shuffled(8));
        let mut sorted = shuffled(7);
        sorted.sort();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    }
}
