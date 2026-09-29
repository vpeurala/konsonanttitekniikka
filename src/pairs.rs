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
    pair("30", "liha"),
    pair("31", "leija"),
    pair("32", "leka"),
    pair("33", "luola"),
    pair("34", "liima"),
    pair("35", "lapio"),
    pair("36", "lyyra"),
    pair("37", "liesi"),
    pair("38", "luoti"),
    pair("39", "laiva"),
    pair("40", "maha"),
    pair("41", "maja"),
    pair("42", "muki"),
    pair("43", "mela"),
    pair("44", "muumio"),
    pair("45", "mopo"),
    pair("46", "muuri"),
    pair("47", "muusi"),
    pair("48", "mato"),
    pair("49", "muovi"),
    pair("50", "pyyhe"),
    pair("51", "poiju"),
    pair("52", "puku"),
    pair("53", "peili"),
    pair("54", "piimä"),
    pair("55", "pipo"),
    pair("56", "pora"),
    pair("57", "paasi"),
    pair("58", "pata"),
    pair("59", "paavi"),
    pair("60", "raha"),
    pair("61", "ryijy"),
    pair("62", "reki"),
    pair("63", "railo"),
    pair("64", "riimu"),
    pair("65", "rapu"),
    pair("66", "ruori"),
    pair("67", "ruusu"),
    pair("68", "rata"),
    pair("69", "rovio"),
    pair("70", "saha"),
    pair("71", "soija"),
    pair("72", "sika"),
    pair("73", "siili"),
    pair("74", "siima"),
    pair("75", "siipi"),
    pair("76", "siru"),
    pair("77", "susi"),
    pair("78", "sota"),
    pair("79", "sauva"),
    pair("80", "tuohi"),
    pair("81", "taiji"),
    pair("82", "tiuku"),
    pair("83", "tiili"),
    pair("84", "taimi"),
    pair("85", "tipu"),
    pair("86", "terä"),
    pair("87", "teesi"),
    pair("88", "toti"),
    pair("89", "tavi"),
    pair("90", "vuohi"),
    pair("91", "vaja"),
    pair("92", "vaaka"),
    pair("93", "viulu"),
    pair("94", "vaimo"),
    pair("95", "vapa"),
    pair("96", "vuori"),
    pair("97", "vaasi"),
    pair("98", "vouti"),
    pair("99", "vauva"),
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
    fn every_number_from_0_to_9_and_00_to_99_has_a_pair() {
        let numbers: HashSet<&str> = PAIRS.iter().map(|p| p.number).collect();
        let expected: Vec<String> = (0..10)
            .map(|n| n.to_string())
            .chain((0..100).map(|n| format!("{n:02}")))
            .collect();
        for number in &expected {
            assert!(numbers.contains(number.as_str()), "missing {number}");
        }
        assert_eq!(PAIRS.len(), 110);
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
