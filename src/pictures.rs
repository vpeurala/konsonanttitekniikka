//! A small picture for every word, to strengthen its mental image.
//!
//! Each picture is drawn in its own square coordinate system running from
//! -1 to 1 on both axes, with y pointing down, so it can be drawn at any
//! size.

use std::f32::consts::TAU;

use macroquad::prelude::*;

/// Draws in a square centered on `center`, `half` pixels from the center
/// to each edge, with coordinates from -1 to 1.
pub struct Canvas {
    center: Vec2,
    half: f32,
}

type Pt = (f32, f32);

impl Canvas {
    fn p(&self, x: f32, y: f32) -> Vec2 {
        self.center + vec2(x, y) * self.half
    }

    fn s(&self, v: f32) -> f32 {
        v * self.half
    }

    fn circle(&self, x: f32, y: f32, r: f32, color: Color) {
        let p = self.p(x, y);
        draw_circle(p.x, p.y, self.s(r), color);
    }

    fn ring(&self, x: f32, y: f32, r: f32, thickness: f32, color: Color) {
        let p = self.p(x, y);
        draw_circle_lines(p.x, p.y, self.s(r), self.s(thickness), color);
    }

    /// An ellipse with radii `rx` and `ry`, rotated by `degrees`.
    fn ellipse(&self, x: f32, y: f32, rx: f32, ry: f32, degrees: f32, color: Color) {
        let p = self.p(x, y);
        draw_ellipse(p.x, p.y, self.s(rx), self.s(ry), degrees, color);
    }

    fn ellipse_lines(&self, x: f32, y: f32, rx: f32, ry: f32, thickness: f32, color: Color) {
        let p = self.p(x, y);
        draw_ellipse_lines(
            p.x,
            p.y,
            self.s(rx),
            self.s(ry),
            0.0,
            self.s(thickness),
            color,
        );
    }

    /// A rectangle by its top left corner and size.
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let p = self.p(x, y);
        draw_rectangle(p.x, p.y, self.s(w), self.s(h), color);
    }

    fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        let p = self.p(x, y);
        draw_rectangle_lines(p.x, p.y, self.s(w), self.s(h), self.s(thickness), color);
    }

    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let (a, b) = (self.p(x1, y1), self.p(x2, y2));
        draw_line(a.x, a.y, b.x, b.y, self.s(thickness), color);
    }

    fn tri(&self, a: Pt, b: Pt, c: Pt, color: Color) {
        draw_triangle(self.p(a.0, a.1), self.p(b.0, b.1), self.p(c.0, c.1), color);
    }

    /// Fills a convex polygon.
    fn poly(&self, points: &[Pt], color: Color) {
        for i in 1..points.len().saturating_sub(1) {
            self.tri(points[0], points[i], points[i + 1], color);
        }
    }

    fn polyline(&self, points: &[Pt], thickness: f32, color: Color) {
        for pair in points.windows(2) {
            self.line(pair[0].0, pair[0].1, pair[1].0, pair[1].1, thickness, color);
            // Round joints keep thick lines from cracking at the bends.
            self.circle(pair[1].0, pair[1].1, thickness / 2.0, color);
        }
    }

    /// Points on a circular arc from `from` to `to` degrees, clockwise
    /// on screen, with 0 pointing right.
    fn arc_points(x: f32, y: f32, r: f32, from: f32, to: f32) -> Vec<Pt> {
        const STEPS: usize = 16;
        (0..=STEPS)
            .map(|i| {
                let a = (from + (to - from) * i as f32 / STEPS as f32).to_radians();
                (x + r * a.cos(), y + r * a.sin())
            })
            .collect()
    }

    // Mirrors macroquad's own drawing functions, which take many numbers.
    #[allow(clippy::too_many_arguments)]
    fn arc(&self, x: f32, y: f32, r: f32, from: f32, to: f32, thickness: f32, color: Color) {
        self.polyline(&Self::arc_points(x, y, r, from, to), thickness, color);
    }

    /// A filled circular sector.
    fn pie(&self, x: f32, y: f32, r: f32, from: f32, to: f32, color: Color) {
        let points = Self::arc_points(x, y, r, from, to);
        for pair in points.windows(2) {
            self.tri((x, y), pair[0], pair[1], color);
        }
    }

    /// Wavy water filling the bottom of the picture from `top` down.
    fn water(&self, top: f32, time: f32) {
        self.rect(-1.0, top, 2.0, 1.0 - top, WATER);
        for i in 0..5 {
            let x = -0.9 + i as f32 * 0.45;
            let bob = (time * 2.0 + i as f32).sin() * 0.03;
            self.arc(x, top + bob, 0.12, 200.0, 340.0, 0.05, WATER_LIGHT);
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

const TILE: Color = rgb(34, 36, 52);
const OUTLINE: Color = rgb(20, 20, 28);
const SKIN: Color = rgb(250, 212, 180);
const SKIN_DARK: Color = rgb(220, 170, 135);
const WOOD: Color = rgb(150, 100, 55);
const WOOD_DARK: Color = rgb(105, 68, 35);
const STEEL: Color = rgb(185, 190, 200);
const STEEL_DARK: Color = rgb(120, 125, 135);
const GOLD_: Color = rgb(235, 190, 60);
const WATER: Color = rgb(45, 105, 190);
const WATER_LIGHT: Color = rgb(140, 190, 240);
const GRASS: Color = rgb(70, 150, 60);
const LEAF: Color = rgb(60, 160, 70);
const LEAF_DARK: Color = rgb(35, 110, 45);
const SNOW: Color = rgb(240, 245, 250);
const STONE: Color = rgb(140, 140, 150);
const STONE_DARK: Color = rgb(95, 95, 105);
const RED: Color = rgb(215, 50, 50);
const BLACK_: Color = rgb(25, 25, 30);
const WHITE_: Color = rgb(245, 245, 245);
const SMOKE: Color = rgba(220, 220, 230, 0.6);

type Painter = fn(&Canvas, f32);

/// Every word's picture, by number.
const PICTURES: &[(&str, Painter)] = &[
    ("0", hai),
    ("1", jaa),
    ("2", kuu),
    ("3", luu),
    ("4", maa),
    ("5", puu),
    ("6", rae),
    ("7", suu),
    ("8", tai),
    ("9", vyo),
    ("00", hiha),
    ("01", hajy),
    ("02", hauki),
    ("03", huilu),
    ("04", haamu),
    ("05", huopa),
    ("06", hiiri),
    ("07", hius),
    ("08", hauta),
    ("09", haavi),
    ("10", jauho),
    ("11", jojo),
    ("12", joki),
    ("13", joulu),
    ("14", juomu),
    ("15", jopo),
    ("16", juuri),
    ("17", jousi),
    ("18", jeti),
    ("19", jyva),
    ("20", koho),
    ("21", koju),
    ("22", keko),
    ("23", kela),
    ("24", kuomu),
    ("25", kupu),
    ("26", koira),
    ("27", kaasu),
    ("28", kota),
    ("29", kavio),
    ("30", liha),
    ("31", leija),
    ("32", leka),
    ("33", luola),
    ("34", liima),
    ("35", lapio),
    ("36", lyyra),
    ("37", liesi),
    ("38", luoti),
    ("39", laiva),
    ("40", maha),
    ("41", maja),
    ("42", muki),
    ("43", mela),
    ("44", muumio),
    ("45", mopo),
    ("46", muuri),
    ("47", muusi),
    ("48", mato),
    ("49", muovi),
    ("50", pyyhe),
    ("51", poiju),
    ("52", puku),
    ("53", peili),
    ("54", piima),
    ("55", pipo),
    ("56", pora),
    ("57", paasi),
    ("58", pata),
    ("59", paavi),
    ("60", raha),
    ("61", ryijy),
    ("62", reki),
    ("63", railo),
    ("64", riimu),
    ("65", rapu),
    ("66", ruori),
    ("67", ruusu),
    ("68", rata),
    ("69", rovio),
    ("70", saha),
    ("71", soija),
    ("72", sika),
    ("73", siili),
    ("74", siima),
    ("75", siipi),
    ("76", siru),
    ("77", susi),
    ("78", sota),
    ("79", sauva),
    ("80", tuohi),
    ("81", taiji),
    ("82", tiuku),
    ("83", tiili),
    ("84", taimi),
    ("85", tipu),
    ("86", tera),
    ("87", teesi),
    ("88", toti),
    ("89", tavi),
    ("90", vuohi),
    ("91", vaja),
    ("92", vaaka),
    ("93", viulu),
    ("94", vaimo),
    ("95", vapa),
    ("96", vuori),
    ("97", vaasi),
    ("98", vouti),
    ("99", vauva),
];

/// Draws the picture of the word for `number` on a tile centered on
/// `center`, `size` pixels across. Returns false if there is none.
pub fn draw_picture(number: &str, center: Vec2, size: f32, time: f32) -> bool {
    let Some((_, paint)) = PICTURES.iter().find(|(n, _)| *n == number) else {
        return false;
    };
    let half = size / 2.0;
    draw_rectangle(center.x - half, center.y - half, size, size, TILE);
    draw_rectangle_lines(center.x - half, center.y - half, size, size, 1.0, OUTLINE);
    // Pictures stay a little inside the tile's edges.
    paint(
        &Canvas {
            center,
            half: half * 0.88,
        },
        time,
    );
    true
}

// ---------------------------------------------------------------------
// 0-9

fn hai(c: &Canvas, _: f32) {
    let body = rgb(110, 125, 145);
    c.tri((-0.55, 0.1), (-1.0, -0.35), (-0.95, 0.5), body);
    c.tri((-0.1, -0.2), (0.25, -0.2), (0.0, -0.75), body);
    c.ellipse(0.05, 0.1, 0.75, 0.32, 0.0, body);
    c.ellipse(0.1, 0.22, 0.6, 0.15, 0.0, rgb(225, 230, 235));
    c.circle(0.5, 0.0, 0.06, BLACK_);
    c.line(0.35, 0.2, 0.72, 0.18, 0.04, BLACK_);
    for i in 0..3 {
        let x = 0.42 + i as f32 * 0.1;
        c.tri((x, 0.19), (x + 0.08, 0.19), (x + 0.04, 0.27), WHITE_);
    }
}

fn jaa(c: &Canvas, _: f32) {
    c.poly(
        &[(-0.6, -0.3), (-0.2, -0.65), (0.65, -0.65), (0.25, -0.3)],
        rgb(210, 240, 255),
    );
    c.poly(
        &[(0.25, -0.3), (0.65, -0.65), (0.65, 0.3), (0.25, 0.65)],
        rgb(120, 180, 225),
    );
    c.rect(-0.6, -0.3, 0.85, 0.95, rgb(170, 215, 245));
    c.line(-0.45, -0.1, -0.3, -0.2, 0.06, WHITE_);
    c.line(-0.45, 0.05, -0.1, -0.2, 0.06, WHITE_);
}

fn kuu(c: &Canvas, _: f32) {
    c.circle(0.0, 0.0, 0.7, rgb(250, 225, 110));
    c.circle(0.35, -0.2, 0.62, TILE);
    for (x, y) in [(0.55, 0.45), (0.7, -0.6), (-0.6, -0.7)] {
        c.circle(x, y, 0.05, WHITE_);
    }
}

fn luu(c: &Canvas, _: f32) {
    let bone = rgb(240, 235, 215);
    c.line(-0.5, 0.5, 0.5, -0.5, 0.3, bone);
    for (x, y) in [(-0.7, 0.45), (-0.45, 0.7), (0.45, -0.7), (0.7, -0.45)] {
        c.circle(x, y, 0.2, bone);
    }
}

fn maa(c: &Canvas, _: f32) {
    c.circle(0.0, 0.0, 0.8, rgb(50, 110, 200));
    c.circle(-0.3, -0.25, 0.3, GRASS);
    c.circle(-0.1, -0.4, 0.25, GRASS);
    c.circle(0.35, 0.3, 0.28, GRASS);
    c.circle(0.15, 0.45, 0.2, GRASS);
    c.ring(0.0, 0.0, 0.8, 0.05, rgb(200, 230, 255));
}

fn puu(c: &Canvas, _: f32) {
    c.rect(-0.12, 0.1, 0.24, 0.8, WOOD);
    c.circle(0.0, -0.3, 0.5, LEAF);
    c.circle(-0.35, 0.0, 0.35, LEAF);
    c.circle(0.35, 0.0, 0.35, LEAF);
    c.circle(-0.15, -0.45, 0.2, rgb(110, 200, 100));
}

fn rae(c: &Canvas, time: f32) {
    let cloud = rgb(150, 155, 170);
    c.circle(-0.35, -0.45, 0.3, cloud);
    c.circle(0.1, -0.55, 0.38, cloud);
    c.circle(0.45, -0.4, 0.28, cloud);
    c.rect(-0.35, -0.45, 0.8, 0.28, cloud);
    for i in 0..5 {
        let x = -0.5 + i as f32 * 0.25;
        let y = 0.05 + ((time * 0.8 + i as f32 * 0.37) % 1.0) * 0.8;
        c.circle(x, y, 0.09, WHITE_);
    }
}

fn suu(c: &Canvas, _: f32) {
    c.ellipse(0.0, 0.0, 0.8, 0.5, 0.0, rgb(210, 60, 80));
    c.ellipse(0.0, 0.05, 0.6, 0.3, 0.0, rgb(90, 20, 35));
    c.rect(-0.4, -0.22, 0.8, 0.14, WHITE_);
    c.ellipse(0.0, 0.2, 0.3, 0.12, 0.0, rgb(240, 120, 140));
}

fn tai(c: &Canvas, _: f32) {
    let body = rgb(185, 160, 120);
    for (i, y) in [-0.2, 0.05, 0.3].iter().enumerate() {
        let spread = 0.55 + i as f32 * 0.05;
        c.line(-spread, y - 0.1, spread, y + 0.1, 0.05, rgb(90, 70, 50));
        c.line(spread, y - 0.1, -spread, y + 0.1, 0.05, rgb(90, 70, 50));
    }
    c.ellipse(0.0, 0.2, 0.3, 0.45, 0.0, body);
    c.circle(0.0, -0.4, 0.2, body);
    c.circle(-0.08, -0.45, 0.04, BLACK_);
    c.circle(0.08, -0.45, 0.04, BLACK_);
    c.line(-0.05, -0.55, -0.2, -0.8, 0.03, rgb(90, 70, 50));
    c.line(0.05, -0.55, 0.2, -0.8, 0.03, rgb(90, 70, 50));
}

fn vyo(c: &Canvas, _: f32) {
    c.rect(-1.0, -0.2, 2.0, 0.4, rgb(120, 70, 35));
    for x in [0.45, 0.62, 0.79] {
        c.circle(x, 0.0, 0.04, rgb(60, 35, 15));
    }
    c.rect_lines(-0.35, -0.35, 0.45, 0.7, 0.1, GOLD_);
    c.line(-0.12, -0.3, -0.12, 0.3, 0.06, GOLD_);
}

// ---------------------------------------------------------------------
// 00-09

fn hiha(c: &Canvas, _: f32) {
    let cloth = rgb(70, 110, 200);
    c.poly(
        &[(-0.8, -0.8), (-0.1, -0.8), (0.55, 0.35), (0.15, 0.6)],
        cloth,
    );
    c.poly(
        &[(0.15, 0.6), (0.55, 0.35), (0.7, 0.55), (0.3, 0.8)],
        WHITE_,
    );
    c.circle(0.6, 0.75, 0.15, SKIN);
    c.line(-0.45, -0.8, 0.3, 0.45, 0.03, rgb(50, 80, 160));
}

fn hajy(c: &Canvas, _: f32) {
    let coat = rgb(35, 35, 45);
    // Dark coat over a white shirt, and a belt with a puukko.
    c.poly(
        &[(-0.7, 1.0), (-0.55, 0.05), (0.55, 0.05), (0.7, 1.0)],
        coat,
    );
    c.tri((-0.18, 0.05), (0.18, 0.05), (0.0, 0.45), WHITE_);
    c.rect(-0.62, 0.6, 1.24, 0.1, rgb(90, 55, 25));
    c.poly(
        &[(0.25, 0.65), (0.4, 0.65), (0.38, 0.98), (0.3, 1.0)],
        rgb(60, 40, 20),
    );
    c.rect(0.27, 0.45, 0.12, 0.2, WOOD);
    // Head with a grim face and a black hat.
    c.circle(0.0, -0.3, 0.32, SKIN);
    c.line(-0.22, -0.42, -0.05, -0.34, 0.06, BLACK_);
    c.line(0.22, -0.42, 0.05, -0.34, 0.06, BLACK_);
    c.circle(-0.12, -0.3, 0.04, BLACK_);
    c.circle(0.12, -0.3, 0.04, BLACK_);
    c.poly(
        &[(-0.22, -0.12), (0.22, -0.12), (0.15, -0.05), (-0.15, -0.05)],
        rgb(80, 50, 25),
    );
    c.rect(-0.55, -0.62, 1.1, 0.1, BLACK_);
    c.rect(-0.3, -0.95, 0.6, 0.35, BLACK_);
}

fn hauki(c: &Canvas, _: f32) {
    let body = rgb(90, 140, 70);
    c.tri((-0.6, 0.0), (-1.0, -0.35), (-1.0, 0.35), body);
    c.poly(
        &[
            (-0.65, -0.18),
            (0.55, -0.2),
            (0.95, 0.0),
            (0.55, 0.2),
            (-0.65, 0.18),
        ],
        body,
    );
    c.ellipse(0.0, 0.08, 0.6, 0.08, 0.0, rgb(200, 215, 150));
    for (x, y) in [(-0.3, -0.08), (0.0, -0.1), (0.25, -0.06), (-0.1, 0.02)] {
        c.circle(x, y, 0.05, rgb(200, 210, 120));
    }
    c.circle(0.62, -0.06, 0.05, BLACK_);
    c.line(0.7, 0.07, 0.95, 0.0, 0.03, BLACK_);
}

fn huilu(c: &Canvas, _: f32) {
    c.line(-0.85, 0.5, 0.85, -0.5, 0.16, STEEL);
    c.line(-0.85, 0.5, 0.85, -0.5, 0.05, WHITE_);
    for i in 0..6 {
        let t = -0.5 + i as f32 * 0.2;
        c.circle(t * 0.85 * 1.2, -t * 0.5 * 1.2, 0.045, STEEL_DARK);
    }
}

fn haamu(c: &Canvas, time: f32) {
    let y = (time * 2.0).sin() * 0.05;
    c.circle(0.0, -0.25 + y, 0.5, WHITE_);
    c.rect(-0.5, -0.25 + y, 1.0, 0.75, WHITE_);
    for i in 0..4 {
        c.circle(-0.375 + i as f32 * 0.25, 0.5 + y, 0.125, WHITE_);
    }
    c.ellipse(-0.18, -0.25 + y, 0.08, 0.13, 0.0, BLACK_);
    c.ellipse(0.18, -0.25 + y, 0.08, 0.13, 0.0, BLACK_);
    c.ellipse(0.0, 0.05 + y, 0.1, 0.12, 0.0, BLACK_);
}

fn huopa(c: &Canvas, _: f32) {
    c.rect(-0.8, -0.6, 1.6, 1.1, rgb(190, 60, 60));
    for x in [-0.4, 0.0, 0.4] {
        c.rect(x - 0.05, -0.6, 0.1, 1.1, rgb(230, 170, 60));
    }
    for y in [-0.25, 0.15] {
        c.rect(-0.8, y - 0.05, 1.6, 0.1, rgb(230, 170, 60));
    }
    for i in 0..9 {
        let x = -0.75 + i as f32 * 0.19;
        c.line(x, 0.5, x, 0.72, 0.04, rgb(230, 200, 150));
    }
}

fn hiiri(c: &Canvas, _: f32) {
    let fur = rgb(150, 150, 160);
    c.arc(-0.5, 0.3, 0.35, 90.0, 300.0, 0.05, rgb(220, 150, 160));
    c.ellipse(0.0, 0.25, 0.55, 0.35, 0.0, fur);
    c.circle(0.45, -0.05, 0.3, fur);
    c.circle(0.35, -0.35, 0.18, fur);
    c.circle(0.35, -0.35, 0.1, rgb(240, 170, 180));
    c.circle(0.55, -0.1, 0.05, BLACK_);
    c.circle(0.75, 0.0, 0.06, rgb(240, 130, 150));
}

fn hius(c: &Canvas, _: f32) {
    let points: Vec<Pt> = (0..=20)
        .map(|i| {
            let t = i as f32 / 20.0;
            (-0.8 + 1.6 * t, 0.5 - t * 0.9 + (t * 9.0).sin() * 0.15)
        })
        .collect();
    c.polyline(&points, 0.05, rgb(110, 70, 35));
    c.ellipse(-0.8, 0.5, 0.06, 0.09, 30.0, rgb(240, 225, 210));
}

fn hauta(c: &Canvas, _: f32) {
    c.ellipse(0.0, 0.75, 0.9, 0.25, 0.0, GRASS);
    c.circle(0.0, -0.3, 0.45, STONE);
    c.rect(-0.45, -0.3, 0.9, 1.0, STONE);
    c.rect(-0.45, 0.62, 0.9, 0.08, STONE_DARK);
    c.line(0.0, -0.45, 0.0, 0.25, 0.1, STONE_DARK);
    c.line(-0.2, -0.25, 0.2, -0.25, 0.1, STONE_DARK);
}

fn haavi(c: &Canvas, _: f32) {
    c.line(0.25, 0.1, 0.95, 0.9, 0.1, WOOD);
    c.tri(
        (-0.75, -0.25),
        (0.25, -0.25),
        (-0.35, 0.65),
        rgba(220, 220, 230, 0.3),
    );
    for i in 0..5 {
        let x = -0.7 + i as f32 * 0.24;
        c.line(x, -0.25, -0.35, 0.65, 0.02, rgb(200, 200, 210));
    }
    c.ellipse_lines(-0.25, -0.25, 0.5, 0.2, 0.06, STEEL);
}

// ---------------------------------------------------------------------
// 10-19

fn jauho(c: &Canvas, _: f32) {
    let sack = rgb(215, 190, 145);
    c.ellipse(0.0, 0.8, 0.8, 0.15, 0.0, WHITE_);
    c.poly(
        &[(-0.55, -0.45), (0.55, -0.45), (0.65, 0.75), (-0.65, 0.75)],
        sack,
    );
    c.poly(
        &[(-0.4, -0.45), (0.4, -0.45), (0.3, -0.75), (-0.3, -0.75)],
        sack,
    );
    c.rect(-0.45, -0.52, 0.9, 0.1, rgb(150, 110, 60));
    // A wheat ear printed on the sack.
    c.line(0.0, 0.55, 0.0, -0.1, 0.04, rgb(170, 120, 40));
    for i in 0..3 {
        let y = 0.3 - i as f32 * 0.17;
        c.ellipse(-0.1, y, 0.1, 0.05, -40.0, rgb(200, 150, 50));
        c.ellipse(0.1, y, 0.1, 0.05, 40.0, rgb(200, 150, 50));
    }
}

fn jojo(c: &Canvas, time: f32) {
    let y = 0.25 + (time * 3.0).sin() * 0.12;
    c.line(0.0, -0.85, 0.0, y, 0.03, WHITE_);
    c.ring(0.0, -0.85, 0.08, 0.03, WHITE_);
    c.circle(0.0, y, 0.45, rgb(220, 50, 60));
    c.ring(0.0, y, 0.3, 0.06, rgb(250, 120, 120));
    c.circle(0.0, y, 0.1, rgb(250, 230, 90));
}

fn joki(c: &Canvas, time: f32) {
    c.rect(-1.0, -1.0, 2.0, 2.0, GRASS);
    let points: Vec<Pt> = (0..=16)
        .map(|i| {
            let t = i as f32 / 16.0;
            (((t * 6.0).sin()) * 0.45, -1.0 + 2.0 * t)
        })
        .collect();
    c.polyline(&points, 0.4, WATER);
    for (i, p) in points.iter().enumerate().step_by(4) {
        let shift = (time * 2.0 + i as f32).sin() * 0.05;
        c.line(
            p.0 - 0.08 + shift,
            p.1,
            p.0 + 0.08 + shift,
            p.1,
            0.03,
            WATER_LIGHT,
        );
    }
}

fn joulu(c: &Canvas, time: f32) {
    c.rect(-0.1, 0.65, 0.2, 0.25, WOOD);
    for (i, (w, y)) in [(0.75, 0.7), (0.6, 0.3), (0.45, -0.1)].iter().enumerate() {
        let top = y - 0.55 + i as f32 * 0.02;
        c.tri((-w, *y), (*w, *y), (0.0, top), LEAF_DARK);
    }
    for (x, y, color) in [
        (-0.3, 0.5, RED),
        (0.25, 0.2, rgb(80, 150, 240)),
        (-0.1, 0.05, GOLD_),
        (0.35, 0.55, GOLD_),
    ] {
        c.circle(x, y, 0.07, color);
    }
    let twinkle = 0.12 + 0.03 * (time * 5.0).sin();
    c.circle(0.0, -0.65, twinkle, rgb(255, 230, 90));
}

fn juomu(c: &Canvas, _: f32) {
    c.rect(-0.85, -0.85, 1.7, 1.7, rgb(240, 150, 40));
    for i in 0..4 {
        let y = -0.65 + i as f32 * 0.42;
        c.tri(
            (-0.85, y - 0.12),
            (-0.85, y + 0.12),
            (0.05, y + 0.05),
            BLACK_,
        );
        c.tri((0.85, y + 0.05), (0.85, y + 0.25), (0.1, y + 0.2), BLACK_);
    }
}

fn jopo(c: &Canvas, _: f32) {
    let frame = rgb(40, 190, 190);
    c.ring(-0.5, 0.35, 0.38, 0.07, BLACK_);
    c.ring(0.55, 0.35, 0.38, 0.07, BLACK_);
    // A low, step-through frame.
    c.polyline(
        &[(-0.5, 0.35), (-0.15, 0.45), (0.3, 0.45), (0.55, 0.35)],
        0.09,
        frame,
    );
    c.line(-0.15, 0.45, -0.3, -0.25, 0.09, frame);
    c.line(0.3, 0.45, 0.45, -0.35, 0.09, frame);
    c.line(0.45, -0.35, 0.55, 0.35, 0.07, frame);
    c.ellipse(-0.32, -0.3, 0.18, 0.06, 0.0, BLACK_);
    c.line(0.3, -0.4, 0.6, -0.4, 0.07, BLACK_);
}

fn juuri(c: &Canvas, _: f32) {
    c.rect(-1.0, -1.0, 2.0, 0.55, rgb(120, 190, 240));
    c.rect(-1.0, -0.45, 2.0, 1.45, rgb(110, 75, 45));
    c.line(0.0, -0.45, 0.0, -0.75, 0.06, LEAF);
    c.ellipse(-0.12, -0.75, 0.12, 0.06, -30.0, LEAF);
    c.ellipse(0.12, -0.75, 0.12, 0.06, 30.0, LEAF);
    let root = rgb(230, 205, 160);
    c.polyline(
        &[(0.0, -0.45), (0.05, 0.1), (-0.05, 0.5), (0.02, 0.85)],
        0.08,
        root,
    );
    c.polyline(&[(0.03, -0.1), (-0.35, 0.2), (-0.6, 0.25)], 0.05, root);
    c.polyline(&[(0.04, 0.1), (0.4, 0.35), (0.55, 0.6)], 0.05, root);
    c.polyline(&[(-0.02, 0.45), (-0.3, 0.7)], 0.04, root);
}

fn jousi(c: &Canvas, _: f32) {
    c.arc(0.55, 0.0, 0.95, 130.0, 230.0, 0.1, WOOD);
    c.line(-0.07, -0.72, -0.07, 0.72, 0.025, WHITE_);
    c.line(-0.07, 0.0, 0.95, 0.0, 0.05, WOOD_DARK);
    c.tri((0.95, -0.1), (0.95, 0.1), (1.0, 0.0), STEEL);
    c.tri((-0.07, 0.0), (0.1, -0.12), (0.15, 0.0), RED);
    c.tri((-0.07, 0.0), (0.1, 0.12), (0.15, 0.0), RED);
}

fn jeti(c: &Canvas, _: f32) {
    let fur = rgb(235, 240, 245);
    c.ellipse(0.0, 0.35, 0.6, 0.55, 0.0, fur);
    c.ellipse(-0.6, 0.3, 0.18, 0.4, 20.0, fur);
    c.ellipse(0.6, 0.3, 0.18, 0.4, -20.0, fur);
    c.circle(0.0, -0.45, 0.4, fur);
    c.ellipse(0.0, -0.4, 0.26, 0.22, 0.0, rgb(150, 170, 200));
    c.circle(-0.1, -0.45, 0.05, BLACK_);
    c.circle(0.1, -0.45, 0.05, BLACK_);
    c.rect(-0.1, -0.3, 0.2, 0.06, BLACK_);
}

fn jyva(c: &Canvas, _: f32) {
    let grain = rgb(225, 180, 70);
    c.line(0.0, 0.95, 0.0, -0.6, 0.05, rgb(190, 150, 60));
    for i in 0..5 {
        let y = 0.4 - i as f32 * 0.22;
        c.ellipse(-0.14, y, 0.16, 0.08, -50.0, grain);
        c.ellipse(0.14, y, 0.16, 0.08, 50.0, grain);
    }
    c.ellipse(0.0, -0.7, 0.08, 0.16, 0.0, grain);
}

// ---------------------------------------------------------------------
// 20-29

fn koho(c: &Canvas, time: f32) {
    let y = (time * 2.0).sin() * 0.05;
    c.water(0.3, time);
    c.line(0.0, -0.9 + y, 0.0, -0.3 + y, 0.06, rgb(240, 200, 50));
    c.pie(0.0, 0.05 + y, 0.35, 180.0, 360.0, RED);
    c.pie(0.0, 0.05 + y, 0.35, 0.0, 180.0, WHITE_);
    c.line(0.0, 0.4 + y, 0.0, 0.9, 0.02, rgb(200, 200, 200));
}

fn koju(c: &Canvas, _: f32) {
    c.rect(-0.7, -0.25, 0.08, 1.1, WOOD_DARK);
    c.rect(0.62, -0.25, 0.08, 1.1, WOOD_DARK);
    c.rect(-0.8, 0.3, 1.6, 0.5, WOOD);
    for i in 0..6 {
        let x = -0.9 + i as f32 * 0.3;
        let color = if i % 2 == 0 { RED } else { WHITE_ };
        c.rect(x, -0.7, 0.3, 0.45, color);
        c.circle(x + 0.15, -0.25, 0.15, color);
    }
    for (x, color) in [
        (-0.4, RED),
        (-0.1, rgb(250, 200, 40)),
        (0.2, GRASS),
        (0.45, rgb(250, 140, 40)),
    ] {
        c.circle(x, 0.25, 0.1, color);
    }
}

fn keko(c: &Canvas, _: f32) {
    c.pie(0.0, 0.7, 0.85, 180.0, 360.0, rgb(140, 95, 50));
    for i in 0..14 {
        let a = (190.0 + i as f32 * 11.0).to_radians();
        let r = 0.2 + (i % 4) as f32 * 0.15;
        let (x, y) = (r * a.cos(), 0.7 + r * a.sin());
        c.line(x, y, x + 0.1, y - 0.06, 0.03, rgb(90, 60, 30));
    }
    for (x, y) in [(-0.3, 0.2), (0.25, 0.35), (0.5, 0.6), (-0.6, 0.6)] {
        c.circle(x, y, 0.05, BLACK_);
        c.circle(x + 0.07, y, 0.04, BLACK_);
    }
}

fn kela(c: &Canvas, _: f32) {
    c.rect(-0.35, -0.5, 0.7, 1.0, rgb(60, 130, 220));
    for i in 0..8 {
        let y = -0.45 + i as f32 * 0.13;
        c.line(-0.35, y, 0.35, y + 0.04, 0.02, rgb(110, 170, 240));
    }
    c.rect(-0.55, -0.75, 1.1, 0.25, WOOD);
    c.rect(-0.55, 0.5, 1.1, 0.25, WOOD);
    c.polyline(
        &[(0.35, 0.1), (0.7, 0.3), (0.85, 0.75)],
        0.03,
        rgb(60, 130, 220),
    );
}

fn kuomu(c: &Canvas, _: f32) {
    let body = rgb(60, 80, 160);
    c.pie(0.0, -0.05, 0.7, 0.0, 180.0, body);
    c.pie(-0.05, -0.05, 0.65, 180.0, 270.0, rgb(40, 55, 120));
    for a in [200.0f32, 225.0, 250.0] {
        let r = a.to_radians();
        c.line(
            -0.05,
            -0.05,
            -0.05 + 0.65 * r.cos(),
            -0.05 + 0.65 * r.sin(),
            0.03,
            rgb(90, 110, 190),
        );
    }
    c.line(0.65, -0.05, 0.95, -0.55, 0.06, BLACK_);
    c.circle(-0.4, 0.75, 0.16, BLACK_);
    c.circle(0.4, 0.75, 0.16, BLACK_);
}

fn kupu(c: &Canvas, _: f32) {
    c.rect(-0.7, 0.6, 1.4, 0.2, WOOD);
    c.line(0.0, 0.6, 0.0, 0.2, 0.04, LEAF_DARK);
    c.circle(0.0, 0.1, 0.15, RED);
    let glass = rgba(180, 220, 250, 0.3);
    c.circle(0.0, -0.15, 0.55, glass);
    c.rect(-0.55, -0.15, 1.1, 0.75, glass);
    c.arc(0.0, -0.15, 0.55, 180.0, 360.0, 0.04, WATER_LIGHT);
    c.line(-0.55, -0.15, -0.55, 0.6, 0.04, WATER_LIGHT);
    c.line(0.55, -0.15, 0.55, 0.6, 0.04, WATER_LIGHT);
    c.circle(0.0, -0.75, 0.08, WATER_LIGHT);
    c.line(-0.3, -0.35, -0.35, 0.2, 0.05, rgba(255, 255, 255, 0.6));
}

fn koira(c: &Canvas, _: f32) {
    let fur = rgb(190, 130, 70);
    c.ellipse(-0.55, -0.1, 0.22, 0.45, 15.0, rgb(120, 75, 35));
    c.ellipse(0.55, -0.1, 0.22, 0.45, -15.0, rgb(120, 75, 35));
    c.circle(0.0, -0.1, 0.55, fur);
    c.ellipse(0.0, 0.25, 0.32, 0.25, 0.0, rgb(240, 215, 180));
    c.circle(-0.2, -0.2, 0.07, BLACK_);
    c.circle(0.2, -0.2, 0.07, BLACK_);
    c.ellipse(0.0, 0.12, 0.12, 0.08, 0.0, BLACK_);
    c.ellipse(0.0, 0.45, 0.08, 0.12, 0.0, rgb(240, 110, 130));
}

fn kaasu(c: &Canvas, time: f32) {
    let gas = rgba(170, 220, 80, 0.7);
    for (i, (x, y, r)) in [
        (-0.4, 0.1, 0.35),
        (0.1, -0.15, 0.45),
        (0.45, 0.2, 0.35),
        (0.0, 0.35, 0.35),
    ]
    .iter()
    .enumerate()
    {
        let puff = r + (time * 1.5 + i as f32).sin() * 0.03;
        c.circle(*x, *y, puff, gas);
    }
    c.arc(-0.1, 0.0, 0.2, 0.0, 270.0, 0.04, rgb(100, 150, 40));
    c.arc(0.35, 0.2, 0.12, 90.0, 360.0, 0.04, rgb(100, 150, 40));
}

fn kota(c: &Canvas, time: f32) {
    let smoke = (time * 1.5).sin() * 0.05;
    c.circle(0.1 + smoke, -0.85, 0.08, SMOKE);
    c.circle(0.2 - smoke, -0.95, 0.06, SMOKE);
    c.line(-0.15, -0.85, 0.1, -0.6, 0.04, WOOD_DARK);
    c.line(0.15, -0.85, -0.1, -0.6, 0.04, WOOD_DARK);
    c.tri((0.0, -0.65), (-0.85, 0.8), (0.85, 0.8), rgb(140, 100, 60));
    c.tri((0.0, 0.2), (-0.25, 0.8), (0.25, 0.8), rgb(60, 40, 25));
    c.line(-0.4, 0.1, 0.4, 0.1, 0.03, rgb(100, 70, 40));
}

fn kavio(c: &Canvas, _: f32) {
    let leg = rgb(140, 90, 50);
    c.rect(-0.25, -1.0, 0.5, 1.0, leg);
    c.ellipse(0.0, 0.0, 0.35, 0.2, 0.0, rgb(230, 225, 210));
    c.poly(
        &[(-0.35, 0.05), (0.35, 0.05), (0.55, 0.75), (-0.55, 0.75)],
        rgb(60, 55, 55),
    );
    c.rect(-0.6, 0.72, 1.2, 0.12, STEEL);
}

// ---------------------------------------------------------------------
// 30-39

fn liha(c: &Canvas, _: f32) {
    let bone = rgb(245, 240, 225);
    c.line(0.2, -0.2, 0.75, -0.75, 0.16, bone);
    c.circle(0.72, -0.88, 0.12, bone);
    c.circle(0.88, -0.72, 0.12, bone);
    c.circle(-0.1, 0.1, 0.62, rgb(245, 225, 210));
    c.circle(-0.12, 0.12, 0.54, rgb(190, 70, 60));
    c.ellipse(-0.25, 0.0, 0.18, 0.08, -30.0, rgb(220, 120, 110));
}

fn leija(c: &Canvas, time: f32) {
    let sway = (time * 1.5).sin() * 0.05;
    let (top, left, right, bottom) = ((0.0, -0.9), (-0.55, -0.2), (0.55, -0.2), (0.0, 0.45));
    c.tri(top, left, (0.0, -0.2), RED);
    c.tri(top, right, (0.0, -0.2), rgb(250, 200, 40));
    c.tri(left, bottom, (0.0, -0.2), rgb(60, 130, 230));
    c.tri(right, bottom, (0.0, -0.2), GRASS);
    let tail: Vec<Pt> = (0..=6)
        .map(|i| {
            let t = i as f32 / 6.0;
            (((t * 7.0) + time * 3.0).sin() * 0.12 + sway, 0.45 + t * 0.5)
        })
        .collect();
    c.polyline(&tail, 0.03, WHITE_);
    for p in tail.iter().skip(2).step_by(2) {
        c.tri((p.0 - 0.08, p.1 - 0.05), (p.0 - 0.08, p.1 + 0.05), *p, RED);
        c.tri((p.0 + 0.08, p.1 - 0.05), (p.0 + 0.08, p.1 + 0.05), *p, RED);
    }
}

fn leka(c: &Canvas, _: f32) {
    c.line(-0.7, 0.85, 0.3, -0.2, 0.14, WOOD);
    c.poly(
        &[(0.0, -0.75), (0.45, -0.3), (0.2, -0.05), (-0.25, -0.5)],
        STEEL_DARK,
    );
    c.poly(
        &[(0.45, -0.3), (0.55, -0.4), (0.1, -0.85), (0.0, -0.75)],
        STEEL,
    );
}

fn luola(c: &Canvas, _: f32) {
    let rock = rgb(120, 110, 100);
    c.circle(-0.4, 0.2, 0.55, rock);
    c.circle(0.35, 0.15, 0.6, rock);
    c.circle(0.0, -0.25, 0.55, rock);
    c.rect(-0.95, 0.2, 1.9, 0.7, rock);
    c.circle(0.0, 0.35, 0.35, BLACK_);
    c.rect(-0.35, 0.35, 0.7, 0.55, BLACK_);
    for x in [-0.2, 0.0, 0.2] {
        c.tri((x - 0.06, 0.02), (x + 0.06, 0.02), (x, 0.2), rock);
    }
}

fn liima(c: &Canvas, _: f32) {
    c.rect(-0.35, -0.3, 0.7, 1.1, WHITE_);
    c.rect(-0.35, 0.05, 0.7, 0.4, rgb(60, 130, 220));
    c.tri((-0.25, -0.3), (0.25, -0.3), (0.0, -0.8), rgb(250, 140, 40));
    c.ellipse(0.15, -0.88, 0.07, 0.1, 0.0, rgba(250, 250, 230, 0.9));
}

fn lapio(c: &Canvas, _: f32) {
    c.line(-0.6, -0.75, 0.2, 0.15, 0.1, WOOD);
    c.line(-0.8, -0.55, -0.45, -0.9, 0.1, WOOD_DARK);
    c.poly(
        &[
            (0.1, 0.0),
            (0.4, -0.25),
            (0.85, 0.35),
            (0.7, 0.8),
            (0.3, 0.55),
        ],
        STEEL,
    );
}

fn lyyra(c: &Canvas, _: f32) {
    c.arc(-0.1, -0.1, 0.55, 90.0, 250.0, 0.12, GOLD_);
    c.arc(0.1, -0.1, 0.55, 290.0, 450.0, 0.12, GOLD_);
    c.line(-0.45, 0.45, 0.45, 0.45, 0.14, GOLD_);
    c.line(-0.45, -0.55, 0.45, -0.55, 0.08, GOLD_);
    for i in 0..5 {
        let x = -0.28 + i as f32 * 0.14;
        c.line(x, -0.55, x, 0.45, 0.02, WHITE_);
    }
}

fn liesi(c: &Canvas, time: f32) {
    c.rect(-0.75, -0.55, 1.5, 1.4, WHITE_);
    c.rect(-0.75, -0.55, 1.5, 0.2, STEEL);
    let glow = 0.7 + 0.3 * (time * 3.0).sin();
    c.ring(-0.35, -0.72, 0.18, 0.06, Color { a: glow, ..RED });
    c.ring(0.35, -0.72, 0.18, 0.06, STEEL_DARK);
    c.rect(-0.55, -0.1, 1.1, 0.7, BLACK_);
    c.rect(-0.45, 0.0, 0.9, 0.5, rgb(60, 40, 30));
    for x in [-0.45, -0.15, 0.15, 0.45] {
        c.circle(x, -0.45, 0.05, BLACK_);
    }
}

fn luoti(c: &Canvas, _: f32) {
    c.rect(-0.25, -0.1, 0.5, 0.85, GOLD_);
    c.rect(-0.25, 0.65, 0.5, 0.1, rgb(190, 145, 40));
    c.rect(-0.2, -0.35, 0.4, 0.3, rgb(190, 110, 60));
    c.circle(0.0, -0.35, 0.2, rgb(190, 110, 60));
    c.line(-0.12, -0.3, -0.12, 0.6, 0.04, rgb(250, 230, 150));
}

fn laiva(c: &Canvas, time: f32) {
    let y = (time * 1.5).sin() * 0.04;
    c.rect(-0.35, -0.75 + y, 0.2, 0.4, BLACK_);
    c.rect(-0.35, -0.65 + y, 0.2, 0.08, RED);
    c.rect(-0.6, -0.35 + y, 1.0, 0.35, WHITE_);
    for x in [-0.45, -0.2, 0.05, 0.25] {
        c.circle(x, -0.18 + y, 0.05, WATER);
    }
    c.poly(
        &[
            (-0.9, 0.0 + y),
            (0.95, 0.0 + y),
            (0.7, 0.45 + y),
            (-0.75, 0.45 + y),
        ],
        rgb(170, 40, 40),
    );
    c.water(0.4, time);
}

// ---------------------------------------------------------------------
// 40-49

fn maha(c: &Canvas, _: f32) {
    // A round belly between a rolled-up shirt and trousers.
    c.poly(
        &[(-0.55, -0.55), (0.55, -0.55), (0.75, 0.6), (-0.75, 0.6)],
        SKIN,
    );
    c.circle(0.0, 0.1, 0.62, SKIN);
    c.rect(-0.75, -0.95, 1.5, 0.4, rgb(90, 150, 220));
    c.rect(-0.75, -0.6, 1.5, 0.1, rgb(70, 125, 195));
    c.rect(-0.8, 0.6, 1.6, 0.4, rgb(50, 60, 110));
    c.ellipse(0.0, 0.2, 0.05, 0.08, 0.0, SKIN_DARK);
    c.arc(-0.35, 0.1, 0.55, 120.0, 160.0, 0.03, SKIN_DARK);
    c.arc(0.35, 0.1, 0.55, 20.0, 60.0, 0.03, SKIN_DARK);
}

fn maja(c: &Canvas, _: f32) {
    c.rect(-0.6, -0.1, 1.2, 0.9, WOOD);
    for i in 0..5 {
        c.line(
            -0.6,
            -0.1 + i as f32 * 0.18,
            0.6,
            -0.1 + i as f32 * 0.18,
            0.02,
            WOOD_DARK,
        );
    }
    c.tri((-0.8, -0.05), (0.8, -0.05), (0.0, -0.7), rgb(120, 70, 40));
    c.rect(-0.15, 0.3, 0.3, 0.5, WOOD_DARK);
    c.line(0.0, -0.7, 0.0, -1.0, 0.03, WOOD_DARK);
    c.tri((0.0, -1.0), (0.0, -0.8), (0.3, -0.9), RED);
}

fn muki(c: &Canvas, time: f32) {
    c.ring(0.5, 0.1, 0.25, 0.1, rgb(60, 130, 220));
    c.rect(-0.55, -0.35, 1.0, 1.05, rgb(60, 130, 220));
    c.ellipse(-0.05, -0.35, 0.5, 0.1, 0.0, rgb(40, 100, 180));
    c.circle(-0.18, 0.15, 0.1, WHITE_);
    c.circle(0.02, 0.15, 0.1, WHITE_);
    c.tri((-0.28, 0.18), (0.12, 0.18), (-0.08, 0.4), WHITE_);
    for i in 0..2 {
        let x = -0.25 + i as f32 * 0.35 + (time * 2.0 + i as f32).sin() * 0.05;
        c.arc(x, -0.6, 0.1, 90.0, 270.0, 0.03, SMOKE);
    }
}

fn mela(c: &Canvas, _: f32) {
    c.line(-0.55, -0.75, 0.3, 0.3, 0.08, WOOD);
    c.line(-0.72, -0.62, -0.45, -0.9, 0.08, WOOD_DARK);
    c.ellipse(0.5, 0.55, 0.18, 0.4, -40.0, rgb(200, 60, 50));
}

/// A mummy from a horror film, come alive: wrapped in bandages, eyes
/// glowing, lurching forward with its arms out and a loose end dangling.
fn muumio(c: &Canvas, time: f32) {
    let wrap = rgb(225, 214, 180);
    let seam = rgb(160, 145, 110);
    let gap = rgb(45, 35, 30);
    // It lurches from side to side, reaching out as it goes.
    let sway = (time * 2.0).sin() * 0.04;
    let reach = (time * 2.0).cos() * 0.04;

    // Legs, stiffly apart.
    for x in [-0.28, 0.06] {
        c.rect(x, 0.35, 0.22, 0.6, wrap);
        for i in 0..4 {
            let y = 0.42 + i as f32 * 0.14;
            c.line(x, y, x + 0.22, y + 0.05, 0.025, seam);
        }
    }

    // The body, leaning with the sway, wound round and round.
    c.poly(
        &[
            (-0.34 + sway, -0.36),
            (0.34 + sway, -0.36),
            (0.3, 0.4),
            (-0.3, 0.4),
        ],
        wrap,
    );
    for i in 0..6 {
        let y = -0.28 + i as f32 * 0.12;
        let lean = sway * (0.4 - y) / 0.76;
        let tilt = if i % 2 == 0 { 0.035 } else { -0.035 };
        c.line(-0.31 + lean, y - tilt, 0.31 + lean, y + tilt, 0.025, seam);
    }

    // Arms held out in front, hands grasping.
    let hands = [(-0.88, -0.42 + reach), (0.88, -0.42 - reach)];
    for (side, (hx, hy)) in [-1.0f32, 1.0].into_iter().zip(hands) {
        let (sx, sy) = (0.3 * side + sway, -0.28);
        let hx = hx + sway;
        c.line(sx, sy, hx, hy, 0.15, wrap);
        for t in [0.3, 0.55, 0.8] {
            let (x, y) = (sx + (hx - sx) * t, sy + (hy - sy) * t);
            c.line(x - 0.02, y - 0.08, x + 0.02, y + 0.08, 0.02, seam);
        }
        c.circle(hx, hy, 0.09, wrap);
        for finger in [-0.06, 0.0, 0.06] {
            c.line(hx, hy + finger, hx + 0.12 * side, hy + finger * 1.5, 0.035, wrap);
        }
    }

    // A loose bandage end swinging from the left arm.
    let (lx, ly) = (-0.6 + sway, -0.38);
    let dangle: Vec<Pt> = (0..5)
        .map(|k| {
            let k = k as f32;
            (lx + (time * 3.0 + k).sin() * 0.03 * k, ly + 0.11 * k)
        })
        .collect();
    c.polyline(&dangle, 0.06, seam);
    c.polyline(&dangle, 0.035, wrap);

    // The head, with a dark slit for the eyes.
    let (hx, hy) = (sway * 1.3, -0.63);
    let (rx, ry) = (0.23, 0.27);
    c.ellipse(hx, hy, rx, ry, 0.0, wrap);
    for dy in [-0.17f32, -0.08, 0.1, 0.19] {
        let half = rx * (1.0 - (dy / ry).powi(2)).max(0.0).sqrt();
        c.line(hx - half, hy + dy - 0.02, hx + half, hy + dy + 0.02, 0.022, seam);
    }
    c.rect(hx - 0.18, hy - 0.05, 0.36, 0.09, gap);
    // Eyes that glow and dim.
    let glow = 0.6 + 0.4 * (time * 3.0).sin();
    for ex in [-0.08, 0.08] {
        c.circle(hx + ex, hy, 0.06, rgba(255, 220, 80, 0.35 * glow));
        c.circle(hx + ex, hy, 0.025, rgb(255, 240, 140));
    }
}

fn mopo(c: &Canvas, _: f32) {
    c.ring(-0.55, 0.45, 0.3, 0.1, BLACK_);
    c.ring(0.55, 0.45, 0.3, 0.1, BLACK_);
    c.poly(&[(-0.6, 0.0), (0.2, 0.0), (0.4, 0.3), (-0.55, 0.35)], RED);
    c.ellipse(-0.3, -0.07, 0.3, 0.08, 0.0, BLACK_);
    c.line(0.3, 0.3, 0.45, -0.45, 0.07, STEEL_DARK);
    c.line(0.3, -0.45, 0.6, -0.5, 0.07, BLACK_);
    c.circle(0.55, -0.3, 0.09, rgb(250, 240, 150));
}

fn muuri(c: &Canvas, _: f32) {
    for row in 0..5 {
        let y = -0.85 + row as f32 * 0.35;
        let offset = if row % 2 == 0 { 0.0 } else { -0.3 };
        for col in 0..5 {
            let x = -1.0 + offset + col as f32 * 0.6;
            let shade = if (row + col) % 3 == 0 {
                STONE_DARK
            } else {
                STONE
            };
            c.rect(
                x.max(-1.0),
                y,
                (x + 0.55).min(1.0) - x.max(-1.0),
                0.3,
                shade,
            );
        }
    }
}

fn muusi(c: &Canvas, time: f32) {
    c.ellipse(0.0, 0.45, 0.9, 0.3, 0.0, WHITE_);
    c.ellipse(0.0, 0.45, 0.7, 0.2, 0.0, rgb(220, 220, 225));
    let puree = rgb(245, 225, 150);
    c.circle(-0.3, 0.15, 0.3, puree);
    c.circle(0.25, 0.15, 0.3, puree);
    c.circle(0.0, -0.05, 0.35, puree);
    c.rect(-0.12, -0.35, 0.24, 0.15, rgb(255, 240, 100));
    c.arc(
        0.0,
        -0.6 + (time * 2.0).sin() * 0.03,
        0.12,
        90.0,
        270.0,
        0.03,
        SMOKE,
    );
}

fn mato(c: &Canvas, time: f32) {
    for i in 0..9 {
        let t = i as f32 / 8.0;
        let x = -0.75 + t * 1.5;
        let y = ((t * 7.0) + time * 3.0).sin() * 0.2 + 0.2;
        c.circle(
            x,
            y,
            0.16,
            if i % 2 == 0 {
                rgb(230, 130, 140)
            } else {
                rgb(210, 110, 125)
            },
        );
    }
    let head_y = (7.0 + time * 3.0).sin() * 0.2 + 0.2;
    c.circle(0.72, head_y - 0.05, 0.04, BLACK_);
}

fn muovi(c: &Canvas, _: f32) {
    let bag = rgba(235, 240, 245, 0.85);
    c.arc(-0.25, -0.45, 0.2, 180.0, 360.0, 0.07, bag);
    c.arc(0.25, -0.45, 0.2, 180.0, 360.0, 0.07, bag);
    c.poly(&[(-0.6, -0.45), (0.6, -0.45), (0.7, 0.8), (-0.7, 0.8)], bag);
    c.line(-0.3, -0.2, -0.15, 0.4, 0.03, rgb(190, 195, 205));
    c.line(0.25, -0.1, 0.35, 0.5, 0.03, rgb(190, 195, 205));
}

// ---------------------------------------------------------------------
// 50-59

fn pyyhe(c: &Canvas, _: f32) {
    c.line(-0.9, -0.6, 0.9, -0.6, 0.08, STEEL);
    c.rect(-0.55, -0.6, 1.1, 1.4, rgb(240, 120, 150));
    for y in [0.3, 0.5] {
        c.rect(-0.55, y, 1.1, 0.08, WHITE_);
    }
    c.rect(-0.55, -0.6, 1.1, 0.15, rgb(210, 90, 120));
}

fn poiju(c: &Canvas, time: f32) {
    let y = (time * 2.0).sin() * 0.05;
    c.water(0.45, time);
    c.line(0.0, -0.85 + y, 0.0, -0.55 + y, 0.06, STEEL_DARK);
    let blink = if (time * 2.0) % 2.0 < 1.0 {
        rgb(255, 240, 120)
    } else {
        rgb(150, 140, 80)
    };
    c.circle(0.0, -0.88 + y, 0.08, blink);
    c.poly(
        &[
            (-0.2, -0.55 + y),
            (0.2, -0.55 + y),
            (0.45, 0.4 + y),
            (-0.45, 0.4 + y),
        ],
        RED,
    );
    c.poly(
        &[
            (-0.29, -0.25 + y),
            (0.29, -0.25 + y),
            (0.36, 0.05 + y),
            (-0.36, 0.05 + y),
        ],
        WHITE_,
    );
    c.ellipse(0.0, 0.45 + y, 0.55, 0.1, 0.0, WATER);
}

fn puku(c: &Canvas, _: f32) {
    let suit = rgb(45, 50, 75);
    c.poly(&[(-0.8, 1.0), (-0.7, -0.6), (0.7, -0.6), (0.8, 1.0)], suit);
    c.tri((-0.3, -0.6), (0.3, -0.6), (0.0, 0.2), WHITE_);
    c.poly(
        &[
            (-0.08, -0.55),
            (0.08, -0.55),
            (0.12, 0.1),
            (0.0, 0.25),
            (-0.12, 0.1),
        ],
        RED,
    );
    c.tri((-0.3, -0.6), (-0.1, 0.25), (-0.45, -0.1), rgb(65, 70, 100));
    c.tri((0.3, -0.6), (0.1, 0.25), (0.45, -0.1), rgb(65, 70, 100));
    c.circle(0.0, 0.5, 0.05, BLACK_);
    c.circle(0.0, 0.75, 0.05, BLACK_);
}

fn peili(c: &Canvas, time: f32) {
    c.ellipse(0.0, 0.0, 0.6, 0.85, 0.0, GOLD_);
    c.ellipse(0.0, 0.0, 0.5, 0.75, 0.0, rgb(170, 210, 235));
    let shine = (time * 0.8).sin() * 0.1;
    c.line(-0.25 + shine, -0.35, 0.0 + shine, -0.6, 0.06, WHITE_);
    c.line(-0.2 + shine, -0.1, 0.2 + shine, -0.5, 0.04, WHITE_);
}

fn piima(c: &Canvas, _: f32) {
    let glass = rgba(200, 220, 240, 0.4);
    c.poly(
        &[(-0.5, -0.7), (0.5, -0.7), (0.4, 0.85), (-0.4, 0.85)],
        glass,
    );
    c.poly(
        &[(-0.46, -0.35), (0.46, -0.35), (0.4, 0.8), (-0.4, 0.8)],
        rgb(250, 248, 235),
    );
    c.ellipse(0.0, -0.35, 0.46, 0.08, 0.0, rgb(240, 238, 220));
    for (x, y) in [(-0.2, 0.0), (0.15, 0.3), (-0.05, 0.55)] {
        c.circle(x, y, 0.06, rgb(235, 230, 205));
    }
    c.line(-0.5, -0.7, -0.4, 0.85, 0.03, WHITE_);
}

fn pipo(c: &Canvas, _: f32) {
    let wool = rgb(200, 60, 100);
    c.circle(0.0, -0.75, 0.2, WHITE_);
    c.pie(0.0, 0.3, 0.75, 180.0, 360.0, wool);
    for i in 0..5 {
        let x = -0.5 + i as f32 * 0.25;
        c.line(x, 0.25, x * 0.4, -0.35, 0.03, rgb(160, 40, 80));
    }
    c.rect(-0.8, 0.2, 1.6, 0.35, rgb(230, 100, 140));
}

fn pora(c: &Canvas, _: f32) {
    let body = rgb(240, 150, 40);
    c.rect(-0.7, -0.45, 1.0, 0.45, body);
    c.circle(-0.7, -0.22, 0.225, body);
    c.poly(
        &[(-0.45, 0.0), (-0.1, 0.0), (-0.2, 0.8), (-0.55, 0.8)],
        BLACK_,
    );
    c.rect(0.3, -0.33, 0.2, 0.22, STEEL_DARK);
    c.line(0.5, -0.22, 0.95, -0.22, 0.06, STEEL);
    c.rect(-0.25, 0.1, 0.1, 0.15, RED);
}

fn paasi(c: &Canvas, _: f32) {
    c.ellipse(0.0, 0.8, 0.95, 0.2, 0.0, GRASS);
    c.poly(
        &[
            (-0.45, 0.8),
            (-0.5, -0.5),
            (-0.2, -0.85),
            (0.35, -0.8),
            (0.5, 0.8),
        ],
        STONE,
    );
    c.poly(
        &[(0.1, 0.8), (0.2, -0.8), (0.35, -0.8), (0.5, 0.8)],
        STONE_DARK,
    );
    c.circle(-0.3, 0.55, 0.12, LEAF_DARK);
    c.circle(-0.15, 0.65, 0.1, LEAF_DARK);
}

fn pata(c: &Canvas, time: f32) {
    for (i, x) in [-0.3, 0.0, 0.3].iter().enumerate() {
        let h = 0.25 + (time * 6.0 + i as f32).sin() * 0.06;
        c.tri(
            (x - 0.15, 0.95),
            (x + 0.15, 0.95),
            (*x, 0.95 - h - 0.1),
            rgb(250, 140, 30),
        );
    }
    c.ellipse(0.0, 0.1, 0.75, 0.55, 0.0, BLACK_);
    c.ellipse(0.0, -0.35, 0.75, 0.15, 0.0, rgb(60, 60, 70));
    c.ellipse(0.0, -0.35, 0.62, 0.1, 0.0, rgb(120, 160, 60));
    c.ring(-0.8, -0.2, 0.12, 0.05, BLACK_);
    c.ring(0.8, -0.2, 0.12, 0.05, BLACK_);
}

fn paavi(c: &Canvas, _: f32) {
    c.poly(
        &[(-0.7, 1.0), (-0.45, 0.1), (0.45, 0.1), (0.7, 1.0)],
        WHITE_,
    );
    c.rect(-0.08, 0.2, 0.16, 0.7, GOLD_);
    c.rect(-0.25, 0.4, 0.5, 0.12, GOLD_);
    c.circle(0.0, -0.15, 0.3, SKIN);
    c.circle(-0.1, -0.15, 0.04, BLACK_);
    c.circle(0.1, -0.15, 0.04, BLACK_);
    c.poly(
        &[
            (-0.28, -0.35),
            (0.28, -0.35),
            (0.2, -0.85),
            (0.0, -1.0),
            (-0.2, -0.85),
        ],
        WHITE_,
    );
    c.line(0.0, -0.85, 0.0, -0.45, 0.06, GOLD_);
    c.line(-0.12, -0.7, 0.12, -0.7, 0.06, GOLD_);
}

// ---------------------------------------------------------------------
// 60-69

fn raha(c: &Canvas, _: f32) {
    c.rect(-0.9, -0.55, 1.1, 0.65, rgb(110, 180, 110));
    c.rect_lines(-0.85, -0.5, 1.0, 0.55, 0.03, rgb(60, 130, 60));
    c.circle(-0.35, -0.22, 0.15, rgb(60, 130, 60));
    for i in 0..4 {
        let y = 0.75 - i as f32 * 0.15;
        c.ellipse(0.4, y, 0.4, 0.12, 0.0, rgb(200, 150, 40));
        c.ellipse(0.4, y - 0.03, 0.4, 0.12, 0.0, GOLD_);
    }
    c.ellipse(0.4, 0.27, 0.2, 0.05, 0.0, rgb(250, 220, 110));
}

fn ryijy(c: &Canvas, _: f32) {
    c.rect(-0.75, -0.85, 1.5, 1.5, rgb(160, 40, 50));
    let colors = [rgb(240, 180, 50), rgb(60, 130, 200), rgb(240, 240, 220)];
    for (i, r) in [0.55, 0.35, 0.15].iter().enumerate() {
        c.poly(
            &[(0.0, -0.1 - r), (*r, -0.1), (0.0, -0.1 + r), (-r, -0.1)],
            colors[i],
        );
    }
    for i in 0..10 {
        let x = -0.7 + i as f32 * 0.155;
        c.line(x, 0.65, x, 0.9, 0.04, rgb(230, 200, 150));
    }
}

fn reki(c: &Canvas, _: f32) {
    c.rect(-0.7, -0.15, 1.3, 0.2, WOOD);
    c.rect(-0.7, -0.4, 1.3, 0.15, WOOD_DARK);
    for x in [-0.5, 0.3] {
        c.line(x, 0.05, x, 0.4, 0.07, WOOD_DARK);
    }
    c.line(-0.75, 0.45, 0.6, 0.45, 0.08, STEEL_DARK);
    c.arc(0.6, 0.2, 0.25, 90.0, -90.0, 0.08, STEEL_DARK);
    c.rect(-1.0, 0.55, 2.0, 0.45, SNOW);
}

fn railo(c: &Canvas, _: f32) {
    c.rect(-1.0, -1.0, 2.0, 2.0, rgb(215, 235, 250));
    c.polyline(
        &[
            (-0.9, -0.6),
            (-0.4, -0.35),
            (-0.5, 0.0),
            (0.0, 0.2),
            (0.1, 0.55),
            (0.6, 0.6),
            (0.9, 0.95),
        ],
        0.14,
        rgb(20, 60, 120),
    );
    c.polyline(&[(-0.5, 0.0), (-0.8, 0.35)], 0.05, rgb(60, 100, 160));
    c.polyline(&[(0.1, 0.55), (-0.1, 0.9)], 0.05, rgb(60, 100, 160));
}

fn riimu(c: &Canvas, _: f32) {
    c.ellipse(0.0, 0.05, 0.6, 0.85, 0.0, STONE);
    c.ellipse(0.1, 0.1, 0.45, 0.72, 0.0, rgb(155, 155, 165));
    // The rune Fehu, carved in red.
    let rune = rgb(190, 40, 40);
    c.line(-0.15, -0.55, -0.15, 0.6, 0.1, rune);
    c.line(-0.15, -0.1, 0.3, -0.45, 0.1, rune);
    c.line(-0.15, 0.2, 0.3, -0.15, 0.1, rune);
}

fn rapu(c: &Canvas, _: f32) {
    let shell = rgb(210, 60, 40);
    for side in [-1.0, 1.0] {
        c.line(side * 0.15, -0.35, side * 0.45, -0.65, 0.07, shell);
        c.ellipse(side * 0.5, -0.75, 0.13, 0.2, side * 30.0, shell);
        c.line(side * 0.05, -0.45, side * 0.3, -0.95, 0.02, shell);
        for i in 0..3 {
            let y = -0.05 + i as f32 * 0.15;
            c.line(side * 0.15, y, side * 0.45, y + 0.1, 0.04, shell);
        }
    }
    c.ellipse(0.0, 0.05, 0.2, 0.4, 0.0, shell);
    for i in 0..3 {
        c.ellipse(
            0.0,
            0.5 + i as f32 * 0.12,
            0.16 - i as f32 * 0.03,
            0.07,
            0.0,
            shell,
        );
    }
    c.circle(-0.08, -0.3, 0.04, BLACK_);
    c.circle(0.08, -0.3, 0.04, BLACK_);
}

fn ruori(c: &Canvas, time: f32) {
    let turn = time * 0.3;
    for i in 0..8 {
        let a = turn + i as f32 / 8.0 * TAU;
        c.line(0.0, 0.0, 0.85 * a.cos(), 0.85 * a.sin(), 0.08, WOOD_DARK);
        c.circle(0.85 * a.cos(), 0.85 * a.sin(), 0.08, WOOD_DARK);
    }
    c.ring(0.0, 0.0, 0.6, 0.12, WOOD);
    c.circle(0.0, 0.0, 0.15, GOLD_);
}

fn ruusu(c: &Canvas, _: f32) {
    c.line(0.0, 0.0, 0.05, 0.95, 0.06, LEAF_DARK);
    c.ellipse(-0.2, 0.55, 0.2, 0.08, -30.0, LEAF);
    c.ellipse(0.25, 0.4, 0.2, 0.08, 30.0, LEAF);
    let petal = rgb(220, 40, 60);
    for (x, y, r) in [
        (-0.2, -0.2, 0.3),
        (0.2, -0.2, 0.3),
        (0.0, 0.0, 0.3),
        (0.0, -0.4, 0.28),
    ] {
        c.circle(x, y, r, petal);
    }
    c.arc(0.0, -0.2, 0.16, 0.0, 300.0, 0.05, rgb(150, 20, 40));
    c.arc(0.0, -0.2, 0.06, 0.0, 300.0, 0.04, rgb(150, 20, 40));
}

fn rata(c: &Canvas, _: f32) {
    c.rect(-1.0, -1.0, 2.0, 2.0, rgb(120, 110, 95));
    for i in 0..7 {
        let t = i as f32 / 6.0;
        let y = -0.8 + t * t * 1.7;
        let half = 0.15 + t * 0.65;
        c.rect(-half - 0.1, y, half * 2.0 + 0.2, 0.04 + t * 0.1, WOOD_DARK);
    }
    c.line(-0.1, -0.85, -0.8, 1.0, 0.07, STEEL);
    c.line(0.1, -0.85, 0.8, 1.0, 0.07, STEEL);
}

fn rovio(c: &Canvas, time: f32) {
    c.line(-0.7, 0.85, 0.6, 0.45, 0.14, WOOD_DARK);
    c.line(0.7, 0.85, -0.6, 0.45, 0.14, WOOD);
    for (i, (x, w, h, color)) in [
        (-0.3, 0.25, 0.9, rgb(240, 90, 30)),
        (0.25, 0.25, 0.8, rgb(240, 90, 30)),
        (0.0, 0.3, 1.2, rgb(250, 150, 40)),
        (0.0, 0.15, 0.7, rgb(255, 230, 100)),
    ]
    .iter()
    .enumerate()
    {
        let flicker = (time * 8.0 + i as f32 * 1.7).sin() * 0.08;
        c.tri(
            (x - w, 0.6),
            (x + w, 0.6),
            (*x + flicker, 0.6 - h - flicker),
            *color,
        );
    }
    let spark = (time * 1.3) % 1.0;
    c.circle(
        0.3 - spark * 0.2,
        -0.3 - spark * 0.6,
        0.03,
        rgb(255, 220, 120),
    );
}

// ---------------------------------------------------------------------
// 70-79

fn saha(c: &Canvas, _: f32) {
    c.poly(
        &[(-0.45, -0.35), (0.95, 0.05), (0.95, 0.25), (-0.45, 0.25)],
        STEEL,
    );
    for i in 0..10 {
        let x = -0.4 + i as f32 * 0.135;
        c.tri((x, 0.25), (x + 0.135, 0.25), (x + 0.07, 0.38), STEEL_DARK);
    }
    c.poly(
        &[(-0.95, -0.45), (-0.45, -0.45), (-0.45, 0.35), (-0.95, 0.35)],
        rgb(200, 60, 40),
    );
    c.ellipse(-0.7, -0.05, 0.13, 0.22, 0.0, TILE);
}

fn soija(c: &Canvas, _: f32) {
    let bottle = rgb(80, 40, 25);
    c.rect(-0.4, -0.2, 0.8, 1.1, bottle);
    c.poly(
        &[(-0.4, -0.2), (0.4, -0.2), (0.15, -0.55), (-0.15, -0.55)],
        bottle,
    );
    c.rect(-0.15, -0.8, 0.3, 0.3, RED);
    c.rect(-0.4, 0.15, 0.8, 0.45, WHITE_);
    c.circle(0.0, 0.37, 0.14, RED);
    c.line(-0.3, -0.1, -0.3, 0.8, 0.04, rgb(140, 80, 50));
}

fn sika(c: &Canvas, _: f32) {
    let pink = rgb(245, 170, 180);
    c.tri(
        (-0.6, -0.3),
        (-0.25, -0.6),
        (-0.7, -0.85),
        rgb(230, 140, 155),
    );
    c.tri((0.6, -0.3), (0.25, -0.6), (0.7, -0.85), rgb(230, 140, 155));
    c.circle(0.0, 0.0, 0.7, pink);
    c.ellipse(0.0, 0.2, 0.3, 0.2, 0.0, rgb(235, 130, 150));
    c.ellipse(-0.1, 0.2, 0.05, 0.08, 0.0, rgb(150, 60, 80));
    c.ellipse(0.1, 0.2, 0.05, 0.08, 0.0, rgb(150, 60, 80));
    c.circle(-0.28, -0.2, 0.06, BLACK_);
    c.circle(0.28, -0.2, 0.06, BLACK_);
}

fn siili(c: &Canvas, _: f32) {
    let spines = rgb(95, 70, 50);
    for i in 0..11 {
        let a = (180.0 + i as f32 * 16.0).to_radians();
        let base = vec2(-0.1 + 0.55 * a.cos(), 0.35 + 0.5 * a.sin());
        let tip = vec2(-0.1 + 0.85 * a.cos(), 0.35 + 0.8 * a.sin());
        let side = vec2(-a.sin(), a.cos()) * 0.12;
        c.tri(
            (base.x - side.x, base.y - side.y),
            (base.x + side.x, base.y + side.y),
            (tip.x, tip.y),
            spines,
        );
    }
    c.pie(-0.1, 0.35, 0.6, 180.0, 360.0, rgb(130, 95, 65));
    c.poly(
        &[(0.3, 0.35), (0.45, -0.05), (0.9, 0.3), (0.5, 0.45)],
        rgb(220, 190, 150),
    );
    c.circle(0.9, 0.3, 0.06, BLACK_);
    c.circle(0.5, 0.15, 0.04, BLACK_);
    c.rect(-0.8, 0.35, 1.7, 0.1, rgb(220, 190, 150));
}

fn siima(c: &Canvas, _: f32) {
    c.polyline(
        &[(-0.6, -0.95), (-0.3, -0.5), (0.1, -0.1), (0.2, 0.3)],
        0.03,
        WHITE_,
    );
    c.circle(0.2, 0.3, 0.07, STEEL_DARK);
    c.line(0.2, 0.3, 0.2, 0.6, 0.05, STEEL);
    c.arc(0.05, 0.6, 0.15, 0.0, 180.0, 0.05, STEEL);
    c.tri((-0.1, 0.6), (-0.1, 0.42), (-0.02, 0.5), STEEL);
}

fn siipi(c: &Canvas, time: f32) {
    // A white feathered wing, rooted at the lower left.
    let flap = (time * 2.0).sin() * 4.0;
    let root = (-0.7, 0.5);
    for i in 0..6 {
        let angle = (-75.0 + i as f32 * 14.0 + flap).to_radians();
        let length = 1.5 - i as f32 * 0.12;
        let tip = (root.0 + length * angle.cos(), root.1 + length * angle.sin());
        let mid = ((root.0 + tip.0) / 2.0, (root.1 + tip.1) / 2.0);
        let shade = 245 - i as u8 * 10;
        c.ellipse(
            mid.0,
            mid.1,
            length / 2.0,
            0.13,
            angle.to_degrees(),
            rgb(shade, shade, 255),
        );
    }
    c.ellipse(-0.45, 0.25, 0.4, 0.28, -40.0, WHITE_);
    for i in 0..3 {
        let x = -0.55 + i as f32 * 0.15;
        c.arc(
            x,
            0.2 - i as f32 * 0.12,
            0.1,
            180.0,
            300.0,
            0.02,
            rgb(200, 200, 225),
        );
    }
}

fn siru(c: &Canvas, _: f32) {
    for i in 0..5 {
        let t = -0.48 + i as f32 * 0.24;
        c.rect(t - 0.05, -0.9, 0.1, 0.25, STEEL);
        c.rect(t - 0.05, 0.65, 0.1, 0.25, STEEL);
        c.rect(-0.9, t - 0.05, 0.25, 0.1, STEEL);
        c.rect(0.65, t - 0.05, 0.25, 0.1, STEEL);
    }
    c.rect(-0.65, -0.65, 1.3, 1.3, rgb(40, 40, 45));
    c.rect(-0.3, -0.3, 0.6, 0.6, GOLD_);
    c.circle(-0.5, -0.5, 0.06, rgb(90, 90, 95));
}

fn susi(c: &Canvas, _: f32) {
    let fur = rgb(130, 135, 145);
    c.tri((-0.6, -0.2), (-0.2, -0.45), (-0.55, -0.9), fur);
    c.tri((0.6, -0.2), (0.2, -0.45), (0.55, -0.9), fur);
    c.poly(
        &[
            (-0.65, -0.3),
            (0.65, -0.3),
            (0.45, 0.3),
            (0.0, 0.85),
            (-0.45, 0.3),
        ],
        fur,
    );
    c.poly(
        &[
            (-0.25, 0.1),
            (0.25, 0.1),
            (0.15, 0.6),
            (0.0, 0.8),
            (-0.15, 0.6),
        ],
        rgb(220, 220, 225),
    );
    c.ellipse(-0.25, -0.1, 0.1, 0.06, 20.0, rgb(250, 210, 60));
    c.ellipse(0.25, -0.1, 0.1, 0.06, -20.0, rgb(250, 210, 60));
    c.circle(-0.25, -0.1, 0.03, BLACK_);
    c.circle(0.25, -0.1, 0.03, BLACK_);
    c.ellipse(0.0, 0.7, 0.1, 0.07, 0.0, BLACK_);
}

fn sota(c: &Canvas, _: f32) {
    let helmet = rgb(85, 105, 60);
    c.pie(0.0, 0.2, 0.75, 180.0, 360.0, helmet);
    c.rect(-0.95, 0.15, 1.9, 0.15, rgb(70, 90, 50));
    c.arc(0.0, 0.2, 0.55, 20.0, 160.0, 0.06, rgb(90, 70, 40));
    c.arc(-0.2, -0.1, 0.4, 200.0, 250.0, 0.06, rgb(120, 140, 90));
}

fn sauva(c: &Canvas, _: f32) {
    c.line(-0.5, 0.95, 0.35, -0.6, 0.12, WOOD);
    c.circle(0.42, -0.72, 0.16, WOOD_DARK);
    for (x, y) in [(-0.2, 0.4), (0.1, -0.15)] {
        c.circle(x, y, 0.07, WOOD_DARK);
    }
}

// ---------------------------------------------------------------------
// 80-89

fn tuohi(c: &Canvas, _: f32) {
    c.rect(-0.75, -0.6, 1.3, 1.2, rgb(245, 240, 230));
    c.ellipse(0.6, 0.0, 0.14, 0.6, 0.0, rgb(220, 200, 170));
    c.ellipse(0.6, 0.0, 0.07, 0.5, 0.0, rgb(200, 150, 100));
    for (x, y, w) in [
        (-0.6, -0.4, 0.3),
        (-0.1, -0.2, 0.4),
        (-0.5, 0.1, 0.25),
        (0.0, 0.35, 0.35),
        (-0.55, 0.45, 0.2),
    ] {
        c.rect(x, y, w, 0.06, BLACK_);
    }
}

fn taiji(c: &Canvas, time: f32) {
    let turn = time * 30.0;
    c.circle(0.0, 0.0, 0.8, WHITE_);
    c.pie(0.0, 0.0, 0.8, 90.0 + turn, 270.0 + turn, BLACK_);
    let a = (90.0 + turn).to_radians();
    let (dx, dy) = (0.4 * a.cos(), 0.4 * a.sin());
    c.circle(dx, dy, 0.4, WHITE_);
    c.circle(-dx, -dy, 0.4, BLACK_);
    c.circle(dx, dy, 0.12, BLACK_);
    c.circle(-dx, -dy, 0.12, WHITE_);
    c.ring(0.0, 0.0, 0.8, 0.04, STEEL_DARK);
}

fn tiuku(c: &Canvas, time: f32) {
    let swing = (time * 6.0).sin() * 0.05;
    c.poly(
        &[(-0.35, -0.8), (0.35, -0.8), (0.15, -0.45), (-0.15, -0.45)],
        RED,
    );
    c.circle(swing, 0.1, 0.55, GOLD_);
    c.line(-0.4 + swing, 0.2, 0.4 + swing, 0.2, 0.07, rgb(160, 120, 30));
    c.circle(swing, 0.4, 0.09, rgb(160, 120, 30));
    c.circle(-0.18 + swing, -0.1, 0.1, rgb(255, 240, 170));
}

fn tiili(c: &Canvas, _: f32) {
    c.poly(
        &[(-0.8, -0.2), (-0.4, -0.55), (0.8, -0.55), (0.4, -0.2)],
        rgb(220, 110, 70),
    );
    c.poly(
        &[(0.4, -0.2), (0.8, -0.55), (0.8, 0.2), (0.4, 0.55)],
        rgb(150, 60, 35),
    );
    c.rect(-0.8, -0.2, 1.2, 0.75, rgb(190, 80, 50));
    for x in [-0.45, -0.1, 0.25] {
        c.ellipse(x + 0.05, -0.37, 0.08, 0.05, 0.0, rgb(120, 45, 25));
    }
}

fn taimi(c: &Canvas, time: f32) {
    c.poly(
        &[(-0.5, 0.3), (0.5, 0.3), (0.38, 0.9), (-0.38, 0.9)],
        rgb(200, 100, 60),
    );
    c.rect(-0.55, 0.22, 1.1, 0.14, rgb(215, 115, 75));
    c.ellipse(0.0, 0.25, 0.48, 0.07, 0.0, rgb(90, 60, 35));
    let sway = (time * 1.5).sin() * 5.0;
    c.line(0.0, 0.25, 0.0, -0.3, 0.06, LEAF_DARK);
    c.ellipse(-0.25, -0.35, 0.25, 0.12, -25.0 + sway, LEAF);
    c.ellipse(0.25, -0.45, 0.25, 0.12, 25.0 + sway, LEAF);
}

fn tipu(c: &Canvas, _: f32) {
    let yellow = rgb(255, 215, 60);
    c.line(-0.15, 0.6, -0.2, 0.9, 0.05, rgb(240, 140, 30));
    c.line(0.15, 0.6, 0.2, 0.9, 0.05, rgb(240, 140, 30));
    c.circle(0.0, 0.25, 0.45, yellow);
    c.circle(0.1, -0.35, 0.32, yellow);
    c.ellipse(-0.3, 0.25, 0.2, 0.12, 30.0, rgb(240, 190, 40));
    c.circle(0.2, -0.4, 0.05, BLACK_);
    c.tri((0.38, -0.35), (0.38, -0.2), (0.6, -0.28), rgb(240, 140, 30));
}

fn tera(c: &Canvas, time: f32) {
    c.poly(
        &[
            (-0.75, 0.05),
            (0.55, 0.05),
            (0.9, -0.2),
            (0.4, -0.35),
            (-0.75, -0.35),
        ],
        STEEL,
    );
    c.line(-0.75, 0.05, 0.55, 0.05, 0.04, WHITE_);
    c.rect(-1.0, -0.3, 0.25, 0.3, WOOD_DARK);
    let glint = (time * 0.7) % 2.0 - 0.5;
    if (-0.7..0.7).contains(&glint) {
        c.line(glint, -0.3, glint + 0.1, 0.0, 0.04, WHITE_);
    }
}

fn teesi(c: &Canvas, _: f32) {
    c.rect(-0.8, -1.0, 1.6, 2.0, WOOD);
    for x in [-0.4, 0.0, 0.4] {
        c.line(x, -1.0, x, 1.0, 0.03, WOOD_DARK);
    }
    c.rect(-0.45, -0.6, 0.8, 1.0, rgb(245, 235, 200));
    for i in 0..7 {
        let y = -0.45 + i as f32 * 0.12;
        c.line(
            -0.35,
            y,
            0.25 - (i % 3) as f32 * 0.1,
            y,
            0.03,
            rgb(90, 80, 70),
        );
    }
    c.circle(-0.05, -0.55, 0.05, STEEL_DARK);
    // Luther's hammer.
    c.line(0.35, 0.95, 0.75, 0.35, 0.08, WOOD_DARK);
    c.poly(
        &[(0.55, 0.2), (0.9, 0.45), (0.8, 0.6), (0.45, 0.35)],
        STEEL_DARK,
    );
}

fn toti(c: &Canvas, time: f32) {
    c.ring(0.55, 0.15, 0.22, 0.07, rgba(200, 220, 240, 0.8));
    c.poly(
        &[(-0.5, -0.4), (0.45, -0.4), (0.4, 0.75), (-0.45, 0.75)],
        rgba(200, 220, 240, 0.35),
    );
    c.poly(
        &[(-0.47, -0.15), (0.43, -0.15), (0.4, 0.72), (-0.44, 0.72)],
        rgb(190, 110, 40),
    );
    c.circle(-0.2, -0.15, 0.18, rgb(250, 230, 80));
    c.circle(-0.2, -0.15, 0.12, rgb(255, 250, 180));
    c.line(0.15, -0.65, 0.25, 0.3, 0.08, rgb(140, 80, 40));
    for i in 0..2 {
        let x = -0.2 + i as f32 * 0.35 + (time * 2.0 + i as f32).sin() * 0.05;
        c.arc(x, -0.7, 0.1, 90.0, 270.0, 0.03, SMOKE);
    }
}

fn tavi(c: &Canvas, time: f32) {
    let y = (time * 2.0).sin() * 0.03;
    c.water(0.35, time);
    c.ellipse(-0.1, 0.15 + y, 0.6, 0.3, 0.0, rgb(150, 150, 155));
    c.tri(
        (-0.7, 0.05 + y),
        (-0.95, -0.1 + y),
        (-0.6, 0.25 + y),
        rgb(120, 120, 125),
    );
    c.rect(-0.35, 0.05 + y, 0.4, 0.08, rgb(40, 130, 90));
    c.circle(0.45, -0.3 + y, 0.28, rgb(150, 70, 40));
    c.ellipse(0.52, -0.33 + y, 0.16, 0.08, -20.0, rgb(40, 130, 90));
    c.circle(0.52, -0.35 + y, 0.04, BLACK_);
    c.tri(
        (0.7, -0.3 + y),
        (0.7, -0.18 + y),
        (0.95, -0.22 + y),
        rgb(60, 60, 60),
    );
}

// ---------------------------------------------------------------------
// 90-99

fn vuohi(c: &Canvas, _: f32) {
    let fur = rgb(235, 230, 220);
    c.arc(-0.35, -0.55, 0.3, 270.0, 450.0, 0.1, rgb(140, 120, 90));
    c.arc(0.35, -0.55, 0.3, 90.0, 270.0, 0.1, rgb(140, 120, 90));
    c.ellipse(-0.6, -0.1, 0.25, 0.1, 20.0, fur);
    c.ellipse(0.6, -0.1, 0.25, 0.1, -20.0, fur);
    c.poly(
        &[
            (-0.4, -0.45),
            (0.4, -0.45),
            (0.3, 0.45),
            (0.0, 0.6),
            (-0.3, 0.45),
        ],
        fur,
    );
    c.tri((-0.15, 0.5), (0.15, 0.5), (0.0, 0.95), rgb(200, 195, 185));
    c.ellipse(-0.2, -0.15, 0.09, 0.05, 0.0, rgb(200, 170, 60));
    c.ellipse(0.2, -0.15, 0.09, 0.05, 0.0, rgb(200, 170, 60));
    c.rect(-0.25, -0.17, 0.1, 0.03, BLACK_);
    c.rect(0.15, -0.17, 0.1, 0.03, BLACK_);
    c.ellipse(0.0, 0.35, 0.12, 0.06, 0.0, rgb(200, 150, 150));
}

fn vaja(c: &Canvas, _: f32) {
    let red = rgb(160, 40, 35);
    c.rect(-0.7, -0.2, 1.4, 1.0, red);
    c.tri((-0.85, -0.15), (0.85, -0.15), (0.0, -0.8), rgb(60, 60, 65));
    c.rect(-0.7, -0.2, 0.08, 1.0, WHITE_);
    c.rect(0.62, -0.2, 0.08, 1.0, WHITE_);
    c.rect(-0.2, 0.2, 0.4, 0.6, rgb(110, 30, 25));
    c.rect_lines(-0.2, 0.2, 0.4, 0.6, 0.04, WHITE_);
    c.rect(-1.0, 0.8, 2.0, 0.2, GRASS);
}

fn vaaka(c: &Canvas, time: f32) {
    let tilt = (time * 1.2).sin() * 0.08;
    c.rect(-0.4, 0.8, 0.8, 0.12, GOLD_);
    c.line(0.0, 0.8, 0.0, -0.6, 0.07, GOLD_);
    c.line(-0.7, -0.5 - tilt, 0.7, -0.5 + tilt, 0.06, GOLD_);
    for (x, dy) in [(-0.7, -tilt), (0.7, tilt)] {
        let y = -0.5 + dy;
        c.line(x, y, x - 0.22, y + 0.55, 0.02, GOLD_);
        c.line(x, y, x + 0.22, y + 0.55, 0.02, GOLD_);
        c.pie(x, y + 0.55, 0.25, 0.0, 180.0, GOLD_);
    }
    c.circle(0.0, -0.62, 0.07, GOLD_);
}

fn viulu(c: &Canvas, _: f32) {
    let wood = rgb(170, 80, 30);
    c.line(0.0, -0.3, 0.0, -0.9, 0.12, BLACK_);
    c.circle(0.0, -0.92, 0.08, wood);
    c.circle(0.0, -0.1, 0.35, wood);
    c.circle(0.0, 0.5, 0.42, wood);
    c.rect(-0.22, 0.05, 0.44, 0.3, wood);
    c.line(0.0, -0.4, 0.0, 0.6, 0.12, BLACK_);
    c.rect(-0.15, 0.6, 0.3, 0.06, BLACK_);
    for x in [-0.2, 0.2] {
        c.arc(x, 0.3, 0.07, 270.0, 450.0, 0.025, BLACK_);
    }
    for i in 0..4 {
        let x = -0.045 + i as f32 * 0.03;
        c.line(x, -0.85, x, 0.6, 0.01, rgb(230, 230, 230));
    }
}

fn vaimo(c: &Canvas, _: f32) {
    let veil = rgba(250, 250, 255, 0.55);
    c.poly(
        &[(-0.55, 1.0), (-0.3, 0.2), (0.3, 0.2), (0.55, 1.0)],
        WHITE_,
    );
    c.poly(
        &[(-0.65, 0.95), (-0.35, -0.55), (0.35, -0.55), (0.65, 0.95)],
        veil,
    );
    c.circle(0.0, -0.2, 0.3, SKIN);
    c.arc(0.0, -0.25, 0.32, 180.0, 360.0, 0.12, rgb(200, 150, 70));
    c.circle(-0.1, -0.2, 0.04, BLACK_);
    c.circle(0.1, -0.2, 0.04, BLACK_);
    c.arc(0.0, -0.1, 0.08, 20.0, 160.0, 0.03, RED);
    for x in [-0.2, 0.0, 0.2] {
        c.circle(x, -0.52, 0.08, rgb(250, 190, 210));
    }
    for (x, color) in [(-0.1, RED), (0.1, rgb(250, 180, 200)), (0.0, WHITE_)] {
        c.circle(x, 0.5, 0.1, color);
    }
}

fn vapa(c: &Canvas, time: f32) {
    c.water(0.55, time);
    c.line(-0.9, 0.75, 0.8, -0.8, 0.07, rgb(50, 50, 50));
    c.line(-0.9, 0.75, -0.55, 0.43, 0.11, WOOD_DARK);
    c.circle(-0.5, 0.52, 0.12, STEEL_DARK);
    c.circle(-0.5, 0.52, 0.05, STEEL);
    let sway = (time * 1.5).sin() * 0.05;
    c.line(0.8, -0.8, 0.75 + sway, 0.5, 0.02, WHITE_);
    c.pie(0.75 + sway, 0.5, 0.08, 180.0, 360.0, RED);
    c.pie(0.75 + sway, 0.5, 0.08, 0.0, 180.0, WHITE_);
}

fn vuori(c: &Canvas, _: f32) {
    c.tri((-1.0, 0.8), (-0.35, -0.3), (0.3, 0.8), STONE_DARK);
    c.tri((-0.5, 0.8), (0.3, -0.85), (1.0, 0.8), STONE);
    c.tri((0.3, -0.85), (0.08, -0.4), (0.52, -0.4), SNOW);
    c.tri((-0.35, -0.3), (-0.48, -0.03), (-0.22, -0.03), SNOW);
    c.rect(-1.0, 0.75, 2.0, 0.25, GRASS);
}

fn vaasi(c: &Canvas, _: f32) {
    for (x, color) in [
        (-0.3, RED),
        (0.0, rgb(250, 200, 40)),
        (0.3, rgb(240, 120, 180)),
    ] {
        c.line(x * 0.3, -0.3, x, -0.7, 0.04, LEAF_DARK);
        c.circle(x, -0.75, 0.13, color);
    }
    let blue = rgb(60, 110, 200);
    c.circle(0.0, 0.4, 0.45, blue);
    c.rect(-0.15, -0.3, 0.3, 0.5, blue);
    c.poly(
        &[(-0.3, -0.4), (0.3, -0.4), (0.15, -0.2), (-0.15, -0.2)],
        blue,
    );
    c.ring(0.0, 0.4, 0.3, 0.05, WHITE_);
    c.circle(0.0, 0.4, 0.08, WHITE_);
}

fn vouti(c: &Canvas, _: f32) {
    // A burly bailiff in an executioner's black hood.
    c.poly(&[(-0.85, 1.0), (-0.7, 0.2), (0.7, 0.2), (0.85, 1.0)], SKIN);
    c.arc(-0.3, 0.45, 0.25, 200.0, 340.0, 0.04, SKIN_DARK);
    c.arc(0.3, 0.45, 0.25, 200.0, 340.0, 0.04, SKIN_DARK);
    c.line(0.0, 0.3, 0.0, 0.95, 0.03, SKIN_DARK);
    c.rect(-0.85, 0.8, 1.7, 0.12, rgb(90, 55, 25));
    c.ellipse(-0.8, 0.35, 0.22, 0.32, 0.0, SKIN);
    c.ellipse(0.8, 0.35, 0.22, 0.32, 0.0, SKIN);
    c.circle(0.0, -0.35, 0.4, BLACK_);
    c.tri((-0.4, -0.35), (0.4, -0.35), (0.0, -1.0), BLACK_);
    c.poly(
        &[(-0.45, 0.2), (0.45, 0.2), (0.3, -0.1), (-0.3, -0.1)],
        BLACK_,
    );
    c.ellipse(-0.15, -0.35, 0.09, 0.06, 0.0, WHITE_);
    c.ellipse(0.15, -0.35, 0.09, 0.06, 0.0, WHITE_);
    c.circle(-0.15, -0.35, 0.03, BLACK_);
    c.circle(0.15, -0.35, 0.03, BLACK_);
}

fn vauva(c: &Canvas, _: f32) {
    c.circle(0.0, 0.05, 0.7, SKIN);
    c.arc(0.05, -0.62, 0.12, 180.0, 450.0, 0.05, rgb(200, 150, 70));
    c.circle(-0.25, -0.05, 0.07, BLACK_);
    c.circle(0.25, -0.05, 0.07, BLACK_);
    c.circle(-0.42, 0.18, 0.12, rgba(250, 150, 160, 0.6));
    c.circle(0.42, 0.18, 0.12, rgba(250, 150, 160, 0.6));
    // A pacifier.
    c.ellipse(0.0, 0.35, 0.22, 0.12, 0.0, rgb(120, 190, 240));
    c.circle(0.0, 0.35, 0.07, rgb(250, 200, 220));
    c.ring(0.0, 0.5, 0.1, 0.03, rgb(120, 190, 240));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

    #[test]
    fn every_pair_has_exactly_one_picture() {
        for pair in PAIRS {
            let count = PICTURES.iter().filter(|(n, _)| *n == pair.number).count();
            assert_eq!(count, 1, "{} = {}", pair.number, pair.word);
        }
        assert_eq!(PICTURES.len(), PAIRS.len());
    }
}
