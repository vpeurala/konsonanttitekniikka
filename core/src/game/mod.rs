//! The game itself: monsters carry numbers or words toward the heroine,
//! and typing the matching word or number casts a spell at them.
//!
//! Everything but `render` is a pure simulation. `Game::update` gets what
//! the outside world says this frame (`Input`: elapsed time, the clock,
//! keys) and reports what should happen outside (`Outputs`: sounds and
//! events). It reads no global state, so a game can be played in a test
//! by feeding it inputs. `render` only reads a `Game`.

mod answer;
mod display;
pub mod enemy;
mod metrics;
mod play;
mod player;
pub mod rules;
mod scene;
mod spawn;
mod stage;
mod vitals;
mod world;

use glam::Vec2;

use crate::curriculum::Curriculum;
use crate::key::Key;
use crate::memory::Memory;
use crate::rng::{Rng, Stream};
use crate::sfx::Sfx;

pub use answer::Slot;
use answer::Typed;
pub use answer::{InputOutcome, resolve_input};
use display::Display;
pub use display::{Banner, Feedback, Tone};
pub use enemy::Enemy;
use player::Player;
use rules::*;
pub use scene::{Scene, SlotView};
use stage::Stage;
use vitals::Vitals;
use world::World;
pub use world::{Spell, SpellTarget};

/// What the outside world says happened this frame.
#[derive(Debug, Default, Clone)]
pub struct Input {
    /// Seconds since the previous frame.
    pub dt: f32,
    /// The current time, in seconds since 1970, for spaced repetition.
    pub now: f64,
    /// The app was in the background since the previous frame.
    pub away: bool,
    /// The characters typed, in order.
    pub keys: Vec<Key>,
    /// The arrow keys held down, each axis -1, 0 or 1.
    pub arrows: Vec2,
    /// The touch joystick's direction; its length, up to 1, is how far it
    /// is pushed.
    pub stick: Vec2,
    /// Space, or the on-screen pause button, was pressed.
    pub pause: bool,
    /// Enter was pressed.
    pub confirm: bool,
    /// Taps in the arena that didn't start the joystick moving.
    pub taps: usize,
}

/// What the game asks the outside world to do.
#[derive(Debug, Default)]
pub struct Outputs {
    /// Sound effects to play.
    pub sfx: Vec<Sfx>,
    /// Things to save or count.
    pub events: Vec<GameEvent>,
}

impl Outputs {
    fn extend(&mut self, later: Outputs) {
        self.sfx.extend(later.sfx);
        self.events.extend(later.events);
    }
}

/// Something the game wants saved, reported in `Outputs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// A game started (or started again) from this level.
    Started { level: u32 },
    /// The game ended on this level.
    Over { level: u32 },
    /// A level was finished (its boss beaten) with this many stars.
    LevelCompleted { level: u32, stars: u8 },
    /// A monster was answered right. `combo` is how many answers in a row
    /// that makes, without a wrong key or a hit in between.
    Answered { quick: bool, long: bool, combo: u32 },
    /// An ordinary monster (not a boss) was defeated.
    MonsterDefeated,
    /// The level just finished, reported right before `LevelCompleted`, had
    /// no wrong key and no hit.
    FlawlessLevel,
}

/// What was typed or tapped since the game last advanced. A frame can be
/// shorter than a step, so these wait for the step that acts on them.
#[derive(Debug, Default)]
struct Pending {
    keys: Vec<Key>,
    taps: usize,
    confirm: bool,
}

/// The game. Its state is a handful of small values, each behind its own
/// boundary: what is in the arena (`World`), how she is doing (`Vitals`,
/// `Stage`, `Player`), what she has typed and what is on screen. `update`
/// is the only way in, and everything that should happen outside comes
/// back from it as `Outputs`.
pub struct Game {
    player: Player,
    world: World,
    vitals: Vitals,
    /// How the current level is going.
    stage: Stage,
    level: u32,
    curriculum: Curriculum,
    typed: Typed,
    spawn_timer: f32,
    /// What is shown without affecting play.
    display: Display,
    /// Every gameplay decision's randomness, the same in every game. It is
    /// the one thing lent out and changed by the parts that need luck.
    rng: Rng,
    /// Whether the game is paused with the space bar.
    paused: bool,
    /// Whether she plays with touch controls, which changes some texts.
    touch: bool,
    /// The level the game started from, where it starts again after a
    /// game over.
    start_level: u32,
    /// Cuts frames into the fixed steps the game advances in.
    timestep: Timestep,
    /// Typing and taps not acted on yet.
    pending: Pending,
    /// Seconds the game has run, not counting pauses. Animations follow
    /// this rather than the clock, so a pause freezes them too.
    play_time: f64,
    /// Whether the start of this game has been reported yet.
    start_reported: bool,
}

impl Game {
    /// A new game starting from `start_level`.
    pub fn new(touch: bool, start_level: u32) -> Self {
        let start_level = start_level.max(1);
        let mut curriculum = Curriculum::new();
        // Every pair of the levels before the start is already met.
        for _ in 1..start_level {
            curriculum.next_level();
        }
        let world = World::for_level(start_level);
        let mut display = Display::default();
        if start_level > 1 {
            display.announce(
                format!("Taso {start_level}"),
                "Onnea matkaan!".to_owned(),
                Tone::Celebrate,
                None,
            );
        }
        Game {
            // She may be standing where an obstacle is.
            player: Player::at_center().pushed_out(world.obstacles()),
            world,
            vitals: Vitals::new(),
            stage: Stage::default(),
            level: start_level,
            curriculum,
            typed: Typed::default(),
            spawn_timer: 1.0,
            display,
            rng: Rng::new(Stream::Gameplay, 0),
            paused: false,
            touch,
            start_level,
            timestep: Timestep::default(),
            pending: Pending::default(),
            play_time: 0.0,
            start_reported: false,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_over(&self) -> bool {
        self.vitals.is_out()
    }

    /// Pauses the game, unless it is already over.
    fn pause(&mut self) {
        if !self.is_over() {
            self.paused = true;
        }
    }

    /// Says a game has started, once: the first time it is asked, and again
    /// after each restart.
    fn report_start(&mut self) -> Outputs {
        let mut out = Outputs::default();
        if !self.start_reported {
            self.start_reported = true;
            out.events.push(GameEvent::Started { level: self.level });
        }
        out
    }

    /// Plays one frame and returns what the outside world should do about
    /// it. What she answers right or wrong is learned into `memory`, which
    /// the game also reads to choose what comes next; the caller owns it.
    pub fn update(&mut self, input: &Input, memory: &mut Memory) -> Outputs {
        let mut out = self.report_start();
        // Coming back after being away finds the game paused, not lost.
        if input.away {
            self.pause();
        }
        let pause_pressed = input.pause
            // A tap anywhere in the arena also resumes.
            || (self.paused && input.taps > 0);
        if pause_pressed && !self.is_over() {
            self.paused = !self.paused;
        }
        // Keys typed during a pause are dropped along with the frame.
        if self.paused {
            self.pending = Pending::default();
        } else {
            self.pending.keys.extend(&input.keys);
            self.pending.taps += input.taps;
            self.pending.confirm |= input.confirm;
            let steps = self.timestep.steps_due(input.dt);
            for step in 0..steps {
                // What was typed is acted on in the first step only.
                let acts = (step == 0).then(|| std::mem::take(&mut self.pending));
                out.extend(self.advance(STEP_SECONDS, input, acts, memory));
            }
        }
        out
    }

    /// Advances the game by `dt` seconds, acting on what was typed or
    /// tapped (`acts`) if there is anything new.
    fn advance(
        &mut self,
        dt: f32,
        input: &Input,
        acts: Option<Pending>,
        memory: &mut Memory,
    ) -> Outputs {
        let mut out = Outputs::default();
        self.play_time += f64::from(dt);
        self.display.update_motion(dt);
        out.extend(self.update_spells(dt));

        if self.is_over() {
            if acts.is_some_and(|a| a.confirm || a.taps > 0) {
                out.extend(self.restart());
            }
            return out;
        }

        self.stage = self.stage.elapsed(dt);
        self.display.update_messages(dt);

        self.player = self
            .player
            .moved(dt, input.arrows, input.stick, self.world.obstacles());
        for key in acts.into_iter().flat_map(|a| a.keys) {
            out.extend(self.handle_key(key, input.now, memory));
        }
        out.extend(self.move_enemies(dt, input.now, memory));
        if self.is_over() {
            out.events.push(GameEvent::Over { level: self.level });
        }

        // No new monsters join a boss fight.
        if !self.stage.boss_fight {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = spawn_interval(self.stage.time);
                out.extend(self.spawn_enemy(input.now, memory));
            }
        }
        out
    }

    /// Starts over from the level this game started from. What she has
    /// learned is not the game's to lose: it lives in the caller's memory.
    fn restart(&mut self) -> Outputs {
        *self = Game::new(self.touch, self.start_level);
        self.report_start()
    }
}

#[cfg(test)]
mod tests;
