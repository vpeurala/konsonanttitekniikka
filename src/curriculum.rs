//! Which pairs the player has met so far. The first level uses only the
//! single-digit pairs; every later level brings in a few new ones.

use macroquad::rand;

use crate::pairs::{PAIRS, Pair};

/// How many pairs each level after the first introduces.
const NEW_PAIRS_PER_LEVEL: usize = 5;

pub struct Curriculum {
    unlocked: Vec<Pair>,
    /// The pairs introduced on the current level.
    new: Vec<Pair>,
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
            unlocked: first.clone(),
            new: first,
        }
    }

    /// Moves to the next level, introducing up to `NEW_PAIRS_PER_LEVEL`
    /// random pairs not yet met. Returns how many were introduced.
    pub fn next_level(&mut self) -> usize {
        let mut locked: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| !self.unlocked.contains(p))
            .copied()
            .collect();
        self.new.clear();
        for _ in 0..NEW_PAIRS_PER_LEVEL.min(locked.len()) {
            let pair = locked.swap_remove(rand::gen_range(0, locked.len()));
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
}
