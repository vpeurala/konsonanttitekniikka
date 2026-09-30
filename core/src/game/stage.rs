//! How the current level is going; starts over with every level. A `Stage`
//! is a small value: each change makes a new one.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    /// Points scored.
    pub points: u32,
    /// Whether no wrong key has been typed and nothing has hit her.
    pub flawless: bool,
    /// Seconds spent.
    pub time: f32,
    /// Whether the boss has been summoned and not yet beaten.
    pub boss_fight: bool,
}

impl Default for Stage {
    fn default() -> Self {
        Stage {
            points: 0,
            flawless: true,
            time: 0.0,
            boss_fight: false,
        }
    }
}

impl Stage {
    pub fn elapsed(self, dt: f32) -> Stage {
        Stage {
            time: self.time + dt,
            ..self
        }
    }

    /// A wrong key or a hit ends the level's clean record.
    pub fn mistaken(self) -> Stage {
        Stage {
            flawless: false,
            ..self
        }
    }

    /// The stage after `points` more, when `needed` summon the boss, and
    /// whether the boss is now due. Nothing counts during a boss fight.
    pub fn scored(self, points: u32, needed: u32) -> (Stage, bool) {
        if self.boss_fight {
            return (self, false);
        }
        let total = self.points + points;
        let due = total >= needed;
        let stage = Stage {
            points: total.min(needed),
            ..self
        };
        (stage, due)
    }

    /// The boss has been summoned.
    pub fn boss_summoned(self) -> Stage {
        Stage {
            boss_fight: true,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_count_up_to_what_is_needed_and_then_the_boss_is_due() {
        let (stage, due) = Stage::default().scored(3, 5);
        assert_eq!((stage.points, due), (3, false));
        let (stage, due) = stage.scored(4, 5);
        assert_eq!((stage.points, due), (5, true));
    }

    #[test]
    fn nothing_counts_during_a_boss_fight() {
        let fight = Stage::default().boss_summoned();
        let (after, due) = fight.scored(10, 5);
        assert_eq!(after, fight);
        assert!(!due);
    }

    #[test]
    fn a_mistake_ends_the_clean_record_and_time_passes() {
        let stage = Stage::default();
        assert!(stage.flawless);
        assert!(!stage.mistaken().flawless);
        assert!((stage.elapsed(1.5).elapsed(0.5).time - 2.0).abs() < 1e-6);
    }
}
