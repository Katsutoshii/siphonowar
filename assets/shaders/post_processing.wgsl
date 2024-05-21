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
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::{globals::Globals}
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(3) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let time = globals.time * 3.0;
    let offset_strength = settings.intensity * 0.01;

    let frequency = 45.0;
    let amplitude = 0.025;

    let uv = in.uv;

    let noise = 0.2 * perlin_noise_2d(uv);
    let pulse = sin(time - frequency * uv);

    // Distance to the center of the screen. The greater the distance, the more distortion.
    let distance_to_center = length(uv.xy - 0.5);
    let shifted_uv = uv + amplitude * noise * vec2(pulse.y, pulse.x);
    let noise_amount = 0.02;
    let shift_amount = max(mix(distance_to_center, noise, noise_amount), 0.15);
    let interp_uv = mix(uv, shifted_uv, shift_amount);

    return vec4<f32>(
        textureSample(screen_texture, texture_sampler, interp_uv + vec2<f32>(offset_strength, -offset_strength)).r,
        textureSample(screen_texture, texture_sampler, interp_uv + vec2<f32>(-offset_strength, 0.0)).g,
        textureSample(screen_texture, texture_sampler, interp_uv + vec2<f32>(0.0, offset_strength)).b,
        1.0
    );
}
