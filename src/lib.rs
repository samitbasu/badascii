pub mod action;
pub mod analyze;
pub mod rect;
pub mod render;
pub mod roughr_egui;
pub mod svg;
pub mod tc;
pub mod text_buffer;
pub struct Resize {
    pub num_rows: u32,
    pub num_cols: u32,
}
