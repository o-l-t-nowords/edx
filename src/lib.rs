use std::{
    os::{
        windows::{
            ffi::{ OsStrExt }
        }
    },
    ffi::{ OsStr, CString },
    ptr::{ null, null_mut, copy_nonoverlapping },
    mem::{ transmute, zeroed, size_of }
};

use winapi::{
    um::{
        winuser::{ WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, WS_OVERLAPPEDWINDOW, GetClientRect, RegisterClassExW, CreateWindowExW, DefWindowProcW, DestroyWindow, UnregisterClassW },
        libloaderapi::{ GetModuleHandleW, GetProcAddress },
        memoryapi::{ VirtualAlloc },
        winnt::{ MEM_COMMIT, PAGE_READWRITE },
        d3dcommon::{ D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0, ID3DBlob, D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST },
        d3d11::{ D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, ID3D11RenderTargetView, ID3D11DepthStencilView, ID3D11Resource, ID3D11Buffer, ID3D11VertexShader, ID3D11PixelShader, ID3D11InputLayout, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_BUFFER_DESC, D3D11_BIND_VERTEX_BUFFER, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_BIND_INDEX_BUFFER, D3D11_VIEWPORT, D3D11_DEPTH_STENCIL_DESC, ID3D11DepthStencilState, ID3D11RasterizerState, D3D11_RASTERIZER_DESC, D3D11_FILL_SOLID, D3D11_CULL_NONE },
        d3dcompiler::{ D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION }
    },
    shared::{
        dxgi::{ DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, IDXGIAdapter, IDXGISwapChain, IDXGIDevice, IDXGISurface },
        dxgitype::{ DXGI_RATIONAL, DXGI_MODE_DESC, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT },
        dxgiformat::{ DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT },
        windef::{ RECT, HWND, HICON, HCURSOR, HBRUSH, HMENU },
        minwindef::{ UINT, HMODULE, FARPROC },
        winerror::{ S_OK },
        ntdef::{ NULL, HRESULT, LPCWSTR }
    },
    Interface
};

#[repr(C)]
#[derive(Clone)]
struct EasyVertex {
    position: [f32; 3],
    color: [f32; 4]
}
impl EasyVertex {
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y, z],
            color,
        }
    }
}
unsafe impl Send for EasyVertex {}
unsafe impl Sync for EasyVertex {}

#[derive(Clone)]
pub struct EasyWindow {
    pub name: Vec<u16>,
    pub class: WNDCLASSEXW,
    pub hwnd: HWND
}
impl EasyWindow {
    pub fn new(name: &str) -> Self {
        Self::create(name)
    }

    fn create(name: &str) -> EasyWindow {
        let window_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: unsafe { GetModuleHandleW(null()) },
            hIcon: NULL as HICON,
            hCursor: NULL as HCURSOR,
            hbrBackground: NULL as HBRUSH,
            lpszMenuName: NULL as LPCWSTR,
            lpszClassName: window_name.as_ptr() as LPCWSTR,
            hIconSm: NULL as HICON,
        };

        if unsafe { RegisterClassExW(&window_class) } == 0 {
            panic!("Failed to register window class");
        }

        let hwnd = unsafe { CreateWindowExW(
            0,
            window_name.as_ptr() as LPCWSTR,
            window_name.as_ptr() as LPCWSTR,
            WS_OVERLAPPEDWINDOW,
            0, 0,
            100, 100,
            NULL as HWND,
            NULL as HMENU,
            window_class.hInstance,
            null_mut()
        ) };

        if hwnd.is_null() {
            panic!("Failed to create window")
        }

        EasyWindow { name: window_name, class: window_class, hwnd: hwnd }
    }

    pub fn destroy(&self) {
        unsafe {
            DestroyWindow(self.class.hInstance as _);
            UnregisterClassW(self.class.lpszClassName, self.class.hInstance);
        }
    }
}
unsafe impl Send for EasyWindow {}
unsafe impl Sync for EasyWindow {}

#[derive(Clone)]
pub struct EasyDirectX {
    pub dxgi: EasyDirectXGI,
    pub d3d: Option<EasyDirect3D>,
    pub shader: Option<EasyShader>,
    pub renderer: Option<EasyRenderer>
}
impl EasyDirectX {
    pub fn new(swapchain: *mut IDXGISwapChain) -> Self {
        let dxgi = EasyDirectXGI::new(swapchain);

        Self { dxgi, d3d: None, shader: None, renderer: None }
    }

    pub fn with_d3d(mut self) -> Self {
        self.d3d = Some(EasyDirect3D::new(self.dxgi.swapchain));
        if let Some(ref mut d3d) = self.d3d {
            self.dxgi.setup(d3d.device, d3d.backbuffer);
            self = self.with_shader(
                br#"
                struct VSInput {
                    float3 pos   : POSITION;
                    float4 color : COLOR;
                };

                struct PSInput {
                    float4 pos   : SV_POSITION;
                    float4 color : COLOR;
                };

                PSInput VSMain(VSInput input) {
                    PSInput output;
                    output.pos = float4(input.pos, 1.0);
                    output.color = input.color;
                    return output;
                }
                "#,
                br#"
                struct PSInput {
                    float4 pos   : SV_POSITION;
                    float4 color : COLOR;
                };

                float4 PSMain(PSInput input) : SV_TARGET {
                    return input.color;
                }
                "#
            )
        }

        self
    }

    pub fn with_shader(mut self, vs_source: &[u8], ps_source: &[u8]) -> Self {
        if let Some(ref d3d) = self.d3d {
            self.shader = Some(EasyShader::new(vs_source, ps_source, d3d.device));
        }

        self
    }

    pub fn with_renderer(mut self) -> Self {
        if let Some(ref d3d) = self.d3d {
            self.renderer = Some(EasyRenderer::new(d3d.device, d3d.context, d3d.rtv, d3d.resolution));
        }

        self
    }
}
unsafe impl Send for EasyDirectX {}
unsafe impl Sync for EasyDirectX {}

#[derive(Clone)]
pub struct EasyDirectXGI {
    pub swapchain: *mut IDXGISwapChain,
    pub device: *mut IDXGIDevice,
    pub adapter: *mut IDXGIAdapter,
    pub surface: *mut IDXGISurface
}
impl EasyDirectXGI {
    pub fn new(swapchain: *mut IDXGISwapChain) -> Self {
        let device = null_mut();
        let adapter = null_mut();
        let surface = null_mut();

        Self { swapchain, device, adapter, surface }
    }

    pub fn setup(&mut self, d3d_device: *mut ID3D11Device, backbuffer: *mut ID3D11Texture2D) {
        self.device = Self::get_device(d3d_device);
        self.adapter = Self::get_adapter(self.device);
        self.surface = Self::get_surface(backbuffer);
    }

    fn get_device(d3d_device: *mut ID3D11Device) -> *mut IDXGIDevice {
        let mut device = null_mut::<IDXGIDevice>();
        let hr = unsafe { (*d3d_device).QueryInterface(
            &IDXGIDevice::uuidof(),
            &mut device as *mut _ as *mut *mut _
        ) };

        if hr != S_OK {
            panic!("Failed to get DXGI Device from D3D11 Device");
        }

        device
    }

    fn get_adapter(device: *mut IDXGIDevice) -> *mut IDXGIAdapter {
        let mut adapter = null_mut::<IDXGIAdapter>();
        let hr = unsafe { (*device).GetAdapter(
            &mut adapter as *mut _ as *mut *mut _
        ) };

        if hr != S_OK {
            panic!("Failed to get DXGI Adapter");
        }

        adapter
    }

    fn get_surface(backbuffer: *mut ID3D11Texture2D) -> *mut IDXGISurface {
        let mut surface: *mut IDXGISurface = null_mut();
        let hr = unsafe { (*backbuffer).QueryInterface(
            &IDXGISurface::uuidof(),
            &mut surface as *mut _ as *mut *mut _
        ) };

        if hr != S_OK {
            panic!("Failed to get DXGI Surface: HRESULT {:x}", hr);
        }

        surface
    }
}
unsafe impl Send for EasyDirectXGI {}
unsafe impl Sync for EasyDirectXGI {}

#[derive(Clone)]
pub struct EasyDirect3D {
    pub device: *mut ID3D11Device,
    pub context: *mut ID3D11DeviceContext,
    pub backbuffer: *mut ID3D11Texture2D,
    pub game_rtv: *mut ID3D11RenderTargetView,
    pub game_dsv: *mut ID3D11DepthStencilView,
    pub rtv: *mut ID3D11RenderTargetView,
    pub dsv: *mut ID3D11DepthStencilView,
    pub dss: *mut ID3D11DepthStencilState,
    pub rss: *mut ID3D11RasterizerState,
    pub desc: DXGI_SWAP_CHAIN_DESC,
    pub hwnd: HWND,
    pub resolution: [u32; 2]
}
impl EasyDirect3D {
    pub fn new(swapchain: *mut IDXGISwapChain) -> Self {
        let device = Self::get_device(swapchain);
        let context = Self::get_context(device);
        let backbuffer = Self::get_backbuffer(swapchain);
        let (game_rtv, game_dsv) = Self::get_render_targets(context);
        let rtv = Self::create_rtv(device, backbuffer);
        let dsv = null_mut::<ID3D11DepthStencilView>();
        let dss = Self::create_dss(device);
        let rss = Self::create_rss(device);
        let desc = Self::get_desc(swapchain);
        let hwnd = Self::get_hwnd(desc);
        let resolution = Self::get_resolution(desc);

        Self { device, context, backbuffer, game_rtv, game_dsv, rtv, dsv, dss, rss, desc, hwnd, resolution }
    }

    pub fn setup(&mut self) {
        self.set_viewport();
        self.set_render_settings();
    }

    pub fn release(&mut self) {
        unsafe {
            (*self.context).OMSetRenderTargets(1, &self.game_rtv, self.game_dsv);

            // if !self.rtv.is_null() {
            //     (*self.rtv).Release();
            //     self.rtv = null_mut();
            // }
            // if !self.dsv.is_null() {
            //     (*self.dsv).Release();
            //     self.dsv = null_mut();
            // }
            // if !self.backbuffer.is_null() {
            //     (*self.backbuffer).Release();
            //     self.backbuffer = null_mut();
            // }
        };
    }

    pub fn get_vtable(window: &EasyWindow) -> usize {
        let module = Self::get_module("d3d11.dll");
        let (feature_level, feature_levels) = Self::get_feature_levels();
        let refresh_rate = Self::get_refresh_rate();
        let buffer_desc = Self::get_buffer_desc(&window, refresh_rate);
        let sample_desc = Self::get_sample_desk();
        let swapchain_desc = Self::get_swapchain_desc(&window, buffer_desc, sample_desc);
        let (swapchain, device, context) = Self::create_device(&window, &swapchain_desc, &feature_levels);
        let methods_table = Self::create_vtable(swapchain, device, context);

        return methods_table as usize;
    }

    fn set_viewport(&self) {
        let mut rect: RECT = unsafe { zeroed() };
        unsafe { GetClientRect( self.hwnd, &mut rect ) };

        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0f32,
            TopLeftY: 0.0f32,
            Width: (rect.right - rect.left) as f32,
            Height: (rect.bottom - rect.top) as f32,
            MinDepth: 0.0f32,
            MaxDepth: 1.0f32,
        };
        unsafe { (*self.context).RSSetViewports( 1, &viewport ) };
    }

    fn set_render_settings(&self) {
        unsafe { (*self.context).IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST) };
        unsafe { (*self.context).OMSetRenderTargets(1, &self.rtv, null_mut()) };
        unsafe { (*self.context).OMSetDepthStencilState(self.dss, 1) };
        unsafe { (*self.context).RSSetState(self.rss) };
    }

    fn get_module(name: &str) -> HMODULE {
        let module_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let module = unsafe { GetModuleHandleW(module_name.as_ptr() as LPCWSTR) };
            
        if module.is_null() {
            panic!("Failed to get module '{}'", name);
        }

        module
    }

    fn get_feature_levels() -> (D3D_FEATURE_LEVEL, [D3D_FEATURE_LEVEL; 2]) {
        (D3D_FEATURE_LEVEL_11_0, [ D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0 ])
    }

    fn get_refresh_rate() -> DXGI_RATIONAL {
        DXGI_RATIONAL {
            Numerator: 60,
            Denominator: 1
        }
    }

    fn get_buffer_desc(window: &EasyWindow, refresh_rate: DXGI_RATIONAL) -> DXGI_MODE_DESC {
        let mut rect: RECT =  unsafe { zeroed() };
        unsafe { GetClientRect(window.hwnd, &mut rect) };
        
        DXGI_MODE_DESC {
            Width: (rect.right - rect.left) as u32,
            Height: (rect.bottom - rect.top) as u32,
            RefreshRate: refresh_rate,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            Scaling: DXGI_MODE_SCALING_UNSPECIFIED
        }
    }

    fn get_sample_desk() -> DXGI_SAMPLE_DESC {
        DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0
        }
    }

    fn get_swapchain_desc(window: &EasyWindow, buffer_desc: DXGI_MODE_DESC, sample_desc: DXGI_SAMPLE_DESC) -> DXGI_SWAP_CHAIN_DESC {
        DXGI_SWAP_CHAIN_DESC {
            BufferDesc: buffer_desc,
            SampleDesc: sample_desc,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 1,
            OutputWindow: window.hwnd,
            Windowed: 1,
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH
        }
    }

    fn get_device(swapchain: *mut IDXGISwapChain) -> *mut ID3D11Device {
        let mut device: *mut ID3D11Device = null_mut();
        let hr = unsafe { (*swapchain).GetDevice(
            &ID3D11Device::uuidof(),
            &mut device as *mut _ as *mut *mut _,
        ) };

        if hr != S_OK {
            panic!("Failed to get device");
        }

        device
    }

    fn get_context(device: *mut ID3D11Device) -> *mut ID3D11DeviceContext {
        let mut context: *mut ID3D11DeviceContext = null_mut();
        unsafe { (*device).GetImmediateContext(&mut context) }

        context
    }

    fn get_backbuffer(swapchain: *mut IDXGISwapChain) -> *mut ID3D11Texture2D {
        let mut backbuffer: *mut ID3D11Texture2D = null_mut();
        let hr = unsafe { (*swapchain).GetBuffer(
            0,
            &ID3D11Texture2D::uuidof(),
            &mut backbuffer as *mut _ as *mut *mut _,
        ) };

        if hr != S_OK {
            panic!("Failed to get backbuffer");
        }

        backbuffer
    }

    fn get_render_targets(context: *mut ID3D11DeviceContext) -> (*mut ID3D11RenderTargetView, *mut ID3D11DepthStencilView) {
        let mut rtv: *mut ID3D11RenderTargetView = null_mut();
        let mut dsv: *mut ID3D11DepthStencilView = null_mut();

        unsafe { (*context).OMGetRenderTargets(1, &mut rtv, &mut dsv) };

        (rtv, dsv)
    }

    pub fn get_desc(swapchain: *mut IDXGISwapChain) -> DXGI_SWAP_CHAIN_DESC {
        let mut desc: DXGI_SWAP_CHAIN_DESC = unsafe { zeroed() };
        let hr = unsafe { (*swapchain).GetDesc(&mut desc) };

        if hr != S_OK {
            panic!("Failed to get desc");
        }

        desc
    }

    fn get_hwnd(desc: DXGI_SWAP_CHAIN_DESC) -> HWND {
        desc.OutputWindow
    }

    fn get_resolution(desc: DXGI_SWAP_CHAIN_DESC) -> [u32; 2] {
        [desc.BufferDesc.Width, desc.BufferDesc.Height]
    }

    fn create_device_and_swapchain(window: &EasyWindow, module: HMODULE) -> FARPROC {
        let device_and_swapchain = unsafe { GetProcAddress(module, "D3D11CreateDeviceAndSwapChain\0".as_ptr() as *const i8) };

        if device_and_swapchain.is_null() {
            window.destroy();
        }

        device_and_swapchain
    }

    fn create_device(window: &EasyWindow, swap_chain_desc: &DXGI_SWAP_CHAIN_DESC, feature_levels: &[D3D_FEATURE_LEVEL]) -> (*mut IDXGISwapChain, *mut ID3D11Device, *mut ID3D11DeviceContext) {
        unsafe {
            let mut swapchain: *mut IDXGISwapChain = null_mut();
            let mut device: *mut ID3D11Device = null_mut();
            let mut context: *mut ID3D11DeviceContext = null_mut();
            let mut feature_level: D3D_FEATURE_LEVEL = 0;

            let func_ptr = D3D11CreateDeviceAndSwapChain as *const ();
            let func: extern "system" fn(
                *mut IDXGIAdapter,
                D3D_DRIVER_TYPE,
                winapi::shared::minwindef::HMODULE,
                winapi::shared::minwindef::UINT,
                *const D3D_FEATURE_LEVEL,
                winapi::shared::minwindef::UINT,
                winapi::shared::minwindef::UINT,
                *const DXGI_SWAP_CHAIN_DESC,
                *mut *mut IDXGISwapChain,
                *mut *mut ID3D11Device,
                *mut D3D_FEATURE_LEVEL,
                *mut *mut ID3D11DeviceContext,
            ) -> HRESULT = transmute(func_ptr);

            let hr = func(
                null_mut(),
                D3D_DRIVER_TYPE_HARDWARE,
                null_mut(),
                0,
                feature_levels.as_ptr(),
                feature_levels.len() as u32,
                D3D11_SDK_VERSION,
                swap_chain_desc,
                &mut swapchain,
                &mut device,
                &mut feature_level,
                &mut context,
            );

            if hr < 0 {
                window.destroy();
                panic!("Failed to create D3D11 device and swap chain");
            } else {
                (swapchain, device, context)
            }
        }
    }

    fn create_vtable(swap_chain: *mut IDXGISwapChain, device: *mut ID3D11Device, context: *mut ID3D11DeviceContext) -> *mut usize {
        const SWAPCHAIN_METHODS: usize = 18;
        const DEVICE_METHODS: usize = 43;
        const CONTEXT_METHODS: usize = 144;
        const TOTAL_METHODS: usize = SWAPCHAIN_METHODS + DEVICE_METHODS + CONTEXT_METHODS;

        let methods_table = unsafe { VirtualAlloc(
            null_mut(),
            TOTAL_METHODS * size_of::<usize>(),
            MEM_COMMIT,
            PAGE_READWRITE,
        ) } as *mut usize;

        if methods_table.is_null() {
            panic!("Methods table is null")
        }

        let swap_chain_vtable = unsafe { *(swap_chain as *mut *mut usize) };
        let device_vtable = unsafe { *(device as *mut *mut usize) };
        let context_vtable = unsafe { *(context as *mut *mut usize) };

        unsafe {
            copy_nonoverlapping(
                swap_chain_vtable,
                methods_table,
                SWAPCHAIN_METHODS,
            );

            copy_nonoverlapping(
                device_vtable,
                methods_table.add(SWAPCHAIN_METHODS),
                DEVICE_METHODS,
            );

            copy_nonoverlapping(
                context_vtable,
                methods_table.add(SWAPCHAIN_METHODS + DEVICE_METHODS),
                CONTEXT_METHODS,
            );
        }

        methods_table
    }

    fn create_rtv(device: *mut ID3D11Device, backbuffer: *mut ID3D11Texture2D) -> *mut ID3D11RenderTargetView {
        let mut rtv: *mut ID3D11RenderTargetView = null_mut();
        let hr = unsafe { (*device).CreateRenderTargetView(
            backbuffer as *mut ID3D11Resource,
            null_mut(),
            &mut rtv
        ) };

        if hr != S_OK || rtv.is_null() {
            panic!("Failed to create RTV: HRESULT {:x}", hr);
        }

        rtv
    }

    fn create_dss(device: *mut ID3D11Device) -> *mut ID3D11DepthStencilState {
        let mut dss: *mut ID3D11DepthStencilState = null_mut();
        let hr = unsafe { (*device).CreateDepthStencilState(
            &D3D11_DEPTH_STENCIL_DESC {
                DepthEnable: 0,
                StencilEnable: 0,
                ..zeroed()
            },
            &mut dss as *mut _ as *mut *mut _
        ) };

        if hr != S_OK || dss.is_null() {
            panic!("Failed to create DSS: HRESULT {:x}", hr);
        }

        dss
    }

    fn create_rss(device: *mut ID3D11Device) -> *mut ID3D11RasterizerState {
        let mut rss: *mut ID3D11RasterizerState = null_mut();
        let hr = unsafe { (*device).CreateRasterizerState(
            &D3D11_RASTERIZER_DESC {
                FillMode: D3D11_FILL_SOLID,
                CullMode: D3D11_CULL_NONE,
                ..zeroed()
            },
            &mut rss as *mut _ as *mut *mut _
        ) };

        if hr != S_OK || rss.is_null() {
            panic!("Failed to create RSS: HRESULT {:x}", hr);
        }

        rss
    }
}
unsafe impl Send for EasyDirect3D {}
unsafe impl Sync for EasyDirect3D {}

#[derive(Clone)]
pub struct EasyShader {
    pub vertex_shader: *mut ID3D11VertexShader,
    pub pixel_shader: *mut ID3D11PixelShader,
    pub input_layout: *mut ID3D11InputLayout,
    pub vs_blob: *mut ID3DBlob
}
impl EasyShader {
    pub fn new(vs_source: &[u8], ps_source: &[u8], device: *mut ID3D11Device) -> Self {
        let vs_blob = Self::compile_shader(vs_source, "VSMain", "vs_5_0");
        let ps_blob = Self::compile_shader(ps_source, "PSMain", "ps_5_0");
        let vertex_shader = Self::create_vertex_shader(device, vs_blob);
        let pixel_shader = Self::create_pixel_shader(device, ps_blob);
        let input_layout = Self::create_input_layout(device, vs_blob);

        unsafe { (*ps_blob).Release() };

        Self { vertex_shader, pixel_shader, input_layout, vs_blob }
    }

    pub fn setup(&self, context: *mut ID3D11DeviceContext) {
        unsafe {
            (*context).VSSetShader(self.vertex_shader, null_mut(), 0);
            (*context).PSSetShader(self.pixel_shader, null_mut(), 0);
            (*context).IASetInputLayout(self.input_layout);
        }
    }

    fn compile_shader(source: &[u8], entry_point: &str, target: &str) -> *mut ID3DBlob {
        unsafe {
            let mut blob: *mut ID3DBlob = null_mut();
            let mut error_blob: *mut ID3DBlob = null_mut();

            let entry_point_cstr = CString::new(entry_point).unwrap();
            let target_cstr = CString::new(target).unwrap();

            let hr = D3DCompile(
                source.as_ptr() as *const _,
                source.len() as _,
                null_mut(),
                null_mut(),
                null_mut(),
                entry_point_cstr.as_ptr(),
                target_cstr.as_ptr(),
                D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
                0,
                &mut blob,
                &mut error_blob
            );

            if hr != S_OK {
                if !error_blob.is_null() {
                    let error_ptr = (*error_blob).GetBufferPointer();
                    let error_len = (*error_blob).GetBufferSize();
                    let error_msg = std::slice::from_raw_parts(error_ptr as *const u8, error_len as usize);
                    eprintln!("Shader compilation error: {}", String::from_utf8_lossy(error_msg));
                    (*error_blob).Release();
                }
                panic!("Failed to compile shader: HRESULT {:?}", hr);
            }

            blob
        }
    }

    fn create_vertex_shader(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> *mut ID3D11VertexShader {
        unsafe {
            let mut shader: *mut ID3D11VertexShader = null_mut();
            let hr = (*device).CreateVertexShader(
                (*shader_blob).GetBufferPointer(),
                (*shader_blob).GetBufferSize(),
                null_mut(),
                &mut shader,
            );

            if hr != S_OK {
                panic!("Failed to create vertex shader");
            }

            shader
        }
    }

    fn create_pixel_shader(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> *mut ID3D11PixelShader {
        unsafe {
            let mut shader: *mut ID3D11PixelShader = null_mut();
            let hr = (*device).CreatePixelShader(
                (*shader_blob).GetBufferPointer(),
                (*shader_blob).GetBufferSize(),
                null_mut(),
                &mut shader,
            );

            if hr != S_OK {
                panic!("Failed to create pixel shader");
            }

            shader
        }
    }

    fn create_input_layout(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> *mut ID3D11InputLayout {
        unsafe {
            let layout_desc = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: b"POSITION\0".as_ptr() as *const _,
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: b"COLOR\0".as_ptr() as *const _,
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 12,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ];
            
            let mut input_layout: *mut ID3D11InputLayout = null_mut();
            let hr = (*device).CreateInputLayout(
                layout_desc.as_ptr(),
                layout_desc.len() as u32,
                (*shader_blob).GetBufferPointer(),
                (*shader_blob).GetBufferSize(),
                &mut input_layout,
            );

            if hr != S_OK {
                panic!("Failed to create input layout");
            }

            input_layout
        }
    }
}
unsafe impl Send for EasyShader {}
unsafe impl Sync for EasyShader {}

#[derive(Clone)]
pub struct EasyRenderer {
    pub device: *mut ID3D11Device,
    pub context: *mut ID3D11DeviceContext,
    pub rtv: *mut ID3D11RenderTargetView,
    pub resolution: [u32; 2],
    pub vertex_buffer: *mut ID3D11Buffer,
    pub vertex_stride: u32,
    pub vertex_count: u32,
    pub index_buffer: *mut ID3D11Buffer,
    pub index_count: u32,
    pub vertices: Vec<EasyVertex>,
    pub indices: Vec<UINT>
}
impl EasyRenderer {
    pub fn new(device: *mut ID3D11Device, context: *mut ID3D11DeviceContext, rtv: *mut ID3D11RenderTargetView, resolution: [u32; 2]) -> Self {
        let vertex_buffer = null_mut();
        let vertex_stride = size_of::<EasyVertex>() as u32;
        let vertex_count = 0;
        let index_buffer = null_mut();
        let index_count = 0;
        let vertices: Vec<EasyVertex> = vec![];
        let indices: Vec<UINT> = vec![];

        Self { device, context, rtv, resolution, vertex_buffer, vertex_stride, vertex_count, index_buffer, index_count, vertices, indices }
    }

    pub fn draw(&mut self) {
        self.flush();
        unsafe { (*self.context).DrawIndexed(self.index_count as u32, 0, 0) };
    }

    pub fn draw_background(&self, color: [f32; 4]) {
        unsafe { (*self.context).ClearRenderTargetView(self.rtv, &color) };
    }

    pub fn draw_line(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4], thickness: f32) {
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let len = (dx * dx + dy * dy).sqrt();

        if len == 0.0 { return };

        let nx = -dy / len;
        let ny =  dx / len;

        let offset_x = nx * (thickness / 2.0);
        let offset_y = ny * (thickness / 2.0);

        let v0 = [start[0] - offset_x, start[1] - offset_y];
        let v1 = [end[0]   - offset_x, end[1]   - offset_y];
        let v2 = [end[0]   + offset_x, end[1]   + offset_y];
        let v3 = [start[0] + offset_x, start[1] + offset_y];
        
        let v0 = self.position_to_ndc(v0);
        let v1 = self.position_to_ndc(v1);
        let v2 = self.position_to_ndc(v2);
        let v3 = self.position_to_ndc(v3);

        let base_index = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            EasyVertex::new(v0.0, v0.1, 0.0, color),
            EasyVertex::new(v1.0, v1.1, 0.0, color),
            EasyVertex::new(v2.0, v2.1, 0.0, color),
            EasyVertex::new(v3.0, v3.1, 0.0, color)
        ]);

        self.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3
        ]);
    }

    pub fn draw_rect(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4], thickness: f32) {
        let x1 = start[0];
        let y1 = start[1];
        let x2 = end[0];
        let y2 = end[1];

        let top_left     = [x1, y1];
        let top_right    = [x2, y1];
        let bottom_right = [x2, y2];
        let bottom_left  = [x1, y2];
        
        self.draw_line(top_left, top_right, color, thickness);
        self.draw_line(top_right, bottom_right, color, thickness);
        self.draw_line(bottom_right, bottom_left, color, thickness);
        self.draw_line(bottom_left, top_left, color, thickness);
    }


    pub fn draw_rect_filled(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
        let x1 = start[0];
        let y1 = start[1];
        let x2 = end[0];
        let y2 = end[1];

        let v0 = [x1, y1];
        let v1 = [x2, y1];
        let v2 = [x2, y2];
        let v3 = [x1, y2];

        let v0 = self.position_to_ndc(v0);
        let v1 = self.position_to_ndc(v1);
        let v2 = self.position_to_ndc(v2);
        let v3 = self.position_to_ndc(v3);

        let base_index = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            EasyVertex::new(v0.0, v0.1, 0.0, color),
            EasyVertex::new(v1.0, v1.1, 0.0, color),
            EasyVertex::new(v2.0, v2.1, 0.0, color),
            EasyVertex::new(v3.0, v3.1, 0.0, color)
        ]);
        
        self.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3
        ]);
    }

    fn position_to_ndc(&self, position: [f32; 2]) -> (f32, f32) {
        let ndc_x = (2.0 * position[0] / self.resolution[0] as f32) - 1.0;
        let ndc_y = 1.0 - (2.0 * position[1] / self.resolution[1] as f32);
        (ndc_x, ndc_y)
    }

    fn thickness_to_ndc(&self, thickness: f32) -> (f32, f32) {
        let px_to_ndc_x = 2.0 / self.resolution[0] as f32;
        let px_to_ndc_y = 2.0 / self.resolution[1] as f32;

        (thickness * px_to_ndc_x, thickness * px_to_ndc_y)
    }

    fn flush(&mut self) {
        if !self.vertices.is_empty() && !self.indices.is_empty() {
            if !self.vertex_buffer.is_null() {
                unsafe { (*self.vertex_buffer).Release(); }
                self.vertex_buffer = null_mut();
            }
            if !self.index_buffer.is_null() {
                unsafe { (*self.index_buffer).Release(); }
                self.index_buffer = null_mut();
            }

            self.vertex_buffer = self.create_vertex_buffer(&self.vertices);
            self.index_buffer = self.create_index_buffer(&self.indices);

            self.vertex_count = self.vertices.len() as u32;
            self.index_count = self.indices.len() as u32;
            self.vertex_stride = size_of::<EasyVertex>() as u32;

            unsafe {
                (*self.context).IASetVertexBuffers(0, 1, &self.vertex_buffer, &self.vertex_stride, &0);
                (*self.context).IASetIndexBuffer(
                    self.index_buffer,
                    DXGI_FORMAT_R32_UINT,
                    0,
                );
                (*self.context).DrawIndexed(self.index_count, 0, 0);
            }
        
            self.vertices.clear();
            self.indices.clear();
        }
    }

    fn set_vertices(&mut self, vertices: &[EasyVertex]) {
        if !self.vertex_buffer.is_null() {
            unsafe { (*self.vertex_buffer).Release(); }
            self.vertex_buffer = null_mut();
        }
        
        self.vertex_buffer = self.create_vertex_buffer(vertices);
        self.vertex_count = vertices.len() as u32;
        self.vertex_stride = size_of::<EasyVertex>() as u32;

        unsafe { (*self.context).IASetVertexBuffers(0, 1, &self.vertex_buffer, &self.vertex_stride, &0) };
    }

    fn set_indices(&mut self, indices: &[UINT]) {
        if !self.index_buffer.is_null() {
            unsafe { (*self.index_buffer).Release(); }
            self.index_buffer = null_mut();
        }

        self.index_buffer = self.create_index_buffer(indices);
        self.index_count = indices.len() as u32;

        unsafe {
            (*self.context).IASetIndexBuffer(
                self.index_buffer,
                if size_of::<UINT>() == 2 {
                    DXGI_FORMAT_R16_UINT
                } else {
                    DXGI_FORMAT_R32_UINT
                },
                0
            );
        }
    }
    
    fn create_vertex_buffer(&self, vertices: &[EasyVertex]) -> *mut ID3D11Buffer {
        unsafe {
            let mut vertex_buffer: *mut ID3D11Buffer = null_mut();
            
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: (vertices.len() * size_of::<EasyVertex>()) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: vertices.as_ptr() as *const _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };
            
            let hr = (*self.device).CreateBuffer(&desc, &data, &mut vertex_buffer);
            
            if hr != S_OK {
                panic!("Failed to create vertex buffer: HRESULT {:x}", hr);
            }
            
            vertex_buffer
        }
    }
    
    fn create_index_buffer(&self, indices: &[UINT]) -> *mut ID3D11Buffer {
        unsafe {
            let mut index_buffer: *mut ID3D11Buffer = null_mut();
            
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: (indices.len() * size_of::<UINT>()) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_INDEX_BUFFER,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: indices.as_ptr() as *const _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };
            
            let hr = (*self.device).CreateBuffer(&desc, &data, &mut index_buffer);
            
            if hr != S_OK {
                panic!("Failed to create index buffer: HRESULT {:x}", hr);
            }
            
            index_buffer
        }
    }
}
unsafe impl Send for EasyRenderer {}
unsafe impl Sync for EasyRenderer {}