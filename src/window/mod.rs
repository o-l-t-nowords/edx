mod window_class_handle;
pub use window_class_handle::WindowClassHandle;

mod window_handle;
pub use window_handle::WindowHandle;

use crate::{ DirectX };

#[derive(Clone)]
pub struct Window {
    pub class_handle: WindowClassHandle,
    pub handle: WindowHandle,
    pub directx: DirectX
}
impl Window {
    pub fn create(name: &str) -> Option<Self> {
        let mut class_handle = WindowClassHandle::build(name);
        class_handle.register();

        let mut handle = WindowHandle::build(&class_handle);
        handle.create();

        let directx = match DirectX::create(&handle) {
            Some(directx) => directx,
            None => return None
        };

        Some(Self {
            class_handle,
            handle,
            directx
        })
    }
}
unsafe impl Send for Window {}
unsafe impl Sync for Window {}