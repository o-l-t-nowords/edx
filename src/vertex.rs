#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4]
}
unsafe impl Send for Vertex {}
unsafe impl Sync for Vertex {}