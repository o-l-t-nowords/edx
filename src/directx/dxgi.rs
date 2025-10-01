use crate::dependencies::{
    null_mut, SUCCEEDED, ID3D11Device, ID3D11Texture2D, IDXGIAdapter, IDXGISwapChain, IDXGIDevice, IDXGISurface, Interface
};

#[derive(Clone)]
pub struct DirectXGI {
    pub swapchain: *mut IDXGISwapChain,
    pub device: *mut IDXGIDevice,
    pub adapter: *mut IDXGIAdapter,
    pub surface: *mut IDXGISurface
}
impl DirectXGI {
    pub fn get(swapchain: *mut IDXGISwapChain, device: *mut ID3D11Device, backbuffer: *mut ID3D11Texture2D) -> Option<Self> {
        let device = match Self::get_device(device) {
            Some(device) => device,
            None => return None
        };
        let adapter = match Self::get_adapter(device) {
            Some(adapter) => adapter,
            None => return None
        };
        let surface = match Self::get_surface(backbuffer) {
            Some(surface) => surface,
            None => return None
        };

        Some(Self { swapchain, device, adapter, surface })
    }

    fn get_device(d3d_device: *mut ID3D11Device) -> Option<*mut IDXGIDevice> {
        let mut device = null_mut::<IDXGIDevice>();
        let hr = unsafe { (*d3d_device).QueryInterface(
            &IDXGIDevice::uuidof(),
            &mut device as *mut _ as *mut *mut _
        ) };

        if SUCCEEDED(hr) { return Some(device) };

        None
    }

    fn get_adapter(device: *mut IDXGIDevice) -> Option<*mut IDXGIAdapter> {
        let mut adapter = null_mut::<IDXGIAdapter>();
        let hr = unsafe { (*device).GetAdapter(
            &mut adapter as *mut _ as *mut *mut _
        ) };

        if SUCCEEDED(hr) { return Some(adapter) };

        None
    }

    fn get_surface(backbuffer: *mut ID3D11Texture2D) -> Option<*mut IDXGISurface> {
        let mut surface: *mut IDXGISurface = null_mut();
        let hr = unsafe { (*backbuffer).QueryInterface(
            &IDXGISurface::uuidof(),
            &mut surface as *mut _ as *mut *mut _
        ) };

        if SUCCEEDED(hr) { return Some(surface) };

        None
    }
}
unsafe impl Send for DirectXGI {}
unsafe impl Sync for DirectXGI {}