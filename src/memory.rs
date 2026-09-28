//! Spaced repetition: how well the player knows each pair, judged by how
//! quickly its monsters are answered, and when she last saw it. Pairs
//! answered slowly, or not at all, come back more often than pairs she
//! answers in a flash, and every pair comes back once it is due again.

use std::collections::HashMap;

use crate::pairs::{PAIRS, Pair};

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

const HOUR: f64 = 3600.0;
/// How long until a pair is due again: from this for an unknown pair...
const SHORTEST_INTERVAL: f64 = 5.0 * HOUR;
/// ...up to this for a learned one.
const LONGEST_INTERVAL: f64 = 6.0 * 24.0 * HOUR;

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

/// How long after being seen a pair of this difficulty is due again.
pub fn review_interval(difficulty: f32) -> f64 {
    let learned = f64::from(1.0 - difficulty.clamp(0.0, 1.0));
    SHORTEST_INTERVAL * (LONGEST_INTERVAL / SHORTEST_INTERVAL).powf(learned)
}

/// What is known about one pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PairRecord {
    /// From 0 (learned) to 1 (unknown).
    pub difficulty: f32,
    /// When it was last answered or missed, in seconds since 1970.
    pub last_seen: f64,
    /// How many times it has been answered or missed.
    pub times_seen: u32,
}

/// Each pair's record, kept across games and, through saving, across
/// launches of the app.
#[derive(Debug, Default, Clone)]
pub struct Memory {
    records: HashMap<&'static str, PairRecord>,
}

impl Memory {
    /// A memory with the given records, keyed by number; unknown numbers
    /// are ignored.
    pub fn from_records(records: impl IntoIterator<Item = (String, PairRecord)>) -> Self {
        let mut memory = Memory::default();
        for (number, record) in records {
            if let Some(pair) = PAIRS.iter().find(|p| p.number == number) {
                memory.records.insert(pair.number, record);
            }
        }
        memory
    }

    pub fn records(&self) -> impl Iterator<Item = (&'static str, &PairRecord)> {
        self.records.iter().map(|(n, r)| (*n, r))
    }

    /// Records that `pair` was answered `seconds` after it appeared, at
    /// time `now`.
    pub fn record_answer(&mut self, pair: Pair, seconds: f32, with_hint: bool, now: f64) {
        self.update(pair, answer_quality(seconds, with_hint), now);
    }

    /// Records that `pair`'s monster reached the player unanswered.
    pub fn record_miss(&mut self, pair: Pair, now: f64) {
        self.update(pair, 0.0, now);
    }

    fn update(&mut self, pair: Pair, quality: f32, now: f64) {
        let record = self.records.entry(pair.number).or_insert(PairRecord {
            difficulty: UNSEEN_DIFFICULTY,
            last_seen: now,
            times_seen: 0,
        });
        record.difficulty += ((1.0 - quality) - record.difficulty) * LEARNING_RATE;
        record.last_seen = now;
        record.times_seen += 1;
    }

    pub fn record(&self, pair: &Pair) -> Option<&PairRecord> {
        self.records.get(pair.number)
    }

    pub fn difficulty(&self, pair: &Pair) -> f32 {
        self.record(pair)
            .map_or(UNSEEN_DIFFICULTY, |r| r.difficulty)
    }

    /// How likely `pair` is to be picked at time `now`, relative to other
    /// pairs: harder pairs more, and pairs past their review time more
    /// than pairs seen just now.
    pub fn weight(&self, pair: &Pair, now: f64) -> f32 {
        let base = MIN_WEIGHT + (MAX_WEIGHT - MIN_WEIGHT) * self.difficulty(pair);
        let Some(record) = self.record(pair) else {
            return base;
        };
        let due = ((now - record.last_seen) / review_interval(record.difficulty)) as f32;
        // Half weight right after being seen, full when due, up to double
        // when long overdue.
        base * (0.5 + 0.75 * due.clamp(0.0, 2.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: f64 = 1_800_000_000.0;

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
        memory.record_answer(a, 1.0, false, NOW);
        memory.record_miss(b, NOW);
        assert!(memory.difficulty(&a) < UNSEEN_DIFFICULTY);
        assert!(memory.difficulty(&b) > UNSEEN_DIFFICULTY);
        assert!(memory.weight(&b, NOW) > memory.weight(&a, NOW));
    }

    #[test]
    fn difficulty_stays_between_zero_and_one() {
        let (a, b) = (PAIRS[0], PAIRS[1]);
        let mut memory = Memory::default();
        for _ in 0..100 {
            memory.record_answer(a, 0.0, false, NOW);
            memory.record_miss(b, NOW);
        }
        assert!((0.0..=1.0).contains(&memory.difficulty(&a)));
        assert!((0.0..=1.0).contains(&memory.difficulty(&b)));
    }

    #[test]
    fn learned_pairs_wait_longer_before_they_are_due() {
        assert!(review_interval(0.0) > review_interval(0.5));
        assert!(review_interval(0.5) > review_interval(1.0));
        assert!((review_interval(1.0) - SHORTEST_INTERVAL).abs() < 1.0);
        assert!((review_interval(0.0) - LONGEST_INTERVAL).abs() < 1.0);
    }

    #[test]
    fn a_pair_comes_back_more_often_once_it_is_due() {
        let pair = PAIRS[3];
        let mut memory = Memory::default();
        memory.record_answer(pair, 5.0, false, NOW);
        let interval = review_interval(memory.difficulty(&pair));
        let fresh = memory.weight(&pair, NOW);
        let due = memory.weight(&pair, NOW + interval);
        let overdue = memory.weight(&pair, NOW + 10.0 * interval);
        assert!(fresh < due && due < overdue, "{fresh} {due} {overdue}");
    }

    #[test]
    fn records_survive_a_round_trip_and_unknown_numbers_are_dropped() {
        let mut memory = Memory::default();
        memory.record_answer(PAIRS[5], 2.0, false, NOW);
        let saved: Vec<(String, PairRecord)> = memory
            .records()
            .map(|(n, r)| (n.to_string(), *r))
            .chain([("xyz".to_string(), *memory.records().next().unwrap().1)])
            .collect();
        let restored = Memory::from_records(saved);
        assert_eq!(restored.records().count(), 1);
        assert_eq!(restored.record(&PAIRS[5]), memory.record(&PAIRS[5]));
    }
}
