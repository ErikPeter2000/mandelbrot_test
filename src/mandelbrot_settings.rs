
/// Settings specifying how to render a region of the Mandelbrot.
pub struct MandelbrotSettings {
    pub width: u32,
    pub height: u32,
    pub max_iterations: u32,
    pub zoom: f32,
    pub zoom_exp: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub gamma: f32,
}