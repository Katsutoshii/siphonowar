#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals;
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d}
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords};

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> size: GridSize;
@group(2) @binding(2) var<storage> grid: array<u32>;

// These are the parameters p = [p1, p2, p3] to the 2d value function f(x, y) = p1(x - p3) + p2(x - p3)
// which can get us the triangles we need for different obstacle positions.
const EMPTY: vec3<f32> = vec3<f32>(1., 1., 1.);
const UPRIGHT: vec3<f32> = vec3<f32>(1., 1., 0.5);
const UPLEFT: vec3<f32> = vec3<f32>(-1., 1., 0.5);
const DOWNRIGHT: vec3<f32> = vec3<f32>(1., -1., 0.5);
const DOWNLEFT: vec3<f32> = vec3<f32>(-1., -1., 0.5);
const FULL: vec3<f32> = vec3<f32>(1., 1., 0.);

// For some reason we can't have an array indexed by a dynamic int.
// So we have to use switch case. Hopefully this has okay performance.
fn get_factors(i: u32) -> vec3<f32> {
    switch (i) {
        case 0u: {
            return EMPTY;
        }
        case 1u: {
            return UPRIGHT;
        }
        case 2u: {
            return UPLEFT;
        }
        case 3u: {
            return DOWNRIGHT;
        }
        case 4u: {
            return DOWNLEFT;
        }
        case 5u: {
            return FULL;
        }
        default: {
            return EMPTY;
        }
    }
}

fn hightlight_v(x: f32, y: f32, i: u32) -> f32 {
    let f = get_factors(i);
    return f[0] * (x - f[2]) + f[1] * (y - f[2]);
}

fn hightlight(x: f32, y: f32, i: u32) -> f32 {
    return f32(hightlight_v(x, y, i) > 0.);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let g = grid_coords(size, mesh.world_position.xy);
    let g_frac = g - floor(g);

    let row = u32(g.y);
    let col = u32(g.x);

    let highlight = hightlight(g_frac.x, g_frac.y, grid[grid_index(size, row, col)]);

    var output_color = color;
    output_color.a = 0.8 * highlight;
    return output_color;
}
