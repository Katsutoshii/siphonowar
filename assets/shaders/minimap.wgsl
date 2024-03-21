#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords, grid_uv};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<uniform> camera_position: vec2<f32>;
@group(2) @binding(3) var<uniform> viewport_size: vec2<f32>;
@group(2) @binding(4) var<storage> grid: array<u32>;
@group(2) @binding(5) var<storage> visibility_grid: array<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_uv(size, mesh.uv);
    let g_frac = g - floor(g);
    let row = u32(g.y);
    let col = u32(g.x);

    var camera_brightness = vec4<f32>(0.);
    let camera_check = abs(g - camera_position);
    if camera_check.x < viewport_size.x && camera_check.y < viewport_size.y {
        camera_brightness = vec4<f32>(0.02);
    }

    let e = 0.01;
    let boundary_check = abs(mesh.uv - 0.5);
    if boundary_check.x > 0.5 - e {
        return vec4<f32>(0.3);
    }
    if boundary_check.y > 0.5 - e {
        return vec4<f32>(0.3);
    }

    var output_color = vec4<f32>(.1, .1, .1, 1.);
    output_color *= f32((col / 8u + row / 8u) % 2u) * (0.3 - 0.2) + 0.2;

    let visible = visibility_grid[grid_index(size, row, col)];
    var fog = visible;
    fog += visibility_grid[grid_index(size, row + 1u, col + 0u)] * g_frac.y;
    fog += visibility_grid[grid_index(size, row + 0u, col + 1u)] * g_frac.x;
    fog += visibility_grid[grid_index(size, row - 1u, col - 0u)] * (1. - g_frac.y);
    fog += visibility_grid[grid_index(size, row - 0u, col - 1u)] * (1. - g_frac.x);
    fog *= 0.3;

    var highlight = 0.;
    highlight += f32(grid[grid_index(size, row, col)]);
    highlight += f32(grid[grid_index(size, row + 1u, col + 0u)]);
    highlight += f32(grid[grid_index(size, row + 0u, col + 1u)]);
    highlight += f32(grid[grid_index(size, row - 1u, col + 0u)]);
    highlight += f32(grid[grid_index(size, row + 0u, col - 1u)]);
    highlight += f32(grid[grid_index(size, row + 1u, col + 1u)]);
    highlight += f32(grid[grid_index(size, row - 1u, col - 1u)]);
    highlight += f32(grid[grid_index(size, row - 1u, col + 1u)]);
    highlight += f32(grid[grid_index(size, row + 1u, col - 1u)]);
    highlight = min(highlight, HIGHLIGHT_LEVEL) * (1. - visible);


    output_color += 30. * color * highlight;


    // fog += f32(visibility_grid[grid_index(size, row, col)]);
    output_color *= 1. - fog;
    output_color += camera_brightness;
    output_color.a = 0.9;
    return output_color;
}
