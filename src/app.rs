//! The whole app as a state machine: which screen is showing, what has
//! been saved, and how each frame moves things on.
//!
//! `App::update` is pure. It gets a `Frame` of input and returns the
//! `Effect`s it wants carried out (sounds, saving, statistics, quitting),
//! and it is the caller's job to perform them. That keeps the game's
//! decisions testable by running frames through it, and every impure
//! call in one place, `main`. `App::draw` only reads the state.

use macroquad::prelude::{BLACK, KeyCode, Rect};

use crate::audio::Sfx;
use crate::frame::Frame;
use crate::game::{self, Game, GameEvent};
use crate::levels::{LevelAction, LevelSelect};
use crate::memory::Memory;
use crate::practice::{PracticeAction, PracticeScreen};
use crate::progress::ProgressScreen;
use crate::save::SaveData;
use crate::title::{TitleAction, TitleScreen};
use crate::touch::{self, Button, TouchControls, TouchInput};
use crate::view::{self, ARENA_H, ARENA_W, View};

/// How often progress is saved while playing, in seconds, so little is lost
/// if the app is closed or killed in the background.
const SAVE_INTERVAL: f64 = 5.0;

/// Something the app wants done outside itself.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Play a sound effect.
    Play(Sfx),
    /// Write this progress to the device.
    Save(Box<SaveData>),
    /// Count an event in the web version's statistics.
    Count { path: String, title: String },
    /// Close the app.
    Quit,
}

impl Effect {
    fn count(path: impl Into<String>, title: impl Into<String>) -> Effect {
        Effect::Count {
            path: path.into(),
            title: title.into(),
        }
    }
}

/// What is on screen.
enum Screen {
    Title,
    /// Choosing the level to start from.
    Levels(LevelSelect),
    Game(Box<Game>),
    Practice(Box<PracticeScreen>),
    Progress(ProgressScreen),
}

pub struct App {
    screen: Screen,
    /// The title screen keeps its scroll position between visits.
    title: TitleScreen,
    controls: TouchControls,
    progress: SaveData,
    /// How well each pair is known, live: `progress` only gets it when
    /// saving, so nothing is converted every frame.
    memory: Memory,
    touch_mode: bool,
    /// A web page can't be quit, only left.
    can_quit: bool,
    /// Seconds of frames shown so far, which the menu screens' animations
    /// follow. Summed from frame times, not read from the clock, so drawing
    /// depends only on state.
    time: f64,
    /// When progress was last saved, in seconds since 1970.
    last_save: f64,
}

/// The area a menu screen shows.
fn menu_rect() -> Rect {
    Rect::new(0.0, 0.0, ARENA_W, ARENA_H)
}

impl App {
    /// The app at its title screen, with `progress` loaded from the
    /// device, at time `now`.
    pub fn new(progress: SaveData, touch_mode: bool, can_quit: bool, now: f64) -> Self {
        App {
            screen: Screen::Title,
            title: TitleScreen::new(touch_mode),
            controls: TouchControls::default(),
            memory: progress.memory(),
            progress,
            touch_mode,
            can_quit,
            time: 0.0,
            last_save: now,
        }
    }

    /// Whether the player has sound switched on: the music and the sound
    /// effects both.
    pub fn music_on(&self) -> bool {
        self.progress.music_on
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
        let mut effects = Vec::new();
        self.time += f64::from(frame.dt);
        // Esc leaves the game or a menu screen, and quits from the title.
        let escape = frame.pressed(KeyCode::Escape);
        if frame.pressed(KeyCode::Tab) {
            toggle_music(&mut self.progress, &self.memory, &mut effects);
        }

        let mut next = None;
        match &mut self.screen {
            Screen::Title => {
                if escape && self.can_quit {
                    effects.push(Effect::Quit);
                    return effects;
                }
                let view = View::fit(menu_rect(), frame.screen);
                match self.title.update(frame, &frame.pointers(&view)) {
                    TitleAction::Stay => {}
                    TitleAction::StartGame => {
                        // With only the first level to start from, there
                        // is nothing to choose.
                        let levels = LevelSelect::new(self.progress.best_level, self.touch_mode);
                        next = Some(if levels.len() > 1 {
                            Screen::Levels(levels)
                        } else {
                            Screen::Game(Box::new(self.new_game(1)))
                        });
                    }
                    TitleAction::Practice => {
                        effects.push(Effect::count("harjoittelu", "Harjoittelu"));
                        next = Some(Screen::Practice(Box::new(PracticeScreen::new(
                            self.progress.best_level,
                            &self.memory,
                            self.touch_mode,
                            frame.now,
                        ))));
                    }
                    TitleAction::Progress => {
                        effects.push(Effect::count("edistyminen", "Edistyminen"));
                        next = Some(Screen::Progress(ProgressScreen));
                    }
                }
            }

            Screen::Levels(levels) => {
                let view = View::fit(menu_rect(), frame.screen);
                match levels.update(frame, &frame.pointers(&view)) {
                    LevelAction::Stay => {}
                    LevelAction::Back => next = Some(Screen::Title),
                    LevelAction::Start(level) => {
                        next = Some(Screen::Game(Box::new(Game::new(
                            self.touch_mode,
                            self.memory.clone(),
                            level,
                        ))));
                    }
                }
            }

            Screen::Progress(screen) => {
                let view = View::fit(menu_rect(), frame.screen);
                if screen.update(frame, &frame.pointers(&view)) {
                    next = Some(Screen::Title);
                }
            }

            Screen::Practice(practice) => {
                let view = View::fit(touch::content_rect(self.touch_mode), frame.screen);
                let input = touch_input(&mut self.controls, self.touch_mode, frame, &view);
                if input.buttons.contains(&Button::Music) {
                    toggle_music(&mut self.progress, &self.memory, &mut effects);
                }
                let mut keys = frame.typed.clone();
                keys.extend(input.keys.iter().copied());
                // The pause button leaves practice.
                let back = escape || input.buttons.contains(&Button::Pause);
                let action =
                    practice.update(frame, &keys, input.arena_taps, back, &mut self.memory);
                effects.extend(practice.take_sfx().into_iter().map(Effect::Play));
                let leaving = matches!(action, PracticeAction::Back);
                save(
                    &mut self.progress,
                    &self.memory,
                    &mut self.last_save,
                    frame.now,
                    leaving,
                    &mut effects,
                );
                if leaving {
                    next = Some(Screen::Title);
                }
            }

            Screen::Game(game) => {
                let view = View::fit(touch::content_rect(self.touch_mode), frame.screen);
                let input = touch_input(&mut self.controls, self.touch_mode, frame, &view);
                if input.buttons.contains(&Button::Music) {
                    toggle_music(&mut self.progress, &self.memory, &mut effects);
                }

                let mut keys = frame.typed.clone();
                keys.extend(input.keys.iter().copied());
                let was_over = game.is_over();
                let outputs = game.update(&game::Input {
                    dt: frame.dt,
                    now: frame.now,
                    away: frame.away,
                    keys,
                    arrows: frame.arrows(),
                    stick: input.movement,
                    pause: frame.pressed(KeyCode::Space) || input.buttons.contains(&Button::Pause),
                    confirm: frame.pressed(KeyCode::Enter),
                    taps: input.arena_taps,
                });
                effects.extend(outputs.sfx.into_iter().map(Effect::Play));
                let mut save_now = escape;
                for event in outputs.events {
                    save_now |= record_event(&mut self.progress, event, &mut effects);
                }
                if !was_over && game.is_over() {
                    effects.push(Effect::Play(Sfx::GameOver));
                    save_now = true;
                }
                if save_now || frame.now - self.last_save > SAVE_INTERVAL {
                    // The game has been learning; take what it knows.
                    self.memory = game.memory().clone();
                }
                save(
                    &mut self.progress,
                    &self.memory,
                    &mut self.last_save,
                    frame.now,
                    save_now,
                    &mut effects,
                );
                if escape {
                    next = Some(Screen::Title);
                }
            }
        }
        if let Some(next) = next {
            self.screen = next;
        }
        // The sound switch (still called music in the save file) silences
        // the sound effects too, not just the music.
        if !self.progress.music_on {
            effects.retain(|effect| !matches!(effect, Effect::Play(_)));
        }
        effects
    }

    fn new_game(&self, level: u32) -> Game {
        Game::new(self.touch_mode, self.memory.clone(), level)
    }

    /// Draws the current screen.
    pub fn draw(&self) {
        match &self.screen {
            Screen::Title => {
                let view = view::begin(menu_rect());
                self.title.draw(&self.progress, view, self.time as f32);
                view::mask_outside(view, menu_rect(), BLACK);
            }
            Screen::Levels(levels) => {
                let view = view::begin(menu_rect());
                levels.draw();
                view::mask_outside(view, menu_rect(), BLACK);
            }
            Screen::Progress(screen) => {
                let view = view::begin(menu_rect());
                screen.draw(&self.progress, &self.memory, self.touch_mode);
                view::mask_outside(view, menu_rect(), BLACK);
            }
            Screen::Practice(practice) => {
                let content = touch::content_rect(self.touch_mode);
                let view = view::begin(content);
                practice.draw(self.time as f32);
                if self.touch_mode {
                    self.controls.draw(self.music_on(), false, false);
                }
                view::mask_outside(view, content, BLACK);
            }
            Screen::Game(game) => {
                let content = touch::content_rect(self.touch_mode);
                let view = view::begin(content);
                game.draw(view);
                if self.touch_mode {
                    self.controls.draw(self.music_on(), game.is_paused(), true);
                }
                view::mask_outside(view, content, BLACK);
            }
        }
    }
}

/// What the touch controls did this frame; nothing without them.
fn touch_input(
    controls: &mut TouchControls,
    touch_mode: bool,
    frame: &Frame,
    view: &View,
) -> TouchInput {
    if touch_mode {
        controls.update(&frame.pointers(view), frame.dt)
    } else {
        TouchInput::default()
    }
}

/// Switches the music on or off and remembers the choice.
fn toggle_music(progress: &mut SaveData, memory: &Memory, effects: &mut Vec<Effect>) {
    progress.music_on = !progress.music_on;
    progress.set_memory(memory);
    effects.push(Effect::Save(Box::new(progress.clone())));
}

/// Saves `progress` if `now` is `forced`, or it has been a while since the
/// last time.
fn save(
    progress: &mut SaveData,
    memory: &Memory,
    last_save: &mut f64,
    now: f64,
    forced: bool,
    effects: &mut Vec<Effect>,
) {
    if forced || now - *last_save > SAVE_INTERVAL {
        progress.set_memory(memory);
        effects.push(Effect::Save(Box::new(progress.clone())));
        *last_save = now;
    }
}

/// Counts and remembers what happened in the game. Returns whether it
/// is worth saving right away.
fn record_event(progress: &mut SaveData, event: GameEvent, effects: &mut Vec<Effect>) -> bool {
    match event {
        GameEvent::Started { level } => {
            effects.push(Effect::count(
                format!("peli-alkoi/taso-{level}"),
                format!("Peli alkoi tasolta {level}"),
            ));
            false
        }
        GameEvent::Over { level } => {
            effects.push(Effect::count(
                format!("peli-paattyi/taso-{level}"),
                format!("Peli päättyi tasolla {level}"),
            ));
            false
        }
        GameEvent::LevelCompleted { level, stars } => {
            effects.push(Effect::count(
                format!("taso-lapaisty/{level}"),
                format!("Taso {level} läpäisty ({stars} tähteä)"),
            ));
            progress.record_level(level, stars);
            true
        }
    }
}

#[cfg(test)]
mod tests;
