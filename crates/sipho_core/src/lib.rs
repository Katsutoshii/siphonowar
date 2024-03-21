/// Core libraries used by Siphonowar.
pub mod aabb;
pub mod rowcol;
pub mod stages;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        rowcol::{RowCol, RowColDistance},
        stages::SystemStage,
        zindex,
    };
}
