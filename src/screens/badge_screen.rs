//! Drawing the badge screen. What it does is `BadgeScreen`, in the core
//! crate.

use macroquad::prelude::*;

use crate::badges::{BADGES, Badge, standing};
use crate::gfx::badge_art::draw_medal;
use crate::gfx::fonts::{self, Style};
use crate::gfx::view::{ARENA_H, ARENA_W};
use crate::progress::Progress;
use lukuloitsu_core::screens::badge_screen::{
    BadgeScreen, LIST_HEIGHT, LIST_TOP, MEDAL_RADIUS, MEDAL_STEP, PANEL_TOP, ROW_HEIGHT,
    back_button, format_day, list_height, max_scroll, medal_center, rows,
};

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const PANEL: Color = Color::new(0.13, 0.13, 0.19, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);

pub fn draw(screen: &BadgeScreen, data: &Progress, hardcore: bool, touch: bool) {
    clear_background(BACKGROUND);
    draw_list(screen, data);
    // The heading and the panel are drawn last, over the list.
    draw_heading(data, touch);
    fonts::draw_mode_tag(hardcore);
    draw_panel(screen, data);
}

/// The medals, a row for each category, and the scrollbar.
fn draw_list(screen: &BadgeScreen, data: &Progress) {
    let rows = rows();
    for (r, row) in rows.iter().enumerate() {
        let top = LIST_TOP + r as f32 * ROW_HEIGHT - screen.scroll();
        fonts::draw(
            BADGES[row[0]].category.title(),
            64.0,
            top + 16.0,
            20,
            GOLD,
            Style::Heading,
        );
        for (c, &index) in row.iter().enumerate() {
            let centre = medal_center(r, c, screen.scroll());
            let badge = &BADGES[index];
            let earned = data.badges.contains_key(badge.id);
            if index == screen.selected() {
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
    draw_scrollbar(screen, rows.len());
}

/// The title, how many badges are earned, and the back button.
fn draw_heading(data: &Progress, touch: bool) {
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
}

fn draw_scrollbar(screen: &BadgeScreen, rows: usize) {
    let max = max_scroll(rows);
    if max <= 0.0 {
        return;
    }
    let track = LIST_HEIGHT - 8.0;
    let thumb = (track * LIST_HEIGHT / list_height(rows)).max(30.0);
    let top = LIST_TOP + 4.0 + (track - thumb) * screen.scroll() / max;
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
fn draw_panel(screen: &BadgeScreen, data: &Progress) {
    draw_rectangle(0.0, PANEL_TOP, ARENA_W, ARENA_H - PANEL_TOP, PANEL);
    draw_line(
        0.0,
        PANEL_TOP,
        ARENA_W,
        PANEL_TOP,
        2.0,
        Color::new(0.3, 0.3, 0.4, 1.0),
    );
    let badge: &Badge = &BADGES[screen.selected()];
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
        None => draw_progress_bar(x, PANEL_TOP + 104.0, badge, data),
    }
}

/// How far the player is from earning `badge`: a bar and the numbers.
fn draw_progress_bar(x: f32, y: f32, badge: &Badge, data: &Progress) {
    let (now, goal) = badge.requirement.progress(&standing(data));
    let (bar_w, bar_h) = (300.0, 14.0);
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
