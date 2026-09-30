//! Checkpoints: a game can start from every fifth level she has reached,
//! so later levels, like those with long numbers, are within reach of a
//! single sitting.

/// A checkpoint every this many levels: 1, 6, 11, ...
pub const LEVELS_PER_CHECKPOINT: u32 = 5;

/// At most this many checkpoints fit on the screen.
const MAX_SHOWN: usize = 25;

/// The levels a game can start from, having reached `best_level`.
pub fn checkpoints(best_level: u32) -> Vec<u32> {
    let all: Vec<u32> = (1..=best_level.max(1))
        .step_by(LEVELS_PER_CHECKPOINT as usize)
        .collect();
    if all.len() <= MAX_SHOWN {
        return all;
    }
    // The first level and the latest ones.
    let mut shown = vec![1];
    shown.extend_from_slice(&all[all.len() - (MAX_SHOWN - 1)..]);
    shown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::long_numbers::first_long_level;

    #[test]
    fn every_fifth_level_reached_is_a_checkpoint() {
        assert_eq!(checkpoints(1), vec![1]);
        assert_eq!(checkpoints(5), vec![1]);
        assert_eq!(checkpoints(6), vec![1, 6]);
        assert_eq!(checkpoints(22), vec![1, 6, 11, 16, 21]);
    }

    #[test]
    fn a_long_list_keeps_the_start_and_the_latest() {
        let shown = checkpoints(1000);
        assert_eq!(shown.len(), MAX_SHOWN);
        assert_eq!(shown[0], 1);
        assert_eq!(*shown.last().unwrap(), 996);
    }

    #[test]
    fn a_checkpoint_comes_just_before_long_numbers() {
        // Reaching the first level with long numbers unlocks a checkpoint
        // at most one level before it.
        let long = first_long_level();
        let last = *checkpoints(long).last().unwrap();
        assert!(long - last <= 1, "checkpoint {last}, long numbers {long}");
    }
}
