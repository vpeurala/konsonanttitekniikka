//! The Finnish Major system: the canonical number-word pairs.
//!
//! This module is the single source of truth for the pairs. Each digit is
//! encoded by one consonant, and every occurrence counts (22 = keko):
//!
//! | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
//! |---|---|---|---|---|---|---|---|---|---|
//! | h | j | k | l | m | p | r | s | t | v |
//!
//! Vowels are filler. No other letters appear in the words.

/// A number and the word that encodes it.
///
/// Numbers are strings because "0" and "00" are distinct pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub number: &'static str,
    pub word: &'static str,
}

const fn pair(number: &'static str, word: &'static str) -> Pair {
    Pair { number, word }
}

/// The consonant that encodes each digit, indexed by digit.
pub const DIGIT_CONSONANTS: [char; 10] = ['h', 'j', 'k', 'l', 'm', 'p', 'r', 's', 't', 'v'];

/// The filler letters.
pub const VOWELS: [char; 8] = ['a', 'e', 'i', 'o', 'u', 'y', 'ä', 'ö'];

/// Whether a lowercase character can appear in an answer.
pub fn is_answer_char(c: char) -> bool {
    c.is_ascii_digit() || DIGIT_CONSONANTS.contains(&c) || VOWELS.contains(&c)
}

pub const PAIRS: &[Pair] = &[
    pair("0", "Hai"),
    pair("1", "Jää"),
    pair("2", "Kuu"),
    pair("3", "Luu"),
    pair("4", "Maa"),
    pair("5", "Puu"),
    pair("6", "Rae"),
    pair("7", "Suu"),
    pair("8", "Täi"),
    pair("9", "Vyö"),
    pair("00", "hiha"),
    pair("01", "häjy"),
    pair("02", "hauki"),
    pair("03", "huilu"),
    pair("04", "haamu"),
    pair("05", "huopa"),
    pair("06", "hiiri"),
    pair("07", "hius"),
    pair("08", "hauta"),
    pair("09", "haavi"),
    pair("10", "jauho"),
    pair("11", "jojo"),
    pair("12", "joki"),
    pair("13", "joulu"),
    pair("14", "juomu"),
    pair("15", "jopo"),
    pair("16", "juuri"),
    pair("17", "jousi"),
    pair("18", "jeti"),
    pair("19", "jyvä"),
    pair("20", "koho"),
    pair("21", "koju"),
    pair("22", "keko"),
    pair("23", "kela"),
    pair("24", "kuomu"),
    pair("25", "kupu"),
    pair("26", "koira"),
    pair("27", "kaasu"),
    pair("28", "kota"),
    pair("29", "kavio"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Decodes a word into its number by mapping each digit consonant to
    /// its digit and ignoring every other letter.
    fn decode(word: &str) -> String {
        word.to_lowercase()
            .chars()
            .filter_map(|c| DIGIT_CONSONANTS.iter().position(|&d| d == c))
            .map(|digit| char::from_digit(digit as u32, 10).unwrap())
            .collect()
    }

    #[test]
    fn every_word_encodes_its_number() {
        for p in PAIRS {
            assert_eq!(decode(p.word), p.number, "{} = {}", p.number, p.word);
        }
    }

    #[test]
    fn words_use_only_digit_consonants_and_vowels() {
        for p in PAIRS {
            for c in p.word.to_lowercase().chars() {
                assert!(
                    DIGIT_CONSONANTS.contains(&c) || VOWELS.contains(&c),
                    "{} = {} contains '{c}'",
                    p.number,
                    p.word
                );
            }
        }
    }

    #[test]
    fn numbers_are_unique() {
        let mut seen = HashSet::new();
        for p in PAIRS {
            assert!(seen.insert(p.number), "duplicate number {}", p.number);
        }
    }

    #[test]
    fn words_are_unique_ignoring_case() {
        let mut seen = HashSet::new();
        for p in PAIRS {
            assert!(
                seen.insert(p.word.to_lowercase()),
                "duplicate word {}",
                p.word
            );
        }
    }
}
