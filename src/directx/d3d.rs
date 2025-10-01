use crate::dependencies::{
    null_mut, zeroed, D3D11CreateDeviceAndSwapChain, SUCCEEDED, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, IDXGISwapChain, HWND, Interface, DXGI_SWAP_CHAIN_DESC, DXGI_MODE_DESC, DXGI_RATIONAL, DXGI_SAMPLE_DESC, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_SWAP_EFFECT_DISCARD, DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0, D3D_DRIVER_TYPE_HARDWARE, D3D11_SDK_VERSION
};

use crate::{ WindowHandle };

#[derive(Clone)]
pub struct Direct3D {
    pub device: *mut ID3D11Device,
    pub context: *mut ID3D11DeviceContext,
    pub backbuffer: *mut ID3D11Texture2D,
    pub desc: DXGI_SWAP_CHAIN_DESC,
    pub hwnd: HWND,
    pub resolution: [u32; 2]
}
impl Direct3D {
    pub fn get(swapchain: *mut IDXGISwapChain) -> Option<Self> {
        let device = match Self::get_device(swapchain) {
            Some(device) => device,
            None => return None
        };
        let context = Self::get_context(device);
        let backbuffer = match Self::get_backbuffer(swapchain) {
            Some(backbuffer) => backbuffer,
            None => return None
        };
        let desc = match Self::get_desc(swapchain) {
            Some(desc) => desc,
            None => return None
        };
        let hwnd = Self::get_hwnd(desc);
        let resolution = Self::get_resolution(desc);

        Some(Self { device, context, backbuffer, desc, hwnd, resolution })
    }

    pub fn create_device_and_swapchain(window_handle: &WindowHandle) -> Option<(*mut IDXGISwapChain, *mut ID3D11Device, *mut ID3D11DeviceContext)> {
        let rect = match window_handle.get_rect() {
            Some(rect) => rect,
            None => return None
        };
        let feature_levels = [D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0];
        let mut feature_level = D3D_FEATURE_LEVEL_11_0;
        let mut swapchain = null_mut::<IDXGISwapChain>();
        let mut device = null_mut::<ID3D11Device>();
        let mut context = null_mut::<ID3D11DeviceContext>();

        let hr = unsafe { D3D11CreateDeviceAndSwapChain(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            0,
            feature_levels.as_ptr(),
            feature_levels.len() as u32,
            D3D11_SDK_VERSION,
            &DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: (rect.right - rect.left) as u32,
                    Height: (rect.bottom - rect.top) as u32,
                    RefreshRate: DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1
                    },
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: DXGI_MODE_SCALING_UNSPECIFIED
                },
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 1,
                OutputWindow: window_handle.hwnd.unwrap(),
                Windowed: 1,
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH
            },
            &mut swapchain,
            &mut device,
            &mut feature_level,
            &mut context
        ) };

        if SUCCEEDED(hr) { return Some((swapchain, device, context)) }

        None
    }

    fn get_device(swapchain: *mut IDXGISwapChain) -> Option<*mut ID3D11Device> {
        let mut device = null_mut::<ID3D11Device>();
        let hr = unsafe { (*swapchain).GetDevice(
            &ID3D11Device::uuidof(),
            &mut device as *mut _ as *mut *mut _,
        ) };

        if SUCCEEDED(hr) { return Some(device) };

        None
    }

    fn get_context(device: *mut ID3D11Device) -> *mut ID3D11DeviceContext {
        let mut context: *mut ID3D11DeviceContext = null_mut();
        unsafe { (*device).GetImmediateContext(&mut context) }

        context
    }

    fn get_backbuffer(swapchain: *mut IDXGISwapChain) -> Option<*mut ID3D11Texture2D> {
        let mut backbuffer = null_mut::<ID3D11Texture2D>();
        let hr = unsafe { (*swapchain).GetBuffer(
            0,
            &ID3D11Texture2D::uuidof(),
            &mut backbuffer as *mut _ as *mut *mut _,
        ) };

        if SUCCEEDED(hr) { return Some(backbuffer) };

        None
    }

    pub fn get_desc(swapchain: *mut IDXGISwapChain) -> Option<DXGI_SWAP_CHAIN_DESC> {
        let mut desc = unsafe { zeroed::<DXGI_SWAP_CHAIN_DESC>() };
        let hr = unsafe { (*swapchain).GetDesc(&mut desc) };

        if SUCCEEDED(hr) { return Some(desc) };

        None
    }

    fn get_hwnd(desc: DXGI_SWAP_CHAIN_DESC) -> HWND {
        desc.OutputWindow
    }

    fn get_resolution(desc: DXGI_SWAP_CHAIN_DESC) -> [u32; 2] {
        [desc.BufferDesc.Width, desc.BufferDesc.Height]
    }
}
unsafe impl Send for Direct3D {}
unsafe impl Sync for Direct3D {}