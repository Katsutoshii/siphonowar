#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput};
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};
#import "shaders/sampler.wgsl"::{textureSample5};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;

@group(2) @binding(2) var visibility_texture: texture_2d<f32>;
@group(2) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);


    let path2 = vec2<f32>(-1.0, 0.33);

    var output_color = color;
    let sin_t = sin(1.4 * globals.time);
    let sin_xt = sin(2.26 * globals.time);
    let noise1_xy = vec2<f32>(g.x + sin_t, g.y - sin_xt) * 0.2;
    let noise1 = perlin_noise_2d(noise1_xy);
    let noise2_xy = vec2<f32>(g.x + 0.2 * sin_xt, g.y + 0.5 * sin_t) * path2 * 0.2;
    let noise2 = perlin_noise_2d(noise2_xy);
    let noise = (noise1 + noise2) / 2;
    let noise_amount = 0.3;

    var uv = mesh.uv;
    uv.y = 1.0 - uv.y;

    let e = 0.004;
    var fog_amount = textureSample5(visibility_texture, texture_sampler, uv, e).a;
    let alpha = 1.0 - fog_amount;
    output_color.a *= alpha + noise_amount * max(noise, 0.);
    return output_color;
    // return output_color;
}
