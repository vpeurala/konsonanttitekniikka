//! Drawing badges: a round medal on a ribbon, in the metal of its tier,
//! with a little picture of its own. Like everything else in the game it is
//! made of shapes drawn by code.

use macroquad::prelude::*;

use crate::badges::{Badge, Tier};
use crate::fonts::{self, Style};

/// What a medal is made of.
struct Metal {
    /// The disc.
    face: Color,
    /// The rim.
    rim: Color,
    /// The picture on the disc, its main colour...
    ink: Color,
    /// ...and its second colour, for the details.
    light: Color,
}

const RIBBON: Color = Color::new(0.8, 0.15, 0.22, 1.0);

fn metal(tier: Tier) -> Metal {
    match tier {
        Tier::Bronze => Metal {
            face: Color::new(0.8, 0.5, 0.26, 1.0),
            rim: Color::new(0.55, 0.32, 0.14, 1.0),
            ink: Color::new(0.33, 0.17, 0.06, 1.0),
            light: Color::new(1.0, 0.9, 0.75, 1.0),
        },
        Tier::Silver => Metal {
            face: Color::new(0.8, 0.82, 0.87, 1.0),
            rim: Color::new(0.5, 0.52, 0.6, 1.0),
            ink: Color::new(0.27, 0.29, 0.36, 1.0),
            light: Color::new(1.0, 1.0, 1.0, 1.0),
        },
        Tier::Gold => Metal {
            face: Color::new(0.97, 0.78, 0.2, 1.0),
            rim: Color::new(0.72, 0.5, 0.08, 1.0),
            ink: Color::new(0.4, 0.26, 0.02, 1.0),
            light: Color::new(1.0, 0.96, 0.75, 1.0),
        },
        Tier::Diamond => Metal {
            face: Color::new(0.5, 0.87, 1.0, 1.0),
            rim: Color::new(0.2, 0.5, 0.82, 1.0),
            ink: Color::new(0.06, 0.24, 0.5, 1.0),
            light: Color::new(1.0, 1.0, 1.0, 1.0),
        },
    }
}

/// The dull look of a badge not yet earned.
const LOCKED: Metal = Metal {
    face: Color::new(0.17, 0.17, 0.22, 1.0),
    rim: Color::new(0.3, 0.3, 0.38, 1.0),
    ink: Color::new(0.3, 0.3, 0.38, 1.0),
    light: Color::new(0.42, 0.42, 0.5, 1.0),
};
const LOCKED_RIBBON: Color = Color::new(0.2, 0.2, 0.26, 1.0);

fn fade(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha)
}

/// The colours a picture is drawn in.
#[derive(Clone, Copy)]
struct Paint {
    /// The main colour.
    dark: Color,
    /// The colour of the details.
    light: Color,
    /// The colour of the disc, for cutting shapes out of the picture.
    face: Color,
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
    let paint = Paint {
        dark: fade(metal.ink, alpha),
        light: fade(metal.light, alpha),
        face: fade(metal.face, alpha),
    };
    draw_emblem(emblem_of(badge.id), centre, radius, paint);
    if earned {
        // A little shine.
        let shine = at(-0.35, -0.4);
        draw_circle(shine.x, shine.y, radius * 0.16, fade(WHITE, 0.3 * alpha));
    }
}

/// The picture on a badge. Every badge has one of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emblem {
    // Monsters
    Cyclops,
    Wand,
    Shield,
    Sword,
    Trophy,
    Ghost,
    Burst,
    Crescent,
    // Bosses
    Crown,
    Hammer,
    Helmet,
    // Levels
    Tent,
    Compass,
    Map,
    Footprints,
    Mountain,
    Castle,
    // Pairs
    Dots,
    Quarter,
    Half,
    ThreeQuarters,
    Book,
    // Combos
    Chain,
    Flame,
    Rocket,
    Comet,
    // Skill
    Star,
    Check,
    ThreeStars,
    Constellation,
    // Long numbers
    Ruler,
    Scroll,
    WitchHat,
    // Practice
    Card,
    Pencil,
    Dumbbell,
    GraduationCap,
    // Days
    Heart,
    Calendar,
    Rainbow,
    Flower,
    Moon,
    Sun,
    // Speed
    Stopwatch,
    Bolt,
    // Legends
    CrystalBall,
    Gem,
}

/// The picture of the badge with this id.
pub fn emblem_of(id: &str) -> Emblem {
    match id {
        "monsters-1" => Emblem::Cyclops,
        "monsters-10" => Emblem::Wand,
        "monsters-20" => Emblem::Shield,
        "monsters-50" => Emblem::Sword,
        "monsters-100" => Emblem::Trophy,
        "monsters-500" => Emblem::Ghost,
        "monsters-1000" => Emblem::Burst,
        "monsters-5000" => Emblem::Crescent,
        "bosses-1" => Emblem::Crown,
        "bosses-10" => Emblem::Hammer,
        "bosses-50" => Emblem::Helmet,
        "levels-5" => Emblem::Tent,
        "levels-10" => Emblem::Compass,
        "levels-21" => Emblem::Map,
        "levels-30" => Emblem::Footprints,
        "levels-50" => Emblem::Mountain,
        "levels-100" => Emblem::Castle,
        "pairs-digits" => Emblem::Dots,
        "pairs-25" => Emblem::Quarter,
        "pairs-50" => Emblem::Half,
        "pairs-75" => Emblem::ThreeQuarters,
        "pairs-110" => Emblem::Book,
        "combo-10" => Emblem::Chain,
        "combo-25" => Emblem::Flame,
        "combo-50" => Emblem::Rocket,
        "combo-100" => Emblem::Comet,
        "skill-star" => Emblem::Star,
        "skill-flawless" => Emblem::Check,
        "skill-stars-10" => Emblem::ThreeStars,
        "skill-stars-25" => Emblem::Constellation,
        "long-1" => Emblem::Ruler,
        "long-25" => Emblem::Scroll,
        "long-100" => Emblem::WitchHat,
        "practice-1" => Emblem::Card,
        "practice-50" => Emblem::Pencil,
        "practice-250" => Emblem::Dumbbell,
        "practice-1000" => Emblem::GraduationCap,
        "days-2" => Emblem::Heart,
        "days-3" => Emblem::Calendar,
        "days-7" => Emblem::Rainbow,
        "days-14" => Emblem::Flower,
        "days-30" => Emblem::Moon,
        "days-100" => Emblem::Sun,
        "speed-50" => Emblem::Stopwatch,
        "speed-500" => Emblem::Bolt,
        "master" => Emblem::CrystalBall,
        "everything" => Emblem::Gem,
        _ => panic!("badge {id} has no picture"),
    }
}

/// A star with `points` points, `outer` from the centre at the tips and
/// `inner` at the dents, the first tip pointing up.
fn draw_spiky(pos: Vec2, points: usize, outer: f32, inner: f32, color: Color) {
    let corner = |i: usize| {
        let r = if i.is_multiple_of(2) { outer } else { inner };
        let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / points as f32;
        pos + Vec2::from_angle(angle) * r
    };
    for i in 0..points * 2 {
        draw_triangle(pos, corner(i), corner(i + 1), color);
    }
}

/// A four-cornered shape, drawn as two triangles.
fn draw_quad(a: Vec2, b: Vec2, c: Vec2, d: Vec2, color: Color) {
    draw_triangle(a, b, c, color);
    draw_triangle(a, c, d, color);
}

/// A line with rounded ends.
fn draw_stroke(from: Vec2, to: Vec2, width: f32, color: Color) {
    draw_line(from.x, from.y, to.x, to.y, width, color);
    draw_circle(from.x, from.y, width / 2.0, color);
    draw_circle(to.x, to.y, width / 2.0, color);
}

/// A pie: the part of a disc of `radius` swept clockwise from the top
/// through `fraction` of a full turn.
fn draw_pie(pos: Vec2, radius: f32, fraction: f32, color: Color) {
    let steps = 24;
    let angle = |i: usize| {
        -std::f32::consts::FRAC_PI_2 + fraction * std::f32::consts::TAU * i as f32 / steps as f32
    };
    for i in 0..steps {
        let (a, b) = (
            pos + Vec2::from_angle(angle(i)) * radius,
            pos + Vec2::from_angle(angle(i + 1)) * radius,
        );
        draw_triangle(pos, a, b, color);
    }
}

/// Draws `emblem` on a medal of `radius` centred on `c`.
fn draw_emblem(emblem: Emblem, c: Vec2, radius: f32, paint: Paint) {
    let r = radius;
    let Paint { dark, light, face } = paint;
    // A point, in medal radii from the centre.
    let at = |dx: f32, dy: f32| c + vec2(dx, dy) * r;
    // A line of a width given in radii.
    let line = |a: Vec2, b: Vec2, w: f32, color: Color| draw_line(a.x, a.y, b.x, b.y, w * r, color);
    let stroke = |a: Vec2, b: Vec2, w: f32, color: Color| draw_stroke(a, b, w * r, color);
    let disc = |p: Vec2, size: f32, color: Color| draw_circle(p.x, p.y, size * r, color);
    let ring = |p: Vec2, size: f32, w: f32, color: Color| {
        draw_circle_lines(p.x, p.y, size * r, w * r, color);
    };
    let rect = |x: f32, y: f32, w: f32, h: f32, color: Color| {
        let p = at(x, y);
        draw_rectangle(p.x, p.y, w * r, h * r, color);
    };
    let oval = |p: Vec2, w: f32, h: f32, color: Color| {
        draw_ellipse(p.x, p.y, w * r, h * r, 0.0, color);
    };
    match emblem {
        Emblem::Cyclops => {
            disc(at(0.0, 0.0), 0.44, dark);
            disc(at(0.0, -0.08), 0.22, light);
            disc(at(0.0, -0.08), 0.1, dark);
            for side in [-1.0, 1.0] {
                draw_triangle(
                    at(side * 0.28, 0.14),
                    at(side * 0.1, 0.14),
                    at(side * 0.19, 0.33),
                    light,
                );
            }
        }
        // A magic wand with a star at its tip.
        Emblem::Wand => {
            line(at(-0.4, 0.42), at(0.16, -0.14), 0.12, dark);
            line(at(-0.4, 0.42), at(-0.28, 0.3), 0.12, light);
            draw_solid_star(at(0.26, -0.26), r * 0.3, dark);
            disc(at(-0.1, -0.36), 0.05, dark);
            disc(at(0.4, 0.1), 0.05, dark);
        }
        Emblem::Shield => {
            draw_quad(
                at(-0.38, -0.4),
                at(0.38, -0.4),
                at(0.38, 0.05),
                at(-0.38, 0.05),
                dark,
            );
            draw_triangle(at(-0.38, 0.05), at(0.38, 0.05), at(0.0, 0.5), dark);
            line(at(0.0, -0.32), at(0.0, 0.38), 0.09, light);
            line(at(-0.3, -0.12), at(0.3, -0.12), 0.09, light);
        }
        Emblem::Sword => {
            let (tip, guard) = (at(0.4, -0.42), at(-0.14, 0.12));
            line(tip, guard, 0.15, dark);
            line(at(0.32, -0.34), at(-0.06, 0.04), 0.04, light);
            let perp = vec2(0.6, 0.6);
            stroke(guard - perp * r * 0.32, guard + perp * r * 0.32, 0.11, dark);
            line(guard, at(-0.34, 0.32), 0.1, dark);
            disc(at(-0.38, 0.36), 0.09, dark);
        }
        Emblem::Trophy => {
            draw_quad(
                at(-0.32, -0.36),
                at(0.32, -0.36),
                at(0.2, 0.06),
                at(-0.2, 0.06),
                dark,
            );
            for side in [-1.0, 1.0] {
                ring(at(side * 0.36, -0.2), 0.12, 0.06, dark);
            }
            rect(-0.05, 0.05, 0.1, 0.22, dark);
            rect(-0.26, 0.26, 0.52, 0.1, dark);
            draw_solid_star(at(0.0, -0.16), r * 0.12, light);
        }
        Emblem::Ghost => {
            disc(at(0.0, -0.08), 0.34, dark);
            rect(-0.34, -0.08, 0.68, 0.4, dark);
            for i in 0..3 {
                let x = -0.34 + i as f32 * 0.2267;
                draw_triangle(at(x, 0.3), at(x + 0.2267, 0.3), at(x + 0.1133, 0.48), dark);
            }
            for side in [-1.0, 1.0] {
                disc(at(side * 0.13, -0.12), 0.08, light);
                disc(at(side * 0.13, -0.1), 0.035, dark);
            }
            disc(at(0.0, 0.1), 0.06, light);
        }
        // An explosion.
        Emblem::Burst => {
            draw_spiky(at(0.0, 0.0), 8, r * 0.52, r * 0.26, dark);
            disc(at(0.0, 0.0), 0.13, light);
        }
        // A crescent moon with a star, for a monster's nightmare.
        Emblem::Crescent => {
            disc(at(-0.04, 0.0), 0.42, dark);
            disc(at(0.14, -0.08), 0.35, face);
            draw_solid_star(at(0.24, 0.1), r * 0.14, light);
        }
        Emblem::Crown => {
            draw_quad(
                at(-0.4, 0.08),
                at(0.4, 0.08),
                at(0.4, 0.32),
                at(-0.4, 0.32),
                dark,
            );
            draw_triangle(at(-0.4, 0.08), at(-0.4, -0.34), at(-0.1, 0.08), dark);
            draw_triangle(at(-0.22, 0.08), at(0.0, -0.44), at(0.22, 0.08), dark);
            draw_triangle(at(0.4, 0.08), at(0.4, -0.34), at(0.1, 0.08), dark);
            for x in [-0.22, 0.0, 0.22] {
                disc(at(x, 0.2), 0.05, light);
            }
        }
        Emblem::Hammer => {
            line(at(-0.36, 0.44), at(0.1, -0.1), 0.11, dark);
            stroke(at(-0.04, -0.32), at(0.36, 0.02), 0.3, dark);
            line(at(0.0, -0.24), at(0.26, -0.02), 0.06, light);
        }
        Emblem::Helmet => {
            disc(at(0.0, 0.02), 0.4, dark);
            rect(-0.5, 0.24, 1.0, 0.4, face);
            rect(-0.4, 0.02, 0.8, 0.22, dark);
            rect(-0.31, 0.06, 0.62, 0.07, light);
            rect(-0.04, 0.06, 0.08, 0.2, light);
            draw_triangle(at(-0.04, -0.38), at(0.1, -0.56), at(0.2, -0.34), dark);
        }
        Emblem::Tent => {
            draw_triangle(at(0.0, -0.42), at(-0.5, 0.38), at(0.5, 0.38), dark);
            draw_triangle(at(0.0, -0.06), at(-0.16, 0.38), at(0.16, 0.38), light);
            line(at(-0.56, 0.4), at(0.56, 0.4), 0.06, dark);
        }
        Emblem::Compass => {
            ring(at(0.0, 0.0), 0.42, 0.07, dark);
            draw_triangle(at(0.12, -0.3), at(-0.09, -0.08), at(0.09, 0.09), dark);
            draw_triangle(at(-0.12, 0.3), at(-0.09, -0.08), at(0.09, 0.09), light);
            disc(at(0.0, 0.0), 0.04, dark);
        }
        Emblem::Map => {
            draw_quad(
                at(-0.42, -0.3),
                at(0.42, -0.3),
                at(0.42, 0.34),
                at(-0.42, 0.34),
                dark,
            );
            for x in [-0.14, 0.14] {
                line(at(x, -0.3), at(x, 0.34), 0.03, light);
            }
            for p in [(-0.3, 0.2), (-0.2, 0.04), (-0.05, -0.02), (0.06, 0.14)] {
                disc(at(p.0, p.1), 0.035, light);
            }
            line(at(0.18, 0.0), at(0.34, 0.16), 0.07, light);
            line(at(0.34, 0.0), at(0.18, 0.16), 0.07, light);
        }
        Emblem::Footprints => {
            for (x, y) in [(-0.2, 0.1), (0.2, -0.16)] {
                oval(at(x, y), 0.11, 0.18, dark);
                oval(at(x, y + 0.29), 0.08, 0.08, dark);
                for dx in [-0.09, 0.0, 0.09] {
                    disc(at(x + dx, y - 0.27), 0.045, dark);
                }
            }
        }
        Emblem::Mountain => {
            draw_triangle(at(-0.2, -0.14), at(-0.55, 0.4), at(0.15, 0.4), dark);
            draw_triangle(at(0.12, -0.3), at(-0.3, 0.4), at(0.54, 0.4), dark);
            draw_triangle(at(0.12, -0.3), at(0.0, -0.08), at(0.24, -0.08), light);
            line(at(0.12, -0.3), at(0.12, -0.5), 0.04, dark);
            draw_triangle(at(0.12, -0.5), at(0.32, -0.43), at(0.12, -0.36), dark);
        }
        Emblem::Castle => {
            rect(-0.4, -0.02, 0.8, 0.42, dark);
            for x in [-0.44, 0.2] {
                rect(x, -0.3, 0.24, 0.7, dark);
                for k in 0..3 {
                    rect(x + k as f32 * 0.09, -0.4, 0.06, 0.1, dark);
                }
            }
            rect(-0.08, 0.14, 0.16, 0.26, light);
            disc(at(0.0, 0.14), 0.08, light);
        }
        // Ten dots for the digits 0–9.
        Emblem::Dots => {
            for row in 0..2 {
                for col in 0..5 {
                    let p = at(-0.4 + col as f32 * 0.2, -0.12 + row as f32 * 0.26);
                    draw_circle(p.x, p.y, r * 0.085, dark);
                }
            }
        }
        Emblem::Quarter | Emblem::Half | Emblem::ThreeQuarters => {
            let fraction = match emblem {
                Emblem::Quarter => 0.25,
                Emblem::Half => 0.5,
                _ => 0.75,
            };
            ring(at(0.0, 0.0), 0.42, 0.06, dark);
            draw_pie(at(0.0, 0.0), r * 0.42, fraction, dark);
        }
        Emblem::Book => {
            for side in [-1.0, 1.0] {
                draw_quad(
                    at(side * 0.03, -0.2),
                    at(side * 0.46, -0.34),
                    at(side * 0.46, 0.3),
                    at(side * 0.03, 0.42),
                    dark,
                );
            }
            draw_solid_star(at(0.0, -0.32), r * 0.16, light);
            for side in [-1.0, 1.0] {
                for y in [-0.08, 0.06, 0.2] {
                    line(at(side * 0.1, y), at(side * 0.38, y - 0.08), 0.03, light);
                }
            }
        }
        Emblem::Chain => {
            for side in [-1.0, 1.0] {
                ring(at(side * 0.2, 0.0), 0.27, 0.1, dark);
            }
        }
        Emblem::Flame => {
            disc(at(0.0, 0.16), 0.28, dark);
            draw_triangle(at(0.04, -0.54), at(-0.24, 0.1), at(0.22, 0.1), dark);
            draw_triangle(at(-0.26, -0.24), at(-0.3, 0.16), at(-0.06, 0.02), dark);
            draw_triangle(at(0.3, -0.14), at(0.06, -0.02), at(0.3, 0.2), dark);
            disc(at(0.0, 0.22), 0.13, light);
            draw_triangle(at(0.0, -0.04), at(-0.11, 0.22), at(0.11, 0.22), light);
        }
        Emblem::Rocket => {
            oval(at(0.0, -0.06), 0.17, 0.4, dark);
            disc(at(0.0, -0.12), 0.08, light);
            for side in [-1.0, 1.0] {
                draw_triangle(
                    at(side * 0.14, 0.12),
                    at(side * 0.34, 0.4),
                    at(side * 0.1, 0.3),
                    dark,
                );
            }
            draw_triangle(at(-0.08, 0.3), at(0.08, 0.3), at(0.0, 0.52), light);
        }
        Emblem::Comet => {
            // A tail that widens towards the head, with a streak in it.
            draw_triangle(at(0.36, -0.1), at(0.1, -0.36), at(-0.46, 0.46), dark);
            line(at(0.14, -0.14), at(-0.3, 0.3), 0.04, light);
            draw_solid_star(at(0.24, -0.24), r * 0.27, dark);
            disc(at(0.24, -0.24), 0.07, light);
        }
        Emblem::Star => draw_solid_star(at(0.0, 0.03), r * 0.52, dark),
        Emblem::Check => {
            stroke(at(-0.32, 0.04), at(-0.1, 0.28), 0.17, dark);
            stroke(at(-0.1, 0.28), at(0.36, -0.3), 0.17, dark);
        }
        Emblem::ThreeStars => {
            draw_solid_star(at(0.0, -0.1), r * 0.3, dark);
            draw_solid_star(at(-0.33, 0.2), r * 0.2, dark);
            draw_solid_star(at(0.33, 0.2), r * 0.2, dark);
        }
        Emblem::Constellation => {
            let stars = [
                (-0.36, 0.2),
                (-0.16, -0.14),
                (0.08, 0.08),
                (0.3, -0.26),
                (0.4, 0.22),
            ];
            for pair in stars.windows(2) {
                line(
                    at(pair[0].0, pair[0].1),
                    at(pair[1].0, pair[1].1),
                    0.035,
                    dark,
                );
            }
            for (i, s) in stars.iter().enumerate() {
                draw_solid_star(at(s.0, s.1), r * if i == 3 { 0.2 } else { 0.14 }, dark);
            }
        }
        Emblem::Ruler => {
            let (a, b) = (at(-0.4, 0.32), at(0.4, -0.32));
            line(a, b, 0.32, dark);
            // Ticks across the ruler, from one edge.
            let along = (b - a).normalize();
            let across = vec2(-along.y, along.x);
            for i in 0..7 {
                let base = a + (b - a) * (0.08 + i as f32 * 0.14) + across * r * 0.16;
                let len = if i % 2 == 0 { 0.16 } else { 0.09 };
                draw_line(
                    base.x,
                    base.y,
                    base.x - across.x * r * len,
                    base.y - across.y * r * len,
                    r * 0.035,
                    light,
                );
            }
        }
        Emblem::Scroll => {
            rect(-0.3, -0.34, 0.6, 0.68, dark);
            rect(-0.38, -0.44, 0.76, 0.13, dark);
            rect(-0.38, 0.31, 0.76, 0.13, dark);
            for y in [-0.16, 0.0, 0.16] {
                line(at(-0.18, y), at(0.18, y), 0.05, light);
            }
        }
        Emblem::WitchHat => {
            draw_triangle(at(0.12, -0.52), at(-0.24, 0.26), at(0.24, 0.26), dark);
            oval(at(0.0, 0.28), 0.5, 0.1, dark);
            draw_quad(
                at(-0.22, 0.12),
                at(0.22, 0.12),
                at(0.24, 0.26),
                at(-0.24, 0.26),
                light,
            );
            draw_solid_star(at(0.02, -0.14), r * 0.09, light);
        }
        Emblem::Card => {
            let (w, h) = (0.62 * r, 0.82 * r);
            draw_rectangle(c.x - w / 2.0, c.y - h / 2.0, w, h, light);
            draw_rectangle_lines(c.x - w / 2.0, c.y - h / 2.0, w, h, 0.09 * r, dark);
            fonts::draw_centered(
                "?",
                c.x,
                c.y + 0.02 * r,
                (r * 0.62) as u16,
                dark,
                Style::Bold,
            );
        }
        Emblem::Pencil => {
            let (tip, end) = (at(-0.42, 0.42), at(0.3, -0.3));
            let start = tip + (end - tip).normalize() * r * 0.24;
            let across = vec2(0.7, 0.7) * r * 0.1;
            draw_triangle(tip, start + across, start - across, dark);
            line(start, end, 0.2, dark);
            line(end, end + (end - tip).normalize() * r * 0.12, 0.2, light);
        }
        Emblem::Dumbbell => {
            line(at(-0.32, 0.0), at(0.32, 0.0), 0.09, dark);
            for side in [-1.0, 1.0] {
                rect(side * 0.42 - 0.05, -0.22, 0.1, 0.44, dark);
                rect(side * 0.31 - 0.04, -0.15, 0.08, 0.3, dark);
            }
        }
        Emblem::GraduationCap => {
            draw_quad(
                at(0.0, -0.32),
                at(0.52, -0.1),
                at(0.0, 0.12),
                at(-0.52, -0.1),
                dark,
            );
            draw_quad(
                at(-0.28, 0.02),
                at(0.28, 0.02),
                at(0.26, 0.26),
                at(-0.26, 0.26),
                dark,
            );
            line(at(-0.28, 0.03), at(0.28, 0.03), 0.04, light);
            line(at(0.42, -0.08), at(0.42, 0.22), 0.04, light);
            disc(at(0.42, 0.26), 0.06, light);
        }
        Emblem::Heart => {
            disc(at(-0.15, -0.12), 0.21, dark);
            disc(at(0.15, -0.12), 0.21, dark);
            draw_triangle(at(-0.34, -0.03), at(0.34, -0.03), at(0.0, 0.42), dark);
            disc(at(-0.2, -0.16), 0.06, light);
        }
        Emblem::Calendar => {
            rect(-0.38, -0.3, 0.76, 0.72, dark);
            rect(-0.38, -0.3, 0.76, 0.16, light);
            for x in [-0.2, 0.14] {
                rect(x, -0.4, 0.06, 0.16, dark);
            }
            for row in 0..2 {
                for col in 0..3 {
                    rect(
                        -0.27 + col as f32 * 0.21,
                        -0.04 + row as f32 * 0.22,
                        0.13,
                        0.13,
                        light,
                    );
                }
            }
        }
        Emblem::Rainbow => {
            disc(at(0.0, 0.1), 0.46, dark);
            disc(at(0.0, 0.1), 0.36, light);
            disc(at(0.0, 0.1), 0.26, dark);
            disc(at(0.0, 0.1), 0.16, face);
            rect(-0.5, 0.1, 1.0, 0.5, face);
            for side in [-1.0, 1.0] {
                disc(at(side * 0.36, 0.14), 0.09, light);
            }
        }
        Emblem::Flower => {
            for i in 0..6 {
                let p = c + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 6.0) * r * 0.25;
                draw_circle(p.x, p.y, r * 0.16, dark);
            }
            disc(at(0.0, 0.0), 0.13, light);
        }
        Emblem::Moon => {
            disc(at(0.0, 0.0), 0.42, dark);
            disc(at(-0.13, -0.1), 0.09, light);
            disc(at(0.14, 0.14), 0.12, light);
            disc(at(0.16, -0.2), 0.06, light);
        }
        Emblem::Sun => {
            disc(at(0.0, 0.0), 0.24, dark);
            for i in 0..8 {
                let dir = Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4);
                line(c + dir * r * 0.36, c + dir * r * 0.58, 0.08, dark);
            }
        }
        Emblem::Stopwatch => {
            ring(at(0.0, 0.08), 0.34, 0.08, dark);
            rect(-0.07, -0.5, 0.14, 0.11, dark);
            line(at(0.0, 0.08), at(0.14, -0.1), 0.06, dark);
            for (x, y) in [(0.0, -0.16), (0.24, 0.08), (0.0, 0.32), (-0.24, 0.08)] {
                disc(at(x, y), 0.03, dark);
            }
        }
        Emblem::Bolt => {
            draw_triangle(at(0.14, -0.56), at(-0.32, 0.1), at(0.06, 0.1), dark);
            draw_triangle(at(-0.14, 0.56), at(0.32, -0.1), at(-0.06, -0.1), dark);
            draw_quad(
                at(0.14, -0.56),
                at(0.06, 0.1),
                at(-0.06, -0.1),
                at(0.06, -0.1),
                dark,
            );
        }
        Emblem::CrystalBall => {
            disc(at(0.0, -0.06), 0.36, dark);
            draw_quad(
                at(-0.28, 0.3),
                at(0.28, 0.3),
                at(0.2, 0.44),
                at(-0.2, 0.44),
                dark,
            );
            disc(at(-0.14, -0.18), 0.08, light);
            draw_solid_star(at(0.06, -0.02), r * 0.15, light);
        }
        Emblem::Gem => {
            draw_quad(
                at(-0.2, -0.4),
                at(0.2, -0.4),
                at(0.38, -0.14),
                at(-0.38, -0.14),
                dark,
            );
            draw_triangle(at(-0.38, -0.08), at(0.38, -0.08), at(0.0, 0.46), dark);
            line(at(-0.1, -0.4), at(-0.2, -0.14), 0.03, light);
            line(at(0.1, -0.4), at(0.2, -0.14), 0.03, light);
            line(at(-0.2, -0.02), at(0.0, 0.3), 0.03, light);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::BADGES;
    use std::collections::HashSet;

    #[test]
    fn every_badge_has_a_picture_of_its_own() {
        let mut seen = HashSet::new();
        for badge in &BADGES {
            assert!(
                seen.insert(emblem_of(badge.id)),
                "{} shares its picture",
                badge.id
            );
        }
        assert_eq!(seen.len(), BADGES.len());
    }
}
