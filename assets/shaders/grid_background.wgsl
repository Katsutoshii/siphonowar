#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput}
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage, read> grid: array<u32>;
@group(2) @binding(3) var<uniform> wave_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var time = globals.time;

    let g = grid_coords(size, mesh.world_position.xy) / 2.0;
    let row = u32(g.y);
    let col = u32(g.x);
    let path1 = vec2<f32>(-1.0, 0.33);
    let path2 = vec2<f32>(.2, -0.8);
    var path3 = vec2<f32>( 0.01 * sin(time * 0.1),  0.001 * cos(time * 0.1));
    let path4 = vec2<f32>(-0.1, -0.1);
    
    var wave = color * 0.1;
    wave += 0.1 * wave_color * perlin_noise_2d(g + path1 * time * 0.3) * cos(time * 0.01);
    wave += 0.1 * wave_color * perlin_noise_2d(g + path2 * time* 0.3);
    wave += 0.2 * color * abs(perlin_noise_2d(g + path3 * time));
    wave += color * perlin_noise_2d(g + path4 * time) * (0.2 + .05 * sin(time * 0.001));
    return (wave * 0.07 + vec4<f32>(0., 0.05 + 0.01 * cos(time * 0.001), 0.2 + 0.05 * sin(time * 0.001), 0.1)) * 0.8;
}
