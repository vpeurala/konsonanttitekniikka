//! Spaced repetition: how well the player knows each pair, judged by how
//! quickly its monsters are answered. Pairs answered slowly, or not at
//! all, come back more often than pairs she answers in a flash.

use std::collections::HashMap;

use crate::pairs::Pair;

/// Answering this soon after a pair appears shows she knows it.
const FAST_SECONDS: f32 = 3.0;
/// Answering this late, or never, shows she doesn't know it yet.
const SLOW_SECONDS: f32 = 12.0;
/// The best an answer given while the hint was showing can count as.
const HINTED_QUALITY: f32 = 0.5;
/// How much the latest answer moves a pair's difficulty.
const LEARNING_RATE: f32 = 0.4;
/// Difficulty assumed for a pair never answered.
const UNSEEN_DIFFICULTY: f32 = 0.5;
/// Spawn weights of a fully learned and a fully unknown pair.
const MIN_WEIGHT: f32 = 1.0;
const MAX_WEIGHT: f32 = 6.0;

/// How good an answer was, from 0 (no better than not answering) to 1
/// (answered at once, without a hint).
pub fn answer_quality(seconds: f32, with_hint: bool) -> f32 {
    let quality = 1.0 - ((seconds - FAST_SECONDS) / (SLOW_SECONDS - FAST_SECONDS)).clamp(0.0, 1.0);
    if with_hint {
        quality.min(HINTED_QUALITY)
    } else {
        quality
    }
}

/// Each pair's difficulty, from 0 (learned) to 1 (unknown).
#[derive(Default)]
pub struct Memory {
    difficulty: HashMap<&'static str, f32>,
}

impl Memory {
    /// Records that `pair` was answered `seconds` after it appeared.
    pub fn record_answer(&mut self, pair: Pair, seconds: f32, with_hint: bool) {
        self.record(pair, answer_quality(seconds, with_hint));
    }

    /// Records that `pair`'s monster reached the player unanswered.
    pub fn record_miss(&mut self, pair: Pair) {
        self.record(pair, 0.0);
    }

    fn record(&mut self, pair: Pair, quality: f32) {
        let difficulty = self
            .difficulty
            .entry(pair.number)
            .or_insert(UNSEEN_DIFFICULTY);
        *difficulty += ((1.0 - quality) - *difficulty) * LEARNING_RATE;
    }

    pub fn difficulty(&self, pair: &Pair) -> f32 {
        self.difficulty
            .get(pair.number)
            .copied()
            .unwrap_or(UNSEEN_DIFFICULTY)
    }

    /// How likely `pair` is to be picked relative to other pairs.
    pub fn weight(&self, pair: &Pair) -> f32 {
        MIN_WEIGHT + (MAX_WEIGHT - MIN_WEIGHT) * self.difficulty(pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

    #[test]
    fn quick_answers_are_good_and_slow_ones_bad() {
        assert_eq!(answer_quality(1.0, false), 1.0);
        assert_eq!(answer_quality(SLOW_SECONDS + 5.0, false), 0.0);
        let middle = answer_quality((FAST_SECONDS + SLOW_SECONDS) / 2.0, false);
        assert!((middle - 0.5).abs() < 0.001);
    }

    #[test]
    fn answers_read_from_a_hint_count_for_less() {
        assert_eq!(answer_quality(1.0, true), HINTED_QUALITY);
    }

    #[test]
    fn quick_answers_make_a_pair_easier_and_misses_harder() {
        let (a, b) = (PAIRS[0], PAIRS[1]);
        let mut memory = Memory::default();
        memory.record_answer(a, 1.0, false);
        memory.record_miss(b);
        assert!(memory.difficulty(&a) < UNSEEN_DIFFICULTY);
        assert!(memory.difficulty(&b) > UNSEEN_DIFFICULTY);
        assert!(memory.weight(&b) > memory.weight(&a));
    }

    #[test]
    fn difficulty_stays_between_zero_and_one() {
        let (a, b) = (PAIRS[0], PAIRS[1]);
        let mut memory = Memory::default();
        for _ in 0..100 {
            memory.record_answer(a, 0.0, false);
            memory.record_miss(b);
        }
        assert!((0.0..=1.0).contains(&memory.difficulty(&a)));
        assert!((0.0..=1.0).contains(&memory.difficulty(&b)));
        assert!(memory.weight(&a) >= MIN_WEIGHT && memory.weight(&b) <= MAX_WEIGHT);
    }
}
