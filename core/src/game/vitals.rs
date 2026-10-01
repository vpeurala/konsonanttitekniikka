//! How she is doing: energy, the run of right answers, the score. Every
//! change makes a new `Vitals`, so what a change may touch is in its name.

use super::rules::{COLLISION_PENALTY, FULL_ENERGY, after_wrong_key};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vitals {
    energy: f32,
    /// Right answers in a row, without a wrong key or a hit in between.
    combo: u32,
    score: u32,
}

impl Vitals {
    pub fn new() -> Vitals {
        Vitals {
            energy: FULL_ENERGY,
            combo: 0,
            score: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn with_energy(self, energy: f32) -> Vitals {
        Vitals { energy, ..self }
    }

    pub fn energy(self) -> f32 {
        self.energy
    }

    /// The energy left as a fraction of the most she can have.
    pub fn energy_fraction(self) -> f32 {
        self.energy / FULL_ENERGY
    }

    pub fn combo(self) -> u32 {
        self.combo
    }

    pub fn score(self) -> u32 {
        self.score
    }

    /// Whether she has no energy left.
    pub fn is_out(self) -> bool {
        self.energy <= 0.0
    }

    /// A right answer.
    pub fn answered(self) -> Vitals {
        Vitals {
            combo: self.combo + 1,
            ..self
        }
    }

    /// `energy` more, up to `cap`. Energy above the cap (there is none,
    /// unless the level's cap was lowered) is left as it is.
    pub fn rewarded(self, energy: f32, cap: f32) -> Vitals {
        Vitals {
            energy: (self.energy + energy).min(cap.max(self.energy)),
            ..self
        }
    }

    pub fn scored(self, points: u32) -> Vitals {
        Vitals {
            score: self.score + points,
            ..self
        }
    }

    /// A mistake ends the run of right answers.
    pub fn mistaken(self) -> Vitals {
        Vitals { combo: 0, ..self }
    }

    /// A wrong key: it costs energy, down to a floor.
    pub fn after_wrong_key(self) -> Vitals {
        Vitals {
            energy: after_wrong_key(self.energy),
            ..self
        }
    }

    /// A monster or boss reached her.
    pub fn hurt(self) -> Vitals {
        Vitals {
            energy: self.energy - COLLISION_PENALTY,
            ..self
        }
    }
}

impl Default for Vitals {
    fn default() -> Self {
        Vitals::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changes_leave_the_original_alone() {
        let start = Vitals::new();
        let hurt = start.hurt();
        assert_eq!(start.energy(), FULL_ENERGY);
        assert_eq!(hurt.energy(), FULL_ENERGY - COLLISION_PENALTY);
    }

    #[test]
    fn answers_build_a_combo_and_a_mistake_ends_it() {
        let v = Vitals::new().answered().answered();
        assert_eq!(v.combo(), 2);
        assert_eq!(v.mistaken().combo(), 0);
        assert_eq!(v.mistaken().energy(), v.energy());
    }

    #[test]
    fn a_reward_never_passes_the_maximum() {
        assert_eq!(Vitals::new().rewarded(15.0, 150.0).energy(), 115.0);
        assert_eq!(Vitals::new().rewarded(150.0, 150.0).energy(), 150.0);
        let low = Vitals::new().with_energy(50.0);
        assert_eq!(low.rewarded(10.0, 150.0).energy(), 60.0);
    }

    #[test]
    fn she_is_out_when_the_energy_is_gone() {
        assert!(!Vitals::new().is_out());
        assert!(Vitals::new().with_energy(0.0).is_out());
        assert!(Vitals::new().with_energy(-5.0).is_out());
    }

    #[test]
    fn score_only_counts_up() {
        assert_eq!(Vitals::new().scored(2).scored(3).score(), 5);
    }
}
