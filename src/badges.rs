//! Badges (kunniamerkit): rewards for milestones, from easy ones like the
//! first monster to very hard ones like fifty levels. The table `BADGES` is
//! the single source of truth; the badge screen, the notices and the
//! booklet all read it, so what they say can't disagree.
//!
//! Everything here is pure. A `Standing` says how far the player has got, a
//! `Requirement` says what a badge needs from it, and `award` records the
//! badges newly earned. An earned badge is remembered in the save file and
//! never taken away, even if, say, a pair is forgotten again later.

use crate::memory::{FAST_SECONDS, Memory};
use crate::pairs::{PAIR_COUNT, PAIRS};
use crate::progress::is_learned;
use crate::save::SaveData;

/// The most any counter is allowed to reach, so a damaged save can't send
/// the numbers to billions.
pub const MAX_COUNT: u32 = 1_000_000_000;

/// Lifetime counters that can't be worked out from the rest of the save.
/// Each has a name in the save file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Ordinary monsters defeated in the game.
    pub monsters: u32,
    /// Bosses defeated.
    pub bosses: u32,
    /// Long numbers answered correctly.
    pub long_answers: u32,
    /// Answers given at once, without a hint.
    pub quick_answers: u32,
    /// Flash cards answered correctly in practice.
    pub practice_correct: u32,
    /// Levels finished without a wrong key or a hit.
    pub flawless_levels: u32,
    /// The most correct answers in a row, in one game, without a wrong key
    /// or a hit in between.
    pub best_combo: u32,
}

impl Stats {
    /// Every counter with its name in the save file.
    pub fn fields(&self) -> [(&'static str, u32); 7] {
        [
            ("monsters", self.monsters),
            ("bosses", self.bosses),
            ("long-answers", self.long_answers),
            ("quick-answers", self.quick_answers),
            ("practice-correct", self.practice_correct),
            ("flawless-levels", self.flawless_levels),
            ("best-combo", self.best_combo),
        ]
    }

    /// Sets the counter called `name`. Returns false if there is none.
    pub fn set(&mut self, name: &str, value: u32) -> bool {
        let value = value.min(MAX_COUNT);
        let field = match name {
            "monsters" => &mut self.monsters,
            "bosses" => &mut self.bosses,
            "long-answers" => &mut self.long_answers,
            "quick-answers" => &mut self.quick_answers,
            "practice-correct" => &mut self.practice_correct,
            "flawless-levels" => &mut self.flawless_levels,
            "best-combo" => &mut self.best_combo,
            _ => return false,
        };
        *field = value;
        true
    }

    /// Adds one to a counter, up to `MAX_COUNT`.
    pub fn bump(counter: &mut u32) {
        *counter = counter.saturating_add(1).min(MAX_COUNT);
    }
}

/// How far the player has got, everything a requirement can ask about.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Standing {
    pub stats: Stats,
    /// The highest level finished.
    pub levels_cleared: u32,
    /// How many of the pairs are learned.
    pub pairs_learned: u32,
    /// How many of the single digits 0–9 are learned.
    pub digits_learned: u32,
    /// The levels finished with three stars.
    pub three_star_levels: u32,
    /// Days in a row played.
    pub streak: u32,
    /// How many badges other than the last one are earned.
    pub others_earned: u32,
}

/// The standing of the player whose save is `data`, with `memory` telling
/// how well each pair is known.
pub fn standing(data: &SaveData, memory: &Memory) -> Standing {
    let learned = |pair: &&crate::pairs::Pair| is_learned(memory, pair);
    Standing {
        stats: data.stats,
        levels_cleared: data.best_level.saturating_sub(1),
        pairs_learned: PAIRS.iter().filter(learned).count() as u32,
        digits_learned: PAIRS
            .iter()
            .filter(|p| p.number.len() == 1)
            .filter(learned)
            .count() as u32,
        three_star_levels: data.stars.values().filter(|&&s| s >= 3).count() as u32,
        streak: data.streak,
        others_earned: BADGES
            .iter()
            .filter(|b| b.requirement != Requirement::Everything)
            .filter(|b| data.badges.contains_key(b.id))
            .count() as u32,
    }
}

/// What a badge asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// Defeat this many monsters.
    Monsters(u32),
    /// Defeat this many bosses.
    Bosses(u32),
    /// Finish this level.
    LevelsCleared(u32),
    /// Learn this many pairs.
    PairsLearned(u32),
    /// Learn all the single digits 0–9.
    DigitsLearned,
    /// Answer this many in a row without a mistake.
    Combo(u32),
    /// Finish this many levels with three stars.
    ThreeStarLevels(u32),
    /// Finish this many levels without a wrong key or a hit.
    FlawlessLevels(u32),
    /// Answer this many long numbers.
    LongAnswers(u32),
    /// Answer this many flash cards in practice.
    PracticeCorrect(u32),
    /// Play this many days in a row.
    Streak(u32),
    /// Answer this many monsters at once, without a hint.
    QuickAnswers(u32),
    /// Learn every pair and finish `MASTER_LEVELS` levels.
    Master,
    /// Earn every other badge.
    Everything,
}

/// The levels the master badge asks for.
pub const MASTER_LEVELS: u32 = 50;

impl Requirement {
    /// How far the player is towards this, and how far they need to get:
    /// the first number never exceeds the second.
    pub fn progress(self, s: &Standing) -> (u32, u32) {
        let (now, goal) = match self {
            Requirement::Monsters(n) => (s.stats.monsters, n),
            Requirement::Bosses(n) => (s.stats.bosses, n),
            Requirement::LevelsCleared(n) => (s.levels_cleared, n),
            Requirement::PairsLearned(n) => (s.pairs_learned, n),
            Requirement::DigitsLearned => (s.digits_learned, 10),
            Requirement::Combo(n) => (s.stats.best_combo, n),
            Requirement::ThreeStarLevels(n) => (s.three_star_levels, n),
            Requirement::FlawlessLevels(n) => (s.stats.flawless_levels, n),
            Requirement::LongAnswers(n) => (s.stats.long_answers, n),
            Requirement::PracticeCorrect(n) => (s.stats.practice_correct, n),
            Requirement::Streak(n) => (s.streak, n),
            Requirement::QuickAnswers(n) => (s.stats.quick_answers, n),
            Requirement::Master => (
                s.pairs_learned + s.levels_cleared.min(MASTER_LEVELS),
                PAIR_COUNT as u32 + MASTER_LEVELS,
            ),
            Requirement::Everything => (s.others_earned, BADGES.len() as u32 - 1),
        };
        (now.min(goal), goal)
    }

    /// Whether `s` has got that far.
    pub fn is_met(self, s: &Standing) -> bool {
        let (now, goal) = self.progress(s);
        now >= goal
    }

    /// What to do, in Finnish.
    pub fn describe(self) -> String {
        match self {
            Requirement::Monsters(1) => "Kaada ensimmäinen hirviö".to_owned(),
            Requirement::Monsters(n) => format!("Kaada {n} hirviötä"),
            Requirement::Bosses(1) => "Kukista ensimmäinen pomo".to_owned(),
            Requirement::Bosses(n) => format!("Kukista {n} pomoa"),
            Requirement::LevelsCleared(n) => format!("Läpäise taso {n}"),
            Requirement::PairsLearned(n) if n as usize == PAIR_COUNT => {
                format!("Opi kaikki {n} paria")
            }
            Requirement::PairsLearned(n) => format!("Opi {n} paria"),
            Requirement::DigitsLearned => "Opi kaikki luvut 0–9".to_owned(),
            Requirement::Combo(n) => format!("Vastaa {n} oikein putkeen ilman virheitä"),
            Requirement::ThreeStarLevels(1) => "Saa kolme tähteä yhdestä tasosta".to_owned(),
            Requirement::ThreeStarLevels(n) => format!("Saa kolme tähteä {n} tasosta"),
            Requirement::FlawlessLevels(1) => {
                "Läpäise taso ilman väärää näppäintä tai osumia".to_owned()
            }
            Requirement::FlawlessLevels(n) => {
                format!("Läpäise {n} tasoa ilman väärää näppäintä tai osumia")
            }
            Requirement::LongAnswers(1) => "Vastaa pitkään lukuun".to_owned(),
            Requirement::LongAnswers(n) => format!("Vastaa {n} pitkään lukuun"),
            Requirement::PracticeCorrect(1) => "Vastaa oikein harjoittelussa".to_owned(),
            Requirement::PracticeCorrect(n) => {
                format!("Vastaa oikein {n} kertaa harjoittelussa")
            }
            Requirement::Streak(n) => format!("Päiviä putkeen: {n}"),
            Requirement::QuickAnswers(n) => format!(
                "Vastaa {n} kertaa alle {} sekunnissa ilman vinkkiä",
                FAST_SECONDS as u32
            ),
            Requirement::Master => {
                format!("Opi kaikki {PAIR_COUNT} paria ja läpäise taso {MASTER_LEVELS}")
            }
            Requirement::Everything => "Ansaitse kaikki muut kunniamerkit".to_owned(),
        }
    }
}

/// What kind of achievement a badge is; the badge screen has a row for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Monsters,
    Bosses,
    Levels,
    Pairs,
    Combo,
    Skill,
    LongNumbers,
    Practice,
    Days,
    Speed,
    Legend,
}

impl Category {
    /// The title of the badge screen's row, in Finnish.
    pub fn title(self) -> &'static str {
        match self {
            Category::Monsters => "Hirviöt",
            Category::Bosses => "Pomot",
            Category::Levels => "Tasot",
            Category::Pairs => "Parit",
            Category::Combo => "Putket",
            Category::Skill => "Taito",
            Category::LongNumbers => "Pitkät luvut",
            Category::Practice => "Harjoittelu",
            Category::Days => "Päivät",
            Category::Speed => "Nopeus",
            Category::Legend => "Legendat",
        }
    }
}

/// How hard a badge is, from easiest to hardest; it sets the medal's metal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
    Diamond,
}

impl Tier {
    /// The tier's name in Finnish.
    pub fn name(self) -> &'static str {
        match self {
            Tier::Bronze => "Pronssi",
            Tier::Silver => "Hopea",
            Tier::Gold => "Kulta",
            Tier::Diamond => "Timantti",
        }
    }
}

/// One badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    /// Short and lowercase, used in the save file and the statistics.
    pub id: &'static str,
    /// The name, in Finnish.
    pub name: &'static str,
    pub category: Category,
    pub tier: Tier,
    pub requirement: Requirement,
}

const fn badge(
    id: &'static str,
    name: &'static str,
    category: Category,
    tier: Tier,
    requirement: Requirement,
) -> Badge {
    Badge {
        id,
        name,
        category,
        tier,
        requirement,
    }
}

use Category as C;
use Requirement as R;
use Tier::{Bronze, Diamond, Gold, Silver};

/// Every badge, grouped by category, easiest first within each.
// One badge to a line, so the table reads as a table.
#[rustfmt::skip]
pub const BADGES: [Badge; 47] = [
    badge("monsters-1", "Ensimmäinen kaato", C::Monsters, Bronze, R::Monsters(1)),
    badge("monsters-10", "Hirviönkaataja", C::Monsters, Bronze, R::Monsters(10)),
    badge("monsters-20", "Rohkea", C::Monsters, Bronze, R::Monsters(20)),
    badge("monsters-50", "Peikonpelätin", C::Monsters, Silver, R::Monsters(50)),
    badge("monsters-100", "Sadan sankari", C::Monsters, Silver, R::Monsters(100)),
    badge("monsters-500", "Kauhun kauhu", C::Monsters, Gold, R::Monsters(500)),
    badge("monsters-1000", "Tuhannen tuho", C::Monsters, Gold, R::Monsters(1000)),
    badge("monsters-5000", "Hirviöiden painajainen", C::Monsters, Diamond, R::Monsters(5000)),
    badge("bosses-1", "Ensimmäinen pomo", C::Bosses, Bronze, R::Bosses(1)),
    badge("bosses-10", "Pomonpieksäjä", C::Bosses, Silver, R::Bosses(10)),
    badge("bosses-50", "Pomojen pelko", C::Bosses, Gold, R::Bosses(50)),
    badge("levels-5", "Retkeilijä", C::Levels, Bronze, R::LevelsCleared(5)),
    badge("levels-10", "Seikkailija", C::Levels, Silver, R::LevelsCleared(10)),
    badge("levels-21", "Kaikki parit tavattu", C::Levels, Silver, R::LevelsCleared(21)),
    badge("levels-30", "Pitkän matkan kulkija", C::Levels, Gold, R::LevelsCleared(30)),
    badge("levels-50", "Puolisataa", C::Levels, Diamond, R::LevelsCleared(50)),
    badge("levels-100", "Sadan tason mestari", C::Levels, Diamond, R::LevelsCleared(100)),
    badge("pairs-digits", "Ykkösten mestari", C::Pairs, Bronze, R::DigitsLearned),
    badge("pairs-25", "Neljännes", C::Pairs, Silver, R::PairsLearned(25)),
    badge("pairs-50", "Puolet", C::Pairs, Silver, R::PairsLearned(50)),
    badge("pairs-75", "Kolme neljäsosaa", C::Pairs, Gold, R::PairsLearned(75)),
    badge("pairs-110", "Kaikki sata ja kymmenen", C::Pairs, Diamond, R::PairsLearned(110)),
    badge("combo-10", "Putki", C::Combo, Bronze, R::Combo(10)),
    badge("combo-25", "Vauhdissa", C::Combo, Silver, R::Combo(25)),
    badge("combo-50", "Pysäyttämätön", C::Combo, Gold, R::Combo(50)),
    badge("combo-100", "Virheetön sadan putki", C::Combo, Diamond, R::Combo(100)),
    badge("skill-star", "Kolme tähteä", C::Skill, Bronze, R::ThreeStarLevels(1)),
    badge("skill-flawless", "Virheetön taso", C::Skill, Silver, R::FlawlessLevels(1)),
    badge("skill-stars-10", "Tähtikokoelma", C::Skill, Silver, R::ThreeStarLevels(10)),
    badge("skill-stars-25", "Tähtitaivas", C::Skill, Gold, R::ThreeStarLevels(25)),
    badge("long-1", "Pitkä luku", C::LongNumbers, Silver, R::LongAnswers(1)),
    badge("long-25", "Pitkien lukujen taitaja", C::LongNumbers, Gold, R::LongAnswers(25)),
    badge("long-100", "Numeroiden noita", C::LongNumbers, Diamond, R::LongAnswers(100)),
    badge("practice-1", "Ensimmäinen kortti", C::Practice, Bronze, R::PracticeCorrect(1)),
    badge("practice-50", "Harjoittelija", C::Practice, Bronze, R::PracticeCorrect(50)),
    badge("practice-250", "Uurastaja", C::Practice, Silver, R::PracticeCorrect(250)),
    badge("practice-1000", "Harjoituksen mestari", C::Practice, Gold, R::PracticeCorrect(1000)),
    badge("days-2", "Uskollinen", C::Days, Bronze, R::Streak(2)),
    badge("days-3", "Kolmen päivän putki", C::Days, Bronze, R::Streak(3)),
    badge("days-7", "Viikon sankari", C::Days, Silver, R::Streak(7)),
    badge("days-14", "Kahden viikon kestäjä", C::Days, Silver, R::Streak(14)),
    badge("days-30", "Kuukauden noita", C::Days, Gold, R::Streak(30)),
    badge("days-100", "Sadan päivän sankari", C::Days, Diamond, R::Streak(100)),
    badge("speed-50", "Salamannopea", C::Speed, Silver, R::QuickAnswers(50)),
    badge("speed-500", "Ukkosnuoli", C::Speed, Gold, R::QuickAnswers(500)),
    badge("master", "Loitsumestari", C::Legend, Diamond, R::Master),
    badge("everything", "Lukuloitsija", C::Legend, Diamond, R::Everything),
];

/// The badge with this id.
pub fn find(id: &str) -> Option<&'static Badge> {
    BADGES.iter().find(|b| b.id == id)
}

/// Records every badge `data` has newly earned, on `day` (days since
/// 1970), and returns them in table order. Whether a pair is learned is
/// read from `memory`, which may be newer than the one in `data`.
pub fn award(data: &mut SaveData, memory: &Memory, day: i64) -> Vec<&'static Badge> {
    let mut earned = Vec::new();
    // The last badge depends on the others, so go round again until
    // nothing more is earned.
    loop {
        let standing = standing(data, memory);
        let new: Vec<&'static Badge> = BADGES
            .iter()
            .filter(|b| !data.badges.contains_key(b.id) && b.requirement.is_met(&standing))
            .collect();
        if new.is_empty() {
            return earned;
        }
        for badge in new {
            data.badges.insert(badge.id.to_owned(), day);
            earned.push(badge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::PairRecord;
    use std::collections::HashSet;

    const DAY: i64 = 20_000;

    /// A memory in which the first `n` pairs are fully learned.
    fn memory_knowing(n: usize) -> Memory {
        Memory::from_records(PAIRS.iter().take(n).map(|p| {
            (
                p.number.to_owned(),
                PairRecord {
                    difficulty: 0.0,
                    last_seen: 1.0,
                    times_seen: 9,
                    streak: 9,
                },
            )
        }))
    }

    fn earned_ids(data: &SaveData, memory: &Memory) -> Vec<&'static str> {
        let mut data = data.clone();
        award(&mut data, memory, DAY).iter().map(|b| b.id).collect()
    }

    #[test]
    fn a_new_player_has_earned_nothing() {
        assert!(earned_ids(&SaveData::default(), &Memory::default()).is_empty());
    }

    #[test]
    fn ids_are_unique_and_safe_for_the_save_file() {
        let mut seen = HashSet::new();
        for b in &BADGES {
            assert!(seen.insert(b.id), "duplicate {}", b.id);
            assert!(
                b.id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{}",
                b.id
            );
            assert!(!b.name.is_empty());
        }
    }

    #[test]
    fn badges_come_grouped_by_category_and_get_harder_within_one() {
        let mut finished = HashSet::new();
        let mut previous: Option<&Badge> = None;
        for b in &BADGES {
            if let Some(p) = previous {
                if p.category != b.category {
                    assert!(
                        finished.insert(p.category.title()),
                        "{} splits",
                        p.category.title()
                    );
                } else {
                    assert!(p.tier <= b.tier, "{} is easier than {}", b.id, p.id);
                }
            }
            previous = Some(b);
        }
    }

    #[test]
    fn every_requirement_can_be_met_and_none_is_met_at_the_start() {
        let start = Standing::default();
        for b in &BADGES {
            let (now, goal) = b.requirement.progress(&start);
            assert!(goal > 0, "{}", b.id);
            assert!(now < goal, "{} is met from the start", b.id);
        }
    }

    #[test]
    fn nothing_asks_for_more_pairs_than_there_are() {
        for b in &BADGES {
            if let Requirement::PairsLearned(n) = b.requirement {
                assert!(n as usize <= PAIR_COUNT, "{}", b.id);
            }
        }
    }

    #[test]
    fn descriptions_say_the_number_they_ask_for() {
        for b in &BADGES {
            let text = b.requirement.describe();
            match b.requirement {
                Requirement::Monsters(n)
                | Requirement::Bosses(n)
                | Requirement::LevelsCleared(n)
                | Requirement::PairsLearned(n)
                | Requirement::Combo(n)
                | Requirement::LongAnswers(n)
                | Requirement::PracticeCorrect(n)
                | Requirement::Streak(n)
                | Requirement::QuickAnswers(n)
                | Requirement::ThreeStarLevels(n)
                | Requirement::FlawlessLevels(n)
                    if n > 1 =>
                {
                    assert!(text.contains(&n.to_string()), "{}: {text}", b.id);
                }
                _ => assert!(!text.is_empty()),
            }
        }
    }

    #[test]
    fn a_counter_earns_its_badge_exactly_at_the_threshold() {
        let mut data = SaveData::default();
        data.stats.monsters = 9;
        assert_eq!(earned_ids(&data, &Memory::default()), ["monsters-1"]);
        data.stats.monsters = 10;
        assert_eq!(
            earned_ids(&data, &Memory::default()),
            ["monsters-1", "monsters-10"]
        );
    }

    #[test]
    fn levels_are_counted_from_the_best_level_reached() {
        // Reaching level 6 means level 5 was finished.
        let at = |best_level| SaveData {
            best_level,
            ..SaveData::default()
        };
        assert!(!earned_ids(&at(5), &Memory::default()).contains(&"levels-5"));
        assert!(earned_ids(&at(6), &Memory::default()).contains(&"levels-5"));
    }

    #[test]
    fn learned_pairs_count_and_the_digits_have_their_own_badge() {
        let data = SaveData::default();
        let ids = earned_ids(&data, &memory_knowing(9));
        assert!(!ids.contains(&"pairs-digits"));
        let ids = earned_ids(&data, &memory_knowing(10));
        assert_eq!(ids, ["pairs-digits"]);
        let ids = earned_ids(&data, &memory_knowing(25));
        assert!(ids.contains(&"pairs-25") && !ids.contains(&"pairs-50"));
    }

    #[test]
    fn only_three_star_levels_count_as_star_levels() {
        let mut data = SaveData::default();
        data.stars.insert(1, 2);
        assert!(earned_ids(&data, &Memory::default()).is_empty());
        data.stars.insert(2, 3);
        assert_eq!(earned_ids(&data, &Memory::default()), ["skill-star"]);
    }

    #[test]
    fn the_streak_earns_the_day_badges() {
        let data = SaveData {
            streak: 7,
            ..SaveData::default()
        };
        let ids = earned_ids(&data, &Memory::default());
        assert_eq!(ids, ["days-2", "days-3", "days-7"]);
    }

    #[test]
    fn an_earned_badge_is_never_taken_back() {
        let mut data = SaveData::default();
        award(&mut data, &memory_knowing(25), DAY);
        assert!(data.badges.contains_key("pairs-25"));
        // The pairs are forgotten again, and nothing is lost or added.
        assert!(award(&mut data, &Memory::default(), DAY + 1).is_empty());
        assert_eq!(data.badges.get("pairs-25"), Some(&DAY));
    }

    #[test]
    fn each_badge_is_earned_once_on_the_day_it_was_earned() {
        let mut data = SaveData::default();
        data.stats.monsters = 1;
        assert_eq!(award(&mut data, &Memory::default(), DAY).len(), 1);
        data.stats.monsters = 10;
        let second = award(&mut data, &Memory::default(), DAY + 3);
        assert_eq!(
            second.iter().map(|b| b.id).collect::<Vec<_>>(),
            ["monsters-10"]
        );
        assert_eq!(data.badges["monsters-1"], DAY);
        assert_eq!(data.badges["monsters-10"], DAY + 3);
    }

    #[test]
    fn the_master_badge_needs_every_pair_and_the_levels() {
        let at = |best_level| SaveData {
            best_level,
            ..SaveData::default()
        };
        let data = at(MASTER_LEVELS + 1);
        assert!(!earned_ids(&data, &memory_knowing(109)).contains(&"master"));
        assert!(earned_ids(&data, &memory_knowing(PAIR_COUNT)).contains(&"master"));
        let data = at(MASTER_LEVELS);
        assert!(!earned_ids(&data, &memory_knowing(PAIR_COUNT)).contains(&"master"));
    }

    #[test]
    fn earning_everything_earns_the_last_badge_in_the_same_go() {
        let mut data = SaveData {
            stats: Stats {
                monsters: 10_000,
                bosses: 100,
                long_answers: 1000,
                quick_answers: 1000,
                practice_correct: 5000,
                flawless_levels: 5,
                best_combo: 500,
            },
            best_level: 101,
            streak: 100,
            ..SaveData::default()
        };
        for level in 1..=30 {
            data.stars.insert(level, 3);
        }
        let earned = award(&mut data, &memory_knowing(PAIR_COUNT), DAY);
        assert_eq!(earned.len(), BADGES.len());
        assert_eq!(earned.last().map(|b| b.id), Some("everything"));
    }

    #[test]
    fn progress_never_passes_the_goal() {
        let mut standing = Standing::default();
        standing.stats.monsters = 99_999;
        assert_eq!(Requirement::Monsters(10).progress(&standing), (10, 10));
        assert_eq!(
            Requirement::Monsters(100_000).progress(&standing),
            (99_999, 100_000)
        );
    }

    #[test]
    fn stats_are_set_by_name_and_capped() {
        let mut stats = Stats::default();
        assert!(stats.set("monsters", 12));
        assert!(stats.set("best-combo", u32::MAX));
        assert!(!stats.set("nonsense", 1));
        assert_eq!(stats.monsters, 12);
        assert_eq!(stats.best_combo, MAX_COUNT);
        for (name, value) in stats.fields() {
            let mut copy = Stats::default();
            assert!(copy.set(name, value));
        }
    }
}
