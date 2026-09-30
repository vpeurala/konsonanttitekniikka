//! Short-lived visual effects: sparks, rings and lightning. Everything
//! here is state that moves on with time, and it is pure; `game/render.rs`
//! draws it.

use glam::{Vec2, vec2};

use crate::color::{Color, SKYBLUE, WHITE, YELLOW};

use crate::rng::{Rng, Stream};

const PARTICLES_PER_EXPLOSION: usize = 40;
const RING_SECONDS: f32 = 0.35;
const LIGHTNING_SECONDS: f32 = 0.45;
const FLASH_SECONDS: f32 = 0.6;
const BOLT_SEGMENTS: usize = 14;
const BOLT_BRANCHES: usize = 3;

pub struct Particle {
    pub pos: Vec2,
    vel: Vec2,
    age: f32,
    lifetime: f32,
    pub size: f32,
    pub color: Color,
}

impl Particle {
    /// How visible it still is, from 1 (new) to 0 (gone).
    pub fn fade(&self) -> f32 {
        1.0 - self.age / self.lifetime
    }
}

pub struct Ring {
    pub pos: Vec2,
    age: f32,
    start_radius: f32,
    pub color: Color,
}

impl Ring {
    /// How far along it is, from 0 (new) to 1 (gone).
    pub fn progress(&self) -> f32 {
        self.age / RING_SECONDS
    }

    /// How wide it has spread.
    pub fn radius(&self) -> f32 {
        self.start_radius * (1.0 + 2.0 * self.progress())
    }
}

/// A jagged lightning bolt, as a polyline.
pub struct Bolt {
    pub points: Vec<Vec2>,
    pub width: f32,
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

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn rings(&self) -> &[Ring] {
        &self.rings
    }

    /// The lightning bolts to draw now, with how bright they are from 0
    /// to 1. Real lightning flickers, so some moments show none.
    pub fn bolts(&self) -> Option<(&[Bolt], f32)> {
        let age = self.lightning_age?;
        if age >= LIGHTNING_SECONDS || (age * 30.0) as u32 % 3 == 2 {
            return None;
        }
        Some((&self.bolts, 1.0 - age / LIGHTNING_SECONDS))
    }

    /// How white the whole screen is right after a lightning strike, from
    /// 0 to 1, if it is at all.
    pub fn flash(&self) -> Option<f32> {
        let age = self.lightning_age?;
        (age < FLASH_SECONDS).then(|| 0.7 * (1.0 - age / FLASH_SECONDS).powi(2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> [Color; 2] {
        [WHITE, YELLOW]
    }

    #[test]
    fn an_explosion_makes_sparks_and_a_ring() {
        let mut effects = Effects::default();
        effects.explode(vec2(100.0, 100.0), 20.0, &palette());
        assert_eq!(effects.particles().len(), PARTICLES_PER_EXPLOSION);
        assert_eq!(effects.rings().len(), 1);
    }

    #[test]
    fn sparks_fade_and_then_disappear() {
        let mut effects = Effects::default();
        effects.explode(Vec2::ZERO, 20.0, &palette());
        assert!(effects.particles().iter().all(|p| p.fade() == 1.0));
        effects.update(0.2);
        assert!(effects.particles().iter().all(|p| p.fade() < 1.0));
        effects.update(5.0);
        assert!(effects.particles().is_empty());
        assert!(effects.rings().is_empty());
    }

    #[test]
    fn sparks_fly_outwards_and_slow_down() {
        let mut effects = Effects::default();
        effects.explode(Vec2::ZERO, 20.0, &palette());
        effects.update(0.1);
        let far = effects
            .particles()
            .iter()
            .filter(|p| p.pos.length() > 1.0)
            .count();
        assert!(far > PARTICLES_PER_EXPLOSION / 2);
    }

    #[test]
    fn a_ring_spreads_out_as_it_ages() {
        let mut effects = Effects::default();
        effects.explode(Vec2::ZERO, 20.0, &palette());
        let before = effects.rings()[0].radius();
        assert_eq!(before, 20.0);
        effects.update(RING_SECONDS / 2.0);
        assert!(effects.rings()[0].radius() > before);
        assert!((effects.rings()[0].progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn lightning_flashes_the_screen_and_then_calms_down() {
        let mut effects = Effects::default();
        assert_eq!(effects.flash(), None);
        effects.lightning(vec2(400.0, 300.0));
        let first = effects.flash().expect("it flashes at once");
        assert!(first > 0.6);
        effects.update(FLASH_SECONDS / 2.0);
        assert!(effects.flash().expect("still flashing") < first);
        effects.update(FLASH_SECONDS);
        assert_eq!(effects.flash(), None);
        assert!(effects.bolts().is_none());
    }

    #[test]
    fn a_bolt_reaches_from_above_the_screen_to_the_target() {
        let mut effects = Effects::default();
        let target = vec2(400.0, 300.0);
        effects.lightning(target);
        let (bolts, brightness) = effects.bolts().expect("the bolt is visible at first");
        assert!(brightness > 0.9);
        let trunk = bolts.last().expect("the trunk comes last");
        assert!(trunk.points.first().unwrap().y < 0.0);
        assert_eq!(*trunk.points.last().unwrap(), target);
        assert_eq!(bolts.len(), BOLT_BRANCHES + 1);
    }

    #[test]
    fn lightning_flickers() {
        let mut effects = Effects::default();
        effects.lightning(vec2(400.0, 300.0));
        let mut shown = 0;
        let mut hidden = 0;
        for _ in 0..27 {
            effects.update(1.0 / 60.0);
            if effects.bolts().is_some() {
                shown += 1;
            } else {
                hidden += 1;
            }
        }
        assert!(shown > 0 && hidden > 0, "{shown} shown, {hidden} hidden");
    }

    #[test]
    fn a_trail_leaves_a_few_short_lived_sparks() {
        let mut effects = Effects::default();
        effects.trail(vec2(10.0, 10.0), &palette());
        assert_eq!(effects.particles().len(), 3);
        effects.update(1.0);
        assert!(effects.particles().is_empty());
    }

    #[test]
    fn effects_are_the_same_every_time() {
        let run = || {
            let mut effects = Effects::default();
            effects.explode(vec2(5.0, 5.0), 10.0, &palette());
            effects.lightning(vec2(100.0, 100.0));
            effects.update(0.1);
            effects
                .particles()
                .iter()
                .map(|p| p.pos)
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}
