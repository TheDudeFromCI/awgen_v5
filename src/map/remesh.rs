//! This module is used to handle the remeshing of voxel chunk.

use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet, Parallel};

use super::blocks::model::BlockModel;
use super::blocks::occlusion::BlockDataOccludedBy;
use super::blocks::shape::BlockShape;
use super::blocks::Block;
use super::chunk::ChunkData;
use crate::math::BlockPos;
use crate::utilities::chunk_iter::ChunkIterator;
use crate::utilities::meshbuf::MeshBuf;

/// This plugin handles the remeshing of chunks.
pub struct ChunkRemeshPlugin;
impl Plugin for ChunkRemeshPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(
            Update,
            (
                remesh,
                update_block_handles,
                on_block_model_updated,
                check_remesh_later,
                remesh_queue_starvation,
            ),
        );
    }
}

/// A component that marks a chunk as needing remeshing.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
#[component(storage = "SparseSet")]
pub struct NeedsRemesh;

/// A component that marks a chunk as needing remeshing, but low priority.
/// Chunks with this component will be marked with [`NeedsRemesh`] as long as
/// there are few chunks being remeshed during the current frame.
///
/// A priority value can be specified to determine the order in which chunks
/// are remeshed. Lower values are remeshed first.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
#[component(storage = "SparseSet")]
pub struct NeedsRemeshLater {
    /// The priority of this chunk. Lower values are remeshed first. Default is
    /// 0.
    pub priority: i32,

    /// If true, the priority value of this component will be reduced by 1 every
    /// frame. This can be used to ensure that chunks are not waiting for too
    /// long to be remeshed.
    pub starvation: bool,
}

impl PartialOrd for NeedsRemeshLater {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NeedsRemeshLater {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// This component stores a set of all unique block entities within a chunk.
/// This component is updated internally when a chunk is marked for remeshing.
/// It is only guaranteed to be up-to-date during the remeshing process.
#[derive(Debug, Default, Clone, Component)]
pub struct UniqueBlocks {
    /// The set of unique block entities within this chunk.
    pub blocks: HashSet<Entity>,
}

/// This system listens for dirty chunks and remeshes them as needed.
pub(crate) fn remesh(
    mut meshes: ResMut<Assets<Mesh>>,
    block_models: Query<&BlockModel>,
    block_shapes: Query<&BlockShape>,
    chunks: Query<(Entity, &ChunkData, Option<&Children>), With<NeedsRemesh>>,
    mut chunk_model_parts: Query<
        (&mut Handle<Mesh>, &mut Handle<StandardMaterial>),
        With<ChunkModelPart>,
    >,
    mut commands: Commands,
) {
    if chunks.is_empty() {
        return;
    }

    let mut queue: Parallel<Vec<(Entity, ChunkModel)>> = Parallel::default();

    chunks.par_iter().for_each_init(
        || queue.borrow_local_mut(),
        |out, (chunk_id, chunk, _)| {
            let models = build_models(chunk, &block_models, &block_shapes);
            for model in models {
                out.push((chunk_id, model));
            }
        },
    );

    let mut chunk_models: HashMap<Entity, Vec<ChunkModel>> = HashMap::default();
    for (chunk_id, model) in queue.drain::<Vec<(Entity, ChunkModel)>>() {
        chunk_models.entry(chunk_id).or_default().push(model);
    }

    for (chunk_id, _, children) in chunks.iter() {
        commands
            .entity(chunk_id)
            .remove::<NeedsRemesh>()
            .remove::<NeedsRemeshLater>();

        // Get all new model parts for this chunk.
        let mut models = chunk_models.remove(&chunk_id).unwrap_or_default();

        // Check through all children of the chunk to see if we can reuse any
        // of them.
        if let Some(children) = children {
            for child in children.iter() {
                let Ok((mesh, mut material)) = chunk_model_parts.get_mut(*child) else {
                    // Ignore non-ChunkModel children.
                    continue;
                };

                // Check if we have more model parts to assign.
                if let Some(model_part) = models.pop() {
                    // Reuse the existing entity.
                    meshes.insert(&*mesh, model_part.mesh);
                    *material = model_part.material;
                } else {
                    // Child is unnecessary, despawn it.
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        // Spawn any remaining model parts directly.
        for model in models {
            commands
                .spawn((
                    ChunkModelPart,
                    MaterialMeshBundle {
                        mesh: meshes.add(model.mesh),
                        material: model.material,
                        ..default()
                    },
                ))
                .set_parent(chunk_id);
        }
    }
}

/// This system iterators over all blocks in chunks that need remeshing and
/// updates the block handles in the [`RemeshWhenBlockLoaded`] components.
pub(crate) fn update_block_handles(
    mut query: Query<(&mut UniqueBlocks, &ChunkData), With<NeedsRemesh>>,
) {
    query.par_iter_mut().for_each(|(mut remesh, chunk)| {
        remesh.blocks = chunk.iter().collect();
    });
}

/// This system listens for changes to [`BlockModel`] components and marks all
/// chunks that contain that block as [`NeedsRemeshLater`] with a priority of 0,
/// no starvation.
///
/// Chunks that are already marked as [`NeedsRemesh`] or [`NeedsRemeshLater`]
/// are ignored.
#[allow(clippy::type_complexity)]
pub(crate) fn on_block_model_updated(
    blocks: Query<Entity, (With<Block>, Changed<BlockModel>)>,
    chunks: Query<
        (Entity, &UniqueBlocks),
        (
            With<ChunkData>,
            Without<NeedsRemesh>,
            Without<NeedsRemeshLater>,
        ),
    >,
    mut commands: Commands,
) {
    for block_id in blocks.iter() {
        let mut count = 0;

        for (chunk_id, unique_blocks) in chunks.iter() {
            if unique_blocks.blocks.contains(&block_id) {
                commands
                    .entity(chunk_id)
                    .insert(NeedsRemeshLater::default());

                count += 1;
            }
        }

        debug!(
            "Block model {block_id} updated, queuing {count} chunks for
remesh."
        );
    }
}

/// This system checks how many chunks are currently being remeshed during this
/// frame, and queues a chunk with [`NeedsRemeshLater`] if there are no chunks
/// currently being remeshed.
pub(crate) fn check_remesh_later(
    queued_chunks: Query<(Entity, &NeedsRemeshLater)>,
    current_chunks: Query<(), With<NeedsRemesh>>,
    mut commands: Commands,
) {
    if !current_chunks.is_empty() {
        return;
    }

    let Some((chunk_id, _)) = queued_chunks.iter().sort::<&NeedsRemeshLater>().next() else {
        return;
    };

    commands
        .entity(chunk_id)
        .remove::<NeedsRemeshLater>()
        .insert(NeedsRemesh);
}

/// This system reduces the priority of chunks with [`NeedsRemeshLater`] that
/// are starving.
pub(crate) fn remesh_queue_starvation(mut chunks: Query<&mut NeedsRemeshLater>) {
    for mut chunk in chunks.iter_mut() {
        if chunk.starvation {
            chunk.priority -= 1;
        }
    }
}

/// This function builds the chunk models from the given block data and
/// materials.
///
/// This function may return an empty list if the chunk contains no visible
/// blocks.
pub fn build_models(
    data: &ChunkData,
    block_models: &Query<&BlockModel>,
    block_shapes: &Query<&BlockShape>,
) -> Vec<ChunkModel> {
    let occlusion = BlockDataOccludedBy::from_block_data(data, block_shapes);
    let mut meshes: HashMap<Handle<StandardMaterial>, MeshBuf> = HashMap::new();

    info!("Occlusion: {:?}", occlusion.get(BlockPos::new(5, 0, 0)));

    for pos in ChunkIterator::default() {
        let block = data.get(pos);
        let Ok(model) = block_models.get(block) else {
            continue;
        };

        let BlockModel::Primitive { material, mesh } = model else {
            continue;
        };

        let mesh_buf = match meshes.contains_key(material) {
            true => meshes.get_mut(material).unwrap(),
            false => meshes.entry(material.clone()).or_insert_with(MeshBuf::new),
        };

        let mut block_mesh = *mesh.clone();
        block_mesh.rotate(Quat::IDENTITY);
        block_mesh.translate(pos.as_vec3());
        block_mesh.append_to(occlusion.get(pos), mesh_buf);
    }

    let mut models = Vec::new();
    for (tileset, mesh) in meshes.into_iter() {
        models.push(ChunkModel {
            mesh: mesh.into(),
            material: tileset,
        });
    }

    models
}

/// A model for a chunk.
pub struct ChunkModel {
    /// The mesh of the chunk model.
    pub mesh: Mesh,

    /// The material of the chunk model.
    pub material: Handle<StandardMaterial>,
}

/// A component that marks a model as part of a chunk model. Entities with this
/// component will be reused when remeshing a chunk.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component)]
pub struct ChunkModelPart;
