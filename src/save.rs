//! Saving progress on the device, so it survives closing the app: how well
//! each pair is known, the best level, stars, the daily streak and the
//! music setting. Nothing leaves the device: in a browser, it stays in the
//! page's local storage.
//!
//! The file is plain text, one fact per line, and a damaged line is
//! skipped rather than losing everything:
//!
//! ```text
//! lukuloitsu-save 1
//! music on
//! best-level 4
//! stars 1 3
//! streak 20356 5
//! pair 22 0.3512 1790000000 7
//! ```

use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::memory::{Memory, PairRecord};
use crate::pairs::PAIRS;

const HEADER: &str = "lukuloitsu-save 1";

/// No saved level is taken to be higher than this: nobody plays this far,
/// and a damaged number must not send the game counting to billions.
const MAX_LEVEL: u32 = 1000;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveData {
    pub pairs: BTreeMap<String, PairRecord>,
    pub music_on: bool,
    /// The highest level reached.
    pub best_level: u32,
    /// The best stars earned on each level, from 1 to 3.
    pub stars: BTreeMap<u32, u8>,
    /// The last day played, as days since 1970, and how many days in a
    /// row ended with it.
    pub streak_day: i64,
    pub streak: u32,
}

impl Default for SaveData {
    fn default() -> Self {
        SaveData {
            pairs: BTreeMap::new(),
            music_on: true,
            best_level: 1,
            stars: BTreeMap::new(),
            streak_day: 0,
            streak: 0,
        }
    }
}

impl SaveData {
    pub fn to_text(&self) -> String {
        let mut text = format!("{HEADER}\n");
        text += &format!("music {}\n", if self.music_on { "on" } else { "off" });
        text += &format!("best-level {}\n", self.best_level);
        for (level, stars) in &self.stars {
            text += &format!("stars {level} {stars}\n");
        }
        text += &format!("streak {} {}\n", self.streak_day, self.streak);
        for (number, r) in &self.pairs {
            text += &format!(
                "pair {number} {:.4} {:.0} {}\n",
                r.difficulty, r.last_seen, r.times_seen
            );
        }
        text
    }

    /// Reads a save file, skipping lines it can't understand. Returns the
    /// defaults if the text isn't a save file at all.
    pub fn from_text(text: &str) -> Self {
        let mut data = SaveData::default();
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return data;
        }
        for line in lines {
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                ["music", setting] => data.music_on = *setting != "off",
                ["best-level", level] => {
                    if let Ok(level) = level.parse::<u32>() {
                        data.best_level = level.clamp(1, MAX_LEVEL);
                    }
                }
                ["stars", level, stars] => {
                    if let (Ok(level), Ok(stars)) = (level.parse(), stars.parse::<u8>()) {
                        data.stars.insert(level, stars.clamp(1, 3));
                    }
                }
                ["streak", day, count] => {
                    if let (Ok(day), Ok(count)) = (day.parse(), count.parse()) {
                        data.streak_day = day;
                        data.streak = count;
                    }
                }
                ["pair", number, difficulty, last_seen, times_seen] => {
                    if let (Ok(difficulty), Ok(last_seen), Ok(times_seen)) = (
                        difficulty.parse::<f32>(),
                        last_seen.parse::<f64>(),
                        times_seen.parse(),
                    ) && difficulty.is_finite()
                        && last_seen.is_finite()
                        && PAIRS.iter().any(|p| p.number == *number)
                    {
                        data.pairs.insert(
                            number.to_string(),
                            PairRecord {
                                difficulty: difficulty.clamp(0.0, 1.0),
                                last_seen,
                                times_seen,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
        data
    }

    pub fn memory(&self) -> Memory {
        Memory::from_records(self.pairs.iter().map(|(n, r)| (n.clone(), *r)))
    }

    pub fn set_memory(&mut self, memory: &Memory) {
        self.pairs = memory.records().map(|(n, r)| (n.to_string(), *r)).collect();
    }

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

/// Where the save file lives, if saving is possible on this platform.
#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<PathBuf> {
    if cfg!(target_os = "android") {
        // The app's private storage, which Android gives every app.
        Some(PathBuf::from(
            "/data/data/fi.lukuloitsu.lukuloitsu/files/save.txt",
        ))
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        // On iOS, HOME is the app's own sandbox.
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/Application Support/Lukuloitsu/save.txt"))
    } else if cfg!(target_family = "unix") || cfg!(target_os = "windows") {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("APPDATA"))?;
        Some(PathBuf::from(home).join(".lukuloitsu/save.txt"))
    } else {
        None
    }
}

/// Loads the saved progress, or the defaults if there is none.
pub fn load() -> SaveData {
    read_text()
        .map(|text| SaveData::from_text(&text))
        .unwrap_or_default()
}

/// Saves progress. Failing to save is not worth stopping the game for, so
/// errors are only logged.
pub fn store(data: &SaveData) {
    write_text(&data.to_text());
}

#[cfg(not(target_arch = "wasm32"))]
fn read_text() -> Option<String> {
    std::fs::read_to_string(path()?).ok()
}

/// Writes a new file and then swaps it in, so an interrupted save never
/// leaves a half-written file behind.
#[cfg(not(target_arch = "wasm32"))]
fn write_text(text: &str) {
    let Some(path) = path() else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    let temporary = path.with_extension("tmp");
    let result = std::fs::create_dir_all(dir)
        .and_then(|_| std::fs::write(&temporary, text))
        .and_then(|_| std::fs::rename(&temporary, &path));
    if let Err(error) = result {
        macroquad::logging::warn!("Saving progress to {} failed: {error}", path.display());
    }
}

// In a browser, progress is kept in the page's local storage.
#[cfg(target_arch = "wasm32")]
fn read_text() -> Option<String> {
    crate::web::load()
}

#[cfg(target_arch = "wasm32")]
fn write_text(text: &str) {
    crate::web::store(text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::{Rng, Stream};

    fn sample() -> SaveData {
        let mut data = SaveData {
            music_on: false,
            best_level: 4,
            streak_day: 20_356,
            streak: 5,
            ..SaveData::default()
        };
        data.stars.insert(1, 3);
        data.stars.insert(2, 1);
        data.pairs.insert(
            "22".to_string(),
            PairRecord {
                difficulty: 0.3512,
                last_seen: 1_790_000_000.0,
                times_seen: 7,
            },
        );
        data
    }

    #[test]
    fn progress_survives_a_round_trip() {
        let data = sample();
        assert_eq!(SaveData::from_text(&data.to_text()), data);
    }

    #[test]
    fn damaged_lines_are_skipped_without_losing_the_rest() {
        let text = sample().to_text().replace("stars 2 1", "stars two ?") + "garbage line\n";
        let data = SaveData::from_text(&text);
        assert_eq!(data.best_level, 4);
        assert_eq!(data.stars.get(&1), Some(&3));
        assert_eq!(data.stars.get(&2), None);
        assert_eq!(data.pairs.len(), 1);
    }

    #[test]
    fn anything_but_a_save_file_gives_the_defaults() {
        assert_eq!(SaveData::from_text("hello"), SaveData::default());
        assert_eq!(SaveData::from_text(""), SaveData::default());
    }

    #[test]
    fn the_streak_grows_on_consecutive_days_and_restarts_after_a_gap() {
        let mut data = SaveData::default();
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
        let mut data = SaveData::default();
        data.record_level(2, 2);
        data.record_level(2, 1);
        assert_eq!(data.stars.get(&2), Some(&2));
        data.record_level(2, 3);
        assert_eq!(data.stars.get(&2), Some(&3));
        assert_eq!(data.best_level, 3);
    }

    /// A save file made of plausible and implausible pieces, as damaged or
    /// tampered local storage might hold.
    fn garbage(seed: u64) -> String {
        const TOKENS: [&str; 26] = [
            "music",
            "best-level",
            "stars",
            "streak",
            "pair",
            "on",
            "off",
            "NaN",
            "inf",
            "-inf",
            "-1",
            "0",
            "1",
            "22",
            "99",
            "0.5",
            "1e400",
            "4294967295",
            "99999999999999999999",
            "9223372036854775807",
            "-9223372036854775808",
            "x",
            "",
            "3.14",
            "1800000000",
            "00",
        ];
        let mut rng = Rng::new(Stream::Gameplay, seed);
        let mut text = format!("{HEADER}\n");
        for _ in 0..rng.index(1..12) {
            for _ in 0..rng.index(1..7) {
                text += rng.pick(&TOKENS);
                text += " ";
            }
            text += "\n";
        }
        text
    }

    #[test]
    fn damaged_save_files_never_panic_and_always_give_usable_data() {
        for seed in 0..3000 {
            let text = garbage(seed);
            let mut data = SaveData::from_text(&text);
            assert!(
                (1..=MAX_LEVEL).contains(&data.best_level),
                "best level {} from {text:?}",
                data.best_level
            );
            for (number, record) in &data.pairs {
                assert!(
                    (0.0..=1.0).contains(&record.difficulty) && record.last_seen.is_finite(),
                    "pair {number} {record:?} from {text:?}"
                );
            }
            // Whatever was read must survive being used.
            let memory = data.memory();
            for pair in PAIRS {
                assert!(memory.weight(pair, 1.8e9).is_finite(), "{text:?}");
            }
            data.record_play_day(20_000);
            data.record_play_day(20_001);
            assert!(crate::levels::checkpoints(data.best_level).len() <= 25);
            // Writing rounds times to whole seconds, but once rounded a
            // save reads back as itself.
            let once = SaveData::from_text(&data.to_text());
            assert_eq!(SaveData::from_text(&once.to_text()), once, "{text:?}");
        }
    }

    #[test]
    fn a_save_cut_off_anywhere_still_loads() {
        let text = sample().to_text();
        for end in 0..=text.len() {
            if text.is_char_boundary(end) {
                SaveData::from_text(&text[..end]);
            }
        }
    }

    #[test]
    fn the_streak_survives_extreme_days() {
        let mut data = SaveData {
            streak_day: i64::MAX,
            streak: u32::MAX,
            ..SaveData::default()
        };
        data.record_play_day(i64::MAX);
        data.record_play_day(i64::MIN);
        data.streak_day = i64::MAX - 1;
        data.streak = u32::MAX;
        data.record_play_day(i64::MAX);
        assert_eq!(data.streak, u32::MAX);
    }

    #[test]
    fn an_absurd_best_level_is_capped() {
        let data = SaveData::from_text(&format!("{HEADER}\nbest-level 4294967295\n"));
        assert_eq!(data.best_level, MAX_LEVEL);
        let data = SaveData::from_text(&format!("{HEADER}\nbest-level 0\n"));
        assert_eq!(data.best_level, 1);
    }

    #[test]
    fn numbers_that_are_not_numbers_are_skipped() {
        let text = format!(
            "{HEADER}\npair 22 NaN 100 1\npair 23 inf 100 1\npair 24 0.5 inf 1\npair 25 0.5 100 1\npair x 0.5 100 1\n"
        );
        let data = SaveData::from_text(&text);
        assert_eq!(data.pairs.keys().collect::<Vec<_>>(), vec!["25"]);
    }
}
