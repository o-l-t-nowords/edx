mod d3d;
pub use d3d::Direct3D;

mod dxgi;
pub use dxgi::DirectXGI;

use crate::dependencies::{
    IDXGISwapChain
};

use crate::{ WindowHandle, Shader, Renderer};

#[derive(Clone)]
pub struct DirectX {
    pub d3d: Direct3D,
    pub dxgi: DirectXGI,
    pub shader: Option<Shader>,
    pub renderer: Option<Renderer>
}
impl DirectX {
    pub fn create(window_handle: &WindowHandle) -> Option<Self> {
        let (swapchain, device, context) = match Direct3D::create_device_and_swapchain(&window_handle) {
            Some((swapchain, device, context)) => (swapchain, device, context),
            None => return None
        };

        Self::get(swapchain)
    }

    pub fn get(swapchain: *mut IDXGISwapChain) -> Option<Self> {
        let d3d = match Direct3D::get(swapchain) {
            Some(d3d) => d3d,
            None => return None
        };
        let dxgi = match DirectXGI::get(swapchain, d3d.device, d3d.backbuffer) {
            Some(dxgi) => dxgi,
            None => return None
        };
        let shader = None;
        let renderer = None;

        Some(Self { dxgi, d3d, shader, renderer })
    }

    pub fn update(&mut self, swapchain: *mut IDXGISwapChain) {
        if self.dxgi.swapchain != swapchain {
            self.release();
            let mut dx = match Self::get(swapchain) {
                Some(dx) => dx,
                None => return
            };
            dx.shader = self.shader.take();
            dx.renderer = Some(Renderer::create(&dx.d3d));
            *self = dx;
        }
    }

    pub fn setup(&mut self) {
        if self.shader.is_none() {
            self.create_shader();
        }

        if let Some(shader) = self.shader.as_ref() {
            shader.setup(self.d3d.context);
        } else {
            panic!("Shader creation failed");
        }

        if self.renderer.is_none() {
            self.renderer = Some(self.create_renderer());
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.setup();
            }
        }
    }

    pub fn release(&mut self) {
        if !self.renderer.is_none() {
            self.renderer.as_mut().unwrap().release();
            self.renderer = None;
        }
    }

    pub fn get_renderer(&mut self) -> &mut Renderer {
        let renderer = self.renderer.as_mut().unwrap();
        renderer.set_own_render();
        renderer
    }

    fn create_shader(&mut self) {
        self.shader = Some(match Shader::build(
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
            "#,
            self.d3d.device
        ) {
            Some(shader) => shader,
            None => return
        });
    }

    fn create_renderer(&mut self) -> Renderer {
        Renderer::create(&self.d3d)
    }
}
unsafe impl Send for DirectX {}
unsafe impl Sync for DirectX {}