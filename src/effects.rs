use macroquad::prelude::*;

use crate::rng::{Rng, Stream};
use crate::view::View;

const PARTICLES_PER_EXPLOSION: usize = 40;
const RING_SECONDS: f32 = 0.35;
const LIGHTNING_SECONDS: f32 = 0.45;
const FLASH_SECONDS: f32 = 0.6;
const BOLT_SEGMENTS: usize = 14;
const BOLT_BRANCHES: usize = 3;

struct Particle {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    lifetime: f32,
    size: f32,
    color: Color,
}

struct Ring {
    pos: Vec2,
    age: f32,
    start_radius: f32,
    color: Color,
}

/// A jagged lightning bolt, as a polyline.
struct Bolt {
    points: Vec<Vec2>,
    width: f32,
}

/// Short-lived visual effects that don't affect gameplay.
pub struct Effects {
    rng: Rng,
    particles: Vec<Particle>,
    rings: Vec<Ring>,
    bolts: Vec<Bolt>,
    /// Seconds since the last lightning strike, while it is visible.
    lightning_age: Option<f32>,
}

impl Default for Effects {
    fn default() -> Self {
        Effects {
            rng: Rng::new(Stream::Effects, 0),
            particles: Vec::new(),
            rings: Vec::new(),
            bolts: Vec::new(),
            lightning_age: None,
        }
    }
}

/// A jagged line from `from` to `to`, wandering sideways by up to
/// `jitter`.
fn jagged(rng: &mut Rng, from: Vec2, to: Vec2, segments: usize, jitter: f32) -> Vec<Vec2> {
    let side = (to - from).perp().normalize_or_zero();
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let wander = if i == 0 || i == segments {
                0.0
            } else {
                rng.range(-jitter, jitter)
            };
            from.lerp(to, t) + side * wander
        })
        .collect()
}

impl Effects {
    /// A burst of sparks and a shockwave ring, tinted with `palette`.
    pub fn explode(&mut self, pos: Vec2, radius: f32, palette: &[Color]) {
        for _ in 0..PARTICLES_PER_EXPLOSION {
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            let speed = self.rng.range(50.0, 280.0);
            self.particles.push(Particle {
                pos,
                vel: Vec2::from_angle(angle) * speed,
                age: 0.0,
                lifetime: self.rng.range(0.4, 0.9),
                size: self.rng.range(2.0, 5.0),
                color: *self.rng.pick(palette),
            });
        }
        self.rings.push(Ring {
            pos,
            age: 0.0,
            start_radius: radius,
            color: palette[0],
        });
    }

    /// `count` explosions scattered up to `spread` away from `pos`.
    pub fn explode_around(
        &mut self,
        pos: Vec2,
        spread: f32,
        count: usize,
        radius: f32,
        palette: &[Color],
    ) {
        for _ in 0..count {
            let offset = vec2(
                self.rng.range(-spread, spread),
                self.rng.range(-spread, spread),
            );
            self.explode(pos + offset, radius, palette);
        }
    }

    /// A few slow sparkles, left behind by something moving through `pos`.
    pub fn trail(&mut self, pos: Vec2, palette: &[Color]) {
        for _ in 0..3 {
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            self.particles.push(Particle {
                pos,
                vel: Vec2::from_angle(angle) * self.rng.range(10.0, 50.0),
                age: 0.0,
                lifetime: self.rng.range(0.2, 0.45),
                size: self.rng.range(1.5, 3.0),
                color: *self.rng.pick(palette),
            });
        }
    }

    /// A lightning bolt from the top of the screen down to `to`, with a
    /// few branches, a burst at the impact and a screen-wide flash.
    pub fn lightning(&mut self, to: Vec2) {
        let from = vec2(to.x + self.rng.range(-120.0, 120.0), -10.0);
        let trunk = jagged(&mut self.rng, from, to, BOLT_SEGMENTS, 25.0);
        self.bolts.clear();
        for _ in 0..BOLT_BRANCHES {
            let start = trunk[self.rng.index(2..BOLT_SEGMENTS - 2)];
            let side = if self.rng.chance(0.5) { -1.0 } else { 1.0 };
            let angle = self.rng.range(0.3, 1.2) * side;
            let end = start
                + Vec2::from_angle(std::f32::consts::FRAC_PI_2 + angle)
                    * self.rng.range(60.0, 140.0);
            self.bolts.push(Bolt {
                points: jagged(&mut self.rng, start, end, 5, 12.0),
                width: 2.0,
            });
        }
        self.bolts.push(Bolt {
            points: trunk,
            width: 4.0,
        });
        self.lightning_age = Some(0.0);
        self.explode(to, 40.0, &[WHITE, SKYBLUE, YELLOW, WHITE]);
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(age) = &mut self.lightning_age {
            *age += dt;
            if *age >= FLASH_SECONDS.max(LIGHTNING_SECONDS) {
                self.lightning_age = None;
                self.bolts.clear();
            }
        }

        let drag = (-3.0 * dt).exp();
        for p in &mut self.particles {
            p.age += dt;
            p.pos += p.vel * dt;
            p.vel *= drag;
        }
        self.particles.retain(|p| p.age < p.lifetime);

        for ring in &mut self.rings {
            ring.age += dt;
        }
        self.rings.retain(|r| r.age < RING_SECONDS);
    }

    pub fn draw(&self) {
        for ring in &self.rings {
            let t = ring.age / RING_SECONDS;
            let radius = ring.start_radius * (1.0 + 2.0 * t);
            let color = Color {
                a: 1.0 - t,
                ..ring.color
            };
            draw_circle_lines(ring.pos.x, ring.pos.y, radius, 3.0, color);
        }
        for p in &self.particles {
            let fade = 1.0 - p.age / p.lifetime;
            let color = Color { a: fade, ..p.color };
            draw_circle(p.pos.x, p.pos.y, p.size * fade.max(0.3), color);
        }
        self.draw_bolts();
    }

    fn draw_bolts(&self) {
        let Some(age) = self.lightning_age else {
            return;
        };
        // Real lightning flickers: skip every third frame-ish slice.
        if age >= LIGHTNING_SECONDS || (age * 30.0) as u32 % 3 == 2 {
            return;
        }
        let fade = 1.0 - age / LIGHTNING_SECONDS;
        let glow = Color::new(0.6, 0.8, 1.0, 0.35 * fade);
        let core = Color::new(1.0, 1.0, 1.0, fade);
        for bolt in &self.bolts {
            for pair in bolt.points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                draw_line(a.x, a.y, b.x, b.y, bolt.width * 4.0, glow);
                draw_line(a.x, a.y, b.x, b.y, bolt.width, core);
            }
        }
    }

    /// Whitens the whole screen right after a lightning strike. Drawn on
    /// top of everything else.
    pub fn draw_flash(&self, view: View) {
        if let Some(age) = self.lightning_age
            && age < FLASH_SECONDS
        {
            let alpha = 0.7 * (1.0 - age / FLASH_SECONDS).powi(2);
            let screen = view.visible();
            draw_rectangle(
                screen.x,
                screen.y,
                screen.w,
                screen.h,
                Color::new(0.9, 0.95, 1.0, alpha),
            );
        }
    }
}
