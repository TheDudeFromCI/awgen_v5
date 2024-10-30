//! This module implements useful system parameters for working with block
//! definitions.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use uuid::Uuid;

use super::Block;

/// This system parameter provides a way to find a block entity by its name.
#[derive(SystemParam)]
pub struct BlockFinder<'w, 's> {
    /// The query for all block entities with their names.
    blocks: Query<'w, 's, (Entity, &'static Name, &'static Block)>,
}

impl<'w, 's> BlockFinder<'w, 's> {
    /// Gets the air block entity. This is the default block type for empty
    /// space and is always present.
    pub fn find_air(&self) -> Entity {
        self.find_by_uuid(super::AIR_BLOCK_UUID).unwrap()
    }

    /// Finds a block by its name. Returns the entity if found, or `None` if the
    /// block does not exist. Name must be an exact match. Warning: There may be
    /// more than one block with the same name, but only the first one found is
    /// returned.
    ///
    /// This method may be slow if called frequently. Values should be cached if
    /// possible.
    pub fn find(&self, name: &str) -> Option<Entity> {
        let block_name: Name = name.into();
        self.blocks
            .iter()
            .find(|(_, name, _)| **name == block_name)
            .map(|(entity, _, _)| entity)
    }

    /// Finds a block by its UUID. Returns the entity if found, or `None` if the
    /// block does not exist. Warning: There may be more than one block with the
    /// same UUID, but only the first one found is returned.
    pub fn find_by_uuid(&self, uuid: Uuid) -> Option<Entity> {
        self.blocks
            .iter()
            .find(|(_, _, block)| block.uuid == uuid)
            .map(|(entity, _, _)| entity)
    }
}
