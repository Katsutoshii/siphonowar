// Samples a texture 5 times with offset epsilon along each axis.
fn textureSample5(texture: texture_2d<f32>, texture_sampler: sampler, uv: vec2<f32>, e: f32) -> vec4<f32> {
    var output = textureSample(texture, texture_sampler, uv);
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(0, e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(0, -e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(e, 0));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(-e, 0));
    return output / 5.;
}

// Samples a texture 9 times with offset epsilon along each axis.
fn textureSample9(texture: texture_2d<f32>, texture_sampler: sampler, uv: vec2<f32>, e: f32) -> vec4<f32> {
    var output = textureSample(texture, texture_sampler, uv);
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(0, e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(0, -e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(e, 0));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(-e, 0));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(e, e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(e, -e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(-e, e));
    output += textureSample(texture, texture_sampler, uv + vec2<f32>(-e, -e));
    return output / 9.;
}

/// Samples a texture with chromatic abberation.
fn textureSampleChromaticAbberation(texture: texture_2d<f32>, texture_sampler: sampler, uv: vec2<f32>, e: f32) -> vec4<f32> {
    return vec4<f32>(
        textureSample(texture, texture_sampler, uv + vec2<f32>(e, -e)).r,
        textureSample(texture, texture_sampler, uv + vec2<f32>(-e, 0.0)).g,
        textureSample(texture, texture_sampler, uv + vec2<f32>(0.0, e)).b,
        1.0
    );
}
