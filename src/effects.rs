use macroquad::prelude::*;

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
#[derive(Default)]
pub struct Effects {
    particles: Vec<Particle>,
    rings: Vec<Ring>,
    bolts: Vec<Bolt>,
    /// Seconds since the last lightning strike, while it is visible.
    lightning_age: Option<f32>,
}

/// A jagged line from `from` to `to`, wandering sideways by up to
/// `jitter`.
fn jagged(from: Vec2, to: Vec2, segments: usize, jitter: f32) -> Vec<Vec2> {
    let side = (to - from).perp().normalize_or_zero();
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let wander = if i == 0 || i == segments {
                0.0
            } else {
                rand::gen_range(-jitter, jitter)
            };
            from.lerp(to, t) + side * wander
        })
        .collect()
}

impl Effects {
    /// A burst of sparks and a shockwave ring, tinted with `palette`.
    pub fn explode(&mut self, pos: Vec2, radius: f32, palette: &[Color]) {
        for _ in 0..PARTICLES_PER_EXPLOSION {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(50.0, 280.0);
            self.particles.push(Particle {
                pos,
                vel: Vec2::from_angle(angle) * speed,
                age: 0.0,
                lifetime: rand::gen_range(0.4, 0.9),
                size: rand::gen_range(2.0, 5.0),
                color: palette[rand::gen_range(0, palette.len())],
            });
        }
        self.rings.push(Ring {
            pos,
            age: 0.0,
            start_radius: radius,
            color: palette[0],
        });
    }

    /// A few slow sparkles, left behind by something moving through `pos`.
    pub fn trail(&mut self, pos: Vec2, palette: &[Color]) {
        for _ in 0..3 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            self.particles.push(Particle {
                pos,
                vel: Vec2::from_angle(angle) * rand::gen_range(10.0, 50.0),
                age: 0.0,
                lifetime: rand::gen_range(0.2, 0.45),
                size: rand::gen_range(1.5, 3.0),
                color: palette[rand::gen_range(0, palette.len())],
            });
        }
    }

    /// A lightning bolt from the top of the screen down to `to`, with a
    /// few branches, a burst at the impact and a screen-wide flash.
    pub fn lightning(&mut self, to: Vec2) {
        let from = vec2(to.x + rand::gen_range(-120.0, 120.0), -10.0);
        let trunk = jagged(from, to, BOLT_SEGMENTS, 25.0);
        self.bolts.clear();
        for _ in 0..BOLT_BRANCHES {
            let start = trunk[rand::gen_range(2, BOLT_SEGMENTS - 2)];
            let angle = rand::gen_range(0.3, 1.2)
                * if rand::gen_range(0, 2) == 0 {
                    -1.0
                } else {
                    1.0
                };
            let end = start
                + Vec2::from_angle(std::f32::consts::FRAC_PI_2 + angle)
                    * rand::gen_range(60.0, 140.0);
            self.bolts.push(Bolt {
                points: jagged(start, end, 5, 12.0),
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
    pub fn draw_flash(&self) {
        if let Some(age) = self.lightning_age
            && age < FLASH_SECONDS
        {
            let alpha = 0.7 * (1.0 - age / FLASH_SECONDS).powi(2);
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.9, 0.95, 1.0, alpha),
            );
        }
    }
}
