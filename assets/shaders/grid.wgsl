// Specifies grid size.
struct GridSize {
    width: f32,
    rows: u32,
    cols: u32
};

/// Computes the grid index from row, col.
fn grid_index(size: GridSize, row: u32, col: u32) -> u32 {
    return row * size.cols + col;
}

/// Computes the offset from the given coordinates.
fn grid_offset(size: GridSize) -> vec2<f32> {
    return vec2<f32>(
        f32(size.cols),
        f32(size.rows)
    ) * size.width / 2.;
}

/// Get fractional rowcol coords from world position.
fn grid_coords(size: GridSize, position: vec2<f32>) -> vec2<f32> {
    return (position + grid_offset(size)) / size.width;
}

/// Get fractional rowcol coords from UV coordinates.
fn grid_uv(size: GridSize, uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(
        f32(size.cols),
        f32(size.rows)
    );
}
