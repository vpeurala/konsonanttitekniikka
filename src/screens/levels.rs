//! Checkpoints: a game can start from every fifth level she has reached,
//! so later levels, like those with long numbers, are within reach of a
//! single sitting.

use macroquad::prelude::*;

use lukuloitsu_core::geometry::Rect;
use lukuloitsu_core::input::{Frame, KeyCode, Phase, Pointer};

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::view::{ARENA_H, ARENA_W};
use crate::long_numbers::first_long_level;
use crate::screens::practice::unlocked_pairs;
use lukuloitsu_core::levels::checkpoints;

const COLUMNS: usize = 5;

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);
const BUTTON_W: f32 = 132.0;
const BUTTON_H: f32 = 70.0;
const GAP: f32 = 14.0;
const GRID_TOP: f32 = 130.0;

pub enum LevelAction {
    Stay,
    Start(u32),
    Back,
}

pub struct LevelSelect {
    levels: Vec<u32>,
    /// A line under each level saying what it holds.
    captions: Vec<String>,
    selected: usize,
    touch: bool,
}

impl LevelSelect {
    pub fn new(best_level: u32, touch: bool) -> Self {
        let levels = checkpoints(best_level);
        let long = first_long_level();
        let captions = levels
            .iter()
            .map(|&level| {
                if level == 1 {
                    "Alusta".to_owned()
                } else if level + 1 >= long {
                    "Pitkiä lukuja".to_owned()
                } else {
                    format!("{} paria", unlocked_pairs(level).len())
                }
            })
            .collect();
        LevelSelect {
            // The furthest checkpoint is the likeliest choice.
            selected: levels.len() - 1,
            levels,
            captions,
            touch,
        }
    }

    /// How many choices there are; with one, there is nothing to pick.
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    fn button_rect(&self, i: usize) -> Rect {
        let columns = COLUMNS.min(self.levels.len());
        let row_width = columns as f32 * BUTTON_W + (columns - 1) as f32 * GAP;
        let (row, column) = (i / COLUMNS, i % COLUMNS);
        Rect::new(
            (ARENA_W - row_width) / 2.0 + column as f32 * (BUTTON_W + GAP),
            GRID_TOP + row as f32 * (BUTTON_H + GAP),
            BUTTON_W,
            BUTTON_H,
        )
    }

    pub fn update(&mut self, frame: &Frame, pointers: &[Pointer]) -> LevelAction {
        if frame.pressed(KeyCode::Escape) || frame.pressed(KeyCode::Backspace) {
            return LevelAction::Back;
        }
        if frame.pressed(KeyCode::Enter) || frame.pressed(KeyCode::Space) {
            return LevelAction::Start(self.levels[self.selected]);
        }
        let last = self.levels.len() - 1;
        if frame.pressed(KeyCode::Left) {
            self.selected = self.selected.saturating_sub(1);
        }
        if frame.pressed(KeyCode::Right) {
            self.selected = (self.selected + 1).min(last);
        }
        if frame.pressed(KeyCode::Up) {
            self.selected = self.selected.saturating_sub(COLUMNS);
        }
        if frame.pressed(KeyCode::Down) && self.selected + COLUMNS <= last {
            self.selected += COLUMNS;
        }
        let Some(tap) = pointers.iter().find(|p| p.phase == Phase::Ended) else {
            return LevelAction::Stay;
        };
        match (0..self.levels.len()).find(|&i| self.button_rect(i).contains(tap.pos)) {
            Some(i) => LevelAction::Start(self.levels[i]),
            // A tap beside the buttons goes back.
            None => LevelAction::Back,
        }
    }

    pub fn draw(&self) {
        clear_background(BACKGROUND);
        let cx = ARENA_W / 2.0;
        fonts::draw_centered("Valitse taso", cx, 50.0, 44, GOLD, Style::Heading);
        draw_centered_text(
            "Aloita alusta tai tasolta, jolle olet jo päässyt",
            cx,
            95.0,
            20,
            DIM,
        );
        for (i, (level, caption)) in self.levels.iter().zip(&self.captions).enumerate() {
            let rect = self.button_rect(i);
            let chosen = i == self.selected && !self.touch;
            let (fill, edge) = if chosen {
                (Color::new(0.85, 0.45, 0.1, 1.0), GOLD)
            } else {
                (
                    Color::new(0.2, 0.18, 0.32, 1.0),
                    Color::new(0.55, 0.45, 0.85, 1.0),
                )
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, edge);
            let c = rect.center();
            let title = format!("Taso {level}");
            fonts::draw_centered(&title, c.x, c.y - 9.0, 26, WHITE, Style::Heading);
            let caption_color = if chosen { WHITE } else { DIM };
            fonts::draw_centered(caption, c.x, c.y + 18.0, 15, caption_color, Style::Body);
        }
        let help = if self.touch {
            "Napauta tasoa aloittaaksesi"
        } else {
            "Nuolet: valitse     Enter: aloita     Esc: takaisin"
        };
        draw_centered_text(help, cx, ARENA_H - 30.0, 20, DIM);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> Frame {
        Frame::default()
    }

    fn pressed(key: KeyCode) -> Frame {
        Frame {
            pressed: vec![key],
            ..frame()
        }
    }

    fn tap_on(pos: Vec2) -> Vec<Pointer> {
        vec![Pointer {
            id: 1,
            pos,
            phase: Phase::Ended,
        }]
    }

    /// A player who has reached level 30, with six levels to start from.
    fn select() -> LevelSelect {
        LevelSelect::new(30, false)
    }

    #[test]
    fn a_new_player_has_nothing_to_choose() {
        assert_eq!(LevelSelect::new(1, false).len(), 1);
        assert_eq!(select().len(), 6);
    }

    #[test]
    fn the_furthest_level_is_chosen_at_first() {
        let mut levels = select();
        assert!(matches!(
            levels.update(&pressed(KeyCode::Enter), &[]),
            LevelAction::Start(26)
        ));
    }

    #[test]
    fn arrow_keys_move_the_choice_within_the_list() {
        let mut levels = select();
        levels.update(&pressed(KeyCode::Right), &[]);
        assert_eq!(levels.selected, 5, "already at the end");
        levels.update(&pressed(KeyCode::Left), &[]);
        assert_eq!(levels.selected, 4);
        levels.update(&pressed(KeyCode::Up), &[]);
        assert_eq!(levels.selected, 0, "up a row, stopping at the first");
        levels.update(&pressed(KeyCode::Left), &[]);
        assert_eq!(levels.selected, 0);
        levels.update(&pressed(KeyCode::Down), &[]);
        assert_eq!(levels.selected, 5, "down a row");
        levels.update(&pressed(KeyCode::Down), &[]);
        assert_eq!(levels.selected, 5, "there is no row below");
    }

    #[test]
    fn down_does_not_jump_to_a_button_that_is_not_there() {
        // Seven levels: the second row has one button.
        let mut levels = LevelSelect::new(31, false);
        assert_eq!(levels.len(), 7);
        levels.selected = 3;
        levels.update(&pressed(KeyCode::Down), &[]);
        assert_eq!(levels.selected, 3, "no button below the fourth");
    }

    #[test]
    fn enter_and_space_start_the_chosen_level() {
        for key in [KeyCode::Enter, KeyCode::Space] {
            let mut levels = select();
            levels.update(&pressed(KeyCode::Left), &[]);
            assert!(matches!(
                levels.update(&pressed(key), &[]),
                LevelAction::Start(21)
            ));
        }
    }

    #[test]
    fn escape_and_backspace_go_back() {
        for key in [KeyCode::Escape, KeyCode::Backspace] {
            assert!(matches!(
                select().update(&pressed(key), &[]),
                LevelAction::Back
            ));
        }
    }

    #[test]
    fn nothing_happens_without_input() {
        assert!(matches!(select().update(&frame(), &[]), LevelAction::Stay));
    }

    #[test]
    fn tapping_a_level_starts_it() {
        let levels = select();
        for (i, level) in levels.levels.clone().into_iter().enumerate() {
            let center = levels.button_rect(i).center();
            assert!(matches!(
                select().update(&frame(), &tap_on(center)),
                LevelAction::Start(l) if l == level
            ));
        }
    }

    #[test]
    fn tapping_beside_the_buttons_goes_back() {
        assert!(matches!(
            select().update(&frame(), &tap_on(vec2(2.0, 2.0))),
            LevelAction::Back
        ));
    }

    #[test]
    fn a_finger_only_touching_down_does_nothing_yet() {
        let started = [Pointer {
            id: 1,
            pos: vec2(2.0, 2.0),
            phase: Phase::Started,
        }];
        assert!(matches!(
            select().update(&frame(), &started),
            LevelAction::Stay
        ));
    }

    #[test]
    fn the_buttons_fit_the_arena_and_do_not_overlap() {
        for best in [6, 30, 500] {
            let levels = LevelSelect::new(best, false);
            for i in 0..levels.len() {
                let r = levels.button_rect(i);
                assert!(r.x >= 0.0 && r.right() <= ARENA_W, "{best}: {i}");
                assert!(r.bottom() <= ARENA_H, "{best}: {i}");
                for j in i + 1..levels.len() {
                    let o = levels.button_rect(j);
                    let apart = r.right() <= o.x
                        || o.right() <= r.x
                        || r.bottom() <= o.y
                        || o.bottom() <= r.y;
                    assert!(apart, "{best}: {i} and {j}");
                }
            }
        }
    }

    #[test]
    fn the_first_level_is_captioned_as_the_beginning() {
        assert_eq!(select().captions[0], "Alusta");
    }
}
