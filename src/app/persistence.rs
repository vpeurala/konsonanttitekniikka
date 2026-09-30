//! What the app knows about the player and when it was last written down:
//! the progress, memory of each pair included, and the time of the last
//! save. Saving returns an `Effect` for the caller to perform.

use super::Effect;
use super::events::{tally, worth_saving};
use crate::badges::{self, Badge, Stats};
use crate::game::GameEvent;
use crate::memory::Lesson;
use crate::platform::save::day_of;
use crate::progress::Progress;

/// How often progress is saved while playing, in seconds, so little is lost
/// if the app is closed or killed in the background.
const SAVE_INTERVAL: f64 = 5.0;

pub struct Persistence {
    /// Everything about the player. The one live copy: the game and the
    /// practice screen learn into its memory, and saving writes it out.
    pub(super) progress: Progress,
    /// When progress was last saved, in seconds since 1970.
    last_save: f64,
}

impl Persistence {
    /// `progress` as loaded from the device, at time `now`.
    pub fn new(mut progress: Progress, now: f64) -> Self {
        // Badges the player already qualifies for, from before badges
        // existed, are given quietly; the badge screen shows them.
        badges::award(&mut progress, day_of(now));
        Persistence {
            progress,
            last_save: now,
        }
    }

    /// The save file's text for the state right now.
    fn snapshot(&self) -> Effect {
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

    /// Learns what the game or practice reported into the memory of the
    /// pairs.
    pub fn learn(&mut self, lessons: &[Lesson]) {
        for lesson in lessons {
            self.progress.memory.learn(lesson);
        }
    }

    /// Counts `n` more flash cards answered right.
    pub fn count_practice_correct(&mut self, n: u32) {
        for _ in 0..n {
            Stats::bump(&mut self.progress.stats.practice_correct);
        }
    }

    /// Records the badges just earned, on the day `now` falls on.
    pub fn award_badges(&mut self, now: f64) -> Vec<&'static Badge> {
        badges::award(&mut self.progress, day_of(now))
    }
}
