//! The game's only source of randomness. Everything random is derived
//! from `GAME_SEED`, so every game plays out the same way given the same
//! input: a level always unlocks the same pairs and has its portals in the
//! same places.
//!
//! Separate concerns draw from separate streams, so that, say, the number
//! of sparks drawn so far can never change which pairs a level unlocks.

use std::ops::Range;

pub const GAME_SEED: u64 = 0x4b6f_6e73_6f6e_616e; // "Konsonan"

/// The independent random streams, one per concern.
#[derive(Debug, Clone, Copy)]
pub enum Stream {
    /// Which pairs each level introduces.
    Curriculum = 1,
    /// Where each level's portals are.
    Portals = 2,
    /// Spawns, pair choices and other gameplay decisions.
    Gameplay = 3,
    /// Sparks, lightning and other visuals that don't affect play.
    Effects = 4,
    /// Where each level's obstacles are and what they look like.
    Obstacles = 5,
}

/// A small, fast generator (SplitMix64).
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// The generator for `stream`, optionally split further by `index` (for
    /// example a level number) so each index gets its own fixed sequence.
    pub fn new(stream: Stream, index: u64) -> Self {
        let mut mixer = Rng(GAME_SEED ^ (stream as u64).wrapping_mul(0xa076_1d64_78bd_642f));
        mixer.0 ^= index.wrapping_mul(0xe703_7ed1_a0b4_28db);
        Rng(mixer.next_u64())
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0.0..1.0`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A number in `min..max`.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.unit()
    }

    /// An index in `range`, which must not be empty.
    pub fn index(&mut self, range: Range<usize>) -> usize {
        let len = (range.end - range.start) as u64;
        range.start + (self.next_u64() % len) as usize
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    /// A random element of `items`, which must not be empty.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.index(0..items.len())]
    }

    /// The index of a random element of `items`, each chosen in proportion
    /// to its `weight`. `items` must not be empty.
    pub fn weighted_index<T>(&mut self, items: &[T], weight: impl Fn(&T) -> f32) -> usize {
        let total: f32 = items.iter().map(&weight).sum();
        let mut target = self.unit() * total;
        for (i, item) in items.iter().enumerate() {
            target -= weight(item);
            if target < 0.0 {
                return i;
            }
        }
        items.len() - 1
    }

    /// Removes and returns a random element of `items`, which must not be
    /// empty.
    pub fn take<T>(&mut self, items: &mut Vec<T>) -> T {
        let i = self.index(0..items.len());
        items.swap_remove(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(stream: Stream, index: u64) -> Vec<u64> {
        let mut rng = Rng::new(stream, index);
        (0..8).map(|_| rng.next_u64()).collect()
    }

    #[test]
    fn same_stream_and_index_give_the_same_sequence() {
        assert_eq!(first(Stream::Gameplay, 0), first(Stream::Gameplay, 0));
    }

    #[test]
    fn streams_and_indexes_are_independent() {
        assert_ne!(first(Stream::Gameplay, 0), first(Stream::Effects, 0));
        assert_ne!(first(Stream::Curriculum, 1), first(Stream::Curriculum, 2));
    }

    #[test]
    fn heavier_items_are_picked_more_often() {
        let mut rng = Rng::new(Stream::Gameplay, 0);
        let weights = [1.0, 3.0];
        let mut counts = [0; 2];
        for _ in 0..10_000 {
            counts[rng.weighted_index(&weights, |w| *w)] += 1;
        }
        // Expect about 1:3.
        assert!((2000..3000).contains(&counts[0]), "{counts:?}");
    }

    #[test]
    fn values_stay_in_their_ranges() {
        let mut rng = Rng::new(Stream::Gameplay, 0);
        for _ in 0..10_000 {
            let x = rng.range(-2.0, 3.0);
            assert!((-2.0..3.0).contains(&x));
            assert!((5..9).contains(&rng.index(5..9)));
        }
    }

    /// Pinned so the generator, and with it every game's sequence of
    /// spawns, can't change by accident.
    #[test]
    fn the_gameplay_stream_starts_with_these_numbers() {
        let mut rng = Rng::new(Stream::Gameplay, 0);
        assert_eq!(
            [rng.next_u64(), rng.next_u64(), rng.next_u64()],
            [
                6_755_087_138_771_210_701,
                6_448_486_015_279_407_272,
                4_376_902_893_971_273_747
            ]
        );
    }

    #[test]
    fn streams_and_indices_give_different_sequences() {
        let first = |stream, index| Rng::new(stream, index).next_u64();
        assert_ne!(first(Stream::Gameplay, 0), first(Stream::Effects, 0));
        assert_ne!(first(Stream::Gameplay, 0), first(Stream::Gameplay, 1));
    }
}
