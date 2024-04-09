// This shader draws a circle with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords, grid_uv, SIZE};
#import "shaders/team.wgsl"::{TEAM_NONE, TEAM_BLUE, TEAM_RED, NUM_TEAMS};
#import "shaders/constants.wgsl"::{HIGHLIGHT_LEVEL};

struct GridEntry {
    visibility: f32,
    team_presence: array<f32, NUM_TEAMS>,
}
struct MinimapUiMaterial {
    @location(0) colors: array<vec4<f32>, NUM_TEAMS>,
    @location(1) size: GridSize,
    @location(2) camera_position: vec2<f32>,
    @location(3) viewport_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> input: MinimapUiMaterial;
@group(1) @binding(1)
var<storage> grid: array<GridEntry>;

fn get_visibility(row: u32, col: u32) -> f32 {
    return grid[grid_index(input.size, row, col)].visibility;
}

fn get_team_presence(row: u32, col: u32) -> vec3<f32> {
    let arr = grid[grid_index(input.size, row, col)].team_presence;
    return vec3<f32>(arr[0], arr[1], arr[2]);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {

    let g = grid_uv(input.size, in.uv);
    let g_frac = g - floor(g);
    let row = u32(g.y);
    let col = u32(g.x);

    var camera_brightness = vec4<f32>(0.);
    let camera_check = abs(g - input.camera_position);
    if camera_check.x < input.viewport_size.x && camera_check.y < input.viewport_size.y {
        camera_brightness = vec4<f32>(0.05);
    }

    let e = 0.01;
    let boundary_check = abs(in.uv - 0.5);
    if boundary_check.x > 0.5 - e {
        return vec4<f32>(0.3);
    }
    if boundary_check.y > 0.5 - e {
        return vec4<f32>(0.3);
    }

    var output_color = vec4<f32>(.1, .1, .1, 1.);
    output_color *= f32((col / 8u + row / 8u) % 2u) * (0.05) + 0.1;

    let visible = grid[grid_index(input.size, row, col)].visibility;
    var fog = visible;
    fog += get_visibility(row + 1u, col + 0u) * g_frac.y;
    fog += get_visibility(row + 0u, col + 1u) * g_frac.x;
    fog += get_visibility(row - 1u, col - 0u) * (1. - g_frac.y);
    fog += get_visibility(row - 0u, col - 1u) * (1. - g_frac.x);
    fog /= 3.;

    if fog >= 1. {
        var highlight = get_team_presence(row, col);
        highlight += get_team_presence(row + 1u, col + 0u);
        highlight += get_team_presence(row + 0u, col + 1u);
        highlight += get_team_presence(row - 1u, col + 0u);
        highlight += get_team_presence(row + 0u, col - 1u);
        highlight += get_team_presence(row + 1u, col + 1u);
        highlight += get_team_presence(row - 1u, col - 1u);
        highlight += get_team_presence(row - 1u, col + 1u);
        highlight += get_team_presence(row + 1u, col - 1u);
        highlight.x = min(highlight.x, HIGHLIGHT_LEVEL);
        highlight.y = min(highlight.y, HIGHLIGHT_LEVEL);
        highlight.z = min(highlight.z, HIGHLIGHT_LEVEL);

        output_color += 30. * input.colors[TEAM_NONE] * highlight.x;
        output_color += 30. * input.colors[TEAM_BLUE] * highlight.y;
        output_color += 30. * input.colors[TEAM_RED] * highlight.z;
        output_color /= 2. * (highlight.x + highlight.y + highlight.z + 1.);
    }

    output_color *= fog;
    output_color += camera_brightness;
    output_color.a = 0.9;
    return output_color;
}

