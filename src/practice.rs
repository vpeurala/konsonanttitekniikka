//! Practice mode: calm flash cards without monsters. Each card shows a
//! word or a number, and the player types its counterpart. Answers feed the
//! same spaced-repetition memory as the game.

use macroquad::prelude::*;

use crate::audio::Sfx;
use crate::curriculum::Curriculum;
use crate::fonts::draw_centered_text;
use crate::fonts::{self, Style};
use crate::frame::Frame;
use crate::game::{InputOutcome, resolve_input};
use crate::keyboard::Key;
use crate::memory::Memory;
use crate::pairs::{self, Pair};
use crate::pictures::draw_picture;
use crate::rng::{Rng, Stream};
use crate::view::{ARENA_H, ARENA_W};

/// How long a right answer stays on screen before the next card.
const CORRECT_SECONDS: f32 = 1.3;
/// How long a shown answer stays on screen before the next card.
const REVEALED_SECONDS: f32 = 3.0;

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const CARD: Color = Color::new(0.14, 0.13, 0.22, 1.0);

/// The pairs unlocked by the time the player reaches `level`, as in the
/// game's curriculum.
pub fn unlocked_pairs(level: u32) -> Vec<Pair> {
    let mut curriculum = Curriculum::new();
    for _ in 1..level {
        if curriculum.next_level() == 0 {
            break;
        }
    }
    curriculum.unlocked().to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Asking,
    /// Answered right; the next card comes after the given seconds.
    Correct(f32),
    /// Answered wrong or given up; the answer is shown for the seconds.
    Revealed(f32),
}

pub enum PracticeAction {
    Stay,
    Back,
}

pub struct PracticeScreen {
    pool: Vec<Pair>,
    rng: Rng,
    pair: Pair,
    shows_word: bool,
    /// Whether the answer is shown as a hint, as on a pair never seen.
    hint: bool,
    typed: String,
    /// Seconds since the card appeared.
    shown_for: f32,
    state: State,
    correct: u32,
    answered: u32,
    sfx: Vec<Sfx>,
    touch: bool,
}

impl PracticeScreen {
    /// Practice with the pairs unlocked up to `best_level`.
    pub fn new(best_level: u32, memory: &Memory, touch: bool, now: f64) -> Self {
        let pool = unlocked_pairs(best_level);
        let mut screen = PracticeScreen {
            pair: pool[0],
            pool,
            rng: Rng::new(Stream::Gameplay, u64::from(best_level) + 1000),
            shows_word: true,
            hint: false,
            typed: String::new(),
            shown_for: 0.0,
            state: State::Asking,
            correct: 0,
            answered: 0,
            sfx: Vec::new(),
            touch,
        };
        screen.next_card(memory, now);
        screen
    }

    fn next_card(&mut self, memory: &Memory, now: f64) {
        let previous = self.pair;
        // Weighted like the game: hard and overdue pairs come up more, and
        // never the same pair twice in a row when there is a choice.
        let weight = |p: &Pair| {
            if *p == previous && self.pool.len() > 1 {
                0.0
            } else {
                memory.weight(p, now)
            }
        };
        let index = self.rng.weighted_index(&self.pool, weight);
        self.pair = self.pool[index];
        self.shows_word = self.rng.chance(0.5);
        self.hint = memory.record(&self.pair).is_none();
        self.typed.clear();
        self.shown_for = 0.0;
        self.state = State::Asking;
    }

    fn answer(&self) -> String {
        if self.shows_word {
            self.pair.number.to_owned()
        } else {
            self.pair.word.to_lowercase()
        }
    }

    /// The sound effects triggered since the previous call.
    pub fn take_sfx(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sfx)
    }

    /// Handles this frame's typed `keys`, taps on the card (`taps`) and
    /// whether the player asked to go back.
    pub fn update(
        &mut self,
        frame: &Frame,
        keys: &[Key],
        taps: usize,
        back: bool,
        memory: &mut Memory,
    ) -> PracticeAction {
        if back || frame.pressed(KeyCode::Escape) {
            return PracticeAction::Back;
        }
        let (dt, now) = (frame.dt, frame.now);
        self.shown_for += dt;
        let skip = taps > 0 || frame.any_pressed(&[KeyCode::Space, KeyCode::Enter]);

        match self.state {
            State::Asking => {
                if skip {
                    self.reveal(memory, now);
                    return PracticeAction::Stay;
                }
                for &key in keys {
                    self.type_key(key, memory, now);
                    if self.state != State::Asking {
                        break;
                    }
                }
            }
            State::Correct(seconds) | State::Revealed(seconds) => {
                let left = seconds - dt;
                if left <= 0.0 || skip {
                    self.next_card(memory, now);
                } else if let State::Correct(_) = self.state {
                    self.state = State::Correct(left);
                } else {
                    self.state = State::Revealed(left);
                }
            }
        }
        PracticeAction::Stay
    }

    fn type_key(&mut self, key: Key, memory: &mut Memory, now: f64) {
        let wants_digits = self.shows_word;
        match key {
            Key::Backspace => {
                self.typed.pop();
            }
            // Only the kind of character the answer is made of counts.
            Key::Char(c) if pairs::is_answer_char(c) && c.is_ascii_digit() == wants_digits => {
                self.typed.push(c);
                self.sfx.push(Sfx::Type);
                let answer = self.answer();
                match resolve_input(&self.typed, [answer.as_str()]) {
                    InputOutcome::Hit(_) => {
                        memory.record_answer(self.pair, self.shown_for, self.hint, now);
                        self.correct += 1;
                        self.answered += 1;
                        self.state = State::Correct(CORRECT_SECONDS);
                        self.sfx.push(Sfx::Cast);
                    }
                    InputOutcome::DeadEnd => self.reveal(memory, now),
                    InputOutcome::Pending => {}
                }
            }
            Key::Char(_) => {}
        }
    }

    /// Shows the answer after a wrong answer or giving up.
    fn reveal(&mut self, memory: &mut Memory, now: f64) {
        memory.record_miss(self.pair, now);
        self.answered += 1;
        self.state = State::Revealed(REVEALED_SECONDS);
        self.sfx.push(Sfx::Wrong);
    }

    pub fn draw(&self, time: f32) {
        clear_background(BACKGROUND);
        let cx = ARENA_W / 2.0;
        fonts::draw_centered("Harjoittele", cx, 40.0, 44, GOLD, Style::Heading);
        let score = format!("Oikein {} / {}", self.correct, self.answered);
        let size = fonts::measure(&score, Style::Body, 22);
        fonts::draw(
            &score,
            ARENA_W - size.width - 20.0,
            30.0,
            22,
            WHITE,
            Style::Body,
        );

        let card = Rect::new(90.0, 80.0, ARENA_W - 180.0, 420.0);
        draw_rectangle(card.x, card.y, card.w, card.h, CARD);
        draw_rectangle_lines(
            card.x,
            card.y,
            card.w,
            card.h,
            2.0,
            Color::new(0.4, 0.35, 0.6, 1.0),
        );

        let prompt = if self.shows_word {
            self.pair.word.to_uppercase()
        } else {
            self.pair.number.to_owned()
        };
        let answered = self.state != State::Asking;
        // A word's picture helps imagine it; a number's picture would give
        // its word away, so it waits until the card is answered.
        let show_picture = self.shows_word || answered;
        let prompt_y = if show_picture { 250.0 } else { 200.0 };
        if show_picture {
            draw_picture(self.pair, vec2(cx, 160.0), 110.0, time);
        }
        fonts::draw_centered(&prompt, cx, prompt_y + 20.0, 64, WHITE, Style::Heading);

        let accent = if self.shows_word { SKYBLUE } else { VIOLET };
        match self.state {
            State::Asking => {
                if self.hint {
                    let hint = format!("= {}", self.display_answer());
                    fonts::draw_centered(&hint, cx, prompt_y + 75.0, 28, LIME, Style::Bold);
                }
                let dead_end = !self.typed.is_empty()
                    && resolve_input(&self.typed, [self.answer().as_str()])
                        == InputOutcome::DeadEnd;
                let slot_color = if dead_end { RED } else { accent };
                let text = if self.typed.is_empty() {
                    "·".to_owned()
                } else {
                    self.typed.to_uppercase()
                };
                let slot = Rect::new(cx - 130.0, prompt_y + 110.0, 260.0, 60.0);
                draw_rectangle(
                    slot.x,
                    slot.y,
                    slot.w,
                    slot.h,
                    Color::new(0.0, 0.0, 0.0, 0.5),
                );
                draw_rectangle_lines(slot.x, slot.y, slot.w, slot.h, 3.0, slot_color);
                let c = slot.center();
                fonts::draw_centered(&text, c.x, c.y, 36, WHITE, Style::Bold);
            }
            State::Correct(_) => {
                fonts::draw_centered("Oikein!", cx, prompt_y + 90.0, 40, GREEN, Style::Heading);
                self.draw_pair_line(cx, prompt_y + 145.0, WHITE);
            }
            State::Revealed(_) => {
                fonts::draw_centered(
                    "Oikea vastaus:",
                    cx,
                    prompt_y + 90.0,
                    28,
                    YELLOW,
                    Style::Bold,
                );
                self.draw_pair_line(cx, prompt_y + 140.0, YELLOW);
            }
        }

        let help = if self.touch {
            "Napauta korttia: näytä vastaus     Tauko-nappi: takaisin"
        } else {
            "Välilyönti: näytä vastaus     Esc: takaisin"
        };
        draw_centered_text(
            help,
            cx,
            ARENA_H - 50.0,
            20,
            Color::new(0.6, 0.6, 0.65, 1.0),
        );
    }

    fn display_answer(&self) -> String {
        if self.shows_word {
            self.pair.number.to_owned()
        } else {
            self.pair.word.to_uppercase()
        }
    }

    /// "KEKO = 22", the whole pair.
    fn draw_pair_line(&self, x: f32, y: f32, color: Color) {
        let line = format!("{} = {}", self.pair.word.to_uppercase(), self.pair.number);
        fonts::draw_centered(&line, x, y, 40, color, Style::Heading);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_starts_with_the_single_digit_pairs() {
        let pool = unlocked_pairs(1);
        assert_eq!(pool.len(), 10);
        assert!(pool.iter().all(|p| p.number.len() == 1));
    }

    #[test]
    fn later_levels_practice_more_pairs() {
        assert!(unlocked_pairs(3).len() > unlocked_pairs(2).len());
        assert_eq!(unlocked_pairs(1000).len(), crate::pairs::PAIRS.len());
    }

    const NOW: f64 = 1_800_000_000.0;
    const DT: f32 = 1.0 / 60.0;

    fn frame(dt: f32) -> Frame {
        Frame {
            dt,
            now: NOW,
            ..Frame::default()
        }
    }

    fn pressed(key: KeyCode) -> Frame {
        Frame {
            pressed: vec![key],
            ..frame(DT)
        }
    }

    fn chars(text: &str) -> Vec<Key> {
        text.chars().map(Key::Char).collect()
    }

    fn screen(level: u32, memory: &Memory) -> PracticeScreen {
        PracticeScreen::new(level, memory, false, NOW)
    }

    fn step(screen: &mut PracticeScreen, memory: &mut Memory, frame: &Frame, keys: &[Key]) {
        screen.update(frame, keys, 0, false, memory);
    }

    /// A key that starts no answer of this card, and is the right kind.
    fn wrong_key(screen: &PracticeScreen) -> Key {
        let answer = screen.answer();
        let pool: &str = if screen.shows_word {
            "0123456789"
        } else {
            "hjklmprstvaeiouyäö"
        };
        Key::Char(pool.chars().find(|c| !answer.starts_with(*c)).unwrap())
    }

    #[test]
    fn a_right_answer_is_counted_remembered_and_celebrated() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let (pair, answer) = (practice.pair, practice.answer());
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer));
        assert!(matches!(practice.state, State::Correct(_)));
        assert_eq!((practice.correct, practice.answered), (1, 1));
        let sfx = practice.take_sfx();
        assert!(sfx.contains(&Sfx::Cast));
        assert!(sfx.contains(&Sfx::Type));
        let record = memory.record(&pair).expect("the answer is remembered");
        assert_eq!(record.times_seen, 1);
        assert!(record.difficulty <= 0.5, "an answer never makes it harder");
    }

    #[test]
    fn answering_a_known_pair_at_once_builds_its_streak() {
        let mut memory = Memory::default();
        // Every pair has been seen, so no card comes with a hint.
        for pair in unlocked_pairs(1) {
            memory.record_miss(pair, NOW);
        }
        let mut practice = screen(1, &memory);
        assert!(!practice.hint);
        let (pair, answer) = (practice.pair, practice.answer());
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer));
        assert_eq!(memory.record(&pair).unwrap().streak, 1);
    }

    #[test]
    fn an_answer_read_from_a_hint_does_not_build_a_streak() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        assert!(practice.hint, "a pair never seen comes with its answer");
        let (pair, answer) = (practice.pair, practice.answer());
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer));
        assert_eq!(memory.record(&pair).unwrap().streak, 0);
    }

    #[test]
    fn typing_the_answer_one_key_at_a_time_works_too() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        for key in chars(&practice.answer()) {
            step(&mut practice, &mut memory, &frame(DT), &[key]);
        }
        assert_eq!(practice.correct, 1);
    }

    #[test]
    fn a_wrong_answer_shows_the_right_one_and_is_a_miss() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let pair = practice.pair;
        let wrong = wrong_key(&practice);
        step(&mut practice, &mut memory, &frame(DT), &[wrong]);
        assert!(matches!(practice.state, State::Revealed(_)));
        assert_eq!((practice.correct, practice.answered), (0, 1));
        assert!(practice.take_sfx().contains(&Sfx::Wrong));
        assert!(memory.difficulty(&pair) > 0.5, "a miss makes it harder");
    }

    #[test]
    fn the_wrong_kind_of_character_is_ignored() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let wrong_kind = if practice.shows_word { 'k' } else { '7' };
        step(
            &mut practice,
            &mut memory,
            &frame(DT),
            &[Key::Char(wrong_kind)],
        );
        assert_eq!(practice.state, State::Asking);
        assert!(practice.typed.is_empty());
        assert!(practice.take_sfx().is_empty());
        assert_eq!(memory.records().count(), 0);
    }

    #[test]
    fn backspace_takes_back_the_last_character() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let answer = practice.answer();
        if answer.len() < 2 {
            // A one-digit answer is over at once; nothing to take back.
            return;
        }
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer[..1]));
        assert_eq!(practice.typed.len(), 1);
        step(&mut practice, &mut memory, &frame(DT), &[Key::Backspace]);
        assert!(practice.typed.is_empty());
    }

    #[test]
    fn space_gives_up_on_a_card_and_then_moves_on() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let first = practice.pair;
        practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
        assert!(matches!(practice.state, State::Revealed(_)));
        assert_eq!(practice.answered, 1);
        practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
        assert_eq!(practice.state, State::Asking);
        assert_ne!(practice.pair, first);
    }

    #[test]
    fn a_tap_and_enter_also_skip() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        practice.update(&frame(DT), &[], 1, false, &mut memory);
        assert!(matches!(practice.state, State::Revealed(_)));
        practice.update(&pressed(KeyCode::Enter), &[], 0, false, &mut memory);
        assert_eq!(practice.state, State::Asking);
    }

    #[test]
    fn the_next_card_comes_by_itself_after_a_right_answer() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let answer = practice.answer();
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer));
        step(
            &mut practice,
            &mut memory,
            &frame(CORRECT_SECONDS / 2.0),
            &[],
        );
        assert!(matches!(practice.state, State::Correct(_)));
        step(&mut practice, &mut memory, &frame(CORRECT_SECONDS), &[]);
        assert_eq!(practice.state, State::Asking);
    }

    #[test]
    fn a_shown_answer_stays_longer_than_a_right_one() {
        const { assert!(REVEALED_SECONDS > CORRECT_SECONDS) };
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
        step(
            &mut practice,
            &mut memory,
            &frame(CORRECT_SECONDS + 0.1),
            &[],
        );
        assert!(matches!(practice.state, State::Revealed(_)));
        step(&mut practice, &mut memory, &frame(REVEALED_SECONDS), &[]);
        assert_eq!(practice.state, State::Asking);
    }

    #[test]
    fn a_slower_answer_counts_for_less() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        let pair = practice.pair;
        let answer = practice.answer();
        // Thinking for a long time before answering.
        step(&mut practice, &mut memory, &frame(20.0), &[]);
        step(&mut practice, &mut memory, &frame(DT), &chars(&answer));
        assert_eq!(
            memory.record(&pair).unwrap().streak,
            0,
            "too slow to be a good one"
        );
    }

    #[test]
    fn escape_and_the_back_button_leave() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        assert!(matches!(
            practice.update(&pressed(KeyCode::Escape), &[], 0, false, &mut memory),
            PracticeAction::Back
        ));
        assert!(matches!(
            practice.update(&frame(DT), &[], 0, true, &mut memory),
            PracticeAction::Back
        ));
        assert!(matches!(
            practice.update(&frame(DT), &[], 0, false, &mut memory),
            PracticeAction::Stay
        ));
    }

    #[test]
    fn the_same_pair_never_comes_twice_in_a_row() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        for _ in 0..300 {
            let before = practice.pair;
            practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
            practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
            assert_ne!(practice.pair, before);
        }
    }

    #[test]
    fn only_unlocked_pairs_are_asked() {
        let mut memory = Memory::default();
        let mut practice = screen(1, &memory);
        for _ in 0..100 {
            assert!(practice.pair.number.len() == 1);
            practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
            practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
        }
    }

    #[test]
    fn only_a_pair_never_seen_comes_with_a_hint() {
        let mut memory = Memory::default();
        assert!(screen(1, &memory).hint);
        for pair in unlocked_pairs(1) {
            memory.record_miss(pair, NOW);
        }
        assert!(!screen(1, &memory).hint);
    }

    #[test]
    fn hard_and_overdue_pairs_come_up_more_often() {
        let mut memory = Memory::default();
        // Every pair but one is known well and was just seen.
        let pool = unlocked_pairs(1);
        for pair in &pool[1..] {
            for _ in 0..10 {
                memory.record_answer(*pair, 1.0, false, NOW);
            }
        }
        memory.record_miss(pool[0], NOW);
        let mut practice = screen(1, &memory);
        let mut hard = 0;
        for _ in 0..600 {
            practice.next_card(&memory, NOW);
            if practice.pair == pool[0] {
                hard += 1;
            }
        }
        // Fairly, one in ten; it should come up clearly more than that.
        assert!(hard > 100, "the hard pair came up {hard} times in 600");
    }

    #[test]
    fn practising_is_the_same_every_time() {
        let run = || {
            let mut memory = Memory::default();
            let mut practice = screen(4, &memory);
            let mut seen = Vec::new();
            for _ in 0..30 {
                seen.push((practice.pair.number, practice.shows_word));
                practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
                practice.update(&pressed(KeyCode::Space), &[], 0, false, &mut memory);
            }
            seen
        };
        assert_eq!(run(), run());
    }
}
