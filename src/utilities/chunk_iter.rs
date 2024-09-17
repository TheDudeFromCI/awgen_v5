//! This module implements an iterator over all block positions within a chunk.

use crate::map::pos::{BlockPos, CHUNK_SIZE};

/// The size of a chunk in blocks. (as an i32)
const SIZE: i32 = CHUNK_SIZE as i32;

/// An iterator over all block positions within a chunk.
#[derive(Debug, Clone)]
pub struct ChunkIterator {
    /// The next position
    next: Option<BlockPos>,
}

impl Default for ChunkIterator {
    fn default() -> Self {
        Self {
            next: Some(BlockPos::new(0, 0, 0)),
        }
    }
}

impl Iterator for ChunkIterator {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;

        let mut next = current;
        next.z += 1;
        if next.z >= SIZE {
            next.z = 0;
            next.y += 1;
            if next.y >= SIZE {
                next.y = 0;
                next.x += 1;
            }
        }

        if next.x < SIZE {
            self.next = Some(next);
        } else {
            self.next = None;
        }

        Some(current)
    }
}
