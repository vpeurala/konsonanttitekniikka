//! What the game's events mean for the lifetime counters and the
//! statistics. Plain functions of their arguments.

use crate::badges::{self, Stats};
use crate::game::GameEvent;
use crate::platform::analytics;

/// The counters after `event` has happened.
pub fn tally(mut stats: Stats, event: &GameEvent) -> Stats {
    match *event {
        GameEvent::Answered { quick, long, combo } => {
            if quick {
                Stats::bump(&mut stats.quick_answers);
            }
            if long {
                Stats::bump(&mut stats.long_answers);
            }
            stats.best_combo = stats.best_combo.max(combo).min(badges::MAX_COUNT);
        }
        GameEvent::MonsterDefeated => Stats::bump(&mut stats.monsters),
        GameEvent::FlawlessLevel => Stats::bump(&mut stats.flawless_levels),
        GameEvent::LevelCompleted { .. } => Stats::bump(&mut stats.bosses),
        GameEvent::Started { .. } | GameEvent::Over { .. } => {}
    }
    stats
}

/// What to count in the statistics, if anything.
pub fn analytics_of(event: &GameEvent) -> Option<analytics::Event> {
    match *event {
        GameEvent::Started { level } => Some(analytics::Event::GameStarted { level }),
        GameEvent::Over { level } => Some(analytics::Event::GameOver { level }),
        GameEvent::LevelCompleted { level, stars } => {
            Some(analytics::Event::LevelCleared { level, stars })
        }
        GameEvent::Answered { .. } | GameEvent::MonsterDefeated | GameEvent::FlawlessLevel => None,
    }
}

/// Whether the event is worth saving right away; the rest wait for the
/// next regular save.
pub fn worth_saving(event: &GameEvent) -> bool {
    matches!(event, GameEvent::LevelCompleted { .. })
}
