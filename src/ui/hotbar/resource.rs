//! This module contains the [`Hotbar`] resource and related data structures.

use bevy::prelude::*;

/// This resource contains information about the current state of the hotbar.
#[derive(Debug, Default, Clone, Resource)]
pub struct Hotbar {
    /// Whether or not the hotbar is currently active.
    active: bool,

    /// The currently selected index in the hotbar.
    selection: usize,

    /// Entity pointers for the hotbar slots.
    slots: Vec<HotbarSlotMeta>,
}

impl Hotbar {
    /// Activates the hotbar.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivates the hotbar and resets all data.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.selection = 0;
        self.slots.clear();
    }

    /// Whether or not the hotbar is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Inserts a new slot entity into the hotbar.
    pub fn insert_slot(&mut self, slot: Entity) {
        self.slots.push(HotbarSlotMeta {
            slot_id: slot,
            data: HotbarSlotData::Empty,
            is_dirty: true,
        });
    }

    /// Selects a slot in the hotbar by index.
    pub fn select_slot(&mut self, index: usize) {
        self.selection = index.min(self.slots.len() - 1);
    }

    /// Replaces the data in the slot at the given index.
    ///
    /// Panics if the index is out of bounds.
    pub fn set_slot(&mut self, index: usize, data: HotbarSlotData) {
        let meta = self.slots.get_mut(index).unwrap();
        meta.data = data;
        meta.is_dirty = true;
    }

    /// Returns the data in the slot at the given index.
    ///
    /// Panics if the index is out of bounds.
    pub fn get_slot(&self, index: usize) -> HotbarSlotData {
        self.slots.get(index).map(|slot| slot.data).unwrap()
    }

    /// Returns the data in the currently selected slot.
    pub fn get_selected(&self) -> HotbarSlotData {
        self.get_slot(self.selection)
    }

    /// Returns the number of slots in the hotbar.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Returns the index of the currently selected slot.
    pub fn get_selected_index(&self) -> usize {
        self.selection
    }

    /// Gets the entity associated with the slot at the given index.
    ///
    /// Panics if the index is out of bounds.
    pub fn get_slot_entity(&self, index: usize) -> Entity {
        self.slots.get(index).map(|slot| slot.slot_id).unwrap()
    }

    /// Returns whether or not the slot at the given index is dirty.
    pub fn is_dirty(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .map(|slot| slot.is_dirty)
            .unwrap_or(false)
    }

    /// Marks all slots as clean.
    pub fn mark_clean(&mut self) {
        for slot in self.slots.iter_mut() {
            slot.is_dirty = false;
        }
    }

    /// Scrolls the selection by the given delta.
    pub fn scroll(&mut self, delta: i32) {
        let new_selection = self.selection as i32 + delta;
        self.selection = new_selection.rem_euclid(self.slots.len() as i32) as usize;
    }
}

/// This component is used to store the data for a hotbar slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotbarSlotData {
    /// The slot is empty.
    Empty,

    /// The slot contains a tool entity.
    Tool(Entity),

    /// The slot contains a block entity.
    Block(Entity),
}

/// This struct contains metadata for a hotbar slot.
#[derive(Debug, Clone)]
struct HotbarSlotMeta {
    /// The entity associated with the slot.
    slot_id: Entity,

    /// The data stored in the slot.
    data: HotbarSlotData,

    /// Whether or not the slot is dirty and needs to be updated.
    is_dirty: bool,
}
