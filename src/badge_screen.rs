//! The badge screen: every badge as a medal, a row for each category, the
//! earned ones in colour and the rest dull. The selected one is described
//! at the bottom, with the day it was earned or how far the player is from
//! earning it.

use macroquad::prelude::*;

use crate::badge_art::draw_medal;
use crate::badges::{BADGES, Badge, standing};
use crate::fonts::{self, Style};
use crate::frame::Frame;
use crate::memory::Memory;
use crate::save::SaveData;
use crate::touch::Pointer;
use crate::view::{ARENA_H, ARENA_W};

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const PANEL: Color = Color::new(0.13, 0.13, 0.19, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);

/// The top of the scrolling list, under the heading.
const LIST_TOP: f32 = 88.0;
/// The top of the panel that describes the selected badge.
const PANEL_TOP: f32 = 470.0;
/// How tall the visible part of the list is.
const LIST_HEIGHT: f32 = PANEL_TOP - LIST_TOP;
const ROW_HEIGHT: f32 = 108.0;
const MEDAL_RADIUS: f32 = 28.0;
const MEDAL_STEP: f32 = 84.0;
/// Where the first medal of a row is centred, sideways.
const FIRST_MEDAL_X: f32 = 106.0;
/// A finger movement shorter than this is a tap, not a drag.
const TAP_SLOP: f32 = 12.0;
/// Pixels scrolled by the page keys.
const PAGE: f32 = LIST_HEIGHT * 0.9;

/// The "back" button's place.
fn back_button() -> Rect {
    Rect::new(16.0, 16.0, 120.0, 40.0)
}

/// The badges grouped into rows, by category: each row holds the positions
/// in `BADGES` of its badges.
pub fn rows() -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for (i, badge) in BADGES.iter().enumerate() {
        match rows.last_mut() {
            Some(row) if BADGES[row[0]].category == badge.category => row.push(i),
            _ => rows.push(vec![i]),
        }
    }
    rows
}

/// Where badge `index` sits: its row and its place in the row.
fn place(rows: &[Vec<usize>], index: usize) -> (usize, usize) {
    rows.iter()
        .enumerate()
        .find_map(|(r, row)| row.iter().position(|&i| i == index).map(|c| (r, c)))
        .expect("every badge is in a row")
}

/// The badge reached by moving `dx` places sideways and `dy` rows down
/// from `index`, staying on the map.
pub fn moved(index: usize, dx: i32, dy: i32) -> usize {
    let rows = rows();
    let (row, col) = place(&rows, index);
    let row = (row as i32 + dy).clamp(0, rows.len() as i32 - 1) as usize;
    let col = (col as i32 + dx).clamp(0, rows[row].len() as i32 - 1) as usize;
    rows[row][col]
}

/// Where the medal of the `col`th badge in row `row` is centred, with the
/// list scrolled by `scroll`.
fn medal_center(row: usize, col: usize, scroll: f32) -> Vec2 {
    vec2(
        FIRST_MEDAL_X + col as f32 * MEDAL_STEP,
        LIST_TOP + row as f32 * ROW_HEIGHT + 58.0 - scroll,
    )
}

fn list_height(rows: usize) -> f32 {
    rows as f32 * ROW_HEIGHT + 12.0
}

/// How far the list can scroll.
fn max_scroll(rows: usize) -> f32 {
    (list_height(rows) - LIST_HEIGHT).max(0.0)
}

/// The scroll position closest to `scroll` that shows all of row `row`.
fn scrolled_to_show(row: usize, scroll: f32, rows: usize) -> f32 {
    let top = row as f32 * ROW_HEIGHT;
    let scroll = scroll.min(top).max(top + ROW_HEIGHT - LIST_HEIGHT);
    scroll.clamp(0.0, max_scroll(rows))
}

/// The day number (days since 1970) as a date in the Finnish way, like
/// "30.9.2026".
pub fn format_day(day: i64) -> String {
    // The civil calendar from a day count, after Howard Hinnant.
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let date = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{date}.{month}.{year}")
}

pub struct BadgeScreen {
    /// The position in `BADGES` of the selected badge.
    selected: usize,
    /// How far the list is scrolled up, in pixels.
    scroll: f32,
    /// The finger (or mouse) dragging the list: its id, where it was last
    /// frame, and how far it has moved in total.
    drag: Option<(u64, f32, f32)>,
}

impl Default for BadgeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl BadgeScreen {
    pub fn new() -> Self {
        BadgeScreen {
            selected: 0,
            scroll: 0.0,
            drag: None,
        }
    }

    /// Handles a frame of input. Returns true when the player wants to go
    /// back to the title.
    pub fn update(&mut self, frame: &Frame, pointers: &[Pointer]) -> bool {
        let rows = rows();
        let mut back = frame.pressed(KeyCode::Escape) || frame.pressed(KeyCode::Backspace);

        let step = |key| i32::from(frame.pressed(key));
        let dx = step(KeyCode::Right) - step(KeyCode::Left);
        let dy = step(KeyCode::Down) - step(KeyCode::Up);
        if dx != 0 || dy != 0 {
            self.selected = moved(self.selected, dx, dy);
            let (row, _) = place(&rows, self.selected);
            self.scroll = scrolled_to_show(row, self.scroll, rows.len());
        }
        if frame.pressed(KeyCode::PageDown) {
            self.scroll += PAGE;
        }
        if frame.pressed(KeyCode::PageUp) {
            self.scroll -= PAGE;
        }
        // The wheel reports how far the content should move down.
        self.scroll -= frame.wheel;

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
                            if back_button().contains(p.pos) {
                                back = true;
                            } else if let Some(index) = self.medal_at(&rows, p.pos) {
                                self.selected = index;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.scroll = self.scroll.clamp(0.0, max_scroll(rows.len()));
        back
    }

    /// The badge whose medal is at `pos`, if any.
    fn medal_at(&self, rows: &[Vec<usize>], pos: Vec2) -> Option<usize> {
        if !(LIST_TOP..PANEL_TOP).contains(&pos.y) {
            return None;
        }
        rows.iter().enumerate().find_map(|(r, row)| {
            row.iter().enumerate().find_map(|(c, &index)| {
                let centre = medal_center(r, c, self.scroll);
                // A little generous, for fingers.
                (pos.distance(centre) <= MEDAL_RADIUS + 8.0).then_some(index)
            })
        })
    }

    pub fn draw(&self, data: &SaveData, memory: &Memory, touch: bool) {
        clear_background(BACKGROUND);
        let rows = rows();
        for (r, row) in rows.iter().enumerate() {
            let top = LIST_TOP + r as f32 * ROW_HEIGHT - self.scroll;
            fonts::draw(
                BADGES[row[0]].category.title(),
                64.0,
                top + 16.0,
                20,
                GOLD,
                Style::Heading,
            );
            for (c, &index) in row.iter().enumerate() {
                let centre = medal_center(r, c, self.scroll);
                let badge = &BADGES[index];
                let earned = data.badges.contains_key(badge.id);
                if index == self.selected {
                    draw_rectangle_lines(
                        centre.x - MEDAL_STEP / 2.0 + 4.0,
                        centre.y - MEDAL_RADIUS - 6.0,
                        MEDAL_STEP - 8.0,
                        MEDAL_RADIUS * 2.6 + 8.0,
                        2.0,
                        WHITE,
                    );
                }
                draw_medal(centre, MEDAL_RADIUS, badge, earned, 1.0);
            }
        }
        self.draw_scrollbar(rows.len());

        // The heading and the panel are drawn last, over the list.
        draw_rectangle(0.0, 0.0, ARENA_W, LIST_TOP, BACKGROUND);
        fonts::draw_centered(
            "Kunniamerkit",
            ARENA_W / 2.0,
            34.0,
            40,
            GOLD,
            Style::Heading,
        );
        let earned = BADGES
            .iter()
            .filter(|b| data.badges.contains_key(b.id))
            .count();
        fonts::draw_centered(
            &format!("{earned} / {}", BADGES.len()),
            ARENA_W / 2.0,
            68.0,
            22,
            WHITE,
            Style::Body,
        );
        let back = back_button();
        draw_rectangle(
            back.x,
            back.y,
            back.w,
            back.h,
            Color::new(0.2, 0.18, 0.32, 1.0),
        );
        draw_rectangle_lines(
            back.x,
            back.y,
            back.w,
            back.h,
            2.0,
            Color::new(0.55, 0.45, 0.85, 1.0),
        );
        fonts::draw_centered(
            "Takaisin",
            back.center().x,
            back.center().y,
            22,
            WHITE,
            Style::Heading,
        );
        let hint = if touch {
            "Napauta kunniamerkkiä"
        } else {
            "Nuolet: valitse   Esc: takaisin"
        };
        fonts::draw(hint, 16.0, 78.0, 16, DIM, Style::Body);

        self.draw_panel(data, memory);
    }

    fn draw_scrollbar(&self, rows: usize) {
        let max = max_scroll(rows);
        if max <= 0.0 {
            return;
        }
        let track = LIST_HEIGHT - 8.0;
        let thumb = (track * LIST_HEIGHT / list_height(rows)).max(30.0);
        let top = LIST_TOP + 4.0 + (track - thumb) * self.scroll / max;
        let x = ARENA_W - 10.0;
        draw_rectangle(
            x,
            LIST_TOP + 4.0,
            4.0,
            track,
            Color::new(1.0, 1.0, 1.0, 0.1),
        );
        draw_rectangle(x, top, 4.0, thumb, Color::new(1.0, 1.0, 1.0, 0.4));
    }

    /// Describes the selected badge.
    fn draw_panel(&self, data: &SaveData, memory: &Memory) {
        draw_rectangle(0.0, PANEL_TOP, ARENA_W, ARENA_H - PANEL_TOP, PANEL);
        draw_line(
            0.0,
            PANEL_TOP,
            ARENA_W,
            PANEL_TOP,
            2.0,
            Color::new(0.3, 0.3, 0.4, 1.0),
        );
        let badge: &Badge = &BADGES[self.selected];
        let earned = data.badges.get(badge.id);
        draw_medal(
            vec2(70.0, PANEL_TOP + 50.0),
            32.0,
            badge,
            earned.is_some(),
            1.0,
        );

        let x = 132.0;
        fonts::draw(badge.name, x, PANEL_TOP + 38.0, 30, WHITE, Style::Heading);
        fonts::draw(
            &format!("{}, {}", badge.category.title(), badge.tier.name()),
            x,
            PANEL_TOP + 60.0,
            16,
            DIM,
            Style::Body,
        );
        fonts::draw(
            &badge.requirement.describe(),
            x,
            PANEL_TOP + 88.0,
            22,
            WHITE,
            Style::Body,
        );
        match earned {
            Some(&day) => fonts::draw(
                &format!("Ansaittu {}", format_day(day)),
                x,
                PANEL_TOP + 116.0,
                20,
                Color::new(0.35, 0.85, 0.4, 1.0),
                Style::Bold,
            ),
            None => {
                let (now, goal) = badge.requirement.progress(&standing(data, memory));
                let (bar_w, bar_h) = (300.0, 14.0);
                let y = PANEL_TOP + 104.0;
                draw_rectangle(x, y, bar_w, bar_h, Color::new(0.25, 0.25, 0.32, 1.0));
                draw_rectangle(
                    x,
                    y,
                    bar_w * now as f32 / goal as f32,
                    bar_h,
                    Color::new(0.96, 0.77, 0.2, 1.0),
                );
                fonts::draw(
                    &format!("{now} / {goal}"),
                    x + bar_w + 14.0,
                    y + 13.0,
                    20,
                    WHITE,
                    Style::Bold,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::Category;

    #[test]
    fn every_badge_is_in_exactly_one_row_of_its_own_category() {
        let rows = rows();
        let mut seen: Vec<usize> = rows.iter().flatten().copied().collect();
        assert_eq!(seen.len(), BADGES.len());
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), BADGES.len());
        for row in &rows {
            let category: Category = BADGES[row[0]].category;
            assert!(row.iter().all(|&i| BADGES[i].category == category));
        }
    }

    #[test]
    fn a_row_fits_across_the_screen_beside_the_scrollbar() {
        let longest = rows().iter().map(Vec::len).max().unwrap();
        let last = medal_center(0, longest - 1, 0.0).x + MEDAL_RADIUS;
        assert!(last < ARENA_W - 20.0, "{last}");
        assert!(medal_center(0, 0, 0.0).x - MEDAL_RADIUS > 0.0);
    }

    #[test]
    fn moving_the_selection_stays_on_the_map() {
        assert_eq!(moved(0, -1, 0), 0);
        assert_eq!(moved(0, 0, -1), 0);
        assert_eq!(moved(0, 1, 0), 1);
        let last = BADGES.len() - 1;
        assert_eq!(moved(last, 1, 0), last);
        assert_eq!(moved(last, 0, 1), last);
    }

    #[test]
    fn moving_down_keeps_the_column_or_the_end_of_a_shorter_row() {
        let rows = rows();
        // From the last medal of the longest row into the next row.
        let long = rows.iter().position(|r| r.len() > rows[1].len()).unwrap();
        let from = *rows[long].last().unwrap();
        let below = moved(from, 0, 1);
        assert!(rows[long + 1].contains(&below));
        assert_eq!(below, *rows[long + 1].last().unwrap());
        // And back up again, staying in the row above.
        assert!(rows[long].contains(&moved(below, 0, -1)));
    }

    #[test]
    fn scrolling_to_a_row_shows_all_of_it_and_never_leaves_the_list() {
        let n = rows().len();
        for row in 0..n {
            let scroll = scrolled_to_show(row, 0.0, n);
            let top = row as f32 * ROW_HEIGHT - scroll;
            assert!(
                top >= -0.01 && top + ROW_HEIGHT <= LIST_HEIGHT + 0.01,
                "{row}"
            );
            assert!((0.0..=max_scroll(n)).contains(&scroll));
        }
        // A row already in view doesn't move the list.
        assert_eq!(scrolled_to_show(0, 0.0, n), 0.0);
    }

    #[test]
    fn dates_are_written_the_finnish_way() {
        assert_eq!(format_day(0), "1.1.1970");
        assert_eq!(format_day(19_723), "1.1.2024");
        assert_eq!(format_day(19_782), "29.2.2024");
        assert_eq!(format_day(19_782 + 306), "31.12.2024");
        assert_eq!(format_day(-1), "31.12.1969");
    }

    #[test]
    fn the_back_button_is_clear_of_the_heading() {
        assert!(back_button().right() < ARENA_W / 2.0 - 100.0);
        assert!(back_button().bottom() < LIST_TOP);
    }

    #[test]
    fn a_tap_selects_the_medal_under_it() {
        let screen = BadgeScreen::new();
        let rows = rows();
        let centre = medal_center(1, 2, 0.0);
        assert_eq!(screen.medal_at(&rows, centre), Some(rows[1][2]));
        assert_eq!(screen.medal_at(&rows, vec2(2.0, LIST_TOP + 200.0)), None);
        // Under the panel nothing can be tapped.
        assert_eq!(
            screen.medal_at(&rows, vec2(centre.x, PANEL_TOP + 20.0)),
            None
        );
    }

    #[test]
    fn the_map_lists_the_same_badges_the_table_does() {
        assert_eq!(BADGES.len(), rows().iter().map(Vec::len).sum::<usize>());
    }
}
