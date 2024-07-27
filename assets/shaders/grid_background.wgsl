#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput}
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage, read> grid: array<u32>;
@group(2) @binding(3) var<uniform> wave_color: vec4<f32>;
@group(2) @binding(4) var sand_texture: texture_2d<f32>;
@group(2) @binding(5) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let path1 = vec2<f32>(-1.0, 0.33);
    let path2 = vec2<f32>(1., -0.8);
    var path3 = vec2<f32>(0.1, 0.1);
    let path4 = vec2<f32>(-0.2, -0.2);
    var time = globals.time;

    let granularity = 10.0;
    let repeat_uv = (mesh.uv * granularity) % 1.0;
    var sand = textureSample(sand_texture, texture_sampler, repeat_uv);
    var wave = color * 0.1;
    let res = (wave * 0.07 + vec4<f32>(0., 0.05 + 0.01 * cos(time * 0.001), 0.2 + 0.05 * sin(time * 0.001), 1.0));
    let tint_color = vec4<f32>(0.0, 0.08, 0.15, 1.0) / 2.0;
    var output = tint_color + (sand / 25.0) + res / 6.0;
    output.a = 0.35;
    return output;
}
