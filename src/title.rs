//! The opening screen: the title, a few monsters, a short explanation of
//! the system and every number-word pair, scrollable, so the player can
//! study before starting.

use macroquad::prelude::*;

use crate::fonts::draw_centered_text;
use crate::fonts::{self, Style};
use crate::frame::Frame;
use crate::pairs::{DIGIT_CONSONANTS, PAIRS, Pair};
use crate::pictures::draw_picture;
use crate::save::SaveData;
use crate::sprites::{draw_boss, draw_cyclops, draw_girl, draw_monster};
use crate::touch::Pointer;
use crate::view::{ARENA_H, ARENA_W, View};

/// Pixels scrolled per second while an arrow key is held.
const KEY_SCROLL_SPEED: f32 = 500.0;
const FOOTER_HEIGHT: f32 = 44.0;
const SIDE_MARGIN: f32 = 60.0;

const TITLE_COLOR: Color = GOLD;
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);
const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);

const EXPLANATION_BEFORE_TABLE: &[&str] = &[
    "Lukuloitsussa opit konsonanttitekniikan,",
    "jolla muistat minkä tahansa luvun.",
    "Jokainen numero vastaa yhtä konsonanttia:",
];

const EXPLANATION_AFTER_TABLE: &[&str] = &[
    "Vokaalit ovat pelkkää täytettä. Muita kirjaimia ei käytetä.",
    "Näin jokaiselle luvulle löytyy sana, jonka voi kuvitella:",
    "KEKO = K ja K = 2 ja 2 = 22.",
    "",
    "Pelissä hirviöissä on luku tai sana. Kirjoita sen vastine",
    "ennen kuin hirviö saa sinut kiinni! Liiku nuolinäppäimillä.",
];

/// The last thing on the screen: what the game does and doesn't do with
/// the player's information. The page has the details.
const PRIVACY_NOTE: &[&str] = &[
    "Peli ei käytä evästeitä eikä kysy nimeäsi tai muita tietoja.",
    "Edistymisesi tallentuu vain omalle laitteellesi.",
    "Verkkosivu laskee nimettömästi käyntejä ja pelitapahtumia.",
    "Lisää: lukuloitsu.fi/tietosuoja",
];

/// A finger movement shorter than this is a tap, not a drag.
const TAP_SLOP: f32 = 12.0;

pub struct TitleScreen {
    /// How far the content is scrolled up, in pixels.
    scroll: f32,
    /// The finger (or mouse) dragging the list: its id, where it was last
    /// frame, and how far it has moved in total.
    drag: Option<(u64, f32, f32)>,
    /// Whether to describe touch controls instead of keys.
    touch: bool,
}

/// What the title screen wants to happen next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAction {
    Stay,
    StartGame,
    Practice,
    Progress,
}

/// Where the menu buttons' centers are, before scrolling.
const MENU_Y: f32 = 262.0;
/// The room the menu and the status line under it take.
const MENU_SPACE: f32 = 105.0;
const BUTTON_W: f32 = 200.0;
const BUTTON_H: f32 = 54.0;
const BUTTON_GAP: f32 = 30.0;

/// The menu: each button's label, key and action.
const MENU: [(&str, &str, TitleAction); 3] = [
    ("Pelaa", "Enter", TitleAction::StartGame),
    ("Harjoittele", "H", TitleAction::Practice),
    ("Edistyminen", "E", TitleAction::Progress),
];

/// Where menu button `i` is, with the content scrolled by `scroll`.
fn button_rect(i: usize, scroll: f32) -> Rect {
    let total = 3.0 * BUTTON_W + 2.0 * BUTTON_GAP;
    let x = (ARENA_W - total) / 2.0 + i as f32 * (BUTTON_W + BUTTON_GAP);
    Rect::new(x, MENU_Y - BUTTON_H / 2.0 - scroll, BUTTON_W, BUTTON_H)
}

impl TitleScreen {
    pub fn new(touch: bool) -> Self {
        TitleScreen {
            scroll: 0.0,
            drag: None,
            touch,
        }
    }

    pub fn update(&mut self, frame: &Frame, pointers: &[Pointer]) -> TitleAction {
        if frame.pressed(KeyCode::Enter) || frame.pressed(KeyCode::Space) {
            return TitleAction::StartGame;
        }
        if frame.pressed(KeyCode::H) {
            return TitleAction::Practice;
        }
        if frame.pressed(KeyCode::E) {
            return TitleAction::Progress;
        }
        // Dragging scrolls the list; a tap on a menu button picks it.
        for p in pointers {
            match p.phase {
                TouchPhase::Started if self.drag.is_none() => {
                    self.drag = Some((p.id, p.pos.y, 0.0));
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if let Some((id, last_y, moved)) = &mut self.drag
                        && *id == p.id
                    {
                        self.scroll -= p.pos.y - *last_y;
                        *moved += (p.pos.y - *last_y).abs();
                        *last_y = p.pos.y;
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some((id, _, moved)) = self.drag
                        && id == p.id
                    {
                        self.drag = None;
                        if moved < TAP_SLOP && p.phase == TouchPhase::Ended {
                            for (i, &(_, _, action)) in MENU.iter().enumerate() {
                                if button_rect(i, self.scroll).contains(p.pos) {
                                    return action;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let dt = frame.dt;
        let page = ARENA_H - FOOTER_HEIGHT;
        if frame.down(KeyCode::Down) {
            self.scroll += KEY_SCROLL_SPEED * dt;
        }
        if frame.down(KeyCode::Up) {
            self.scroll -= KEY_SCROLL_SPEED * dt;
        }
        if frame.pressed(KeyCode::PageDown) {
            self.scroll += page * 0.9;
        }
        if frame.pressed(KeyCode::PageUp) {
            self.scroll -= page * 0.9;
        }
        if frame.pressed(KeyCode::Home) {
            self.scroll = 0.0;
        }
        if frame.pressed(KeyCode::End) {
            self.scroll = f32::INFINITY;
        }
        // The wheel reports how far the content should move down.
        self.scroll -= frame.wheel;

        self.scroll = self.scroll.clamp(0.0, self.max_scroll());
        TitleAction::Stay
    }

    fn max_scroll(&self) -> f32 {
        (content_height() - (ARENA_H - FOOTER_HEIGHT)).max(0.0)
    }

    pub fn draw(&self, progress: &SaveData, view: View, time: f32) {
        clear_background(BACKGROUND);
        let cx = ARENA_W / 2.0;
        let mut y = 70.0 - self.scroll;

        draw_logo(cx, y, time, view);
        y += 90.0;
        draw_cast(cx, y, time);
        y += 90.0;
        self.draw_menu(progress);
        y += MENU_SPACE;

        for line in EXPLANATION_BEFORE_TABLE {
            draw_text(line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
            y += 28.0;
        }
        y += 10.0;
        draw_consonant_table(y);
        y += CONSONANT_TABLE_SPACE;
        for line in EXPLANATION_AFTER_TABLE {
            // On touch screens she steers with the joystick instead.
            let line = if self.touch {
                line.replace("nuolinäppäimillä", "ohjaussauvalla")
            } else {
                line.to_string()
            };
            draw_text(&line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
            y += 28.0;
        }

        y += 30.0;
        fonts::draw_centered("Kaikki parit", cx, y, 36, TITLE_COLOR, Style::Heading);
        y += 40.0;
        draw_pair_table(y, time);
        y += PAIR_ROW_HEIGHT * pair_rows().len() as f32;

        y += 30.0;
        fonts::draw_centered("Tietosuoja", cx, y, 28, TITLE_COLOR, Style::Heading);
        y += 40.0;
        for line in PRIVACY_NOTE {
            draw_text(line, SIDE_MARGIN, y, 20.0, DIM);
            y += 28.0;
        }

        self.draw_scrollbar();
        draw_footer(self.touch);
    }

    fn draw_menu(&self, progress: &SaveData) {
        for (i, (label, key, _)) in MENU.iter().enumerate() {
            let rect = button_rect(i, self.scroll);
            let primary = i == 0;
            let (fill, edge) = if primary {
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
            fonts::draw_centered(label, c.x, c.y - 4.0, 28, WHITE, Style::Heading);
            // The keyboard shortcut, for those with a keyboard.
            if !self.touch {
                fonts::draw_centered(key, c.x, c.y + 17.0, 13, DIM, Style::Body);
            }
        }
        let status = format!(
            "Päiviä putkeen: {}     Paras taso: {}",
            progress.streak, progress.best_level
        );
        let y = MENU_Y + BUTTON_H / 2.0 + 26.0 - self.scroll;
        draw_centered_text(&status, ARENA_W / 2.0, y, 20, DIM);
    }

    fn draw_scrollbar(&self) {
        let max = self.max_scroll();
        if max <= 0.0 {
            return;
        }
        let track = ARENA_H - FOOTER_HEIGHT - 20.0;
        let visible = (ARENA_H - FOOTER_HEIGHT) / content_height();
        let thumb = (track * visible).max(30.0);
        let top = 10.0 + (track - thumb) * self.scroll / max;
        let x = ARENA_W - 10.0;
        draw_rectangle(x, 10.0, 4.0, track, Color::new(1.0, 1.0, 1.0, 0.1));
        draw_rectangle(x, top, 4.0, thumb, Color::new(1.0, 1.0, 1.0, 0.4));
    }
}

const LOGO_TEXT: &str = "Lukuloitsu";
const LOGO_SIZE: u16 = 100;
const LOGO_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.45);
const LOGO_OUTLINE: Color = Color::new(0.28, 0.1, 0.03, 1.0);
const LOGO_FILL: Color = Color::new(1.0, 0.5, 0.1, 1.0);
const LOGO_GLOSS: Color = Color::new(1.0, 0.86, 0.3, 1.0);
const LOGO_SHINE: Color = Color::new(1.0, 0.97, 0.8, 1.0);

/// The game's name as a logo: chunky letters with a dark outline and a
/// drop shadow, an orange fill with a glossy golden top, bobbing in a
/// gentle wave. Centered on (cx, y).
fn draw_logo(cx: f32, y: f32, time: f32, view: View) {
    let measure = |text: &str| fonts::measure(text, Style::Heading, LOGO_SIZE);
    let whole = measure(LOGO_TEXT);
    let left = cx - whole.width / 2.0;
    let baseline = y + whole.offset_y / 2.0;

    // Each letter's position, measured from the width of the text before
    // it so the spacing matches the font's.
    let letters: Vec<(String, f32, f32)> = LOGO_TEXT
        .char_indices()
        .enumerate()
        .map(|(i, (byte, ch))| {
            let x = left + measure(&LOGO_TEXT[..byte]).width;
            let bob = (time * 2.5 - i as f32 * 0.35).sin() * 4.0;
            (ch.to_string(), x, baseline + bob)
        })
        .collect();
    let draw_all = |dx: f32, dy: f32, color: Color| {
        for (letter, x, y) in &letters {
            fonts::draw(letter, x + dx, y + dy, LOGO_SIZE, color, Style::Heading);
        }
    };

    draw_all(5.0, 7.0, LOGO_SHADOW);
    // Many copies around each letter, at two distances, make a smooth
    // outline even at large sizes.
    for radius in [3.0, 6.0] {
        for i in 0..32 {
            let a = i as f32 / 32.0 * std::f32::consts::TAU;
            draw_all(a.cos() * radius, a.sin() * radius, LOGO_OUTLINE);
        }
    }
    draw_all(0.0, 0.0, LOGO_FILL);

    // The gloss: the same letters in gold, clipped to a band over the top
    // of the lowercase letters, with a pale shine at its very top.
    let x_height = measure("o").offset_y;
    let top = baseline - whole.offset_y - 8.0;
    let gloss_bottom = baseline - x_height * 0.5;
    with_clip(
        view,
        left - 10.0,
        top,
        whole.width + 20.0,
        gloss_bottom - top,
        || {
            draw_all(0.0, 0.0, LOGO_GLOSS);
        },
    );
    let shine_bottom = baseline - x_height * 0.85;
    with_clip(
        view,
        left - 10.0,
        top,
        whole.width + 20.0,
        shine_bottom - top,
        || {
            draw_all(0.0, 0.0, LOGO_SHINE);
        },
    );
}

/// Runs `draw` with drawing clipped to the given screen rectangle.
fn with_clip(view: View, x: f32, y: f32, w: f32, h: f32, draw: impl FnOnce()) {
    // Clipping works in physical pixels, so convert from virtual units.
    let dpi = macroquad::miniquad::window::dpi_scale();
    let top_left = view.to_screen(vec2(x, y)) * dpi;
    let bottom_right = view.to_screen(vec2(x + w, y + h)) * dpi;
    let size = (bottom_right - top_left).max(Vec2::ZERO);
    let rect = (
        top_left.x as i32,
        top_left.y as i32,
        size.x as i32,
        size.y as i32,
    );
    // Safe as long as nothing else holds the internal GL context, which
    // is true inside ordinary drawing code.
    unsafe { get_internal_gl() }.quad_gl.scissor(Some(rect));
    draw();
    unsafe { get_internal_gl() }.quad_gl.scissor(None);
}

/// The heroine among some of the monsters she will meet.
fn draw_cast(cx: f32, y: f32, time: f32) {
    draw_boss(vec2(cx - 250.0, y), 32.0, time, 0.0);
    draw_monster(vec2(cx - 115.0, y + 10.0), 20.0, time, 1.3);
    draw_girl(vec2(cx, y + 6.0), time, false, None);
    let girl = vec2(cx, y);
    draw_cyclops(vec2(cx + 115.0, y + 10.0), 20.0, time, 2.1, girl);
    draw_cyclops(vec2(cx + 250.0, y + 6.0), 22.0, time, 4.7, girl);
}

/// Each digit above the consonant that stands for it.
fn draw_consonant_table(y: f32) {
    let cell = (ARENA_W - 2.0 * SIDE_MARGIN) / 10.0;
    for (digit, consonant) in DIGIT_CONSONANTS.iter().enumerate() {
        let x = SIDE_MARGIN + cell * (digit as f32 + 0.5);
        draw_rectangle_lines(x - cell / 2.0 + 3.0, y - 4.0, cell - 6.0, 58.0, 1.5, DIM);
        draw_centered_text(&digit.to_string(), x, y + 12.0, 22, WHITE);
        fonts::draw_centered(
            &consonant.to_uppercase().to_string(),
            x,
            y + 38.0,
            24,
            LIME,
            Style::Bold,
        );
    }
}

/// The pairs grouped into rows: the single digits, then one row per
/// first digit of the two-digit numbers, each pair in the column of its
/// last digit.
pub fn pair_rows() -> Vec<Vec<Pair>> {
    let mut rows: Vec<Vec<Pair>> = Vec::new();
    let singles: Vec<Pair> = PAIRS
        .iter()
        .filter(|p| p.number.len() == 1)
        .copied()
        .collect();
    if !singles.is_empty() {
        rows.push(singles);
    }
    for first in '0'..='9' {
        let row: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| p.number.len() == 2 && p.number.starts_with(first))
            .copied()
            .collect();
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

const PAIR_ROW_HEIGHT: f32 = 118.0;
/// The size of each word's picture, leaving a gap between cells.
const PICTURE_MARGIN: f32 = 10.0;
/// The consonant table's height plus room before the next line.
const CONSONANT_TABLE_SPACE: f32 = 90.0;

fn draw_pair_table(y: f32, time: f32) {
    let cell = (ARENA_W - 2.0 * SIDE_MARGIN) / 10.0;
    for (i, row) in pair_rows().iter().enumerate() {
        let row_y = y + i as f32 * PAIR_ROW_HEIGHT;
        if i % 2 == 0 {
            draw_rectangle(
                SIDE_MARGIN,
                row_y - 6.0,
                cell * 10.0,
                PAIR_ROW_HEIGHT - 4.0,
                Color::new(1.0, 1.0, 1.0, 0.04),
            );
        }
        for pair in row {
            let last_digit = pair.number.chars().last().and_then(|c| c.to_digit(10));
            let column = last_digit.unwrap_or(0) as f32;
            let x = SIDE_MARGIN + cell * (column + 0.5);
            let size = cell - PICTURE_MARGIN;
            draw_picture(*pair, vec2(x, row_y + size / 2.0), size, time);
            draw_centered_text(pair.number, x, row_y + size + 14.0, 16, DIM);
            fonts::draw_centered(
                &pair.word.to_uppercase(),
                x,
                row_y + size + 34.0,
                14,
                WHITE,
                Style::Bold,
            );
        }
    }
}

/// The height of everything that scrolls, matching `draw`.
fn content_height() -> f32 {
    let explanation =
        28.0 * (EXPLANATION_BEFORE_TABLE.len() + EXPLANATION_AFTER_TABLE.len()) as f32;
    70.0 + 90.0
        + 90.0
        + MENU_SPACE
        + explanation
        + 10.0
        + CONSONANT_TABLE_SPACE
        + 30.0
        + 40.0
        + PAIR_ROW_HEIGHT * pair_rows().len() as f32
        + 30.0
        + 40.0
        + 28.0 * PRIVACY_NOTE.len() as f32
        + 20.0
}

const AUTHOR: &str = "Ville Peurala";

/// The controls on the left and the author's name in the lower right
/// corner, fixed below the scrolling content.
fn draw_footer(touch: bool) {
    const FONT_SIZE: u16 = 20;
    let top = ARENA_H - FOOTER_HEIGHT;
    draw_rectangle(0.0, top, ARENA_W, FOOTER_HEIGHT, BACKGROUND);
    draw_line(0.0, top, ARENA_W, top, 1.0, DIM);

    let controls = if touch {
        "Vedä: selaa"
    } else {
        "Nuolet tai hiiri: selaa"
    };
    let size = measure_text(controls, None, FONT_SIZE, 1.0);
    let baseline = top + FOOTER_HEIGHT / 2.0 + size.offset_y / 2.0;
    draw_text(controls, 16.0, baseline, FONT_SIZE as f32, GOLD);

    let author = measure_text(AUTHOR, None, FONT_SIZE, 1.0);
    draw_text(
        AUTHOR,
        ARENA_W - author.width - 16.0,
        baseline,
        FONT_SIZE as f32,
        DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pair_is_in_the_table_once() {
        let listed: usize = pair_rows().iter().map(Vec::len).sum();
        assert_eq!(listed, PAIRS.len());
    }

    #[test]
    fn the_explanation_mentions_the_arrow_keys_that_touch_mode_replaces() {
        assert!(
            EXPLANATION_AFTER_TABLE
                .iter()
                .any(|l| l.contains("nuolinäppäimillä"))
        );
    }

    #[test]
    fn the_example_in_the_explanation_is_a_real_pair() {
        assert!(PAIRS.iter().any(|p| p.number == "22" && p.word == "keko"));
    }

    const DT: f32 = 1.0 / 60.0;

    fn frame() -> Frame {
        Frame {
            dt: DT,
            ..Frame::default()
        }
    }

    fn pressed(key: KeyCode) -> Frame {
        Frame {
            pressed: vec![key],
            ..frame()
        }
    }

    fn holding(key: KeyCode, dt: f32) -> Frame {
        Frame {
            down: vec![key],
            dt,
            ..Frame::default()
        }
    }

    fn pointer(phase: TouchPhase, pos: Vec2) -> Pointer {
        Pointer { id: 1, pos, phase }
    }

    fn tap_on(pos: Vec2) -> Vec<Pointer> {
        vec![
            pointer(TouchPhase::Started, pos),
            pointer(TouchPhase::Ended, pos),
        ]
    }

    #[test]
    fn keys_pick_the_menu_actions() {
        let mut title = TitleScreen::new(false);
        assert_eq!(
            title.update(&pressed(KeyCode::Enter), &[]),
            TitleAction::StartGame
        );
        assert_eq!(
            title.update(&pressed(KeyCode::Space), &[]),
            TitleAction::StartGame
        );
        assert_eq!(
            title.update(&pressed(KeyCode::H), &[]),
            TitleAction::Practice
        );
        assert_eq!(
            title.update(&pressed(KeyCode::E), &[]),
            TitleAction::Progress
        );
        assert_eq!(title.update(&pressed(KeyCode::X), &[]), TitleAction::Stay);
    }

    #[test]
    fn tapping_a_button_picks_its_action() {
        for (i, (_, _, action)) in MENU.iter().enumerate() {
            let mut title = TitleScreen::new(true);
            let center = button_rect(i, 0.0).center();
            assert_eq!(title.update(&frame(), &tap_on(center)), *action);
        }
    }

    #[test]
    fn tapping_beside_the_buttons_does_nothing() {
        let mut title = TitleScreen::new(true);
        assert_eq!(
            title.update(&frame(), &tap_on(vec2(5.0, 5.0))),
            TitleAction::Stay
        );
    }

    #[test]
    fn a_tap_on_a_scrolled_button_uses_where_it_is_now() {
        let mut title = TitleScreen::new(true);
        title.scroll = 40.0;
        let moved = button_rect(0, 40.0).center();
        assert_eq!(
            title.update(&frame(), &tap_on(moved)),
            TitleAction::StartGame
        );
        // Where it used to be, another button or nothing is under the finger.
        let mut title = TitleScreen::new(true);
        title.scroll = 300.0;
        let old = button_rect(0, 0.0).center();
        assert_eq!(title.update(&frame(), &tap_on(old)), TitleAction::Stay);
    }

    #[test]
    fn dragging_scrolls_and_does_not_pick_a_button() {
        let mut title = TitleScreen::new(true);
        let button = button_rect(0, 0.0).center();
        let up = button - vec2(0.0, 100.0);
        let pointers = [
            pointer(TouchPhase::Started, button),
            pointer(TouchPhase::Moved, up),
            pointer(TouchPhase::Ended, up),
        ];
        assert_eq!(title.update(&frame(), &pointers), TitleAction::Stay);
        assert!((title.scroll - 100.0).abs() < 1e-3, "{}", title.scroll);
    }

    #[test]
    fn a_wobbly_tap_is_still_a_tap() {
        let mut title = TitleScreen::new(true);
        let center = button_rect(1, 0.0).center();
        let pointers = [
            pointer(TouchPhase::Started, center),
            pointer(TouchPhase::Moved, center + vec2(2.0, 3.0)),
            pointer(TouchPhase::Ended, center + vec2(2.0, 3.0)),
        ];
        assert_eq!(title.update(&frame(), &pointers), TitleAction::Practice);
    }

    #[test]
    fn holding_down_scrolls_at_a_steady_speed_and_up_scrolls_back() {
        let mut title = TitleScreen::new(false);
        title.update(&holding(KeyCode::Down, 0.2), &[]);
        assert!((title.scroll - KEY_SCROLL_SPEED * 0.2).abs() < 1e-3);
        title.update(&holding(KeyCode::Up, 0.2), &[]);
        assert!(title.scroll.abs() < 1e-3);
    }

    #[test]
    fn the_scroll_stays_between_the_top_and_the_end() {
        let mut title = TitleScreen::new(false);
        title.update(&holding(KeyCode::Up, 1.0), &[]);
        assert_eq!(title.scroll, 0.0);
        title.update(&pressed(KeyCode::End), &[]);
        assert_eq!(title.scroll, title.max_scroll());
        assert!(title.max_scroll() > 0.0, "there is more than fits");
        title.update(&holding(KeyCode::Down, 1.0), &[]);
        assert_eq!(title.scroll, title.max_scroll());
        title.update(&pressed(KeyCode::Home), &[]);
        assert_eq!(title.scroll, 0.0);
    }

    #[test]
    fn page_down_and_up_move_by_most_of_a_page() {
        let mut title = TitleScreen::new(false);
        title.update(&pressed(KeyCode::PageDown), &[]);
        let page = ARENA_H - FOOTER_HEIGHT;
        assert!((title.scroll - page * 0.9).abs() < 1e-3);
        title.update(&pressed(KeyCode::PageUp), &[]);
        assert_eq!(title.scroll, 0.0);
    }

    #[test]
    fn the_mouse_wheel_scrolls() {
        let mut title = TitleScreen::new(false);
        title.scroll = 100.0;
        title.update(
            &Frame {
                wheel: 30.0,
                ..frame()
            },
            &[],
        );
        assert_eq!(title.scroll, 70.0);
        title.update(
            &Frame {
                wheel: -50.0,
                ..frame()
            },
            &[],
        );
        assert_eq!(title.scroll, 120.0);
    }

    #[test]
    fn the_buttons_fit_side_by_side_in_the_arena() {
        for i in 0..MENU.len() {
            let r = button_rect(i, 0.0);
            assert!(r.x >= 0.0 && r.right() <= ARENA_W);
        }
        for i in 1..MENU.len() {
            assert!(button_rect(i - 1, 0.0).right() < button_rect(i, 0.0).x);
        }
    }
}
