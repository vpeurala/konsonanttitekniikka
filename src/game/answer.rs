//! Reading what the player typed: which answers on screen it matches.

use macroquad::prelude::{Color, SKYBLUE, VIOLET};

/// What the typed text means given the answers currently on screen.
#[derive(Debug, PartialEq, Eq)]
pub enum InputOutcome {
    /// Nothing typed yet, or the text is the start of some answer.
    Pending,
    /// The text is the complete answer for the answers at these positions.
    Hit(Vec<usize>),
    /// The text cannot become any on-screen answer.
    DeadEnd,
}

/// Resolves typed text against the answers on screen.
///
/// A complete answer wins immediately, even when it is also the start of a
/// longer answer: with both "0" and "00" on screen, typing "0" hits "0".
pub fn resolve_input<'a>(
    typed: &str,
    answers: impl IntoIterator<Item = &'a str> + Clone,
) -> InputOutcome {
    if typed.is_empty() {
        return InputOutcome::Pending;
    }
    let hits: Vec<usize> = answers
        .clone()
        .into_iter()
        .enumerate()
        .filter(|(_, answer)| *answer == typed)
        .map(|(i, _)| i)
        .collect();
    if !hits.is_empty() {
        InputOutcome::Hit(hits)
    } else if answers.into_iter().any(|answer| answer.starts_with(typed)) {
        InputOutcome::Pending
    } else {
        InputOutcome::DeadEnd
    }
}

/// Where typed characters go: digits to the number slot, letters to the
/// word slot. Each slot answers the enemies showing the other kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Slot {
    Number,
    Word,
}

impl Slot {
    /// The slot a typed character goes to.
    pub(super) fn of_char(c: char) -> Slot {
        if c.is_ascii_digit() {
            Slot::Number
        } else {
            Slot::Word
        }
    }

    /// The slot's color, which the monsters it answers share.
    pub(super) fn accent(self) -> Color {
        match self {
            Slot::Number => SKYBLUE,
            Slot::Word => VIOLET,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_pending() {
        assert_eq!(resolve_input("", ["kuu"]), InputOutcome::Pending);
    }

    #[test]
    fn prefix_of_an_answer_is_pending() {
        assert_eq!(resolve_input("ku", ["kuu", "3"]), InputOutcome::Pending);
    }

    #[test]
    fn complete_answer_hits_every_matching_enemy() {
        assert_eq!(
            resolve_input("2", ["2", "kuu", "2"]),
            InputOutcome::Hit(vec![0, 2])
        );
    }

    #[test]
    fn complete_answer_wins_over_longer_answer() {
        assert_eq!(resolve_input("0", ["00", "0"]), InputOutcome::Hit(vec![1]));
    }

    #[test]
    fn prefix_of_longer_answer_waits() {
        assert_eq!(resolve_input("0", ["00", "5"]), InputOutcome::Pending);
    }

    #[test]
    fn text_matching_nothing_is_a_dead_end() {
        assert_eq!(resolve_input("ka", ["kuu", "3"]), InputOutcome::DeadEnd);
    }

    #[test]
    fn typing_with_no_enemies_is_a_dead_end() {
        assert_eq!(resolve_input("1", []), InputOutcome::DeadEnd);
    }

    #[test]
    fn digits_go_to_the_number_slot_and_letters_to_the_word_slot() {
        assert_eq!(Slot::of_char('7'), Slot::Number);
        assert_eq!(Slot::of_char('s'), Slot::Word);
        assert_eq!(Slot::of_char('ä'), Slot::Word);
    }
}
