#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@binding(0) @group(2) var tileset: texture_2d_array<f32>;
@binding(1) @group(2) var tileset_sampler: sampler;

struct VertexIn {
  @builtin(instance_index) instance_index: u32,
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) tile: u32,
}

struct VertexOut {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) tile: u32,
}

struct FragmentIn {
  @location(0) uv: vec2<f32>,
  @location(1) tile: u32,
}

struct FragmentOut {
  @location(0) color: vec4<f32>,
}

@vertex
fn vertex(vertex: VertexIn) -> VertexOut {
  var out: VertexOut;

  out.clip_position = mesh_position_local_to_clip(
    get_world_from_local(vertex.instance_index),
    vec4<f32>(vertex.position, 1.0),
  );

  out.uv = vertex.uv;
  out.tile = vertex.tile;

  return out;
}

@fragment
fn fragment(fragment: FragmentIn) -> FragmentOut {
  var out: FragmentOut;
  out.color = textureSample(tileset, tileset_sampler, fragment.uv, fragment.tile);
  return out;
}
