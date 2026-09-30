//! Drawing badges: a round medal on a ribbon, in the metal of its tier,
//! with a small picture for its category. Like everything else in the game
//! it is made of shapes drawn by code.

use macroquad::prelude::*;

use crate::badges::{Badge, Category, Tier};
use crate::fonts::{self, Style};

/// What a medal is made of.
struct Metal {
    /// The disc.
    face: Color,
    /// The rim.
    rim: Color,
    /// The picture on the disc.
    ink: Color,
}

const RIBBON: Color = Color::new(0.8, 0.15, 0.22, 1.0);

fn metal(tier: Tier) -> Metal {
    match tier {
        Tier::Bronze => Metal {
            face: Color::new(0.8, 0.5, 0.26, 1.0),
            rim: Color::new(0.55, 0.32, 0.14, 1.0),
            ink: Color::new(0.33, 0.17, 0.06, 1.0),
        },
        Tier::Silver => Metal {
            face: Color::new(0.8, 0.82, 0.87, 1.0),
            rim: Color::new(0.5, 0.52, 0.6, 1.0),
            ink: Color::new(0.27, 0.29, 0.36, 1.0),
        },
        Tier::Gold => Metal {
            face: Color::new(0.97, 0.78, 0.2, 1.0),
            rim: Color::new(0.72, 0.5, 0.08, 1.0),
            ink: Color::new(0.4, 0.26, 0.02, 1.0),
        },
        Tier::Diamond => Metal {
            face: Color::new(0.5, 0.87, 1.0, 1.0),
            rim: Color::new(0.2, 0.5, 0.82, 1.0),
            ink: Color::new(0.06, 0.24, 0.5, 1.0),
        },
    }
}

/// The dull look of a badge not yet earned.
const LOCKED: Metal = Metal {
    face: Color::new(0.17, 0.17, 0.22, 1.0),
    rim: Color::new(0.3, 0.3, 0.38, 1.0),
    ink: Color::new(0.3, 0.3, 0.38, 1.0),
};
const LOCKED_RIBBON: Color = Color::new(0.2, 0.2, 0.26, 1.0);

fn fade(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha)
}

/// Draws `badge` as a medal of the given `radius` centred on `centre`, with
/// its ribbon hanging below. A badge not `earned` is dull. `alpha` fades
/// the whole thing, from 0 to 1.
pub fn draw_medal(centre: Vec2, radius: f32, badge: &Badge, earned: bool, alpha: f32) {
    let metal = if earned { metal(badge.tier) } else { LOCKED };
    let ribbon = fade(if earned { RIBBON } else { LOCKED_RIBBON }, alpha);
    let at = |dx: f32, dy: f32| centre + vec2(dx, dy) * radius;
    // The ribbon: two tails with a notch at the bottom.
    for side in [-1.0, 1.0] {
        let (a, b) = (at(side * 0.7, 0.3), at(side * 0.15, 0.6));
        let (c, d, e) = (
            at(side * 0.1, 1.6),
            at(side * 0.42, 1.3),
            at(side * 0.74, 1.6),
        );
        draw_triangle(a, b, c, ribbon);
        draw_triangle(a, c, d, ribbon);
        draw_triangle(a, d, e, ribbon);
    }
    draw_circle(centre.x, centre.y, radius, fade(metal.rim, alpha));
    draw_circle(centre.x, centre.y, radius * 0.86, fade(metal.face, alpha));
    if earned {
        // A little shine.
        let shine = at(-0.35, -0.4);
        draw_circle(shine.x, shine.y, radius * 0.22, fade(WHITE, 0.3 * alpha));
    }
    draw_emblem(badge.category, centre, radius, fade(metal.ink, alpha));
}

/// A star of `radius` in one colour.
fn draw_solid_star(pos: Vec2, radius: f32, color: Color) {
    let point = |i: usize| {
        let r = if i.is_multiple_of(2) {
            radius
        } else {
            radius * 0.45
        };
        let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        pos + Vec2::from_angle(angle) * r
    };
    for i in 0..10 {
        draw_triangle(pos, point(i), point(i + 1), color);
    }
}

/// A four-cornered shape, drawn as two triangles.
fn draw_quad(a: Vec2, b: Vec2, c: Vec2, d: Vec2, color: Color) {
    draw_triangle(a, b, c, color);
    draw_triangle(a, c, d, color);
}

/// The picture for a category, on a medal of `radius`.
fn draw_emblem(category: Category, c: Vec2, radius: f32, ink: Color) {
    let r = radius;
    let at = |dx: f32, dy: f32| c + vec2(dx, dy) * r;
    match category {
        // A one-eyed monster.
        Category::Monsters => {
            draw_circle(c.x, c.y, r * 0.44, ink);
            let eye = at(0.0, -0.08);
            draw_circle(eye.x, eye.y, r * 0.22, WHITE);
            draw_circle(eye.x, eye.y, r * 0.1, ink);
            for side in [-1.0, 1.0] {
                draw_triangle(
                    at(side * 0.28, 0.14),
                    at(side * 0.1, 0.14),
                    at(side * 0.19, 0.33),
                    WHITE,
                );
            }
        }
        // A crown.
        Category::Bosses => {
            draw_rectangle(c.x - 0.4 * r, c.y + 0.08 * r, 0.8 * r, 0.24 * r, ink);
            draw_triangle(at(-0.4, 0.08), at(-0.4, -0.34), at(-0.1, 0.08), ink);
            draw_triangle(at(-0.22, 0.08), at(0.0, -0.44), at(0.22, 0.08), ink);
            draw_triangle(at(0.4, 0.08), at(0.4, -0.34), at(0.1, 0.08), ink);
        }
        // A flag on a pole.
        Category::Levels => {
            draw_line(
                c.x - 0.28 * r,
                c.y + 0.42 * r,
                c.x - 0.28 * r,
                c.y - 0.42 * r,
                0.1 * r,
                ink,
            );
            draw_triangle(at(-0.28, -0.42), at(0.42, -0.2), at(-0.28, 0.02), ink);
        }
        // An open book.
        Category::Pairs => {
            for side in [-1.0, 1.0] {
                draw_quad(
                    at(side * 0.03, -0.2),
                    at(side * 0.46, -0.34),
                    at(side * 0.46, 0.3),
                    at(side * 0.03, 0.42),
                    ink,
                );
            }
        }
        // Two linked rings.
        Category::Combo => {
            for side in [-1.0, 1.0] {
                let ring = at(side * 0.2, 0.0);
                draw_circle_lines(ring.x, ring.y, r * 0.27, 0.1 * r, ink);
            }
        }
        // A star.
        Category::Skill => draw_solid_star(at(0.0, 0.03), r * 0.52, ink),
        // Digits.
        Category::LongNumbers => {
            fonts::draw_centered("0123", c.x, c.y, (r * 0.55) as u16, ink, Style::Bold);
        }
        // A flash card with a question on it.
        Category::Practice => {
            let (w, h) = (0.62 * r, 0.82 * r);
            draw_rectangle(c.x - w / 2.0, c.y - h / 2.0, w, h, WHITE);
            draw_rectangle_lines(c.x - w / 2.0, c.y - h / 2.0, w, h, 0.09 * r, ink);
            fonts::draw_centered(
                "?",
                c.x,
                c.y + 0.02 * r,
                (r * 0.62) as u16,
                ink,
                Style::Bold,
            );
        }
        // A sun.
        Category::Days => {
            draw_circle(c.x, c.y, r * 0.24, ink);
            for i in 0..8 {
                let dir = Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4);
                let (from, to) = (c + dir * r * 0.36, c + dir * r * 0.58);
                draw_line(from.x, from.y, to.x, to.y, 0.08 * r, ink);
            }
        }
        // A lightning bolt.
        Category::Speed => {
            draw_triangle(at(0.14, -0.56), at(-0.32, 0.1), at(0.06, 0.1), ink);
            draw_triangle(at(-0.14, 0.56), at(0.32, -0.1), at(-0.06, -0.1), ink);
            draw_quad(
                at(0.14, -0.56),
                at(0.06, 0.1),
                at(-0.06, -0.1),
                at(0.06, -0.1),
                ink,
            );
        }
        // A gem.
        Category::Legend => {
            draw_quad(
                at(-0.2, -0.4),
                at(0.2, -0.4),
                at(0.38, -0.14),
                at(-0.38, -0.14),
                ink,
            );
            draw_triangle(at(-0.38, -0.08), at(0.38, -0.08), at(0.0, 0.46), ink);
        }
    }
}
