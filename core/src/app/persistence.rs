//! What the app knows about the player and when it was last written down:
//! the progress in each mode, memory of each pair included, and the time
//! of the last save. Saving returns an `Effect` for the caller to perform.

use super::Effect;
use super::events::{tally, worth_saving};
use crate::badges::{self, Badge, Stats};
use crate::game::GameEvent;
use crate::memory::Lesson;
use crate::progress::{Profile, Progress, day_of};

/// How often progress is saved while playing, in seconds, so little is lost
/// if the app is closed or killed in the background.
const SAVE_INTERVAL: f64 = 5.0;

pub struct Persistence {
    /// Everything about the player. The one live copy: the game and the
    /// practice screen learn into the memory of the mode being played, and
    /// saving writes it out.
    pub(super) profile: Profile,
    /// When progress was last saved, in seconds since 1970.
    last_save: f64,
}

impl Persistence {
    /// `profile` as loaded from the device, at time `now`.
    pub fn new(mut profile: Profile, now: f64) -> Self {
        // Badges the player already qualifies for, from before badges
        // existed, are given quietly; the badge screen shows them.
        badges::award(&mut profile.easy, day_of(now));
        badges::award(&mut profile.hard, day_of(now));
        Persistence {
            profile,
            last_save: now,
        }
    }

    /// The progress in the mode being played.
    pub fn progress(&self) -> &Progress {
        self.profile.current()
    }

    /// The save file's text for the state right now.
    fn snapshot(&self) -> Effect {
        Effect::Save(self.profile.to_text())
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
        self.profile.music_on = !self.profile.music_on;
        self.snapshot()
    }

    /// Switches between the easy mode and hardcore mode and saves the
    /// choice. Playing in the new mode counts as a day played there.
    pub fn toggle_hardcore(&mut self, now: f64) -> Effect {
        self.profile.hardcore = !self.profile.hardcore;
        self.profile.current_mut().record_play_day(day_of(now));
        self.snapshot()
    }

    /// Counts and remembers what happened in the game. Returns whether it
    /// is worth saving right away.
    pub fn record(&mut self, event: &GameEvent) -> bool {
        let progress = self.profile.current_mut();
        progress.stats = tally(progress.stats, event);
        if let GameEvent::LevelCompleted { level, stars } = *event {
            progress.record_level(level, stars);
        }
        worth_saving(event)
    }

    /// Learns what the game or practice reported into the memory of the
    /// pairs.
    pub fn learn(&mut self, lessons: &[Lesson]) {
        for lesson in lessons {
            self.profile.current_mut().memory.learn(lesson);
        }
    }

    /// Counts `n` more flash cards answered right.
    pub fn count_practice_correct(&mut self, n: u32) {
        for _ in 0..n {
            Stats::bump(&mut self.profile.current_mut().stats.practice_correct);
        }
    }

    /// Records the badges just earned, on the day `now` falls on.
    pub fn award_badges(&mut self, now: f64) -> Vec<&'static Badge> {
        badges::award(self.profile.current_mut(), day_of(now))
    }
}
