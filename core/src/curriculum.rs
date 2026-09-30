//! Which pairs the player has met so far. The first level uses only the
//! single-digit pairs; every later level brings in a few new ones, the
//! same ones in every game.

use crate::pairs::{PAIRS, Pair};
use crate::rng::{Rng, Stream};

/// How many pairs each level after the first introduces.
pub const NEW_PAIRS_PER_LEVEL: usize = 5;

pub struct Curriculum {
    level: u32,
    unlocked: Vec<Pair>,
    /// The pairs introduced on the current level.
    new: Vec<Pair>,
}

impl Default for Curriculum {
    fn default() -> Self {
        Curriculum::new()
    }
}

impl Curriculum {
    /// The first level: all single-digit pairs, which are all new.
    pub fn new() -> Self {
        let first: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| p.number.len() == 1)
            .copied()
            .collect();
        Curriculum {
            level: 1,
            unlocked: first.clone(),
            new: first,
        }
    }

    /// Moves to the next level, introducing up to `NEW_PAIRS_PER_LEVEL`
    /// random pairs not yet met, always the same ones for a given level.
    /// Returns how many were introduced.
    pub fn next_level(&mut self) -> usize {
        self.level += 1;
        let mut rng = Rng::new(Stream::Curriculum, u64::from(self.level));
        let mut locked: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| !self.unlocked.contains(p))
            .copied()
            .collect();
        self.new.clear();
        for _ in 0..NEW_PAIRS_PER_LEVEL.min(locked.len()) {
            let pair = rng.take(&mut locked);
            self.unlocked.push(pair);
            self.new.push(pair);
        }
        self.new.len()
    }

    pub fn unlocked(&self) -> &[Pair] {
        &self.unlocked
    }

    /// The pairs introduced on the current level, in number order.
    pub fn new_pairs(&self) -> Vec<Pair> {
        let mut new = self.new.clone();
        new.sort_by_key(|p| (p.number.len(), p.number));
        new
    }

    pub fn is_new(&self, pair: &Pair) -> bool {
        self.new.contains(pair)
    }
}

/// The pairs unlocked by the time the player reaches `level`, as in a
/// game's curriculum.
pub fn unlocked_pairs(level: u32) -> Vec<Pair> {
    let mut curriculum = Curriculum::new();
    for _ in 1..level {
        if curriculum.next_level() == 0 {
            break;
        }
    }
    curriculum.unlocked().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_level_uses_only_single_digit_pairs() {
        let curriculum = Curriculum::new();
        assert_eq!(curriculum.unlocked().len(), 10);
        assert!(curriculum.unlocked().iter().all(|p| p.number.len() == 1));
        assert_eq!(curriculum.new_pairs().len(), 10);
    }

    #[test]
    fn each_level_introduces_pairs_not_met_before() {
        let mut curriculum = Curriculum::new();
        let before = curriculum.unlocked().to_vec();
        assert_eq!(curriculum.next_level(), NEW_PAIRS_PER_LEVEL);
        let new = curriculum.new_pairs();
        assert_eq!(new.len(), NEW_PAIRS_PER_LEVEL);
        assert!(new.iter().all(|p| !before.contains(p)));
        assert!(new.iter().all(|p| curriculum.unlocked().contains(p)));
    }

    #[test]
    fn every_game_introduces_the_same_pairs() {
        let (mut a, mut b) = (Curriculum::new(), Curriculum::new());
        while a.next_level() > 0 {
            b.next_level();
            assert_eq!(a.new_pairs(), b.new_pairs());
        }
    }

    #[test]
    fn eventually_every_pair_is_met_and_nothing_is_new() {
        let mut curriculum = Curriculum::new();
        while curriculum.next_level() > 0 {}
        assert_eq!(curriculum.unlocked().len(), PAIRS.len());
        assert!(curriculum.new_pairs().is_empty());
    }

    #[test]
    fn new_pairs_are_listed_in_number_order() {
        let mut curriculum = Curriculum::new();
        while curriculum.next_level() > 0 {
            let numbers: Vec<_> = curriculum
                .new_pairs()
                .iter()
                .map(|p| (p.number.len(), p.number))
                .collect();
            assert!(numbers.is_sorted());
        }
    }

    /// The promise that every game introduces the same pairs in the same
    /// order is only kept if these never change by accident. Change them
    /// on purpose only: everyone's progress on later levels shifts with
    /// them.
    #[test]
    fn the_first_levels_introduce_these_pairs() {
        let mut curriculum = Curriculum::new();
        let mut levels = Vec::new();
        for _ in 2..=6 {
            curriculum.next_level();
            let numbers: Vec<&str> = curriculum.new_pairs().iter().map(|p| p.number).collect();
            levels.push(numbers);
        }
        assert_eq!(
            levels,
            [
                ["39", "49", "52", "64", "77"],
                ["01", "51", "55", "66", "82"],
                ["11", "41", "48", "71", "87"],
                ["13", "28", "31", "34", "36"],
                ["07", "20", "63", "90", "92"],
            ]
        );
    }
}
