#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};
#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage, read> grid: array<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);
    let row = u32(g.y);
    let col = u32(g.x);

    var output_color = color;
    output_color.a = grid[grid_index(size, row, col)];
    return output_color;
}
