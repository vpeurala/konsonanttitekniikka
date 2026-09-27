use macroquad::prelude::*;

const PARTICLES_PER_EXPLOSION: usize = 40;
const RING_SECONDS: f32 = 0.35;

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

/// Short-lived visual effects that don't affect gameplay.
#[derive(Default)]
pub struct Effects {
    particles: Vec<Particle>,
    rings: Vec<Ring>,
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

    pub fn update(&mut self, dt: f32) {
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
    }
}
