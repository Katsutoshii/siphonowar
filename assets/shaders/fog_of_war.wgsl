#import bevy_sprite::mesh2d_vertex_output::VertexOutput;
#import bevy_sprite::mesh2d_view_bindings::globals;
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage> grid: array<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);
    let g_frac = g - floor(g);

    let row = u32(g.y);
    let col = u32(g.x);

    // Sample visibility of nearby cells to get a smoother range of visibility.
    var alpha = grid[grid_index(size, row, col)];
    alpha += grid[grid_index(size, row + 1u, col + 0u)] * g_frac.y;
    alpha += grid[grid_index(size, row + 0u, col + 1u)] * g_frac.x;
    alpha += grid[grid_index(size, row - 1u, col - 0u)] * (1. - g_frac.y);
    alpha += grid[grid_index(size, row - 0u, col - 1u)] * (1. - g_frac.x);
    alpha *= 0.4;

    var output_color = color;
    let noise_amount = 0.13;
    let sin_t = sin(0.2 * globals.time);
    let sin_xt = sin(0.13 * globals.time);
    let noise = (2. + sin_t) / 3. * perlin_noise_2d(vec2<f32>(g.x + sin_t, g.y - sin_xt));

    output_color.a *= (1.0 - noise_amount) * alpha + noise_amount * noise;
    return output_color;
}
