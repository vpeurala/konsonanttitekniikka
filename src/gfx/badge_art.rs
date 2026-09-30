//! Drawing badges: a round medal on a ribbon, in the metal of its tier,
//! with a little picture of its own. Like everything else in the game it is
//! made of shapes drawn by code.

use macroquad::prelude::*;

use crate::badges::{Badge, Tier};
use crate::gfx::fonts::{self, Style};

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

/// What the emblems are drawn with: where the medal is, how big it is and
/// its colours. Sizes and positions are in medal radii from the centre.
struct Pen {
    c: Vec2,
    r: f32,
    dark: Color,
    light: Color,
    face: Color,
}

impl Pen {
    /// A point, in medal radii from the centre.
    fn at(&self, dx: f32, dy: f32) -> Vec2 {
        self.c + vec2(dx, dy) * self.r
    }

    /// A line of a width given in radii.
    fn line(&self, a: Vec2, b: Vec2, w: f32, color: Color) {
        draw_line(a.x, a.y, b.x, b.y, w * self.r, color);
    }

    fn stroke(&self, a: Vec2, b: Vec2, w: f32, color: Color) {
        draw_stroke(a, b, w * self.r, color);
    }

    fn disc(&self, p: Vec2, size: f32, color: Color) {
        draw_circle(p.x, p.y, size * self.r, color);
    }

    fn ring(&self, p: Vec2, size: f32, w: f32, color: Color) {
        draw_circle_lines(p.x, p.y, size * self.r, w * self.r, color);
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let p = self.at(x, y);
        draw_rectangle(p.x, p.y, w * self.r, h * self.r, color);
    }

    fn oval(&self, p: Vec2, w: f32, h: f32, color: Color) {
        draw_ellipse(p.x, p.y, w * self.r, h * self.r, 0.0, color);
    }
}

/// Draws `emblem` on a medal of `radius` centred on `c`.
fn draw_emblem(emblem: Emblem, c: Vec2, radius: f32, paint: Paint) {
    let Paint { dark, light, face } = paint;
    let pen = Pen {
        c,
        r: radius,
        dark,
        light,
        face,
    };
    match emblem {
        Emblem::Cyclops => cyclops(&pen),
        Emblem::Wand => wand(&pen),
        Emblem::Shield => shield(&pen),
        Emblem::Sword => sword(&pen),
        Emblem::Trophy => trophy(&pen),
        Emblem::Ghost => ghost(&pen),
        Emblem::Burst => burst(&pen),
        Emblem::Crescent => crescent(&pen),
        Emblem::Crown => crown(&pen),
        Emblem::Hammer => hammer(&pen),
        Emblem::Helmet => helmet(&pen),
        Emblem::Tent => tent(&pen),
        Emblem::Compass => compass(&pen),
        Emblem::Map => map(&pen),
        Emblem::Footprints => footprints(&pen),
        Emblem::Mountain => mountain(&pen),
        Emblem::Castle => castle(&pen),
        Emblem::Dots => dots(&pen),
        Emblem::Quarter | Emblem::Half | Emblem::ThreeQuarters => fraction(&pen, emblem),
        Emblem::Book => book(&pen),
        Emblem::Chain => chain(&pen),
        Emblem::Flame => flame(&pen),
        Emblem::Rocket => rocket(&pen),
        Emblem::Comet => comet(&pen),
        Emblem::Star => star(&pen),
        Emblem::Check => check(&pen),
        Emblem::ThreeStars => three_stars(&pen),
        Emblem::Constellation => constellation(&pen),
        Emblem::Ruler => ruler(&pen),
        Emblem::Scroll => scroll(&pen),
        Emblem::WitchHat => witch_hat(&pen),
        Emblem::Card => card(&pen),
        Emblem::Pencil => pencil(&pen),
        Emblem::Dumbbell => dumbbell(&pen),
        Emblem::GraduationCap => graduation_cap(&pen),
        Emblem::Heart => heart(&pen),
        Emblem::Calendar => calendar(&pen),
        Emblem::Rainbow => rainbow(&pen),
        Emblem::Flower => flower(&pen),
        Emblem::Moon => moon(&pen),
        Emblem::Sun => sun(&pen),
        Emblem::Stopwatch => stopwatch(&pen),
        Emblem::Bolt => bolt(&pen),
        Emblem::CrystalBall => crystal_ball(&pen),
        Emblem::Gem => gem(&pen),
    }
}

fn cyclops(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.0), 0.44, pen.dark);
    pen.disc(pen.at(0.0, -0.08), 0.22, pen.light);
    pen.disc(pen.at(0.0, -0.08), 0.1, pen.dark);
    for side in [-1.0, 1.0] {
        draw_triangle(
            pen.at(side * 0.28, 0.14),
            pen.at(side * 0.1, 0.14),
            pen.at(side * 0.19, 0.33),
            pen.light,
        );
    }
}

/// A magic wand with a star at its tip.
fn wand(pen: &Pen) {
    pen.line(pen.at(-0.4, 0.42), pen.at(0.16, -0.14), 0.12, pen.dark);
    pen.line(pen.at(-0.4, 0.42), pen.at(-0.28, 0.3), 0.12, pen.light);
    draw_solid_star(pen.at(0.26, -0.26), pen.r * 0.3, pen.dark);
    pen.disc(pen.at(-0.1, -0.36), 0.05, pen.dark);
    pen.disc(pen.at(0.4, 0.1), 0.05, pen.dark);
}

fn shield(pen: &Pen) {
    draw_quad(
        pen.at(-0.38, -0.4),
        pen.at(0.38, -0.4),
        pen.at(0.38, 0.05),
        pen.at(-0.38, 0.05),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.38, 0.05),
        pen.at(0.38, 0.05),
        pen.at(0.0, 0.5),
        pen.dark,
    );
    pen.line(pen.at(0.0, -0.32), pen.at(0.0, 0.38), 0.09, pen.light);
    pen.line(pen.at(-0.3, -0.12), pen.at(0.3, -0.12), 0.09, pen.light);
}

fn sword(pen: &Pen) {
    let (tip, guard) = (pen.at(0.4, -0.42), pen.at(-0.14, 0.12));
    pen.line(tip, guard, 0.15, pen.dark);
    pen.line(pen.at(0.32, -0.34), pen.at(-0.06, 0.04), 0.04, pen.light);
    let perp = vec2(0.6, 0.6);
    pen.stroke(
        guard - perp * pen.r * 0.32,
        guard + perp * pen.r * 0.32,
        0.11,
        pen.dark,
    );
    pen.line(guard, pen.at(-0.34, 0.32), 0.1, pen.dark);
    pen.disc(pen.at(-0.38, 0.36), 0.09, pen.dark);
}

fn trophy(pen: &Pen) {
    draw_quad(
        pen.at(-0.32, -0.36),
        pen.at(0.32, -0.36),
        pen.at(0.2, 0.06),
        pen.at(-0.2, 0.06),
        pen.dark,
    );
    for side in [-1.0, 1.0] {
        pen.ring(pen.at(side * 0.36, -0.2), 0.12, 0.06, pen.dark);
    }
    pen.rect(-0.05, 0.05, 0.1, 0.22, pen.dark);
    pen.rect(-0.26, 0.26, 0.52, 0.1, pen.dark);
    draw_solid_star(pen.at(0.0, -0.16), pen.r * 0.12, pen.light);
}

fn ghost(pen: &Pen) {
    pen.disc(pen.at(0.0, -0.08), 0.34, pen.dark);
    pen.rect(-0.34, -0.08, 0.68, 0.4, pen.dark);
    for i in 0..3 {
        let x = -0.34 + i as f32 * 0.2267;
        draw_triangle(
            pen.at(x, 0.3),
            pen.at(x + 0.2267, 0.3),
            pen.at(x + 0.1133, 0.48),
            pen.dark,
        );
    }
    for side in [-1.0, 1.0] {
        pen.disc(pen.at(side * 0.13, -0.12), 0.08, pen.light);
        pen.disc(pen.at(side * 0.13, -0.1), 0.035, pen.dark);
    }
    pen.disc(pen.at(0.0, 0.1), 0.06, pen.light);
}

/// An explosion.
fn burst(pen: &Pen) {
    draw_spiky(pen.at(0.0, 0.0), 8, pen.r * 0.52, pen.r * 0.26, pen.dark);
    pen.disc(pen.at(0.0, 0.0), 0.13, pen.light);
}

/// A crescent moon with a star, for a monster's nightmare.
fn crescent(pen: &Pen) {
    pen.disc(pen.at(-0.04, 0.0), 0.42, pen.dark);
    pen.disc(pen.at(0.14, -0.08), 0.35, pen.face);
    draw_solid_star(pen.at(0.24, 0.1), pen.r * 0.14, pen.light);
}

fn crown(pen: &Pen) {
    draw_quad(
        pen.at(-0.4, 0.08),
        pen.at(0.4, 0.08),
        pen.at(0.4, 0.32),
        pen.at(-0.4, 0.32),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.4, 0.08),
        pen.at(-0.4, -0.34),
        pen.at(-0.1, 0.08),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.22, 0.08),
        pen.at(0.0, -0.44),
        pen.at(0.22, 0.08),
        pen.dark,
    );
    draw_triangle(
        pen.at(0.4, 0.08),
        pen.at(0.4, -0.34),
        pen.at(0.1, 0.08),
        pen.dark,
    );
    for x in [-0.22, 0.0, 0.22] {
        pen.disc(pen.at(x, 0.2), 0.05, pen.light);
    }
}

fn hammer(pen: &Pen) {
    pen.line(pen.at(-0.36, 0.44), pen.at(0.1, -0.1), 0.11, pen.dark);
    pen.stroke(pen.at(-0.04, -0.32), pen.at(0.36, 0.02), 0.3, pen.dark);
    pen.line(pen.at(0.0, -0.24), pen.at(0.26, -0.02), 0.06, pen.light);
}

fn helmet(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.02), 0.4, pen.dark);
    pen.rect(-0.5, 0.24, 1.0, 0.4, pen.face);
    pen.rect(-0.4, 0.02, 0.8, 0.22, pen.dark);
    pen.rect(-0.31, 0.06, 0.62, 0.07, pen.light);
    pen.rect(-0.04, 0.06, 0.08, 0.2, pen.light);
    draw_triangle(
        pen.at(-0.04, -0.38),
        pen.at(0.1, -0.56),
        pen.at(0.2, -0.34),
        pen.dark,
    );
}

fn tent(pen: &Pen) {
    draw_triangle(
        pen.at(0.0, -0.42),
        pen.at(-0.5, 0.38),
        pen.at(0.5, 0.38),
        pen.dark,
    );
    draw_triangle(
        pen.at(0.0, -0.06),
        pen.at(-0.16, 0.38),
        pen.at(0.16, 0.38),
        pen.light,
    );
    pen.line(pen.at(-0.56, 0.4), pen.at(0.56, 0.4), 0.06, pen.dark);
}

fn compass(pen: &Pen) {
    pen.ring(pen.at(0.0, 0.0), 0.42, 0.07, pen.dark);
    draw_triangle(
        pen.at(0.12, -0.3),
        pen.at(-0.09, -0.08),
        pen.at(0.09, 0.09),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.12, 0.3),
        pen.at(-0.09, -0.08),
        pen.at(0.09, 0.09),
        pen.light,
    );
    pen.disc(pen.at(0.0, 0.0), 0.04, pen.dark);
}

fn map(pen: &Pen) {
    draw_quad(
        pen.at(-0.42, -0.3),
        pen.at(0.42, -0.3),
        pen.at(0.42, 0.34),
        pen.at(-0.42, 0.34),
        pen.dark,
    );
    for x in [-0.14, 0.14] {
        pen.line(pen.at(x, -0.3), pen.at(x, 0.34), 0.03, pen.light);
    }
    for p in [(-0.3, 0.2), (-0.2, 0.04), (-0.05, -0.02), (0.06, 0.14)] {
        pen.disc(pen.at(p.0, p.1), 0.035, pen.light);
    }
    pen.line(pen.at(0.18, 0.0), pen.at(0.34, 0.16), 0.07, pen.light);
    pen.line(pen.at(0.34, 0.0), pen.at(0.18, 0.16), 0.07, pen.light);
}

fn footprints(pen: &Pen) {
    for (x, y) in [(-0.2, 0.1), (0.2, -0.16)] {
        pen.oval(pen.at(x, y), 0.11, 0.18, pen.dark);
        pen.oval(pen.at(x, y + 0.29), 0.08, 0.08, pen.dark);
        for dx in [-0.09, 0.0, 0.09] {
            pen.disc(pen.at(x + dx, y - 0.27), 0.045, pen.dark);
        }
    }
}

fn mountain(pen: &Pen) {
    draw_triangle(
        pen.at(-0.2, -0.14),
        pen.at(-0.55, 0.4),
        pen.at(0.15, 0.4),
        pen.dark,
    );
    draw_triangle(
        pen.at(0.12, -0.3),
        pen.at(-0.3, 0.4),
        pen.at(0.54, 0.4),
        pen.dark,
    );
    draw_triangle(
        pen.at(0.12, -0.3),
        pen.at(0.0, -0.08),
        pen.at(0.24, -0.08),
        pen.light,
    );
    pen.line(pen.at(0.12, -0.3), pen.at(0.12, -0.5), 0.04, pen.dark);
    draw_triangle(
        pen.at(0.12, -0.5),
        pen.at(0.32, -0.43),
        pen.at(0.12, -0.36),
        pen.dark,
    );
}

fn castle(pen: &Pen) {
    pen.rect(-0.4, -0.02, 0.8, 0.42, pen.dark);
    for x in [-0.44, 0.2] {
        pen.rect(x, -0.3, 0.24, 0.7, pen.dark);
        for k in 0..3 {
            pen.rect(x + k as f32 * 0.09, -0.4, 0.06, 0.1, pen.dark);
        }
    }
    pen.rect(-0.08, 0.14, 0.16, 0.26, pen.light);
    pen.disc(pen.at(0.0, 0.14), 0.08, pen.light);
}

/// Ten dots for the digits 0–9.
fn dots(pen: &Pen) {
    for row in 0..2 {
        for col in 0..5 {
            let p = pen.at(-0.4 + col as f32 * 0.2, -0.12 + row as f32 * 0.26);
            draw_circle(p.x, p.y, pen.r * 0.085, pen.dark);
        }
    }
}

fn fraction(pen: &Pen, emblem: Emblem) {
    let fraction = match emblem {
        Emblem::Quarter => 0.25,
        Emblem::Half => 0.5,
        _ => 0.75,
    };
    pen.ring(pen.at(0.0, 0.0), 0.42, 0.06, pen.dark);
    draw_pie(pen.at(0.0, 0.0), pen.r * 0.42, fraction, pen.dark);
}

fn book(pen: &Pen) {
    for side in [-1.0, 1.0] {
        draw_quad(
            pen.at(side * 0.03, -0.2),
            pen.at(side * 0.46, -0.34),
            pen.at(side * 0.46, 0.3),
            pen.at(side * 0.03, 0.42),
            pen.dark,
        );
    }
    draw_solid_star(pen.at(0.0, -0.32), pen.r * 0.16, pen.light);
    for side in [-1.0, 1.0] {
        for y in [-0.08, 0.06, 0.2] {
            pen.line(
                pen.at(side * 0.1, y),
                pen.at(side * 0.38, y - 0.08),
                0.03,
                pen.light,
            );
        }
    }
}

fn chain(pen: &Pen) {
    for side in [-1.0, 1.0] {
        pen.ring(pen.at(side * 0.2, 0.0), 0.27, 0.1, pen.dark);
    }
}

fn flame(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.16), 0.28, pen.dark);
    draw_triangle(
        pen.at(0.04, -0.54),
        pen.at(-0.24, 0.1),
        pen.at(0.22, 0.1),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.26, -0.24),
        pen.at(-0.3, 0.16),
        pen.at(-0.06, 0.02),
        pen.dark,
    );
    draw_triangle(
        pen.at(0.3, -0.14),
        pen.at(0.06, -0.02),
        pen.at(0.3, 0.2),
        pen.dark,
    );
    pen.disc(pen.at(0.0, 0.22), 0.13, pen.light);
    draw_triangle(
        pen.at(0.0, -0.04),
        pen.at(-0.11, 0.22),
        pen.at(0.11, 0.22),
        pen.light,
    );
}

fn rocket(pen: &Pen) {
    pen.oval(pen.at(0.0, -0.06), 0.17, 0.4, pen.dark);
    pen.disc(pen.at(0.0, -0.12), 0.08, pen.light);
    for side in [-1.0, 1.0] {
        draw_triangle(
            pen.at(side * 0.14, 0.12),
            pen.at(side * 0.34, 0.4),
            pen.at(side * 0.1, 0.3),
            pen.dark,
        );
    }
    draw_triangle(
        pen.at(-0.08, 0.3),
        pen.at(0.08, 0.3),
        pen.at(0.0, 0.52),
        pen.light,
    );
}

fn comet(pen: &Pen) {
    // A tail that widens towards the head, with a streak in it.
    draw_triangle(
        pen.at(0.36, -0.1),
        pen.at(0.1, -0.36),
        pen.at(-0.46, 0.46),
        pen.dark,
    );
    pen.line(pen.at(0.14, -0.14), pen.at(-0.3, 0.3), 0.04, pen.light);
    draw_solid_star(pen.at(0.24, -0.24), pen.r * 0.27, pen.dark);
    pen.disc(pen.at(0.24, -0.24), 0.07, pen.light);
}

fn star(pen: &Pen) {
    draw_solid_star(pen.at(0.0, 0.03), pen.r * 0.52, pen.dark);
}

fn check(pen: &Pen) {
    pen.stroke(pen.at(-0.32, 0.04), pen.at(-0.1, 0.28), 0.17, pen.dark);
    pen.stroke(pen.at(-0.1, 0.28), pen.at(0.36, -0.3), 0.17, pen.dark);
}

fn three_stars(pen: &Pen) {
    draw_solid_star(pen.at(0.0, -0.1), pen.r * 0.3, pen.dark);
    draw_solid_star(pen.at(-0.33, 0.2), pen.r * 0.2, pen.dark);
    draw_solid_star(pen.at(0.33, 0.2), pen.r * 0.2, pen.dark);
}

fn constellation(pen: &Pen) {
    let stars = [
        (-0.36, 0.2),
        (-0.16, -0.14),
        (0.08, 0.08),
        (0.3, -0.26),
        (0.4, 0.22),
    ];
    for pair in stars.windows(2) {
        pen.line(
            pen.at(pair[0].0, pair[0].1),
            pen.at(pair[1].0, pair[1].1),
            0.035,
            pen.dark,
        );
    }
    for (i, s) in stars.iter().enumerate() {
        draw_solid_star(
            pen.at(s.0, s.1),
            pen.r * if i == 3 { 0.2 } else { 0.14 },
            pen.dark,
        );
    }
}

fn ruler(pen: &Pen) {
    let (a, b) = (pen.at(-0.4, 0.32), pen.at(0.4, -0.32));
    pen.line(a, b, 0.32, pen.dark);
    // Ticks across the ruler, from one edge.
    let along = (b - a).normalize();
    let across = vec2(-along.y, along.x);
    for i in 0..7 {
        let base = a + (b - a) * (0.08 + i as f32 * 0.14) + across * pen.r * 0.16;
        let len = if i % 2 == 0 { 0.16 } else { 0.09 };
        draw_line(
            base.x,
            base.y,
            base.x - across.x * pen.r * len,
            base.y - across.y * pen.r * len,
            pen.r * 0.035,
            pen.light,
        );
    }
}

fn scroll(pen: &Pen) {
    pen.rect(-0.3, -0.34, 0.6, 0.68, pen.dark);
    pen.rect(-0.38, -0.44, 0.76, 0.13, pen.dark);
    pen.rect(-0.38, 0.31, 0.76, 0.13, pen.dark);
    for y in [-0.16, 0.0, 0.16] {
        pen.line(pen.at(-0.18, y), pen.at(0.18, y), 0.05, pen.light);
    }
}

fn witch_hat(pen: &Pen) {
    draw_triangle(
        pen.at(0.12, -0.52),
        pen.at(-0.24, 0.26),
        pen.at(0.24, 0.26),
        pen.dark,
    );
    pen.oval(pen.at(0.0, 0.28), 0.5, 0.1, pen.dark);
    draw_quad(
        pen.at(-0.22, 0.12),
        pen.at(0.22, 0.12),
        pen.at(0.24, 0.26),
        pen.at(-0.24, 0.26),
        pen.light,
    );
    draw_solid_star(pen.at(0.02, -0.14), pen.r * 0.09, pen.light);
}

fn card(pen: &Pen) {
    let (w, h) = (0.62 * pen.r, 0.82 * pen.r);
    draw_rectangle(pen.c.x - w / 2.0, pen.c.y - h / 2.0, w, h, pen.light);
    draw_rectangle_lines(
        pen.c.x - w / 2.0,
        pen.c.y - h / 2.0,
        w,
        h,
        0.09 * pen.r,
        pen.dark,
    );
    fonts::draw_centered(
        "?",
        pen.c.x,
        pen.c.y + 0.02 * pen.r,
        (pen.r * 0.62) as u16,
        pen.dark,
        Style::Bold,
    );
}

fn pencil(pen: &Pen) {
    let (tip, end) = (pen.at(-0.42, 0.42), pen.at(0.3, -0.3));
    let start = tip + (end - tip).normalize() * pen.r * 0.24;
    let across = vec2(0.7, 0.7) * pen.r * 0.1;
    draw_triangle(tip, start + across, start - across, pen.dark);
    pen.line(start, end, 0.2, pen.dark);
    pen.line(
        end,
        end + (end - tip).normalize() * pen.r * 0.12,
        0.2,
        pen.light,
    );
}

fn dumbbell(pen: &Pen) {
    pen.line(pen.at(-0.32, 0.0), pen.at(0.32, 0.0), 0.09, pen.dark);
    for side in [-1.0, 1.0] {
        pen.rect(side * 0.42 - 0.05, -0.22, 0.1, 0.44, pen.dark);
        pen.rect(side * 0.31 - 0.04, -0.15, 0.08, 0.3, pen.dark);
    }
}

fn graduation_cap(pen: &Pen) {
    draw_quad(
        pen.at(0.0, -0.32),
        pen.at(0.52, -0.1),
        pen.at(0.0, 0.12),
        pen.at(-0.52, -0.1),
        pen.dark,
    );
    draw_quad(
        pen.at(-0.28, 0.02),
        pen.at(0.28, 0.02),
        pen.at(0.26, 0.26),
        pen.at(-0.26, 0.26),
        pen.dark,
    );
    pen.line(pen.at(-0.28, 0.03), pen.at(0.28, 0.03), 0.04, pen.light);
    pen.line(pen.at(0.42, -0.08), pen.at(0.42, 0.22), 0.04, pen.light);
    pen.disc(pen.at(0.42, 0.26), 0.06, pen.light);
}

fn heart(pen: &Pen) {
    pen.disc(pen.at(-0.15, -0.12), 0.21, pen.dark);
    pen.disc(pen.at(0.15, -0.12), 0.21, pen.dark);
    draw_triangle(
        pen.at(-0.34, -0.03),
        pen.at(0.34, -0.03),
        pen.at(0.0, 0.42),
        pen.dark,
    );
    pen.disc(pen.at(-0.2, -0.16), 0.06, pen.light);
}

fn calendar(pen: &Pen) {
    pen.rect(-0.38, -0.3, 0.76, 0.72, pen.dark);
    pen.rect(-0.38, -0.3, 0.76, 0.16, pen.light);
    for x in [-0.2, 0.14] {
        pen.rect(x, -0.4, 0.06, 0.16, pen.dark);
    }
    for row in 0..2 {
        for col in 0..3 {
            pen.rect(
                -0.27 + col as f32 * 0.21,
                -0.04 + row as f32 * 0.22,
                0.13,
                0.13,
                pen.light,
            );
        }
    }
}

fn rainbow(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.1), 0.46, pen.dark);
    pen.disc(pen.at(0.0, 0.1), 0.36, pen.light);
    pen.disc(pen.at(0.0, 0.1), 0.26, pen.dark);
    pen.disc(pen.at(0.0, 0.1), 0.16, pen.face);
    pen.rect(-0.5, 0.1, 1.0, 0.5, pen.face);
    for side in [-1.0, 1.0] {
        pen.disc(pen.at(side * 0.36, 0.14), 0.09, pen.light);
    }
}

fn flower(pen: &Pen) {
    for i in 0..6 {
        let p = pen.c + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 6.0) * pen.r * 0.25;
        draw_circle(p.x, p.y, pen.r * 0.16, pen.dark);
    }
    pen.disc(pen.at(0.0, 0.0), 0.13, pen.light);
}

fn moon(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.0), 0.42, pen.dark);
    pen.disc(pen.at(-0.13, -0.1), 0.09, pen.light);
    pen.disc(pen.at(0.14, 0.14), 0.12, pen.light);
    pen.disc(pen.at(0.16, -0.2), 0.06, pen.light);
}

fn sun(pen: &Pen) {
    pen.disc(pen.at(0.0, 0.0), 0.24, pen.dark);
    for i in 0..8 {
        let dir = Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4);
        pen.line(
            pen.c + dir * pen.r * 0.36,
            pen.c + dir * pen.r * 0.58,
            0.08,
            pen.dark,
        );
    }
}

fn stopwatch(pen: &Pen) {
    pen.ring(pen.at(0.0, 0.08), 0.34, 0.08, pen.dark);
    pen.rect(-0.07, -0.5, 0.14, 0.11, pen.dark);
    pen.line(pen.at(0.0, 0.08), pen.at(0.14, -0.1), 0.06, pen.dark);
    for (x, y) in [(0.0, -0.16), (0.24, 0.08), (0.0, 0.32), (-0.24, 0.08)] {
        pen.disc(pen.at(x, y), 0.03, pen.dark);
    }
}

fn bolt(pen: &Pen) {
    draw_triangle(
        pen.at(0.14, -0.56),
        pen.at(-0.32, 0.1),
        pen.at(0.06, 0.1),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.14, 0.56),
        pen.at(0.32, -0.1),
        pen.at(-0.06, -0.1),
        pen.dark,
    );
    draw_quad(
        pen.at(0.14, -0.56),
        pen.at(0.06, 0.1),
        pen.at(-0.06, -0.1),
        pen.at(0.06, -0.1),
        pen.dark,
    );
}

fn crystal_ball(pen: &Pen) {
    pen.disc(pen.at(0.0, -0.06), 0.36, pen.dark);
    draw_quad(
        pen.at(-0.28, 0.3),
        pen.at(0.28, 0.3),
        pen.at(0.2, 0.44),
        pen.at(-0.2, 0.44),
        pen.dark,
    );
    pen.disc(pen.at(-0.14, -0.18), 0.08, pen.light);
    draw_solid_star(pen.at(0.06, -0.02), pen.r * 0.15, pen.light);
}

fn gem(pen: &Pen) {
    draw_quad(
        pen.at(-0.2, -0.4),
        pen.at(0.2, -0.4),
        pen.at(0.38, -0.14),
        pen.at(-0.38, -0.14),
        pen.dark,
    );
    draw_triangle(
        pen.at(-0.38, -0.08),
        pen.at(0.38, -0.08),
        pen.at(0.0, 0.46),
        pen.dark,
    );
    pen.line(pen.at(-0.1, -0.4), pen.at(-0.2, -0.14), 0.03, pen.light);
    pen.line(pen.at(0.1, -0.4), pen.at(0.2, -0.14), 0.03, pen.light);
    pen.line(pen.at(-0.2, -0.02), pen.at(0.0, 0.3), 0.03, pen.light);
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
