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

const CYCLOPS_BODY: Color = Color::new(0.04, 0.16, 0.26, 1.0);
const CYCLOPS_EDGE: Color = Color::new(0.2, 0.55, 0.85, 1.0);
const CYCLOPS_IRIS: Color = Color::new(0.9, 0.1, 0.1, 1.0);

/// A wobbling one-eyed blob with dangling tentacles, centered on `pos`,
/// whose eye follows `look_at`. `phase` keeps monsters from moving in
/// sync.
pub fn draw_cyclops(pos: Vec2, radius: f32, time: f32, phase: f32, look_at: Vec2) {
    const SEGMENTS: usize = 24;
    const TENTACLES: usize = 4;
    let t = time + phase;
    let (x, y) = (pos.x, pos.y);

    draw_ellipse(x, y + radius + 8.0, radius * 0.8, 4.0, 0.0, SHADOW);

    // Tentacles hang below the body and sway.
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

    // Body: a circle whose outline ripples.
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

    // One big eye, glaring at the target, blinking now and then.
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

    // A wide mouth full of fangs.
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

    // Bat wings: a fan of bony fingers with membrane between them.
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

    // Body: a round, gently rippling mass.
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

    // A golden crown with five spikes and a jewel.
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

    // Three glowing eyes: two below and one on the forehead.
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

    // A wide black maw with two long fangs and a row of small teeth.
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
        Kind::Stone => {
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
        Kind::Tree => {
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
        Kind::Lake => {
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
    }
}
