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
//! stat monsters 120
//! badge monsters-100 20357
//! pair 22 0.3512 1790000000 7
//! ```

use std::fmt::Write;

use crate::badges;
use crate::memory::{Memory, PairRecord};
use crate::pairs;
pub use crate::progress::day_of;
use crate::progress::{MAX_LEVEL, Progress};

const HEADER: &str = "lukuloitsu-save 1";

impl Progress {
    pub fn to_text(&self) -> String {
        // Writing to a `String` can't fail.
        let mut text = String::new();
        let _ = self.write_text(&mut text);
        text
    }

    fn write_text(&self, text: &mut String) -> std::fmt::Result {
        writeln!(text, "{HEADER}")?;
        writeln!(text, "music {}", if self.music_on { "on" } else { "off" })?;
        writeln!(
            text,
            "hardcore {}",
            if self.hardcore { "on" } else { "off" }
        )?;
        writeln!(text, "best-level {}", self.best_level)?;
        for (level, stars) in &self.stars {
            writeln!(text, "stars {level} {stars}")?;
        }
        writeln!(text, "streak {} {}", self.streak_day, self.streak)?;
        for (name, value) in self.stats.fields() {
            if value > 0 {
                writeln!(text, "stat {name} {value}")?;
            }
        }
        for (id, day) in &self.badges {
            writeln!(text, "badge {id} {day}")?;
        }
        let mut records: Vec<_> = self.memory.records().collect();
        records.sort_by_key(|(number, _)| *number);
        for (number, r) in records {
            writeln!(
                text,
                "pair {number} {:.4} {:.0} {} {}",
                r.difficulty, r.last_seen, r.times_seen, r.streak
            )?;
        }
        Ok(())
    }

    /// Reads a save file, skipping lines it can't understand. Returns the
    /// defaults if the text isn't a save file at all.
    pub fn from_text(text: &str) -> Self {
        let mut data = Progress::default();
        let mut records = Vec::new();
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return data;
        }
        for line in lines {
            let words: Vec<&str> = line.split_whitespace().collect();
            data.read_line(&words, &mut records);
        }
        data.memory = Memory::from_records(records);
        data
    }

    /// Takes in one line of the file, already split into words: a fact
    /// about the player, or a pair's record, which is added to `records`.
    /// A line that isn't understood is skipped.
    fn read_line(&mut self, words: &[&str], records: &mut Vec<(String, PairRecord)>) {
        match words {
            ["music", setting] => self.music_on = *setting != "off",
            ["hardcore", setting] => self.hardcore = *setting == "on",
            ["best-level", level] => {
                if let Ok(level) = level.parse::<u32>() {
                    self.best_level = level.clamp(1, MAX_LEVEL);
                }
            }
            ["stars", level, stars] => {
                if let (Ok(level), Ok(stars)) = (level.parse(), stars.parse::<u8>()) {
                    self.stars.insert(level, stars.clamp(1, 3));
                }
            }
            ["streak", day, count] => {
                if let (Ok(day), Ok(count)) = (day.parse(), count.parse()) {
                    self.streak_day = day;
                    self.streak = count;
                }
            }
            ["stat", name, value] => {
                if let Ok(value) = value.parse() {
                    self.stats.set(name, value);
                }
            }
            ["badge", id, day] => {
                if let (Some(badge), Ok(day)) = (badges::find(id), day.parse()) {
                    self.badges.insert(badge.id.to_owned(), day);
                }
            }
            // The streak came later, so older saves lack it.
            ["pair", number, difficulty, last_seen, times_seen, rest @ ..] if rest.len() <= 1 => {
                let streak = rest.first().map_or(Some(0), |s| s.parse().ok());
                records.extend(pair_record(
                    number, difficulty, last_seen, times_seen, streak,
                ));
            }
            _ => {}
        }
    }
}

/// A pair's record from its words in the file, if they are all sound.
fn pair_record(
    number: &str,
    difficulty: &str,
    last_seen: &str,
    times_seen: &str,
    streak: Option<u32>,
) -> Option<(String, PairRecord)> {
    let difficulty: f32 = difficulty.parse().ok()?;
    let last_seen: f64 = last_seen.parse().ok()?;
    let sound = difficulty.is_finite() && last_seen.is_finite() && pairs::find(number).is_some();
    sound.then_some((
        number.to_owned(),
        PairRecord {
            difficulty: difficulty.clamp(0.0, 1.0),
            last_seen,
            times_seen: times_seen.parse().ok()?,
            streak: streak?,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::Stats;
    use crate::pairs::PAIRS;
    use crate::rng::{Rng, Stream};

    /// The numbers of the pairs the save knows something about, in order.
    fn numbers(data: &Progress) -> Vec<&'static str> {
        let mut numbers: Vec<_> = data.memory.records().map(|(number, _)| number).collect();
        numbers.sort_unstable();
        numbers
    }

    fn sample() -> Progress {
        let mut data = Progress {
            music_on: false,
            hardcore: true,
            best_level: 4,
            streak_day: 20_356,
            streak: 5,
            ..Progress::default()
        };
        data.stars.insert(1, 3);
        data.stars.insert(2, 1);
        data.stats.monsters = 120;
        data.stats.best_combo = 31;
        data.badges.insert("monsters-100".to_string(), 20_357);
        data.memory = Memory::from_records([(
            "22".to_string(),
            PairRecord {
                difficulty: 0.3512,
                last_seen: 1_790_000_000.0,
                times_seen: 7,
                streak: 3,
            },
        )]);
        data
    }

    #[test]
    fn progress_survives_a_round_trip() {
        let data = sample();
        assert_eq!(Progress::from_text(&data.to_text()), data);
    }

    #[test]
    fn damaged_lines_are_skipped_without_losing_the_rest() {
        let text = sample().to_text().replace("stars 2 1", "stars two ?") + "garbage line\n";
        let data = Progress::from_text(&text);
        assert_eq!(data.best_level, 4);
        assert_eq!(data.stars.get(&1), Some(&3));
        assert_eq!(data.stars.get(&2), None);
        assert_eq!(numbers(&data), ["22"]);
    }

    #[test]
    fn anything_but_a_save_file_gives_the_defaults() {
        assert_eq!(Progress::from_text("hello"), Progress::default());
        assert_eq!(Progress::from_text(""), Progress::default());
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
            let mut data = Progress::from_text(&text);
            assert!(
                (1..=MAX_LEVEL).contains(&data.best_level),
                "best level {} from {text:?}",
                data.best_level
            );
            for (number, record) in data.memory.records() {
                assert!(
                    (0.0..=1.0).contains(&record.difficulty) && record.last_seen.is_finite(),
                    "pair {number} {record:?} from {text:?}"
                );
            }
            // Whatever was read must survive being used.
            let memory = &data.memory;
            for pair in PAIRS {
                assert!(memory.weight(pair, 1.8e9).is_finite(), "{text:?}");
            }
            data.record_play_day(20_000);
            data.record_play_day(20_001);
            assert!(crate::levels::checkpoints(data.best_level).len() <= 25);
            // Writing rounds times to whole seconds, but once rounded a
            // save reads back as itself.
            let once = Progress::from_text(&data.to_text());
            assert_eq!(Progress::from_text(&once.to_text()), once, "{text:?}");
        }
    }

    #[test]
    fn a_save_cut_off_anywhere_still_loads() {
        let text = sample().to_text();
        for end in 0..=text.len() {
            if text.is_char_boundary(end) {
                Progress::from_text(&text[..end]);
            }
        }
    }

    #[test]
    fn an_absurd_best_level_is_capped() {
        let data = Progress::from_text(&format!("{HEADER}\nbest-level 4294967295\n"));
        assert_eq!(data.best_level, MAX_LEVEL);
        let data = Progress::from_text(&format!("{HEADER}\nbest-level 0\n"));
        assert_eq!(data.best_level, 1);
    }

    #[test]
    fn numbers_that_are_not_numbers_are_skipped() {
        let text = format!(
            "{HEADER}\npair 22 NaN 100 1\npair 23 inf 100 1\npair 24 0.5 inf 1\npair 25 0.5 100 1\npair x 0.5 100 1\n"
        );
        let data = Progress::from_text(&text);
        assert_eq!(numbers(&data), ["25"]);
    }

    #[test]
    fn saves_from_before_streaks_still_load() {
        let text = format!("{HEADER}\npair 22 0.3512 1790000000 7\n");
        let data = Progress::from_text(&text);
        let record = data
            .memory
            .record(&pairs::find("22").unwrap())
            .expect("the pair is kept");
        assert_eq!((record.times_seen, record.streak), (7, 0));
    }

    #[test]
    fn a_streak_is_saved_and_read_back() {
        let text = format!("{HEADER}\npair 22 0.3512 1790000000 7 4\n");
        let data = Progress::from_text(&text);
        assert_eq!(
            data.memory
                .record(&pairs::find("22").unwrap())
                .map(|r| r.streak),
            Some(4)
        );
        assert!(data.to_text().contains("pair 22 0.3512 1790000000 7 4"));
    }

    #[test]
    fn a_pair_line_with_a_broken_streak_or_extra_words_is_skipped() {
        let text = format!(
            "{HEADER}\npair 22 0.5 100 1 x\npair 23 0.5 100 1 2 3\npair 24 0.5 100 1 -1\npair 25 0.5 100 1 2\n"
        );
        let data = Progress::from_text(&text);
        assert_eq!(numbers(&data), ["25"]);
    }

    #[test]
    fn stats_and_badges_survive_a_round_trip() {
        let data = Progress::from_text(&sample().to_text());
        assert_eq!(data.stats.monsters, 120);
        assert_eq!(data.stats.best_combo, 31);
        assert_eq!(data.badges.get("monsters-100"), Some(&20_357));
        assert_eq!(data, sample());
    }

    #[test]
    fn unknown_stats_and_badges_and_broken_numbers_are_skipped() {
        let text = format!(
            "{HEADER}\nstat mystery 5\nstat monsters many\nstat bosses 3\n\
             badge no-such-badge 5\nbadge bosses-1 someday\nbadge levels-5 7\n\
             stat monsters 99999999999\n"
        );
        let data = Progress::from_text(&text);
        assert_eq!(data.stats.bosses, 3);
        assert_eq!(data.stats.monsters, 0);
        assert_eq!(data.badges.len(), 1);
        assert_eq!(data.badges.get("levels-5"), Some(&7));
    }

    #[test]
    fn a_huge_counter_is_capped() {
        let data = Progress::from_text(&format!("{HEADER}\nstat monsters 4000000000\n"));
        assert_eq!(data.stats.monsters, badges::MAX_COUNT);
    }

    #[test]
    fn saves_from_before_badges_still_load() {
        let data = Progress::from_text(&format!("{HEADER}\nmusic on\nbest-level 3\n"));
        assert_eq!(data.stats, Stats::default());
        assert!(data.badges.is_empty());
        assert_eq!(data.best_level, 3);
    }
}
