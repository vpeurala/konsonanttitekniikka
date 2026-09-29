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

/// Which of the pairs it is: its position in `PAIRS`. Small, cheap to
/// compare and hash, and it can't name a pair that doesn't exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PairId(u8);

impl PairId {
    /// The position in `PAIRS`.
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// A number and the word that encodes it.
///
/// Numbers are strings because "0" and "00" are distinct pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub id: PairId,
    pub number: &'static str,
    pub word: &'static str,
}

/// The consonant that encodes each digit, indexed by digit.
pub const DIGIT_CONSONANTS: [char; 10] = ['h', 'j', 'k', 'l', 'm', 'p', 'r', 's', 't', 'v'];

/// The filler letters.
pub const VOWELS: [char; 8] = ['a', 'e', 'i', 'o', 'u', 'y', 'ä', 'ö'];

/// Whether a lowercase character can appear in an answer.
pub fn is_answer_char(c: char) -> bool {
    c.is_ascii_digit() || DIGIT_CONSONANTS.contains(&c) || VOWELS.contains(&c)
}

/// The pair for `number`, like "22", if there is one.
pub fn find(number: &str) -> Option<Pair> {
    PAIRS.iter().find(|p| p.number == number).copied()
}

/// The pair with this id.
pub fn get(id: PairId) -> Pair {
    PAIRS[id.index()]
}

/// How many pairs there are: 0-9 and 00-99.
pub const PAIR_COUNT: usize = 110;

/// The pairs as (number, word), in the order they are numbered by.
const WORDS: [(&str, &str); PAIR_COUNT] = [
    ("0", "hai"),
    ("1", "jää"),
    ("2", "kuu"),
    ("3", "luu"),
    ("4", "maa"),
    ("5", "puu"),
    ("6", "rae"),
    ("7", "suu"),
    ("8", "täi"),
    ("9", "vyö"),
    ("00", "hiha"),
    ("01", "häjy"),
    ("02", "hauki"),
    ("03", "huilu"),
    ("04", "haamu"),
    ("05", "huopa"),
    ("06", "hiiri"),
    ("07", "hius"),
    ("08", "hauta"),
    ("09", "haavi"),
    ("10", "jauho"),
    ("11", "jojo"),
    ("12", "joki"),
    ("13", "joulu"),
    ("14", "juomu"),
    ("15", "jopo"),
    ("16", "juuri"),
    ("17", "jousi"),
    ("18", "jeti"),
    ("19", "jyvä"),
    ("20", "koho"),
    ("21", "koju"),
    ("22", "keko"),
    ("23", "kela"),
    ("24", "kuomu"),
    ("25", "kupu"),
    ("26", "koira"),
    ("27", "kaasu"),
    ("28", "kota"),
    ("29", "kavio"),
    ("30", "liha"),
    ("31", "leija"),
    ("32", "leka"),
    ("33", "luola"),
    ("34", "liima"),
    ("35", "lapio"),
    ("36", "lyyra"),
    ("37", "liesi"),
    ("38", "luoti"),
    ("39", "laiva"),
    ("40", "maha"),
    ("41", "maja"),
    ("42", "muki"),
    ("43", "mela"),
    ("44", "muumio"),
    ("45", "mopo"),
    ("46", "muuri"),
    ("47", "muusi"),
    ("48", "mato"),
    ("49", "muovi"),
    ("50", "pyyhe"),
    ("51", "poiju"),
    ("52", "puku"),
    ("53", "peili"),
    ("54", "piimä"),
    ("55", "pipo"),
    ("56", "pora"),
    ("57", "paasi"),
    ("58", "pata"),
    ("59", "paavi"),
    ("60", "raha"),
    ("61", "ryijy"),
    ("62", "reki"),
    ("63", "railo"),
    ("64", "riimu"),
    ("65", "rapu"),
    ("66", "ruori"),
    ("67", "ruusu"),
    ("68", "rata"),
    ("69", "rovio"),
    ("70", "saha"),
    ("71", "soija"),
    ("72", "sika"),
    ("73", "siili"),
    ("74", "siima"),
    ("75", "siipi"),
    ("76", "siru"),
    ("77", "susi"),
    ("78", "sota"),
    ("79", "sauva"),
    ("80", "tuohi"),
    ("81", "taiji"),
    ("82", "tiuku"),
    ("83", "tiili"),
    ("84", "taimi"),
    ("85", "tipu"),
    ("86", "terä"),
    ("87", "teesi"),
    ("88", "toti"),
    ("89", "tavi"),
    ("90", "vuohi"),
    ("91", "vaja"),
    ("92", "vaaka"),
    ("93", "viulu"),
    ("94", "vaimo"),
    ("95", "vapa"),
    ("96", "vuori"),
    ("97", "vaasi"),
    ("98", "vouti"),
    ("99", "vauva"),
];

const fn build() -> [Pair; PAIR_COUNT] {
    let mut pairs = [Pair {
        id: PairId(0),
        number: "",
        word: "",
    }; PAIR_COUNT];
    let mut i = 0;
    while i < PAIR_COUNT {
        pairs[i] = Pair {
            id: PairId(i as u8),
            number: WORDS[i].0,
            word: WORDS[i].1,
        };
        i += 1;
    }
    pairs
}

const TABLE: [Pair; PAIR_COUNT] = build();

/// Every pair, in the order of their ids.
pub const PAIRS: &[Pair] = &TABLE;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Decodes a word into its number by mapping each digit consonant to
    /// its digit and ignoring every other letter.
    fn decode(word: &str) -> String {
        word.chars()
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
            for c in p.word.chars() {
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
    fn words_are_lowercase() {
        for p in PAIRS {
            assert_eq!(p.word, p.word.to_lowercase(), "{} = {}", p.number, p.word);
        }
    }

    #[test]
    fn words_are_unique() {
        let mut seen = HashSet::new();
        for p in PAIRS {
            assert!(seen.insert(p.word), "duplicate word {}", p.word);
        }
    }

    #[test]
    fn a_pair_is_found_by_its_number() {
        assert_eq!(find("22").map(|p| p.word), Some("keko"));
        assert_eq!(find("0").map(|p| p.word), Some("hai"));
        assert_eq!(find("00").map(|p| p.word), Some("hiha"));
        assert_eq!(find("100"), None);
        assert_eq!(find(""), None);
    }

    #[test]
    fn ids_are_the_positions_in_the_table() {
        for (i, pair) in PAIRS.iter().enumerate() {
            assert_eq!(pair.id.index(), i);
            assert_eq!(get(pair.id), *pair);
        }
    }

    #[test]
    fn a_pair_found_by_number_has_the_matching_id() {
        for pair in PAIRS {
            assert_eq!(find(pair.number).map(|p| p.id), Some(pair.id));
        }
    }
}
