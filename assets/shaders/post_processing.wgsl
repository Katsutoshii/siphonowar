// This shader computes the chromatic aberration effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
#import bevy_render::{globals::Globals};
#import "shaders/perlin_noise_2d.wgsl"::perlin_noise_2d;
#import "shaders/sampler.wgsl"::{textureSampleChromaticAbberation};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
    camera_position: vec3<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(3) var<uniform> settings: PostProcessSettings;

fn clip_to_ndc(clip: vec4<f32>) -> vec3<f32> {
    return clip.xyz * vec3<f32>(1., -1., 1.) / clip.w;
}
fn clip_to_world(clip: vec4<f32>) -> vec3<f32> {
    let ndc = clip_to_ndc(clip);

    let world = settings.view * vec4<f32>(ndc, 1.0);
    return world.xyz;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let time = globals.time * 3.0;
    let offset_strength = settings.intensity * 0.01;

    let amplitude = 0.001;
    let frequency = 1.0;

    var uv = in.uv;


    let world = clip_to_world(in.position);


    let noise_width = 128.;
    let noise_xy = world.xy / noise_width + vec2<f32>(sin(time), cos(1.2 * time)) * 0.2;
    let noise = 0.5 * amplitude * perlin_noise_2d(noise_xy);

    let pulse = amplitude * sin(time - frequency * world.xy / 64.);
    let uv_offset = (pulse.xy + noise);

    // Debug
    // return vec4<f32>(abs(uv_offset), 0., 1.0);

    uv += uv_offset;
    // return textureSample(screen_texture, texture_sampler, uv);
    return textureSampleChromaticAbberation(screen_texture, texture_sampler, uv, offset_strength);
}
