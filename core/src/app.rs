//! The whole app as a state machine: which screen is showing, what has
//! been saved, and how each frame moves things on.
//!
//! `App::update` is pure. It gets a `Frame` of input and returns the
//! `Effect`s it wants carried out (sounds, saving, statistics, quitting),
//! and it is the caller's job to perform them. That keeps the game's
//! decisions testable by running frames through it, and every impure
//! call in one place, `main`. Drawing the app, which only reads its state
//! through the accessors below, is the shell's job.

mod events;
mod persistence;

use crate::analytics;
use crate::arena::{ARENA_H, ARENA_W};
use crate::game::rules::MAX_FRAME_SECONDS;
use crate::game::{self, Game};
use crate::geometry::Rect;
use crate::input::{Frame, Key, KeyCode, Pointer};
use crate::progress::{Profile, Progress, day_of};
use crate::screens::badge_screen::BadgeScreen;
use crate::screens::levels::{LevelAction, LevelSelect};
use crate::screens::practice::{PracticeAction, PracticeScreen};
use crate::screens::progress_map;
use crate::screens::title::{TitleAction, TitleScreen};
use crate::sfx::Sfx;
use crate::toast::Toasts;
use crate::touch::{self, Button, TouchControls, TouchInput};
use events::analytics_of;
use persistence::Persistence;

/// Something the app wants done outside itself.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Play a sound effect.
    Play(Sfx),
    /// Write this text, the save file's contents, to the device.
    Save(String),
    /// Count an event in the web version's statistics.
    Count(analytics::Event),
    /// Close the app.
    Quit,
}

/// What is on screen.
pub enum Screen {
    Title,
    /// Choosing the level to start from.
    Levels(LevelSelect),
    Game(Box<Game>),
    Practice(Box<PracticeScreen>),
    Progress,
    Badges(Box<BadgeScreen>),
}

/// What one screen made of a frame: the screen to move on to, if it is
/// leaving, and what to do outside.
#[derive(Default)]
struct Step {
    next: Option<Screen>,
    effects: Vec<Effect>,
}

impl Step {
    fn go(next: Screen) -> Step {
        Step {
            next: Some(next),
            effects: Vec::new(),
        }
    }

    /// Goes to `next` and counts `event`.
    fn go_counting(next: Screen, event: analytics::Event) -> Step {
        Step {
            next: Some(next),
            effects: vec![Effect::Count(event)],
        }
    }
}

pub struct App {
    screen: Screen,
    /// Everything else, so that a screen can be worked on (`&mut Screen`)
    /// while the rest of the app is used.
    context: Context,
}

/// What the screens share: the rest of the app.
struct Context {
    /// The title screen keeps its scroll position between visits.
    title: TitleScreen,
    controls: TouchControls,
    /// The save data, the live memory and when they were last saved.
    data: Persistence,
    touch_mode: bool,
    /// A web page can't be quit, only left.
    can_quit: bool,
    /// Seconds of frames shown so far, which the menu screens' animations
    /// follow. Summed from frame times, not read from the clock, so drawing
    /// depends only on state.
    time: f64,
    /// Notices of badges just earned.
    toasts: Toasts,
}

/// The area a menu screen shows.
pub fn menu_rect() -> Rect {
    Rect::new(0.0, 0.0, ARENA_W, ARENA_H)
}

impl App {
    /// The app at its title screen, launched at time `now` with `progress`
    /// loaded from the device. Today counts as a day played, which is
    /// saved right away.
    pub fn start(
        mut profile: Profile,
        touch_mode: bool,
        can_quit: bool,
        now: f64,
    ) -> (Self, Vec<Effect>) {
        profile.current_mut().record_play_day(day_of(now));
        let mut data = Persistence::new(profile, now);
        let saved = data.save(now);
        let app = App {
            screen: Screen::Title,
            context: Context {
                title: TitleScreen::new(touch_mode),
                controls: TouchControls::default(),
                data,
                touch_mode,
                can_quit,
                time: 0.0,
                toasts: Toasts::default(),
            },
        };
        (app, vec![saved])
    }

    /// What is on screen.
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// The title screen, which keeps its scroll position between visits.
    pub fn title(&self) -> &TitleScreen {
        &self.context.title
    }

    pub fn controls(&self) -> &TouchControls {
        &self.context.controls
    }

    /// What is known about the player.
    pub fn progress(&self) -> &Progress {
        self.context.data.progress()
    }

    /// Whether hardcore mode is on: no hints, and a progress of its own.
    pub fn hardcore(&self) -> bool {
        self.context.data.profile.hardcore
    }

    pub fn toasts(&self) -> &Toasts {
        &self.context.toasts
    }

    /// Whether the touch controls are shown.
    pub fn touch_mode(&self) -> bool {
        self.context.touch_mode
    }

    /// Seconds of frames shown so far, which the menu screens' animations
    /// follow.
    pub fn time(&self) -> f64 {
        self.context.time
    }

    /// Whether the player has sound switched on: the music and the sound
    /// effects both.
    pub fn music_on(&self) -> bool {
        self.context.data.profile.music_on
    }

    /// Whether the screen wants music playing, if the player allows it:
    /// menus always do, a game only while it is going.
    pub fn wants_music(&self) -> bool {
        match &self.screen {
            Screen::Game(game) => !game.is_over() && !game.is_paused(),
            _ => true,
        }
    }

    /// Plays one frame, returning what should be done about it.
    pub fn update(&mut self, frame: &Frame) -> Vec<Effect> {
        let context = &mut self.context;
        let mut effects = Vec::new();
        context.time += f64::from(frame.dt);
        context.toasts.update(frame.dt.min(MAX_FRAME_SECONDS));
        if frame.pressed(KeyCode::Tab) {
            effects.push(context.data.toggle_music());
        }

        let step = context.update_screen(&mut self.screen, frame);
        effects.extend(step.effects);
        if let Some(next) = step.next {
            self.screen = next;
        }

        context.award_badges(frame, &mut effects);
        // The sound switch (still called music in the save file) silences
        // the sound effects too, not just the music.
        if !self.music_on() {
            effects.retain(|effect| !matches!(effect, Effect::Play(_)));
        }
        effects
    }
}

impl Context {
    fn update_screen(&mut self, screen: &mut Screen, frame: &Frame) -> Step {
        // Esc leaves the game or a menu screen, and quits from the title.
        let escape = frame.pressed(KeyCode::Escape);
        match screen {
            Screen::Title => self.update_title(frame, escape),
            Screen::Levels(levels) => self.update_levels(levels, frame),
            Screen::Progress => back_to_title(progress_map::update(frame, &menu_pointers(frame))),
            Screen::Badges(screen) => back_to_title(screen.update(frame, &menu_pointers(frame))),
            Screen::Practice(practice) => self.update_practice(practice, frame, escape),
            Screen::Game(game) => self.update_game(game, frame, escape),
        }
    }

    fn update_title(&mut self, frame: &Frame, escape: bool) -> Step {
        if escape && self.can_quit {
            return Step {
                next: None,
                effects: vec![Effect::Quit],
            };
        }
        match self.title.update(frame, &menu_pointers(frame)) {
            TitleAction::Stay => Step::default(),
            TitleAction::StartGame => {
                // With only the first level to start from, there is
                // nothing to choose.
                let levels = LevelSelect::new(self.data.progress().best_level, self.touch_mode);
                Step::go(if levels.choices() > 1 {
                    Screen::Levels(levels)
                } else {
                    Screen::Game(Box::new(self.new_game(1)))
                })
            }
            TitleAction::Practice => Step::go_counting(
                Screen::Practice(Box::new(PracticeScreen::new(
                    self.data.progress().best_level,
                    &self.data.progress().memory,
                    self.touch_mode,
                    frame.now,
                ))),
                analytics::Event::Practice {
                    hardcore: self.data.profile.hardcore,
                },
            ),
            TitleAction::Progress => Step::go_counting(
                Screen::Progress,
                analytics::Event::Progress {
                    hardcore: self.data.profile.hardcore,
                },
            ),
            TitleAction::ToggleHardcore => Step {
                next: None,
                effects: vec![self.data.toggle_hardcore(frame.now)],
            },
            TitleAction::Badges => Step::go_counting(
                Screen::Badges(Box::default()),
                analytics::Event::Badges {
                    hardcore: self.data.profile.hardcore,
                },
            ),
        }
    }

    fn update_levels(&mut self, levels: &mut LevelSelect, frame: &Frame) -> Step {
        match levels.update(frame, &menu_pointers(frame)) {
            LevelAction::Stay => Step::default(),
            LevelAction::Back => Step::go(Screen::Title),
            LevelAction::Start(level) => Step::go(Screen::Game(Box::new(self.new_game(level)))),
        }
    }

    fn update_practice(
        &mut self,
        practice: &mut PracticeScreen,
        frame: &Frame,
        escape: bool,
    ) -> Step {
        let mut step = Step::default();
        let (input, keys) = self.arena_input(frame, &mut step.effects);
        // The pause button leaves practice.
        let back = escape || input.buttons.contains(&Button::Pause);
        let outcome = practice.update(
            frame,
            &keys,
            input.arena_taps,
            back,
            &self.data.progress().memory,
        );
        self.data.learn(&outcome.lessons);
        self.data.count_practice_correct(outcome.answered_right);
        step.effects
            .extend(outcome.sfx.into_iter().map(Effect::Play));
        let leaving = matches!(outcome.action, PracticeAction::Back);
        step.effects
            .extend(self.data.save_if_due(frame.now, leaving));
        if leaving {
            step.next = Some(Screen::Title);
        }
        step
    }

    fn update_game(&mut self, game: &mut Game, frame: &Frame, escape: bool) -> Step {
        let mut step = Step::default();
        let (input, keys) = self.arena_input(frame, &mut step.effects);
        let was_over = game.is_over();
        let outputs = game.update(
            &game::Input {
                dt: frame.dt,
                now: frame.now,
                away: frame.away,
                keys,
                arrows: frame.arrows(),
                stick: input.movement,
                pause: frame.pressed(KeyCode::Space) || input.buttons.contains(&Button::Pause),
                confirm: frame.pressed(KeyCode::Enter),
                taps: input.arena_taps,
            },
            &self.data.progress().memory,
        );
        self.data.learn(&outputs.lessons);
        step.effects
            .extend(outputs.sfx.into_iter().map(Effect::Play));
        let mut save_now = escape;
        for event in &outputs.events {
            save_now |= self.data.record(event);
            step.effects
                .extend(analytics_of(event, self.data.profile.hardcore).map(Effect::Count));
        }
        if !was_over && game.is_over() {
            step.effects.push(Effect::Play(Sfx::GameOver));
            save_now = true;
        }
        step.effects
            .extend(self.data.save_if_due(frame.now, save_now));
        if escape {
            step.next = Some(Screen::Title);
        }
        step
    }

    /// What the touch controls did this frame (nothing without them) and
    /// every key typed, on the keyboard or on screen. Pressing the sound
    /// button switches it and saves.
    fn arena_input(&mut self, frame: &Frame, effects: &mut Vec<Effect>) -> (TouchInput, Vec<Key>) {
        let input = if self.touch_mode {
            self.controls
                .update(&frame.pointers(touch::content_rect(true)), frame.dt)
        } else {
            TouchInput::default()
        };
        if input.buttons.contains(&Button::Music) {
            effects.push(self.data.toggle_music());
        }
        let mut keys = frame.typed.clone();
        keys.extend(input.keys.iter().copied());
        (input, keys)
    }

    /// Gives the badges the player has just earned: a notice, a sound, a
    /// count for the statistics, and saving right away.
    fn award_badges(&mut self, frame: &Frame, effects: &mut Vec<Effect>) {
        let earned = self.data.award_badges(frame.now);
        if earned.is_empty() {
            return;
        }
        for badge in earned {
            self.toasts.push(badge);
            effects.push(Effect::Play(Sfx::Badge));
            effects.push(Effect::Count(analytics::Event::BadgeEarned {
                id: badge.id,
                name: badge.name,
                hardcore: self.data.profile.hardcore,
            }));
        }
        effects.push(self.data.save(frame.now));
    }

    fn new_game(&self, level: u32) -> Game {
        Game::new(self.touch_mode, level, self.data.profile.hardcore)
    }
}

/// The touches and clicks as a menu screen sees them.
fn menu_pointers(frame: &Frame) -> Vec<Pointer> {
    frame.pointers(menu_rect())
}

/// Leaves for the title screen if the screen said it is done.
fn back_to_title(done: bool) -> Step {
    if done {
        Step::go(Screen::Title)
    } else {
        Step::default()
    }
}

#[cfg(test)]
mod tests;
