//! Procedurally drawn characters and scenery.

use macroquad::prelude::*;

use crate::obstacles::{Kind, Obstacle};

const SKIN: Color = Color::new(1.0, 0.86, 0.75, 1.0);
const HAIR: Color = Color::new(0.98, 0.84, 0.42, 1.0);
const DRESS: Color = Color::new(1.0, 0.41, 0.71, 1.0);
const DRESS_LIGHT: Color = Color::new(1.0, 0.75, 0.87, 1.0);
const SHOES: Color = Color::new(0.85, 0.2, 0.5, 1.0);
const CHEEKS: Color = Color::new(1.0, 0.5, 0.6, 0.6);
const EYES: Color = Color::new(0.15, 0.1, 0.1, 1.0);

const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.35);

use lukuloitsu_core::arena::{girl_hand, side_toward};

/// A little girl in a pink dress, centered on `pos`. `time` animates her
/// walk while `moving` is true. While `casting` toward a point, she points
/// a glowing hand at it.
pub fn draw_girl(pos: Vec2, time: f32, moving: bool, casting: Option<Vec2>) {
    let step = if moving { (time * 14.0).sin() } else { 0.0 };
    let x = pos.x;
    let y = pos.y - step.abs() * 2.0;

    draw_ellipse(pos.x, pos.y + 22.0, 13.0, 4.0, 0.0, SHADOW);

    draw_girl_legs(x, y, step);
    draw_girl_arms(pos, y, step, time, casting);
    draw_girl_dress(x, y);
    draw_girl_pigtails(x, y);
    draw_girl_head(x, y);
}

/// Legs and shoes, stepping in turn.
fn draw_girl_legs(x: f32, y: f32, step: f32) {
    for (side, lift) in [(-1.0, step.max(0.0)), (1.0, (-step).max(0.0))] {
        let foot_y = y + 20.0 - lift * 3.0;
        draw_line(x + side * 4.0, y + 12.0, x + side * 4.0, foot_y, 3.0, SKIN);
        draw_ellipse(x + side * 5.0, foot_y + 1.0, 4.0, 2.5, 0.0, SHOES);
    }
}

/// Arms swing against the legs; a casting arm points at its target.
fn draw_girl_arms(pos: Vec2, y: f32, step: f32, time: f32, casting: Option<Vec2>) {
    let x = pos.x;
    let cast_side = casting.map(|toward| side_toward(pos, toward));
    for side in [-1.0, 1.0] {
        let hand = match casting {
            Some(toward) if cast_side == Some(side) => girl_hand(vec2(x, y), toward),
            _ => vec2(x + side * 13.0, y + 5.0 - side * step * 3.0),
        };
        draw_line(x + side * 6.0, y - 3.0, hand.x, hand.y, 3.0, SKIN);
        draw_circle(hand.x, hand.y, 2.0, SKIN);
        if cast_side == Some(side) {
            let glow = 0.5 + 0.3 * (time * 25.0).sin();
            draw_circle(hand.x, hand.y, 7.0, Color::new(1.0, 0.5, 0.85, glow));
            draw_circle(hand.x, hand.y, 3.0, WHITE);
        }
    }
}

/// A flared skirt with a collar and a belt.
fn draw_girl_dress(x: f32, y: f32) {
    draw_triangle(
        vec2(x, y - 6.0),
        vec2(x - 14.0, y + 14.0),
        vec2(x + 14.0, y + 14.0),
        DRESS,
    );
    draw_rectangle(x - 6.0, y - 6.0, 12.0, 10.0, DRESS);
    draw_rectangle(x - 9.0, y + 3.0, 18.0, 2.0, DRESS_LIGHT);
    draw_triangle(
        vec2(x - 5.0, y - 6.0),
        vec2(x + 5.0, y - 6.0),
        vec2(x, y - 2.0),
        DRESS_LIGHT,
    );
}

/// Pigtails with pink bows.
fn draw_girl_pigtails(x: f32, y: f32) {
    for side in [-1.0, 1.0] {
        let tail = vec2(x + side * 13.0, y - 15.0);
        draw_circle(tail.x, tail.y, 5.0, HAIR);
        let bow = vec2(x + side * 10.0, y - 21.0);
        draw_triangle(bow, bow + vec2(-4.0, -3.0), bow + vec2(-4.0, 3.0), DRESS);
        draw_triangle(bow, bow + vec2(4.0, -3.0), bow + vec2(4.0, 3.0), DRESS);
        draw_circle(bow.x, bow.y, 1.5, DRESS_LIGHT);
    }
}

/// Hair behind, the face, bangs and a smile.
fn draw_girl_head(x: f32, y: f32) {
    draw_circle(x, y - 16.0, 11.5, HAIR);
    draw_circle(x, y - 14.0, 10.0, SKIN);
    draw_ellipse(x, y - 22.0, 9.0, 4.0, 0.0, HAIR);

    // Face.
    for side in [-1.0, 1.0] {
        draw_circle(x + side * 4.0, y - 14.0, 2.0, EYES);
        draw_circle(x + side * 4.0 + 0.7, y - 14.7, 0.7, WHITE);
        draw_circle(x + side * 6.5, y - 10.5, 2.0, CHEEKS);
    }
    // A smile: a shallow curve.
    let smile: Vec<Vec2> = (0..=6)
        .map(|i| {
            let t = i as f32 / 6.0 * 2.0 - 1.0;
            vec2(x + t * 2.8, y - 8.3 - (1.0 - t * t) * -1.2 - 1.2)
        })
        .collect();
    for pair in smile.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 1.1, EYES);
    }
}

const MONSTER_BODY: Color = Color::new(0.16, 0.04, 0.18, 1.0);
const MONSTER_EDGE: Color = Color::new(0.55, 0.05, 0.12, 1.0);
const MONSTER_EYE: Color = Color::new(1.0, 0.85, 0.2, 1.0);
const MONSTER_GLOW: Color = Color::new(1.0, 0.1, 0.05, 0.35);
const MONSTER_MOUTH: Color = Color::new(0.35, 0.0, 0.05, 1.0);

/// A spiky shadow monster with glowing eyes and jagged teeth, centered on
/// `pos` with roughly the given `radius`. `phase` keeps monsters from
/// moving in sync.
pub fn draw_monster(pos: Vec2, radius: f32, time: f32, phase: f32) {
    let t = time + phase;
    let (x, y) = (pos.x, pos.y);

    draw_ellipse(x, y + radius + 4.0, radius * 0.8, 4.0, 0.0, SHADOW);

    draw_monster_body(pos, radius, t);
    draw_monster_eyes(pos, radius, t);
    draw_monster_grin(pos, radius);
}

/// A writhing star, drawn as a fan of triangles from the center.
fn draw_monster_body(pos: Vec2, radius: f32, t: f32) {
    const SPIKES: usize = 11;
    let point = |i: usize, scale: f32| {
        let spike = i.is_multiple_of(2);
        let wobble = (t * 6.0 + i as f32 * 1.7).sin() * 0.08;
        let r = radius * if spike { 1.25 + wobble } else { 0.8 } * scale;
        let angle = i as f32 / (SPIKES * 2) as f32 * std::f32::consts::TAU + t * 0.4;
        pos + Vec2::from_angle(angle) * r
    };
    for (scale, color) in [(1.0, MONSTER_EDGE), (0.85, MONSTER_BODY)] {
        for i in 0..SPIKES * 2 {
            draw_triangle(pos, point(i, scale), point(i + 1, scale), color);
        }
    }
}

/// Glowing, slanted eyes that flicker.
fn draw_monster_eyes(pos: Vec2, radius: f32, t: f32) {
    let (x, y) = (pos.x, pos.y);
    let flicker = 0.8 + 0.2 * (t * 9.0).sin();
    for side in [-1.0, 1.0] {
        let eye = vec2(x + side * radius * 0.35, y - radius * 0.2);
        draw_circle(eye.x, eye.y, radius * 0.35, MONSTER_GLOW);
        draw_ellipse(
            eye.x,
            eye.y,
            radius * 0.22,
            radius * 0.12,
            side * 25.0,
            Color {
                a: flicker,
                ..MONSTER_EYE
            },
        );
        draw_circle(eye.x, eye.y, radius * 0.06, BLACK);
        // Angry brow slanting down toward the middle.
        draw_line(
            eye.x - side * radius * 0.28,
            eye.y - radius * 0.3,
            eye.x + side * radius * 0.2,
            eye.y - radius * 0.14,
            2.5,
            BLACK,
        );
    }
}

/// A jagged grin.
fn draw_monster_grin(pos: Vec2, radius: f32) {
    let (x, y) = (pos.x, pos.y);
    let mouth_w = radius * 0.9;
    let mouth_y = y + radius * 0.3;
    draw_rectangle(
        x - mouth_w / 2.0,
        mouth_y,
        mouth_w,
        radius * 0.3,
        MONSTER_MOUTH,
    );
    const TEETH: usize = 5;
    let tooth_w = mouth_w / TEETH as f32;
    for i in 0..TEETH {
        let left = x - mouth_w / 2.0 + i as f32 * tooth_w;
        draw_triangle(
            vec2(left, mouth_y),
            vec2(left + tooth_w, mouth_y),
            vec2(left + tooth_w / 2.0, mouth_y + radius * 0.2),
            WHITE,
        );
    }
}

const CYCLOPS_BODY: Color = Color::new(0.04, 0.16, 0.26, 1.0);
const CYCLOPS_EDGE: Color = Color::new(0.2, 0.55, 0.85, 1.0);
const CYCLOPS_IRIS: Color = Color::new(0.9, 0.1, 0.1, 1.0);

/// A wobbling one-eyed blob with dangling tentacles, centered on `pos`,
/// whose eye follows `look_at`. `phase` keeps monsters from moving in
/// sync.
pub fn draw_cyclops(pos: Vec2, radius: f32, time: f32, phase: f32, look_at: Vec2) {
    let t = time + phase;
    let (x, y) = (pos.x, pos.y);

    draw_ellipse(x, y + radius + 8.0, radius * 0.8, 4.0, 0.0, SHADOW);

    draw_cyclops_tentacles(pos, radius, t);
    draw_cyclops_body(pos, radius, t);
    draw_cyclops_eye(pos, radius, t, look_at);
    draw_cyclops_mouth(pos, radius);
}

/// Tentacles hang below the body and sway.
fn draw_cyclops_tentacles(pos: Vec2, radius: f32, t: f32) {
    const TENTACLES: usize = 4;
    let (x, y) = (pos.x, pos.y);
    for i in 0..TENTACLES {
        let across = (i as f32 + 0.5) / TENTACLES as f32 * 2.0 - 1.0;
        let root = vec2(x + across * radius * 0.7, y + radius * 0.5);
        let sway = (t * 5.0 + i as f32 * 1.3).sin() * radius * 0.3;
        let tip = root + vec2(sway, radius * 0.9);
        let half = radius * 0.16;
        draw_triangle(
            root - vec2(half, 0.0),
            root + vec2(half, 0.0),
            tip,
            CYCLOPS_EDGE,
        );
    }
}

/// A circle whose outline ripples.
fn draw_cyclops_body(pos: Vec2, radius: f32, t: f32) {
    const SEGMENTS: usize = 24;
    let point = |i: usize, scale: f32| {
        let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let ripple = 1.0 + 0.08 * (angle * 3.0 + t * 3.0).sin();
        pos + Vec2::from_angle(angle) * radius * ripple * scale
    };
    for (scale, color) in [(1.12, CYCLOPS_EDGE), (1.0, CYCLOPS_BODY)] {
        for i in 0..SEGMENTS {
            draw_triangle(pos, point(i, scale), point(i + 1, scale), color);
        }
    }
}

/// One big eye, glaring at the target, blinking now and then, under a heavy brow.
fn draw_cyclops_eye(pos: Vec2, radius: f32, t: f32, look_at: Vec2) {
    let (x, y) = (pos.x, pos.y);
    let eye = vec2(x, y - radius * 0.2);
    let blink = (t * 0.7).sin() > 0.97;
    let eye_height = if blink { 0.05 } else { 0.4 };
    draw_ellipse(eye.x, eye.y, radius * 0.45, radius * eye_height, 0.0, WHITE);
    if !blink {
        let gaze = (look_at - eye).normalize_or_zero() * radius * 0.18;
        let iris = eye + gaze;
        draw_circle(iris.x, iris.y, radius * 0.22, CYCLOPS_IRIS);
        draw_ellipse(iris.x, iris.y, radius * 0.05, radius * 0.18, 0.0, BLACK);
    }
    // A heavy brow over the eye.
    draw_line(
        eye.x - radius * 0.5,
        eye.y - radius * 0.45,
        eye.x + radius * 0.5,
        eye.y - radius * 0.3,
        3.0,
        BLACK,
    );
}

/// A wide mouth full of fangs.
fn draw_cyclops_mouth(pos: Vec2, radius: f32) {
    let (x, y) = (pos.x, pos.y);
    let mouth_y = y + radius * 0.35;
    let mouth_w = radius * 1.1;
    const FANGS: usize = 6;
    let fang_w = mouth_w / FANGS as f32;
    draw_line(
        x - mouth_w / 2.0,
        mouth_y,
        x + mouth_w / 2.0,
        mouth_y,
        2.0,
        BLACK,
    );
    for i in 0..FANGS {
        let left = x - mouth_w / 2.0 + i as f32 * fang_w;
        let (top, down) = if i % 2 == 0 {
            (mouth_y, radius * 0.22)
        } else {
            (mouth_y, -radius * 0.14)
        };
        draw_triangle(
            vec2(left, top),
            vec2(left + fang_w, top),
            vec2(left + fang_w / 2.0, top + down),
            WHITE,
        );
    }
}

const MOULD_SLIME: Color = Color::new(0.45, 0.6, 0.16, 1.0);
const MOULD_DARK: Color = Color::new(0.24, 0.34, 0.1, 1.0);
const MOULD_GLOSS: Color = Color::new(0.85, 0.95, 0.5, 0.55);
const MOULD_CAP: Color = Color::new(0.62, 0.45, 0.55, 1.0);
const MOULD_CAP_SPOT: Color = Color::new(0.93, 0.85, 0.7, 1.0);
const MOULD_STEM: Color = Color::new(0.85, 0.8, 0.6, 1.0);
const MOULD_ROT: Color = Color::new(0.55, 0.5, 0.12, 1.0);
const MOULD_PUS: Color = Color::new(0.92, 0.9, 0.55, 1.0);
const MOULD_SPORE: Color = Color::new(0.85, 0.9, 0.75, 0.9);
const MOULD_EYE: Color = Color::new(0.95, 0.92, 0.55, 1.0);

/// A wobbling slimy blob with a toadstool or two growing on it, `r` big,
/// centered on `pos`. The blobs of a trail and the head are both this.
fn draw_mould_blob(pos: Vec2, r: f32, t: f32, seed: f32) {
    let wobble = (t * 2.0 + seed).sin() * 0.06;
    draw_ellipse(pos.x, pos.y + r * 0.75, r * 1.0, r * 0.28, 0.0, SHADOW);
    draw_ellipse(
        pos.x,
        pos.y + r * 0.1,
        r * (1.05 + wobble),
        r * (0.9 - wobble),
        0.0,
        MOULD_DARK,
    );
    draw_ellipse(
        pos.x,
        pos.y,
        r * (0.95 + wobble),
        r * (0.8 - wobble),
        0.0,
        MOULD_SLIME,
    );
    // Lumps along the edge.
    for i in 0..5 {
        let a = seed + i as f32 * 1.3;
        let lump = pos + vec2(a.cos() * r * 0.8, a.sin() * r * 0.55);
        draw_circle(lump.x, lump.y, r * 0.25, MOULD_SLIME);
    }
    // Blotches of rot, pale pustules and a furry fringe of spores.
    for i in 0..6 {
        let a = seed * 1.7 + i as f32 * 1.05;
        let spot = pos + vec2(a.cos() * r * 0.55, a.sin() * r * 0.4);
        let color = if i % 2 == 0 { MOULD_ROT } else { MOULD_DARK };
        draw_circle(spot.x, spot.y, r * (0.12 + 0.05 * (i % 3) as f32), color);
    }
    for i in 0..3 {
        let a = seed * 0.7 + i as f32 * 2.3;
        let pustule = pos + vec2(a.cos() * r * 0.6, a.sin() * r * 0.4);
        draw_circle(pustule.x, pustule.y, r * 0.09, MOULD_PUS);
    }
    for i in 0..9 {
        let a = -3.1 + i as f32 * 0.4;
        let fuzz = pos + vec2(a.cos() * r * 0.85, a.sin() * r * 0.7);
        draw_circle(fuzz.x, fuzz.y, r * 0.05, MOULD_SPORE);
    }
    // Slime hangs from the bottom in strings.
    for i in 0..3 {
        let x = pos.x + (i as f32 - 1.0) * r * 0.5;
        let length = r * (0.25 + 0.12 * ((t * 1.5 + seed + i as f32 * 2.0).sin() * 0.5 + 0.5));
        draw_line(
            x,
            pos.y + r * 0.6,
            x,
            pos.y + r * 0.6 + length,
            r * 0.08,
            MOULD_SLIME,
        );
        draw_circle(x, pos.y + r * 0.6 + length, r * 0.07, MOULD_SLIME);
    }
    // The wet shine.
    draw_ellipse(
        pos.x - r * 0.3,
        pos.y - r * 0.35,
        r * 0.28,
        r * 0.14,
        -20.0,
        MOULD_GLOSS,
    );
}

/// A little toadstool: a stem, a spotted cap.
fn draw_toadstool(base: Vec2, size: f32, bob: f32) {
    let top = base + vec2(0.0, -size * (1.0 + bob));
    draw_line(base.x, base.y, top.x, top.y, size * 0.35, MOULD_STEM);
    draw_ellipse(top.x, top.y, size * 0.7, size * 0.38, 0.0, MOULD_CAP);
    draw_circle(
        top.x - size * 0.25,
        top.y - size * 0.05,
        size * 0.1,
        MOULD_CAP_SPOT,
    );
    draw_circle(
        top.x + size * 0.2,
        top.y - size * 0.1,
        size * 0.08,
        MOULD_CAP_SPOT,
    );
}

/// The mould's body: one slimy tube from the tail through each of its
/// bends to the head, `radius` thick, with blotches, pustules and a
/// toadstool or two along it. Drawn under the monsters.
pub fn draw_mould_body(body: &[Vec2], radius: f32, time: f32, phase: f32) {
    if body.len() < 2 {
        return;
    }
    let t = time + phase;
    let r = radius;
    // Points along the body, close enough to look like one piece.
    let mut samples: Vec<(Vec2, f32)> = Vec::new();
    let mut along = 0.0;
    for pair in body.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let length = a.distance(b);
        let steps = (length / 4.0).ceil().max(1.0) as usize;
        for k in 0..steps {
            samples.push((
                a.lerp(b, k as f32 / steps as f32),
                along + length * k as f32 / steps as f32,
            ));
        }
        along += length;
    }
    samples.push((body[body.len() - 1], along));
    let swell = |arc: f32| 1.0 + 0.17 * (arc * 0.045 + t * 1.5).sin();
    for &(p, _) in &samples {
        draw_ellipse(p.x, p.y + r * 0.7, r * 1.05, r * 0.3, 0.0, SHADOW);
    }
    for &(p, arc) in &samples {
        draw_circle(p.x, p.y + r * 0.1, r * swell(arc) * 1.1, MOULD_DARK);
    }
    for &(p, arc) in &samples {
        draw_circle(p.x, p.y, r * swell(arc), MOULD_SLIME);
    }
    // Slime strings hang from the underside, and the wet shine runs along
    // the top.
    for (k, &(p, arc)) in samples.iter().enumerate() {
        if k % 6 == 0 {
            draw_ellipse(
                p.x - r * 0.25,
                p.y - r * 0.4,
                r * 0.3,
                r * 0.12,
                -10.0,
                MOULD_GLOSS,
            );
        }
        if k % 9 == 4 {
            let length = r * (0.25 + 0.15 * (t * 1.5 + arc * 0.1).sin().abs());
            draw_line(
                p.x,
                p.y + r * 0.7,
                p.x,
                p.y + r * 0.7 + length,
                r * 0.1,
                MOULD_SLIME,
            );
            draw_circle(p.x, p.y + r * 0.7 + length, r * 0.09, MOULD_SLIME);
        }
    }
    // The decorations sit in the middle of each stretch, each from its
    // own bend, so they stay put as the head moves on.
    for pair in body.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if a.distance(b) < r {
            continue;
        }
        let seed = a.x * 0.013 + a.y * 0.017;
        let mid = a.lerp(b, 0.5);
        for i in 0..4 {
            let angle = seed * 3.0 + i as f32 * 1.7;
            let offset = a.lerp(b, 0.2 + 0.2 * i as f32)
                + vec2(angle.cos() * r * 0.35, angle.sin() * r * 0.35);
            let color = if i % 2 == 0 { MOULD_ROT } else { MOULD_DARK };
            draw_circle(offset.x, offset.y, r * (0.14 + 0.04 * i as f32), color);
        }
        let pustule = mid + vec2(seed.cos() * r * 0.4, -r * 0.2);
        draw_circle(pustule.x, pustule.y, r * 0.1, MOULD_PUS);
        draw_toadstool(
            mid + vec2(r * 0.2, -r * 0.3),
            r * 0.5,
            (t * 1.5 + seed).sin() * 0.05,
        );
    }
    for &(p, arc) in samples.iter().step_by(5) {
        let a = arc * 0.9 + phase;
        draw_circle(
            p.x + a.cos() * r * 0.5,
            p.y - r * 0.85,
            r * 0.05,
            MOULD_SPORE,
        );
    }
}

/// The head of the mould: a disgusting blob of slime and toadstools with a
/// vague face, a big pale eye, a small one, and a drooping mouth with a
/// drip. It wobbles; `phase` keeps moulds from wobbling in sync.
pub fn draw_mould(pos: Vec2, radius: f32, time: f32, phase: f32) {
    let t = time + phase;
    let r = radius;
    draw_mould_blob(pos, r, t, phase);
    draw_toadstool(
        pos + vec2(-r * 0.45, -r * 0.45),
        r * 0.4,
        (t * 1.3).sin() * 0.05,
    );
    draw_toadstool(
        pos + vec2(r * 0.5, -r * 0.35),
        r * 0.3,
        (t * 1.7 + 1.0).sin() * 0.05,
    );
    // A vague face: sunken eyes of different sizes, half-closed under
    // heavy lids, and a wide crooked mouth with a strand of drool.
    let big = pos + vec2(-r * 0.28, -r * 0.1);
    let small = pos + vec2(r * 0.3, -r * 0.02);
    for (eye, size) in [(big, r * 0.22), (small, r * 0.14)] {
        draw_circle(eye.x, eye.y, size * 1.25, MOULD_DARK);
        draw_circle(eye.x, eye.y, size, MOULD_EYE);
        draw_circle(eye.x + size * 0.1, eye.y + size * 0.25, size * 0.4, BLACK);
        // The lid droops over the upper half.
        draw_ellipse(
            eye.x,
            eye.y - size * 0.45,
            size * 1.15,
            size * 0.6,
            0.0,
            MOULD_SLIME,
        );
    }
    draw_line(
        pos.x - r * 0.4,
        pos.y + r * 0.32,
        pos.x + r * 0.1,
        pos.y + r * 0.4,
        r * 0.12,
        MOULD_DARK,
    );
    draw_line(
        pos.x + r * 0.1,
        pos.y + r * 0.4,
        pos.x + r * 0.42,
        pos.y + r * 0.26,
        r * 0.12,
        MOULD_DARK,
    );
    for tooth in [-0.2f32, 0.05, 0.25] {
        draw_triangle(
            vec2(pos.x + r * tooth, pos.y + r * 0.37),
            vec2(pos.x + r * (tooth + 0.07), pos.y + r * 0.36),
            vec2(pos.x + r * (tooth + 0.03), pos.y + r * 0.5),
            MOULD_PUS,
        );
    }
    let drool = r * (0.2 + 0.12 * (t * 2.5).sin().abs());
    draw_line(
        pos.x + r * 0.3,
        pos.y + r * 0.3,
        pos.x + r * 0.3,
        pos.y + r * 0.45 + drool,
        r * 0.06,
        MOULD_GLOSS,
    );
    draw_circle(
        pos.x + r * 0.3,
        pos.y + r * 0.45 + drool,
        r * 0.07,
        MOULD_GLOSS,
    );
}

const BIRD_FEATHER: Color = Color::new(0.2, 0.17, 0.3, 1.0);
const BIRD_FEATHER_EDGE: Color = Color::new(0.38, 0.33, 0.5, 1.0);
const BIRD_BONE: Color = Color::new(0.93, 0.9, 0.8, 1.0);
const BIRD_BONE_SHADE: Color = Color::new(0.68, 0.64, 0.56, 1.0);
const BIRD_SOCKET: Color = Color::new(0.08, 0.05, 0.1, 1.0);
const BIRD_EYE: Color = Color::new(1.0, 0.75, 0.2, 1.0);
const IRON: Color = Color::new(0.52, 0.57, 0.64, 1.0);
const IRON_DARK: Color = Color::new(0.3, 0.33, 0.4, 1.0);
const IRON_SHINE: Color = Color::new(0.85, 0.9, 0.95, 1.0);

/// A half-skeletal bird seen from the front, centered on `pos`: a round
/// skull with hollow, glowing eyes, an iron beak and iron claws, bony wings
/// with a few ragged feathers and an open ribcage. It beats its wings
/// harder, and opens its beak, while `attacking`. `phase` keeps birds from
/// flapping in sync.
pub fn draw_bird(pos: Vec2, radius: f32, time: f32, phase: f32, attacking: bool) {
    let t = time + phase;
    let r = radius;
    let flap = (t * if attacking { 14.0 } else { 6.0 }).sin();
    // It bobs with each wingbeat.
    let pos = pos + vec2(0.0, flap * r * 0.1);
    draw_ellipse(pos.x, pos.y + r * 1.9, r * 0.8, 4.0, 0.0, SHADOW);

    draw_bird_wings(pos, r, flap);
    draw_bird_claws(pos, r);
    draw_bird_body(pos, r);
    draw_bird_skull(pos, r, t, attacking);
}

/// Each wing: a bony arm bent at the elbow and wrist, with ragged feathers
/// along it and gaps where the bones show through.
fn draw_bird_wings(pos: Vec2, r: f32, flap: f32) {
    for side in [-1.0f32, 1.0] {
        let lift = flap * r * 0.8;
        let shoulder = pos + vec2(side * r * 0.5, -r * 0.1);
        let elbow = pos + vec2(side * r * 1.4, -r * 0.55 - lift * 0.5);
        let wrist = pos + vec2(side * r * 2.2, -r * 0.2 - lift);
        let tip = pos + vec2(side * r * 2.7, r * 0.35 - lift * 1.1);
        // Feathers hang from the arm; some are missing.
        for (i, along) in [0.25f32, 0.5, 0.75, 1.0].into_iter().enumerate() {
            if i == 2 {
                continue;
            }
            let base = elbow.lerp(tip, along);
            let length = r * (1.5 - 0.15 * i as f32);
            let end = base + vec2(side * r * 0.12, length * 0.9 + lift * 0.3);
            let width = r * 0.32;
            draw_triangle(
                base - vec2(width, 0.0),
                base + vec2(width, 0.0),
                end,
                BIRD_FEATHER_EDGE,
            );
            draw_triangle(
                base - vec2(width * 0.6, 0.0),
                base + vec2(width * 0.6, 0.0),
                base + (end - base) * 0.88,
                BIRD_FEATHER,
            );
        }
        for (a, b) in [(shoulder, elbow), (elbow, wrist), (wrist, tip)] {
            draw_line(a.x, a.y, b.x, b.y, r * 0.17, BIRD_BONE_SHADE);
            draw_line(a.x, a.y - 1.0, b.x, b.y - 1.0, r * 0.1, BIRD_BONE);
        }
        for joint in [elbow, wrist] {
            draw_circle(joint.x, joint.y, r * 0.16, BIRD_BONE);
        }
    }
}

/// Bony legs ending in iron talons.
fn draw_bird_claws(pos: Vec2, r: f32) {
    for side in [-1.0f32, 1.0] {
        let hip = pos + vec2(side * r * 0.3, r * 0.8);
        let foot = pos + vec2(side * r * 0.45, r * 1.5);
        draw_line(hip.x, hip.y, foot.x, foot.y, r * 0.14, BIRD_BONE);
        for toe in [-1.0f32, 0.0, 1.0] {
            let tip = foot + vec2((side * 0.3 + toe * 0.45) * r, r * 0.4);
            draw_triangle(
                foot + vec2(-r * 0.12, 0.0),
                foot + vec2(r * 0.12, 0.0),
                tip,
                IRON,
            );
            draw_line(foot.x, foot.y, tip.x, tip.y, 1.0, IRON_SHINE);
        }
    }
}

/// A feathered body whose chest is open to show the ribs.
fn draw_bird_body(pos: Vec2, r: f32) {
    draw_ellipse(
        pos.x,
        pos.y + r * 0.4,
        r * 0.85,
        r * 0.95,
        0.0,
        BIRD_FEATHER_EDGE,
    );
    draw_ellipse(
        pos.x,
        pos.y + r * 0.4,
        r * 0.75,
        r * 0.85,
        0.0,
        BIRD_FEATHER,
    );
    // The open chest, with curved ribs across it and a bony spine.
    draw_ellipse(pos.x, pos.y + r * 0.5, r * 0.42, r * 0.6, 0.0, BIRD_SOCKET);
    for i in 0..3 {
        let y = pos.y + r * (0.2 + 0.27 * i as f32);
        let half = r * (0.4 - 0.04 * i as f32);
        draw_line(pos.x - half, y, pos.x, y + r * 0.1, r * 0.08, BIRD_BONE);
        draw_line(pos.x + half, y, pos.x, y + r * 0.1, r * 0.08, BIRD_BONE);
    }
    draw_line(
        pos.x,
        pos.y + r * 0.1,
        pos.x,
        pos.y + r * 0.95,
        r * 0.07,
        BIRD_BONE_SHADE,
    );
}

/// A big round skull, friendlier than frightening: hollow eyes with a
/// warm glow, and an iron beak that opens in the attack.
fn draw_bird_skull(pos: Vec2, r: f32, t: f32, attacking: bool) {
    let head = pos + vec2(0.0, -r * 0.65);
    draw_circle(head.x, head.y, r * 0.78, BIRD_BONE_SHADE);
    draw_circle(head.x, head.y - r * 0.03, r * 0.72, BIRD_BONE);
    let glow = 0.8 + 0.2 * (t * 7.0).sin();
    for side in [-1.0f32, 1.0] {
        let eye = head + vec2(side * r * 0.3, -r * 0.05);
        draw_circle(eye.x, eye.y, r * 0.24, BIRD_SOCKET);
        draw_circle(
            eye.x,
            eye.y,
            r * 0.12,
            Color {
                a: glow,
                ..BIRD_EYE
            },
        );
        // A small spark of light keeps the eyes lively.
        draw_circle(eye.x - r * 0.04, eye.y - r * 0.05, r * 0.035, WHITE);
    }
    // The beak: an iron hook, split in two while attacking.
    let top = head + vec2(0.0, r * 0.15);
    let gap = if attacking { r * 0.18 } else { 0.0 };
    draw_triangle(
        top + vec2(-r * 0.26, 0.0),
        top + vec2(r * 0.26, 0.0),
        top + vec2(r * 0.04, r * 0.8 - gap),
        IRON,
    );
    draw_triangle(
        top + vec2(-r * 0.12, r * 0.2 + gap),
        top + vec2(r * 0.12, r * 0.2 + gap),
        top + vec2(0.0, r * 0.55 + gap),
        IRON_DARK,
    );
    draw_line(
        top.x - r * 0.08,
        top.y + r * 0.04,
        top.x + r * 0.0,
        top.y + r * 0.5 - gap,
        1.5,
        IRON_SHINE,
    );
}

const BOSS_AURA: Color = Color::new(0.7, 0.0, 0.15, 0.25);
const BOSS_BODY: Color = Color::new(0.5, 0.03, 0.1, 1.0);
const BOSS_EDGE: Color = Color::new(0.2, 0.0, 0.04, 1.0);
const BOSS_WING: Color = Color::new(0.2, 0.02, 0.12, 0.95);
const BOSS_BONE: Color = Color::new(0.08, 0.0, 0.04, 1.0);
const BOSS_EYE: Color = Color::new(0.5, 1.0, 0.3, 1.0);
const BOSS_EYE_GLOW: Color = Color::new(0.3, 1.0, 0.2, 0.3);
const CROWN: Color = Color::new(1.0, 0.8, 0.15, 1.0);
const CROWN_SHADE: Color = Color::new(0.75, 0.5, 0.05, 1.0);
const JEWEL: Color = Color::new(0.9, 0.05, 0.2, 1.0);

/// The level boss: a crowned, three-eyed demon on flapping bat wings, in
/// a pulsing red aura.
pub fn draw_boss(pos: Vec2, radius: f32, time: f32, phase: f32) {
    let t = time + phase;
    let r = radius;
    // It hovers, bobbing up and down with its wingbeats.
    let flap = (t * 6.0).sin();
    let pos = pos + vec2(0.0, flap * r * 0.06);
    let (x, y) = (pos.x, pos.y);

    let pulse = 1.0 + 0.1 * (t * 4.0).sin();
    draw_circle(x, y, r * 1.7 * pulse, BOSS_AURA);
    draw_ellipse(x, y + r * 1.25, r * 0.7, 5.0, 0.0, SHADOW);

    draw_boss_wings(pos, r, flap);
    draw_boss_body(pos, r, t);
    draw_boss_crown(pos, r);
    draw_boss_eyes(pos, r, t);
    draw_boss_maw(pos, r);
}

/// Bat wings: a fan of bony fingers with membrane between them.
fn draw_boss_wings(pos: Vec2, r: f32, flap: f32) {
    for side in [-1.0f32, 1.0] {
        let root = pos + vec2(side * r * 0.6, -r * 0.15);
        let fingers = [(-0.9, 1.35), (-0.35, 1.55), (0.2, 1.15)];
        let tips: Vec<Vec2> = fingers
            .iter()
            .map(|&(angle, length)| {
                let angle = angle - flap * 0.35;
                root + vec2(side * angle.cos(), angle.sin()) * r * length
            })
            .collect();
        for pair in tips.windows(2) {
            draw_triangle(root, pair[0], pair[1], BOSS_WING);
        }
        for tip in &tips {
            draw_line(root.x, root.y, tip.x, tip.y, 2.5, BOSS_BONE);
            draw_circle(tip.x, tip.y, 2.5, BOSS_BONE);
        }
    }
}

/// A round, gently rippling mass.
fn draw_boss_body(pos: Vec2, r: f32, t: f32) {
    const SEGMENTS: usize = 28;
    let point = |i: usize, scale: f32| {
        let angle = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let ripple = 1.0 + 0.04 * (angle * 5.0 + t * 2.0).sin();
        pos + Vec2::from_angle(angle) * r * ripple * scale
    };
    for (scale, color) in [(1.08, BOSS_EDGE), (1.0, BOSS_BODY)] {
        for i in 0..SEGMENTS {
            draw_triangle(pos, point(i, scale), point(i + 1, scale), color);
        }
    }
}

/// A golden crown with five spikes and a jewel.
fn draw_boss_crown(pos: Vec2, r: f32) {
    let (x, y) = (pos.x, pos.y);
    let band_w = r * 1.1;
    let band_h = r * 0.24;
    let band_top = y - r * 0.98;
    const SPIKES: usize = 5;
    let spike_w = band_w / SPIKES as f32;
    for i in 0..SPIKES {
        let left = x - band_w / 2.0 + i as f32 * spike_w;
        let height = if i == SPIKES / 2 { r * 0.42 } else { r * 0.3 };
        let tip = vec2(left + spike_w / 2.0, band_top - height);
        draw_triangle(
            vec2(left, band_top),
            vec2(left + spike_w, band_top),
            tip,
            CROWN,
        );
        draw_circle(tip.x, tip.y, r * 0.05, CROWN);
    }
    draw_rectangle(x - band_w / 2.0, band_top, band_w, band_h, CROWN);
    draw_rectangle(
        x - band_w / 2.0,
        band_top + band_h * 0.7,
        band_w,
        band_h * 0.3,
        CROWN_SHADE,
    );
    draw_circle(x, band_top + band_h / 2.0, r * 0.09, JEWEL);
}

/// Three glowing eyes: two below and one on the forehead.
fn draw_boss_eyes(pos: Vec2, r: f32, t: f32) {
    let (x, y) = (pos.x, pos.y);
    let flicker = 0.85 + 0.15 * (t * 11.0).sin();
    let eyes = [
        (vec2(x - r * 0.38, y - r * 0.12), r * 0.17),
        (vec2(x + r * 0.38, y - r * 0.12), r * 0.17),
        (vec2(x, y - r * 0.5), r * 0.13),
    ];
    for (eye, size) in eyes {
        draw_circle(eye.x, eye.y, size * 1.9, BOSS_EYE_GLOW);
        draw_circle(
            eye.x,
            eye.y,
            size,
            Color {
                a: flicker,
                ..BOSS_EYE
            },
        );
        draw_ellipse(eye.x, eye.y, size * 0.25, size * 0.85, 0.0, BLACK);
    }
}

/// A wide black maw with two long fangs and a row of small teeth.
fn draw_boss_maw(pos: Vec2, r: f32) {
    let (x, y) = (pos.x, pos.y);
    let mouth_y = y + r * 0.42;
    let mouth_w = r * 0.95;
    draw_ellipse(x, mouth_y, mouth_w / 2.0, r * 0.2, 0.0, BLACK);
    const TEETH: usize = 7;
    let tooth_w = mouth_w * 0.7 / TEETH as f32;
    for i in 0..TEETH {
        let left = x - mouth_w * 0.35 + i as f32 * tooth_w;
        draw_triangle(
            vec2(left, mouth_y - r * 0.12),
            vec2(left + tooth_w, mouth_y - r * 0.12),
            vec2(left + tooth_w / 2.0, mouth_y),
            WHITE,
        );
    }
    for side in [-1.0, 1.0] {
        let base = x + side * mouth_w * 0.32;
        draw_triangle(
            vec2(base - r * 0.07, mouth_y - r * 0.14),
            vec2(base + r * 0.07, mouth_y - r * 0.14),
            vec2(base, mouth_y + r * 0.35),
            WHITE,
        );
    }
}

const PORTAL_GLOW: Color = Color::new(0.5, 0.2, 0.9, 0.18);
const PORTAL_RIM: Color = Color::new(0.6, 0.4, 1.0, 0.9);
const PORTAL_CORE: Color = Color::new(0.03, 0.0, 0.08, 1.0);
const PORTAL_ARM: [Color; 2] = [
    Color::new(0.4, 0.8, 1.0, 1.0),
    Color::new(0.8, 0.4, 1.0, 1.0),
];

/// A swirling vortex lying flat on the ground, centered on `pos`.
pub fn draw_portal(pos: Vec2, time: f32) {
    const RADIUS: f32 = 30.0;
    /// Seen at an angle, so squashed vertically.
    const TILT: f32 = 0.55;
    const ARMS: usize = 3;
    const DOTS: usize = 14;

    let pulse = 1.0 + 0.06 * (time * 3.0).sin();
    let r = RADIUS * pulse;
    draw_ellipse(pos.x, pos.y, r * 1.6, r * 1.6 * TILT, 0.0, PORTAL_GLOW);
    draw_ellipse(pos.x, pos.y, r, r * TILT, 0.0, PORTAL_CORE);
    draw_ellipse_lines(pos.x, pos.y, r, r * TILT, 0.0, 2.5, PORTAL_RIM);

    // Spiral arms of dots, turning inward and shrinking toward the middle.
    for arm in 0..ARMS {
        let offset = arm as f32 / ARMS as f32 * std::f32::consts::TAU;
        for dot in 0..DOTS {
            let along = dot as f32 / DOTS as f32;
            let angle = offset + along * 4.0 - time * 2.5;
            let dist = r * (1.0 - along) * 0.95;
            let p = pos + vec2(angle.cos() * dist, angle.sin() * dist * TILT);
            let color = PORTAL_ARM[(arm + dot) % 2];
            let size = 2.8 * (1.0 - along) + 0.8;
            draw_circle(
                p.x,
                p.y,
                size,
                Color {
                    a: 1.0 - along * 0.6,
                    ..color
                },
            );
        }
    }
}

const STONE: Color = Color::new(0.45, 0.45, 0.5, 1.0);
const STONE_DARK: Color = Color::new(0.28, 0.28, 0.32, 1.0);
const STONE_LIGHT: Color = Color::new(0.62, 0.62, 0.68, 1.0);
const TRUNK: Color = Color::new(0.4, 0.26, 0.12, 1.0);
const LEAVES_DARK: Color = Color::new(0.08, 0.3, 0.12, 1.0);
const LEAVES: Color = Color::new(0.14, 0.45, 0.18, 1.0);
const LEAVES_LIGHT: Color = Color::new(0.3, 0.62, 0.28, 1.0);
const SHORE: Color = Color::new(0.35, 0.5, 0.25, 1.0);
const WATER_DEEP: Color = Color::new(0.1, 0.25, 0.55, 1.0);
const WATER: Color = Color::new(0.2, 0.42, 0.75, 1.0);
const WATER_SHINE: Color = Color::new(0.75, 0.9, 1.0, 0.7);

/// A closed outline around `pos` whose radius at each point is
/// `radius` scaled by `min..max` according to `shape`.
fn outline(pos: Vec2, radius: f32, shape: &[f32], min: f32, max: f32) -> Vec<Vec2> {
    shape
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let angle = i as f32 / shape.len() as f32 * std::f32::consts::TAU;
            pos + Vec2::from_angle(angle) * radius * (min + (max - min) * s)
        })
        .collect()
}

/// Fills a star-shaped polygon as a fan of triangles from `center`.
fn fill_fan(center: Vec2, points: &[Vec2], color: Color) {
    for i in 0..points.len() {
        let next = points[(i + 1) % points.len()];
        draw_triangle(center, points[i], next, color);
    }
}

/// Draws a stone, tree or lake filling its blocked circle.
pub fn draw_obstacle(obstacle: &Obstacle, time: f32) {
    let (pos, r, shape) = (obstacle.pos, obstacle.radius, &obstacle.shape);
    match obstacle.kind {
        Kind::Stone => draw_stone(pos, r, shape),
        Kind::Tree => draw_tree(pos, r, shape),
        Kind::Lake => draw_lake(pos, r, shape, time),
    }
}

/// A grey boulder with a highlight and a crack.
fn draw_stone(pos: Vec2, r: f32, shape: &[f32]) {
    draw_ellipse(
        pos.x + 4.0,
        pos.y + r * 0.6,
        r * 1.05,
        r * 0.45,
        0.0,
        SHADOW,
    );
    fill_fan(pos, &outline(pos, r, shape, 0.9, 1.1), STONE_DARK);
    fill_fan(pos, &outline(pos, r, shape, 0.78, 0.95), STONE);
    // A highlight on the upper left, and a crack.
    let light = pos + vec2(-r * 0.3, -r * 0.3);
    draw_ellipse(light.x, light.y, r * 0.35, r * 0.2, -30.0, STONE_LIGHT);
    let crack = [
        pos + vec2(r * 0.1, -r * 0.1),
        pos + vec2(r * 0.25, r * 0.15),
        pos + vec2(r * 0.15, r * 0.4),
    ];
    for pair in crack.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, 1.5, STONE_DARK);
    }
}

/// A trunk under a canopy of overlapping leaf clusters.
fn draw_tree(pos: Vec2, r: f32, shape: &[f32]) {
    draw_ellipse(pos.x + 6.0, pos.y + r * 0.7, r * 1.1, r * 0.45, 0.0, SHADOW);
    draw_rectangle(pos.x - r * 0.15, pos.y, r * 0.3, r * 0.75, TRUNK);
    // A canopy of overlapping leaf clusters.
    draw_circle(pos.x, pos.y, r, LEAVES_DARK);
    for i in 0..5 {
        let angle = i as f32 / 5.0 * std::f32::consts::TAU + shape[0] * 3.0;
        let cluster = pos + Vec2::from_angle(angle) * r * 0.45;
        draw_circle(
            cluster.x,
            cluster.y,
            r * (0.45 + 0.15 * shape[i + 1]),
            LEAVES,
        );
    }
    draw_circle(pos.x - r * 0.25, pos.y - r * 0.3, r * 0.35, LEAVES_LIGHT);
}

/// A lake with a shore, a deeper middle and glints drifting across it.
fn draw_lake(pos: Vec2, r: f32, shape: &[f32], time: f32) {
    fill_fan(pos, &outline(pos, r, shape, 1.0, 1.12), SHORE);
    fill_fan(pos, &outline(pos, r, shape, 0.88, 1.0), WATER);
    fill_fan(pos, &outline(pos, r * 0.6, shape, 0.85, 1.0), WATER_DEEP);
    // Glints drifting slowly across the surface.
    for (i, s) in shape.iter().take(3).enumerate() {
        let t = time * 0.3 + s * 10.0;
        let glint = pos
            + vec2(
                (t + i as f32 * 2.0).sin() * r * 0.45,
                (t * 0.7 + i as f32).cos() * r * 0.35,
            );
        let alpha = 0.4 + 0.3 * (time * 2.0 + i as f32).sin();
        draw_line(
            glint.x - r * 0.12,
            glint.y,
            glint.x + r * 0.12,
            glint.y,
            2.0,
            Color {
                a: alpha,
                ..WATER_SHINE
            },
        );
    }
}

/// A five-pointed star centered on `pos`: filled gold if earned, or a dim
/// outline if not.
pub fn draw_star(pos: Vec2, radius: f32, earned: bool) {
    let point = |i: usize| {
        let r = if i.is_multiple_of(2) {
            radius
        } else {
            radius * 0.45
        };
        let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        pos + Vec2::from_angle(angle) * r
    };
    if earned {
        for i in 0..10 {
            draw_triangle(pos, point(i), point(i + 1), GOLD);
        }
    }
    let edge = if earned {
        Color::new(1.0, 0.95, 0.6, 1.0)
    } else {
        Color::new(1.0, 1.0, 1.0, 0.35)
    };
    for i in 0..10 {
        let (a, b) = (point(i), point(i + 1));
        draw_line(a.x, a.y, b.x, b.y, 2.0, edge);
    }
}
