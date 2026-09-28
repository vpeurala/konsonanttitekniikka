//! Long numbers: three or more digits remembered as several words. A long
//! number is read greedily, two digits at a time from the left, and an odd
//! digit left over at the end is a single-digit word: 201 = KOHO JÄÄ,
//! 1377 = JOULU SUSI.
//!
//! They come only once every pair has been met, as the bosses' last hits.
//! One more of a boss's hits is long on each level after that, and the
//! numbers grow a digit longer every few levels, so the step up is gentle.

use crate::curriculum::Curriculum;
use crate::pairs::{PAIRS, Pair};
use crate::rng::Rng;

/// On this many levels, starting with the first one with long numbers,
/// they are shown split into their pairs ("20 1") and made of pairs she
/// knows well.
const EASY_LONG_LEVELS: u32 = 5;
/// Long numbers start with this many digits...
const SHORTEST_LONG: usize = 3;
/// ...grow a digit longer every this many levels...
const LEVELS_PER_EXTRA_DIGIT: u32 = 3;
/// ...up to this many.
const LONGEST_LONG: usize = 6;

/// What a monster asks: one pair, or a long number made of several.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pairs: Vec<Pair>,
}

impl Question {
    pub fn single(pair: Pair) -> Self {
        Question { pairs: vec![pair] }
    }

    /// The question for `digits`, read greedily: two digits at a time from
    /// the left, and a single digit at the end if one is left over. None if
    /// it isn't all digits.
    pub fn for_number(digits: &str) -> Option<Self> {
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let pairs = digits
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let number = std::str::from_utf8(chunk).ok()?;
                PAIRS.iter().find(|p| p.number == number).copied()
            })
            .collect::<Option<Vec<Pair>>>()?;
        Some(Question { pairs })
    }

    pub fn is_long(&self) -> bool {
        self.pairs.len() > 1
    }

    /// The pairs it is made of, in order.
    pub fn pairs(&self) -> &[Pair] {
        &self.pairs
    }

    /// The first pair, which stands for the question in a single pair's
    /// place.
    pub fn first(&self) -> Pair {
        self.pairs[0]
    }

    /// Its number, like "201", or "20 1" when `split`.
    pub fn number(&self, split: bool) -> String {
        let separator = if split { " " } else { "" };
        self.pairs
            .iter()
            .map(|p| p.number)
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Its words in capitals, like "KOHO JÄÄ".
    pub fn words(&self) -> String {
        self.pairs
            .iter()
            .map(|p| p.word.to_uppercase())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Its words as typed, without spaces: "kohojää".
    pub fn typed_words(&self) -> String {
        self.pairs.iter().map(|p| p.word.to_lowercase()).collect()
    }
}

/// The first level with long numbers: the one after the last level that
/// brings new pairs.
pub fn first_long_level() -> u32 {
    let mut curriculum = Curriculum::new();
    let mut level = 1;
    while curriculum.next_level() > 0 {
        level += 1;
    }
    level + 1
}

/// How many levels with long numbers came before `level`.
fn long_levels_before(level: u32) -> Option<u32> {
    level.checked_sub(first_long_level())
}

/// How many of the `hits` of the boss of `level` are long numbers: none
/// before `first_long_level`, then one more on each level.
pub fn long_hits(level: u32, hits: usize) -> usize {
    long_levels_before(level).map_or(0, |before| (before as usize + 1).min(hits))
}

/// The most digits a long number on `level` can have: three at first, one
/// more every few levels.
pub fn max_digits(level: u32) -> usize {
    let before = long_levels_before(level).unwrap_or(0);
    (SHORTEST_LONG + (before / LEVELS_PER_EXTRA_DIGIT) as usize).min(LONGEST_LONG)
}

/// Whether long numbers on `level` are still the easy kind: split into
/// their pairs and made of pairs she knows well.
pub fn easy_long_numbers(level: u32) -> bool {
    long_levels_before(level).is_some_and(|before| before < EASY_LONG_LEVELS)
}

/// A long number for `level`. The newest length comes up most, the shorter
/// ones now and then. Each pair is picked from `pairs` by `weight`.
pub fn random_long_number(
    level: u32,
    pairs: &[Pair],
    rng: &mut Rng,
    weight: impl Fn(&Pair) -> f32,
) -> Question {
    let max = max_digits(level);
    let digits = if max > SHORTEST_LONG && rng.chance(0.3) {
        rng.index(SHORTEST_LONG..max)
    } else {
        max
    };
    let doubles: Vec<Pair> = pairs
        .iter()
        .filter(|p| p.number.len() == 2)
        .copied()
        .collect();
    let singles: Vec<Pair> = pairs
        .iter()
        .filter(|p| p.number.len() == 1)
        .copied()
        .collect();

    let mut chosen = Vec::new();
    let mut pick = |from: &[Pair], chosen: &mut Vec<Pair>| {
        if !from.is_empty() {
            chosen.push(from[rng.weighted_index(from, &weight)]);
        }
    };
    for _ in 0..digits / 2 {
        pick(&doubles, &mut chosen);
    }
    if digits % 2 == 1 {
        pick(&singles, &mut chosen);
    }
    Question { pairs: chosen }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Stream;

    #[test]
    fn numbers_are_read_two_digits_at_a_time() {
        let question = Question::for_number("201").unwrap();
        assert_eq!(question.words(), "KOHO JÄÄ");
        assert_eq!(question.typed_words(), "kohojää");
        assert_eq!(question.number(true), "20 1");
        assert_eq!(question.number(false), "201");

        let question = Question::for_number("1377").unwrap();
        assert_eq!(question.words(), "JOULU SUSI");
        assert_eq!(question.number(true), "13 77");
    }

    #[test]
    fn a_single_pair_reads_as_itself() {
        let question = Question::for_number("22").unwrap();
        assert!(!question.is_long());
        assert_eq!(question, Question::single(question.first()));
        assert_eq!(question.typed_words(), "keko");
        assert!(Question::for_number("").is_none());
        assert!(Question::for_number("2x").is_none());
    }

    #[test]
    fn long_numbers_wait_until_every_pair_has_been_met() {
        let first = first_long_level();
        let mut curriculum = Curriculum::new();
        for _ in 2..first {
            curriculum.next_level();
        }
        // The level before them still brought new pairs.
        assert!(!curriculum.new_pairs().is_empty());
        assert_eq!(curriculum.next_level(), 0);
        assert_eq!(long_hits(first - 1, 10), 0);
        assert!(!easy_long_numbers(first - 1));
    }

    #[test]
    fn each_level_after_that_has_one_more_long_hit() {
        let first = first_long_level();
        assert_eq!(long_hits(first, 10), 1);
        assert_eq!(long_hits(first + 1, 10), 2);
        assert_eq!(long_hits(first + 50, 10), 10);
        assert_eq!(long_hits(first + 50, 4), 4);
    }

    #[test]
    fn long_numbers_start_at_three_digits_and_grow_slowly() {
        let first = first_long_level();
        assert_eq!(max_digits(first), 3);
        assert_eq!(max_digits(first + LEVELS_PER_EXTRA_DIGIT - 1), 3);
        assert_eq!(max_digits(first + LEVELS_PER_EXTRA_DIGIT), 4);
        assert_eq!(max_digits(first + 1000), LONGEST_LONG);
    }

    #[test]
    fn the_first_long_levels_are_easy() {
        let first = first_long_level();
        assert!(easy_long_numbers(first));
        assert!(easy_long_numbers(first + EASY_LONG_LEVELS - 1));
        assert!(!easy_long_numbers(first + EASY_LONG_LEVELS));
    }

    #[test]
    fn random_long_numbers_read_back_the_same_way() {
        let mut rng = Rng::new(Stream::Gameplay, 1);
        for level in first_long_level()..first_long_level() + 20 {
            for _ in 0..50 {
                let question = random_long_number(level, PAIRS, &mut rng, |_| 1.0);
                let digits = question.number(false);
                assert!((SHORTEST_LONG..=max_digits(level)).contains(&digits.len()));
                assert_eq!(Question::for_number(&digits), Some(question));
            }
        }
    }
}
