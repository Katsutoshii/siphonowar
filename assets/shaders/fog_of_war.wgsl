#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput};
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;

@group(2) @binding(2) var texture: texture_2d<f32>;
@group(2) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);

    var output_color = color;
    let noise_amount = 0.13;
    let sin_t = sin(0.2 * globals.time);
    let sin_xt = sin(0.13 * globals.time);
    let noise = (2. + sin_t) / 3. * perlin_noise_2d(vec2<f32>(g.x + sin_t, g.y - sin_xt) * 0.1);

    var uv = mesh.uv;
    uv.y = 1 - uv.y;

    let e = 0.004;
    var sample_color = textureSample(texture, texture_sampler, uv);
    sample_color += textureSample(texture, texture_sampler, uv + vec2<f32>(0, e));
    sample_color += textureSample(texture, texture_sampler, uv + vec2<f32>(0, -e));
    sample_color += textureSample(texture, texture_sampler, uv + vec2<f32>(e, 0));
    sample_color += textureSample(texture, texture_sampler, uv + vec2<f32>(-e, 0));
    sample_color /= 5.0;
    let alpha = 1.0 - sample_color.a;
    output_color.a *= 0.05 + (1.0 - noise_amount) * alpha + noise_amount * noise;
    return output_color;
    // return output_color;
}
