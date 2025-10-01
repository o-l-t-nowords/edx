use crate::dependencies::{
    null_mut, copy_nonoverlapping, VirtualAlloc, SUCCEEDED, IDXGIFactory, Interface, MEM_COMMIT, PAGE_READWRITE
};

use crate::{ DirectX };

#[derive(Clone)]
pub struct VTable {
    pub ptr: *mut usize
}
impl VTable {
    pub fn get_swapchain(dx: &DirectX) -> Option<Self> {
        const SWAPCHAIN_METHODS: usize = 18;
        const DEVICE_METHODS: usize = 43;
        const CONTEXT_METHODS: usize = 144;
        const TOTAL_METHODS: usize = SWAPCHAIN_METHODS + DEVICE_METHODS + CONTEXT_METHODS;

        let vtable = unsafe { VirtualAlloc(
            null_mut(),
            TOTAL_METHODS * size_of::<usize>(),
            MEM_COMMIT,
            PAGE_READWRITE
        ) } as *mut usize;

        if vtable.is_null() { return None };

        let swapchain_vtable = unsafe { *(dx.dxgi.swapchain as *mut *mut usize) };
        let device_vtable = unsafe { *(dx.d3d.device as *mut *mut usize) };
        let context_vtable = unsafe { *(dx.d3d.context as *mut *mut usize) };

        unsafe {
            copy_nonoverlapping(
                swapchain_vtable,
                vtable,
                SWAPCHAIN_METHODS,
            );

            copy_nonoverlapping(
                device_vtable,
                vtable.add(SWAPCHAIN_METHODS),
                DEVICE_METHODS,
            );

            copy_nonoverlapping(
                context_vtable,
                vtable.add(SWAPCHAIN_METHODS + DEVICE_METHODS),
                CONTEXT_METHODS,
            );
        }

        Some(Self { ptr: vtable })
    }

    pub fn get_factory(dx: &DirectX) -> Option<Self> {
        let mut factory: *mut IDXGIFactory = null_mut();
        let hr = unsafe { (*dx.dxgi.swapchain).GetParent(
            &IDXGIFactory::uuidof(),
            &mut factory as *mut _ as *mut *mut _,
        ) };

        if !SUCCEEDED(hr) || factory.is_null() { return None };

        let vtable = unsafe { *(factory as *mut *mut usize) };

        Some(Self { ptr: vtable })
    }
}
unsafe impl Send for VTable {}
unsafe impl Sync for VTable {}