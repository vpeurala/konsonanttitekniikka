//! Ways for the game's tests to look inside a `World` and set it up, which
//! the game itself never needs.

use super::World;
use crate::game::enemy::{Enemy, EnemyId};

impl World {
    /// The enemy called `id`, if it is still in play.
    pub fn enemy(&self, id: EnemyId) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.id == id)
    }

    pub fn enemies_mut(&mut self) -> &mut Vec<Enemy> {
        &mut self.enemies
    }
}
