//! This module implements useful system parameters for working with block
//! definitions.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Block;

/// This system parameter provides a way to find a block entity by its name.
#[derive(SystemParam)]
pub struct BlockFinder<'w, 's> {
    /// The query for all block entities with their names.
    blocks: Query<'w, 's, (Entity, &'static Name), With<Block>>,
}

impl<'w, 's> BlockFinder<'w, 's> {
    /// Finds a block by its name. Returns the entity if found, or `None` if the
    /// block does not exist. Name must be an exact match.
    ///
    /// This method may be slow if called frequently. Values should be cached if
    /// possible.
    pub fn find(&self, name: &str) -> Option<Entity> {
        let block_name: Name = name.into();
        self.blocks
            .iter()
            .find(|(_, name)| **name == block_name)
            .map(|(entity, _)| entity)
    }
}
