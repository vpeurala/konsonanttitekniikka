//! What is known about the player: how well each pair is known, the best
//! level, stars, the daily streak, the lifetime counters, the badges and
//! the sound setting. The one copy of it all; `save` only turns it into
//! text and back.

use std::collections::BTreeMap;

use crate::badges::Stats;
use crate::memory::Memory;

/// No saved level is taken to be higher than this: nobody plays this far,
/// and a damaged number must not send the game counting to billions.
pub const MAX_LEVEL: u32 = 1000;

#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    /// How well each pair is known.
    pub memory: Memory,
    pub music_on: bool,
    /// Hardcore mode: monsters never show hints.
    pub hardcore: bool,
    /// The highest level reached.
    pub best_level: u32,
    /// The best stars earned on each level, from 1 to 3.
    pub stars: BTreeMap<u32, u8>,
    /// The last day played, as days since 1970, and how many days in a
    /// row ended with it.
    pub streak_day: i64,
    pub streak: u32,
    /// Lifetime counters that badges ask about.
    pub stats: Stats,
    /// The badges earned, by id, with the day each was earned (days since
    /// 1970).
    pub badges: BTreeMap<String, i64>,
}

impl Default for Progress {
    fn default() -> Self {
        Progress {
            memory: Memory::default(),
            music_on: true,
            hardcore: false,
            best_level: 1,
            stars: BTreeMap::new(),
            streak_day: 0,
            streak: 0,
            stats: Stats::default(),
            badges: BTreeMap::new(),
        }
    }
}

impl Progress {
    /// Records playing on `day` (days since 1970): the streak grows if the
    /// last day played was yesterday, and starts over after a gap.
    pub fn record_play_day(&mut self, day: i64) {
        if day == self.streak_day {
            return;
        }
        self.streak = if self.streak_day.checked_add(1) == Some(day) {
            self.streak.saturating_add(1)
        } else {
            1
        };
        self.streak_day = day;
    }

    /// Records finishing `level` with `stars`, keeping the best.
    pub fn record_level(&mut self, level: u32, stars: u8) {
        let best = self.stars.entry(level).or_insert(stars);
        *best = (*best).max(stars);
        self.best_level = self.best_level.max(level + 1);
    }
}

/// The day number (days since 1970, in UTC) for a time in seconds. In
/// Finland the day changes over in the small hours, which suits a streak.
pub fn day_of(seconds: f64) -> i64 {
    (seconds / 86_400.0).floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_streak_grows_on_consecutive_days_and_restarts_after_a_gap() {
        let mut data = Progress::default();
        data.record_play_day(100);
        assert_eq!(data.streak, 1);
        data.record_play_day(100);
        assert_eq!(data.streak, 1);
        data.record_play_day(101);
        data.record_play_day(102);
        assert_eq!(data.streak, 3);
        data.record_play_day(105);
        assert_eq!(data.streak, 1);
    }

    #[test]
    fn levels_keep_their_best_stars() {
        let mut data = Progress::default();
        data.record_level(2, 2);
        data.record_level(2, 1);
        assert_eq!(data.stars.get(&2), Some(&2));
        data.record_level(2, 3);
        assert_eq!(data.stars.get(&2), Some(&3));
        assert_eq!(data.best_level, 3);
    }

    #[test]
    fn the_streak_survives_extreme_days() {
        let mut data = Progress {
            streak_day: i64::MAX,
            streak: u32::MAX,
            ..Progress::default()
        };
        data.record_play_day(i64::MAX);
        data.record_play_day(i64::MIN);
        data.streak_day = i64::MAX - 1;
        data.streak = u32::MAX;
        data.record_play_day(i64::MAX);
        assert_eq!(data.streak, u32::MAX);
    }
}
