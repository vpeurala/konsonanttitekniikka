//! The sound effects the game can ask for. Playing them is the shell's job.

/// A sound effect the game asks to be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Type,
    Cast,
    Explode,
    Hurt,
    Wrong,
    LevelUp,
    GameOver,
    Boss,
    Thunder,
    /// A badge was earned.
    Badge,
}
