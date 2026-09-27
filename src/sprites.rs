//! Procedurally drawn characters.

use macroquad::prelude::*;

const SKIN: Color = Color::new(1.0, 0.86, 0.75, 1.0);
const HAIR: Color = Color::new(0.98, 0.84, 0.42, 1.0);
const DRESS: Color = Color::new(1.0, 0.41, 0.71, 1.0);
const DRESS_LIGHT: Color = Color::new(1.0, 0.75, 0.87, 1.0);
const SHOES: Color = Color::new(0.85, 0.2, 0.5, 1.0);
const CHEEKS: Color = Color::new(1.0, 0.5, 0.6, 0.6);
const EYES: Color = Color::new(0.15, 0.1, 0.1, 1.0);

const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.35);

/// Which side of her body faces `toward`: -1.0 for left, 1.0 for right.
fn side_toward(pos: Vec2, toward: Vec2) -> f32 {
    if toward.x < pos.x { -1.0 } else { 1.0 }
}

/// Where her raised hand is while casting toward `toward`.
pub fn girl_hand(pos: Vec2, toward: Vec2) -> Vec2 {
    let side = side_toward(pos, toward);
    let shoulder = vec2(pos.x + side * 6.0, pos.y - 3.0);
    shoulder + (toward - shoulder).normalize_or(vec2(side, 0.0)) * 13.0
}

/// A little girl in a pink dress, centered on `pos`. `time` animates her
/// walk while `moving` is true. While `casting` toward a point, she points
/// a glowing hand at it.
pub fn draw_girl(pos: Vec2, time: f32, moving: bool, casting: Option<Vec2>) {
    let step = if moving { (time * 14.0).sin() } else { 0.0 };
    let x = pos.x;
    let y = pos.y - step.abs() * 2.0;

    draw_ellipse(pos.x, pos.y + 22.0, 13.0, 4.0, 0.0, SHADOW);

    // Legs and shoes, stepping in turn.
    for (side, lift) in [(-1.0, step.max(0.0)), (1.0, (-step).max(0.0))] {
        let foot_y = y + 20.0 - lift * 3.0;
        draw_line(x + side * 4.0, y + 12.0, x + side * 4.0, foot_y, 3.0, SKIN);
        draw_ellipse(x + side * 5.0, foot_y + 1.0, 4.0, 2.5, 0.0, SHOES);
    }

    // Arms swing against the legs; a casting arm points at its target.
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

    // Dress: a flared skirt with a collar and a belt.
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

    // Pigtails with pink bows.
    for side in [-1.0, 1.0] {
        let tail = vec2(x + side * 13.0, y - 15.0);
        draw_circle(tail.x, tail.y, 5.0, HAIR);
        let bow = vec2(x + side * 10.0, y - 21.0);
        draw_triangle(bow, bow + vec2(-4.0, -3.0), bow + vec2(-4.0, 3.0), DRESS);
        draw_triangle(bow, bow + vec2(4.0, -3.0), bow + vec2(4.0, 3.0), DRESS);
        draw_circle(bow.x, bow.y, 1.5, DRESS_LIGHT);
    }

    // Head: hair behind, face, then bangs.
    draw_circle(x, y - 16.0, 11.5, HAIR);
    draw_circle(x, y - 14.0, 10.0, SKIN);
    draw_ellipse(x, y - 22.0, 9.0, 4.0, 0.0, HAIR);

    // Face.
    for side in [-1.0, 1.0] {
        draw_circle(x + side * 4.0, y - 14.0, 2.0, EYES);
        draw_circle(x + side * 4.0 + 0.7, y - 14.7, 0.7, WHITE);
        draw_circle(x + side * 6.5, y - 10.5, 2.0, CHEEKS);
    }
    draw_line(x - 2.5, y - 9.5, x, y - 8.5, 1.2, EYES);
    draw_line(x, y - 8.5, x + 2.5, y - 9.5, 1.2, EYES);
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
    const SPIKES: usize = 11;
    let t = time + phase;
    let (x, y) = (pos.x, pos.y);

    draw_ellipse(x, y + radius + 4.0, radius * 0.8, 4.0, 0.0, SHADOW);

    // Body: a writhing star, drawn as a fan of triangles from the center.
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

    // Glowing, slanted eyes that flicker.
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

    // Jagged grin.
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
