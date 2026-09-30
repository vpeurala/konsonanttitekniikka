//! The opening screen: the title, a few monsters, a short explanation of
//! the system and every number-word pair, scrollable, so the player can
//! study before starting.

use crate::arena::{ARENA_H, ARENA_W};
use crate::geometry::Rect;
use crate::input::{Frame, KeyCode, Phase, Pointer};
use crate::pairs::{PAIRS, Pair};

/// Pixels scrolled per second while an arrow key is held.
const KEY_SCROLL_SPEED: f32 = 500.0;
pub const FOOTER_HEIGHT: f32 = 44.0;
pub const SIDE_MARGIN: f32 = 60.0;

pub const EXPLANATION_BEFORE_TABLE: &[&str] = &[
    "Lukuloitsussa opit konsonanttitekniikan,",
    "jolla muistat minkä tahansa luvun.",
    "Jokainen numero vastaa yhtä konsonanttia:",
];

pub const EXPLANATION_AFTER_TABLE: &[&str] = &[
    "Vokaalit ovat pelkkää täytettä. Muita kirjaimia ei käytetä.",
    "Näin jokaiselle luvulle löytyy sana, jonka voi kuvitella:",
    "KEKO = K ja K = 2 ja 2 = 22.",
    "",
    "Pelissä hirviöissä on luku tai sana. Kirjoita sen vastine",
    "ennen kuin hirviö saa sinut kiinni! Liiku nuolinäppäimillä.",
];

/// The last thing on the screen: what the game does and doesn't do with
/// the player's information. The page has the details.
pub const PRIVACY_NOTE: &[&str] = &[
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
    Badges,
}

/// Where the menu buttons' centers are, before scrolling.
pub const MENU_Y: f32 = 262.0;
/// The room the menu and the status line under it take.
pub const MENU_SPACE: f32 = 105.0;
pub const BUTTON_W: f32 = 170.0;
pub const BUTTON_H: f32 = 54.0;
const BUTTON_GAP: f32 = 20.0;

/// The menu: each button's label, key and action.
pub const MENU: [(&str, &str, TitleAction); 4] = [
    ("Pelaa", "Enter", TitleAction::StartGame),
    ("Harjoittele", "H", TitleAction::Practice),
    ("Edistyminen", "E", TitleAction::Progress),
    ("Kunniamerkit", "K", TitleAction::Badges),
];

/// Where menu button `i` is, with the content scrolled by `scroll`.
pub fn button_rect(i: usize, scroll: f32) -> Rect {
    let total = MENU.len() as f32 * BUTTON_W + (MENU.len() - 1) as f32 * BUTTON_GAP;
    let x = (ARENA_W - total) / 2.0 + i as f32 * (BUTTON_W + BUTTON_GAP);
    Rect::new(x, MENU_Y - BUTTON_H / 2.0 - scroll, BUTTON_W, BUTTON_H)
}

/// What a key pressed this frame picks from the menu, if any.
fn key_action(frame: &Frame) -> Option<TitleAction> {
    if frame.pressed(KeyCode::Enter) || frame.pressed(KeyCode::Space) {
        Some(TitleAction::StartGame)
    } else if frame.pressed(KeyCode::H) {
        Some(TitleAction::Practice)
    } else if frame.pressed(KeyCode::E) {
        Some(TitleAction::Progress)
    } else if frame.pressed(KeyCode::K) {
        Some(TitleAction::Badges)
    } else {
        None
    }
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
        if let Some(action) = key_action(frame) {
            return action;
        }
        if let Some(action) = self.follow_pointers(pointers) {
            return action;
        }
        self.scroll_by_keys_and_wheel(frame);
        TitleAction::Stay
    }

    /// Dragging scrolls the list; a tap on a menu button picks it.
    fn follow_pointers(&mut self, pointers: &[Pointer]) -> Option<TitleAction> {
        for p in pointers {
            match p.phase {
                Phase::Started if self.drag.is_none() => {
                    self.drag = Some((p.id, p.pos.y, 0.0));
                }
                Phase::Moved | Phase::Stationary => {
                    if let Some((id, last_y, moved)) = &mut self.drag
                        && *id == p.id
                    {
                        self.scroll -= p.pos.y - *last_y;
                        *moved += (p.pos.y - *last_y).abs();
                        *last_y = p.pos.y;
                    }
                }
                Phase::Ended | Phase::Cancelled => {
                    if let Some((id, _, moved)) = self.drag
                        && id == p.id
                    {
                        self.drag = None;
                        if moved < TAP_SLOP && p.phase == Phase::Ended {
                            let tapped = (0..MENU.len())
                                .find(|&i| button_rect(i, self.scroll).contains(p.pos));
                            if let Some(i) = tapped {
                                return Some(MENU[i].2);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The arrow keys, the page keys, Home, End and the wheel scroll the
    /// list, which stays between its top and its end.
    fn scroll_by_keys_and_wheel(&mut self, frame: &Frame) {
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

        self.scroll = self.scroll.clamp(0.0, Self::max_scroll());
    }

    pub fn max_scroll() -> f32 {
        (content_height() - (ARENA_H - FOOTER_HEIGHT)).max(0.0)
    }

    /// How far the content is scrolled up, in pixels.
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// Whether to describe touch controls instead of keys.
    pub fn touch(&self) -> bool {
        self.touch
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

pub const PAIR_ROW_HEIGHT: f32 = 118.0;
/// The size of each word's picture, leaving a gap between cells.
pub const PICTURE_MARGIN: f32 = 10.0;
/// The consonant table's height plus room before the next line.
pub const CONSONANT_TABLE_SPACE: f32 = 90.0;

/// The height of everything that scrolls, matching `draw`.
pub fn content_height() -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, vec2};

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

    fn pointer(phase: Phase, pos: Vec2) -> Pointer {
        Pointer { id: 1, pos, phase }
    }

    fn tap_on(pos: Vec2) -> Vec<Pointer> {
        vec![pointer(Phase::Started, pos), pointer(Phase::Ended, pos)]
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
        assert_eq!(title.update(&pressed(KeyCode::K), &[]), TitleAction::Badges);
        assert_eq!(title.update(&pressed(KeyCode::A), &[]), TitleAction::Stay);
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
            pointer(Phase::Started, button),
            pointer(Phase::Moved, up),
            pointer(Phase::Ended, up),
        ];
        assert_eq!(title.update(&frame(), &pointers), TitleAction::Stay);
        assert!((title.scroll - 100.0).abs() < 1e-3, "{}", title.scroll);
    }

    #[test]
    fn a_wobbly_tap_is_still_a_tap() {
        let mut title = TitleScreen::new(true);
        let center = button_rect(1, 0.0).center();
        let pointers = [
            pointer(Phase::Started, center),
            pointer(Phase::Moved, center + vec2(2.0, 3.0)),
            pointer(Phase::Ended, center + vec2(2.0, 3.0)),
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
        assert_eq!(title.scroll, TitleScreen::max_scroll());
        assert!(TitleScreen::max_scroll() > 0.0, "there is more than fits");
        title.update(&holding(KeyCode::Down, 1.0), &[]);
        assert_eq!(title.scroll, TitleScreen::max_scroll());
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
