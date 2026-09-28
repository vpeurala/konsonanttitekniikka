//! Practice mode: calm flash cards without monsters. Each card shows a
//! word or a number, and the player types its counterpart. Answers feed the
//! same spaced-repetition memory as the game.

use macroquad::prelude::*;

use crate::audio::Sfx;
use crate::curriculum::Curriculum;
use crate::fonts::{self, Style};
use crate::game::{InputOutcome, draw_centered_text, resolve_input};
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
    pub fn new(best_level: u32, memory: &Memory, touch: bool) -> Self {
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
        screen.next_card(memory);
        screen
    }

    fn next_card(&mut self, memory: &Memory) {
        let now = macroquad::miniquad::date::now();
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
        keys: &[Key],
        taps: usize,
        back: bool,
        dt: f32,
        memory: &mut Memory,
    ) -> PracticeAction {
        if back || is_key_pressed(KeyCode::Escape) {
            return PracticeAction::Back;
        }
        self.shown_for += dt;
        let skip = taps > 0 || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter);

        match self.state {
            State::Asking => {
                if skip {
                    self.reveal(memory);
                    return PracticeAction::Stay;
                }
                for &key in keys {
                    self.type_key(key, memory);
                    if self.state != State::Asking {
                        break;
                    }
                }
            }
            State::Correct(seconds) | State::Revealed(seconds) => {
                let left = seconds - dt;
                if left <= 0.0 || skip {
                    self.next_card(memory);
                } else if let State::Correct(_) = self.state {
                    self.state = State::Correct(left);
                } else {
                    self.state = State::Revealed(left);
                }
            }
        }
        PracticeAction::Stay
    }

    fn type_key(&mut self, key: Key, memory: &mut Memory) {
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
                        let now = macroquad::miniquad::date::now();
                        memory.record_answer(self.pair, self.shown_for, self.hint, now);
                        self.correct += 1;
                        self.answered += 1;
                        self.state = State::Correct(CORRECT_SECONDS);
                        self.sfx.push(Sfx::Cast);
                    }
                    InputOutcome::DeadEnd => self.reveal(memory),
                    InputOutcome::Pending => {}
                }
            }
            Key::Char(_) => {}
        }
    }

    /// Shows the answer after a wrong answer or giving up.
    fn reveal(&mut self, memory: &mut Memory) {
        memory.record_miss(self.pair, macroquad::miniquad::date::now());
        self.answered += 1;
        self.state = State::Revealed(REVEALED_SECONDS);
        self.sfx.push(Sfx::Wrong);
    }

    pub fn draw(&self) {
        clear_background(BACKGROUND);
        let time = get_time() as f32;
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
            draw_picture(self.pair.number, vec2(cx, 160.0), 110.0, time);
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
}
