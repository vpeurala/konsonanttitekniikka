//! Spaced repetition: how well the player knows each pair, judged by how
//! quickly its monsters are answered, and when she last saw it. Pairs
//! answered slowly, or not at all, come back more often than pairs she
//! answers in a flash, and every pair comes back once it is due again.

use std::collections::HashMap;

use crate::pairs::{self, Pair, PairId};

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

/// An answer at least this good counts towards a pair's streak.
const GOOD_QUALITY: f32 = 0.75;

const HOUR: f64 = 3600.0;
const DAY: f64 = 24.0 * HOUR;
/// How long until a pair is due again: from this for an unknown pair...
const SHORTEST_INTERVAL: f64 = 5.0 * HOUR;
/// ...up to this for one that is learned...
const LEARNED_INTERVAL: f64 = 6.0 * DAY;
/// ...and up to this for one answered well again and again.
const LONGEST_INTERVAL: f64 = 21.0 * DAY;
/// A learned pair's interval starts to grow after this many good answers
/// in a row, and then by this factor with each one more.
const STREAK_BEFORE_GROWTH: u32 = 5;
const STREAK_GROWTH: f64 = 1.5;

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

/// How long after being seen a pair of this difficulty is due again, if it
/// has been answered well `streak` times in a row. The difficulty sets the
/// interval up to `LEARNED_INTERVAL`; a long streak stretches it further,
/// up to `LONGEST_INTERVAL`.
pub fn review_interval(difficulty: f32, streak: u32) -> f64 {
    let learned = f64::from(1.0 - difficulty.clamp(0.0, 1.0));
    let interval = SHORTEST_INTERVAL * (LEARNED_INTERVAL / SHORTEST_INTERVAL).powf(learned);
    let growth = STREAK_GROWTH.powf(f64::from(streak.saturating_sub(STREAK_BEFORE_GROWTH)));
    (interval * growth).min(LONGEST_INTERVAL)
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
    /// How many times in a row it has been answered well, quickly and
    /// without a hint. Any miss, slow answer or hint starts it over.
    pub streak: u32,
}

/// Each pair's record, kept across games and, through saving, across
/// launches of the app.
#[derive(Debug, Default, Clone)]
pub struct Memory {
    records: HashMap<PairId, PairRecord>,
}

impl Memory {
    /// A memory with the given records, keyed by number; unknown numbers
    /// are ignored.
    pub fn from_records(records: impl IntoIterator<Item = (String, PairRecord)>) -> Self {
        let mut memory = Memory::default();
        for (number, record) in records {
            if let Some(pair) = pairs::find(&number) {
                memory.records.insert(pair.id, record);
            }
        }
        memory
    }

    /// Every record, by the number of its pair.
    pub fn records(&self) -> impl Iterator<Item = (&'static str, &PairRecord)> {
        self.records
            .iter()
            .map(|(id, r)| (pairs::get(*id).number, r))
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
        let record = self.records.entry(pair.id).or_insert(PairRecord {
            difficulty: UNSEEN_DIFFICULTY,
            last_seen: now,
            times_seen: 0,
            streak: 0,
        });
        record.difficulty += ((1.0 - quality) - record.difficulty) * LEARNING_RATE;
        record.last_seen = now;
        record.times_seen = record.times_seen.saturating_add(1);
        record.streak = if quality >= GOOD_QUALITY {
            record.streak.saturating_add(1)
        } else {
            0
        };
    }

    pub fn record(&self, pair: &Pair) -> Option<&PairRecord> {
        self.records.get(&pair.id)
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
        let due =
            ((now - record.last_seen) / review_interval(record.difficulty, record.streak)) as f32;
        // Half weight right after being seen, full when due, up to double
        // when long overdue.
        base * (0.5 + 0.75 * due.clamp(0.0, 2.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

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
        assert!(review_interval(0.0, 0) > review_interval(0.5, 0));
        assert!(review_interval(0.5, 0) > review_interval(1.0, 0));
        assert!((review_interval(1.0, 0) - SHORTEST_INTERVAL).abs() < 1.0);
        assert!((review_interval(0.0, 0) - LEARNED_INTERVAL).abs() < 1.0);
    }

    #[test]
    fn a_long_streak_stretches_the_interval_up_to_three_weeks() {
        let plain = review_interval(0.0, 0);
        assert_eq!(review_interval(0.0, STREAK_BEFORE_GROWTH), plain);
        let mut previous = plain;
        for streak in STREAK_BEFORE_GROWTH + 1..STREAK_BEFORE_GROWTH + 4 {
            let interval = review_interval(0.0, streak);
            assert!(interval > previous || interval == LONGEST_INTERVAL);
            previous = interval;
        }
        assert_eq!(review_interval(0.0, 1000), LONGEST_INTERVAL);
        assert_eq!(review_interval(0.0, u32::MAX), LONGEST_INTERVAL);
    }

    #[test]
    fn a_streak_never_stretches_a_pair_that_is_not_learned() {
        // Only a small stretch is possible for a hard pair: it is still due
        // far sooner than an easy one with the same streak.
        assert!(review_interval(1.0, 8) < review_interval(0.0, 8));
        assert!(review_interval(1.0, 0) == SHORTEST_INTERVAL);
    }

    #[test]
    fn good_answers_in_a_row_build_a_streak_and_anything_else_ends_it() {
        let pair = PAIRS[4];
        let mut memory = Memory::default();
        for expected in 1..=4 {
            memory.record_answer(pair, 1.0, false, NOW);
            assert_eq!(memory.record(&pair).unwrap().streak, expected);
        }
        // A hint, a slow answer and a miss each start over.
        memory.record_answer(pair, 1.0, true, NOW);
        assert_eq!(memory.record(&pair).unwrap().streak, 0);
        memory.record_answer(pair, 1.0, false, NOW);
        memory.record_answer(pair, SLOW_SECONDS, false, NOW);
        assert_eq!(memory.record(&pair).unwrap().streak, 0);
        memory.record_answer(pair, 1.0, false, NOW);
        memory.record_miss(pair, NOW);
        assert_eq!(memory.record(&pair).unwrap().streak, 0);
    }

    #[test]
    fn a_pair_with_a_long_streak_comes_back_less_often() {
        let pair = PAIRS[6];
        let mut steady = Memory::default();
        let mut shaky = Memory::default();
        for _ in 0..10 {
            steady.record_answer(pair, 1.0, false, NOW);
        }
        for i in 0..10 {
            // The same answers, but one slow one near the end.
            let seconds = if i == 8 { SLOW_SECONDS } else { 1.0 };
            shaky.record_answer(pair, seconds, false, NOW);
        }
        let later = NOW + 10.0 * DAY;
        assert!(steady.weight(&pair, later) < shaky.weight(&pair, later));
    }

    #[test]
    fn a_pair_comes_back_more_often_once_it_is_due() {
        let pair = PAIRS[3];
        let mut memory = Memory::default();
        memory.record_answer(pair, 5.0, false, NOW);
        let interval = review_interval(memory.difficulty(&pair), 0);
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

    /// The days until a pair is next due, after each of `answers` in turn:
    /// each is given `seconds` after it appeared, and the first one reads
    /// its hint. The pair is asked again exactly when it comes due.
    fn schedule(seconds: f32, answers: usize, slip_at: Option<usize>) -> Vec<f64> {
        let pair = PAIRS[3];
        let mut memory = Memory::default();
        let mut now = NOW;
        (1..=answers)
            .map(|n| {
                if Some(n) == slip_at {
                    memory.record_miss(pair, now);
                } else {
                    memory.record_answer(pair, seconds, n == 1, now);
                }
                let record = memory.record(&pair).unwrap();
                let interval = review_interval(record.difficulty, record.streak);
                now += interval;
                interval / DAY
            })
            .collect()
    }

    #[test]
    fn a_quick_learner_is_asked_less_and_less_often() {
        let days = schedule(1.0, 14, None);
        assert!(days.is_sorted(), "{days:?}");
        assert!(
            days[0] > 0.5 && days[0] < 2.0,
            "the first review is soon: {days:?}"
        );
        assert_eq!(*days.last().unwrap(), 21.0);
        // The cap is reached after a handful of reviews, not dozens.
        let capped = days.iter().position(|&d| d == 21.0).unwrap();
        assert!((6..=12).contains(&capped), "{days:?}");
    }

    #[test]
    fn a_quick_learner_has_learned_it_within_about_two_months() {
        let days = schedule(1.0, 10, None);
        let total: f64 = days.iter().sum();
        assert!((30.0..90.0).contains(&total), "{total} days");
    }

    #[test]
    fn a_learner_who_struggles_keeps_seeing_the_pair_within_a_day() {
        let days = schedule(9.0, 30, None);
        assert!(days.iter().all(|&d| d < 1.5), "{days:?}");
    }

    #[test]
    fn a_slip_sends_a_well_known_pair_back_to_a_short_interval() {
        let days = schedule(1.0, 14, Some(8));
        assert!(days[6] > 5.0, "well known before the slip: {days:?}");
        assert!(days[7] < 2.0, "soon again after it: {days:?}");
        assert!(days[13] > days[8], "and it builds up again: {days:?}");
    }

    #[test]
    fn a_pair_answered_in_five_seconds_still_counts_as_good() {
        let days = schedule(5.0, 14, None);
        assert_eq!(*days.last().unwrap(), 21.0, "{days:?}");
    }
}
