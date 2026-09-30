//! What the app knows about the player and when it was last written down:
//! the save data, the live memory of each pair and the time of the last
//! save. Saving returns an `Effect` for the caller to perform.

use super::Effect;
use super::events::{tally, worth_saving};
use crate::badges::{self, Badge, Stats};
use crate::game::GameEvent;
use crate::memory::Memory;
use crate::save::{self, SaveData};

/// How often progress is saved while playing, in seconds, so little is lost
/// if the app is closed or killed in the background.
const SAVE_INTERVAL: f64 = 5.0;

pub struct Persistence {
    pub progress: SaveData,
    /// How well each pair is known, live: `progress` only gets it when
    /// saving, so nothing is converted every frame. This is the only live
    /// copy; the game and the practice screen learn into it.
    pub memory: Memory,
    /// When progress was last saved, in seconds since 1970.
    last_save: f64,
}

impl Persistence {
    /// `progress` as loaded from the device, at time `now`.
    pub fn new(mut progress: SaveData, now: f64) -> Self {
        let memory = progress.memory();
        // Badges the player already qualifies for, from before badges
        // existed, are given quietly; the badge screen shows them.
        badges::award(&mut progress, &memory, save::day_of(now));
        Persistence {
            progress,
            memory,
            last_save: now,
        }
    }

    /// The save file's text for the state right now.
    fn snapshot(&mut self) -> Effect {
        self.progress.set_memory(&self.memory);
        Effect::Save(self.progress.to_text())
    }

    /// Saves now.
    pub fn save(&mut self, now: f64) -> Effect {
        self.last_save = now;
        self.snapshot()
    }

    /// Saves if `forced`, or it has been a while since the last time.
    pub fn save_if_due(&mut self, now: f64, forced: bool) -> Option<Effect> {
        (forced || now - self.last_save > SAVE_INTERVAL).then(|| self.save(now))
    }

    /// Switches the sound on or off and saves the choice.
    pub fn toggle_music(&mut self) -> Effect {
        self.progress.music_on = !self.progress.music_on;
        self.snapshot()
    }

    /// Counts and remembers what happened in the game. Returns whether it
    /// is worth saving right away.
    pub fn record(&mut self, event: &GameEvent) -> bool {
        self.progress.stats = tally(self.progress.stats, event);
        if let GameEvent::LevelCompleted { level, stars } = *event {
            self.progress.record_level(level, stars);
        }
        worth_saving(event)
    }

    /// Counts `n` more flash cards answered right.
    pub fn count_practice_correct(&mut self, n: u32) {
        for _ in 0..n {
            Stats::bump(&mut self.progress.stats.practice_correct);
        }
    }

    /// Records the badges just earned, on the day `now` falls on.
    pub fn award_badges(&mut self, now: f64) -> Vec<&'static Badge> {
        badges::award(&mut self.progress, &self.memory, save::day_of(now))
    }
}
