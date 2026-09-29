//! How wide the labels are. The simulation needs this to keep labels apart
//! and on screen, so it is a fixed table rather than a measurement: a game
//! must play out the same whatever the font machinery of the device does.
//!
//! The table holds the advance widths of Nunito Bold, the labels' font,
//! for the characters a label can hold, in thousandths of the font size.
//! If the font ever changes, only how snugly labels sit changes.

/// The advance width of `c` in thousandths of the font size.
fn advance(c: char) -> u32 {
    match c {
        ' ' => 271,
        'A' | 'Å' | 'Ä' => 744,
        'B' => 688,
        'C' => 680,
        'D' => 762,
        'E' => 597,
        'F' | 'L' => 562,
        'G' => 736,
        'H' => 773,
        'I' => 282,
        'J' => 354,
        'K' => 665,
        'M' => 868,
        'N' => 748,
        'O' | 'Q' | 'Ö' => 785,
        'P' => 652,
        'R' => 686,
        'S' => 631,
        'T' => 621,
        'U' => 738,
        'V' => 713,
        'W' => 1113,
        'X' => 672,
        'Y' => 618,
        'Z' => 605,
        // Digits and "=" are 600, and so is anything unforeseen.
        _ => 600,
    }
}

/// The width of a label's `text` at `size`, in virtual units.
pub fn text_width(text: &str, size: u16) -> f32 {
    let thousandths: u32 = text.chars().map(advance).sum();
    thousandths as f32 * f32::from(size) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

    #[test]
    fn digits_are_all_equally_wide() {
        assert_eq!(text_width("0", 20), 12.0);
        assert_eq!(text_width("1234", 20), 48.0);
    }

    #[test]
    fn width_grows_with_size_and_length() {
        assert!(text_width("MAA", 22) > text_width("MAA", 20));
        assert!(text_width("MAA", 22) > text_width("MA", 22));
    }

    #[test]
    fn narrow_letters_are_narrower_than_wide_ones() {
        assert!(text_width("I", 22) < text_width("W", 22));
    }

    #[test]
    fn every_letter_of_every_word_has_its_own_width() {
        for pair in PAIRS {
            for c in pair.word.to_uppercase().chars() {
                assert_ne!(advance(c), 0, "{c}");
            }
        }
    }
}
