#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage> grid: array<u32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);
    let row = u32(g.y);
    let col = u32(g.x);

    var output_color = color;
    output_color *= f32((col + row) % u32(2)) * (CHECKERBOARD_LIGHT - CHECKERBOARD_DARK) + CHECKERBOARD_DARK;
    let highlight = f32(grid[grid_index(size, row, col)]) * HIGHLIGHT_LEVEL;
    output_color.r += highlight;
    output_color.g += highlight;
    output_color.b += highlight;
    return output_color / 1.5;
}
