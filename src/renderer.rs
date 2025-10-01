use crate::dependencies::{
    null_mut, size_of, SUCCEEDED, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, ID3D11RenderTargetView, ID3D11DepthStencilView, ID3D11Resource, ID3D11Buffer, D3D11_BUFFER_DESC, D3D11_BIND_VERTEX_BUFFER, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_BIND_INDEX_BUFFER, UINT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT
};

use crate::{ Vertex, Direct3D };

#[derive(Clone)]
pub struct Renderer {
    pub device: *mut ID3D11Device,
    pub context: *mut ID3D11DeviceContext,
    pub backbuffer: *mut ID3D11Texture2D,
    pub resolution: [u32; 2],
    pub game_rtv: *mut ID3D11RenderTargetView,
    pub game_dsv: *mut ID3D11DepthStencilView,
    pub rtv: *mut ID3D11RenderTargetView,
    pub dsv: *mut ID3D11DepthStencilView,
    pub vertex_buffer: *mut ID3D11Buffer,
    pub vertex_stride: u32,
    pub vertex_count: u32,
    pub index_buffer: *mut ID3D11Buffer,
    pub index_count: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<UINT>
}
impl Renderer {
    pub fn create(d3d: &Direct3D) -> Self {
        let device = d3d.device;
        let context = d3d.context;
        let backbuffer = d3d.backbuffer;
        let resolution = d3d.resolution;
        let (game_rtv, game_dsv) = Self::get_render_targets(context);
        let (rtv, dsv) = (null_mut::<ID3D11RenderTargetView>(), null_mut::<ID3D11DepthStencilView>());
        let vertex_buffer = null_mut();
        let vertex_stride = size_of::<Vertex>() as u32;
        let vertex_count = 0;
        let index_buffer = null_mut();
        let index_count = 0;
        let vertices: Vec<Vertex> = vec![];
        let indices: Vec<UINT> = vec![];

        Self { device, context, backbuffer, resolution, game_rtv, game_dsv, rtv, dsv, vertex_buffer, vertex_stride, vertex_count, index_buffer, index_count, vertices, indices }
    }

    pub fn setup(&mut self) {
        self.create_rtv();
    }

    pub fn release(&mut self) {
        self.release_rtv();
    }

    pub fn flush(&mut self) {
        if !self.vertices.is_empty() && !self.indices.is_empty() {
            if !self.vertex_buffer.is_null() {
                unsafe { (*self.vertex_buffer).Release(); }
                self.vertex_buffer = null_mut();
            }
            if !self.index_buffer.is_null() {
                unsafe { (*self.index_buffer).Release(); }
                self.index_buffer = null_mut();
            }

            self.vertex_buffer = match self.create_vertex_buffer(&self.vertices) {
                Some(vertex_buffer) => vertex_buffer,
                None => return
            };
            self.index_buffer = match self.create_index_buffer(&self.indices) {
                Some(index_buffer) => index_buffer,
                None => return
            };

            self.vertex_count = self.vertices.len() as u32;
            self.index_count = self.indices.len() as u32;
            self.vertex_stride = size_of::<Vertex>() as u32;

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

        self.set_game_render();
    }

    pub fn set_own_render(&mut self) {
        unsafe { (*self.context).OMSetRenderTargets(1, &self.rtv, self.dsv) };
    }

    pub fn set_game_render(&mut self) {
        unsafe { (*self.context).OMSetRenderTargets(1, &self.game_rtv, self.game_dsv) };
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
            Vertex { position: [v0.0, v0.1, 0.0], color },
            Vertex { position: [v1.0, v1.1, 0.0], color },
            Vertex { position: [v2.0, v2.1, 0.0], color },
            Vertex { position: [v3.0, v3.1, 0.0], color }
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
            Vertex { position: [v0.0, v0.1, 0.0], color },
            Vertex { position: [v1.0, v1.1, 0.0], color },
            Vertex { position: [v2.0, v2.1, 0.0], color },
            Vertex { position: [v3.0, v3.1, 0.0], color }
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

    fn set_vertices(&mut self, vertices: &[Vertex]) {
        if !self.vertex_buffer.is_null() {
            unsafe { (*self.vertex_buffer).Release(); }
            self.vertex_buffer = null_mut();
        }
        
        self.vertex_buffer = match self.create_vertex_buffer(vertices) {
            Some(vertex_buffer) => vertex_buffer,
            None => return
        };
        self.vertex_count = vertices.len() as u32;
        self.vertex_stride = size_of::<Vertex>() as u32;

        unsafe { (*self.context).IASetVertexBuffers(0, 1, &self.vertex_buffer, &self.vertex_stride, &0) };
    }

    fn set_indices(&mut self, indices: &[UINT]) {
        if !self.index_buffer.is_null() {
            unsafe { (*self.index_buffer).Release(); }
            self.index_buffer = null_mut();
        }

        self.index_buffer = match self.create_index_buffer(indices) {
            Some(index_buffer) => index_buffer,
            None => return
        };
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
    
    fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Option<*mut ID3D11Buffer> {
        unsafe {
            let mut vertex_buffer: *mut ID3D11Buffer = null_mut();
            
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: (vertices.len() * size_of::<Vertex>()) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0
            };
            
            let data = D3D11_SUBRESOURCE_DATA {
                pSysMem: vertices.as_ptr() as *const _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0
            };
            
            let hr = (*self.device).CreateBuffer(&desc, &data, &mut vertex_buffer);
            
            if SUCCEEDED(hr) { return Some(vertex_buffer) };
            
            None
        }
    }
    
    fn create_index_buffer(&self, indices: &[UINT]) -> Option<*mut ID3D11Buffer> {
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
            
            if SUCCEEDED(hr) { return Some(index_buffer) };
            
            None
        }
    }

    fn get_render_targets(context: *mut ID3D11DeviceContext) -> (*mut ID3D11RenderTargetView, *mut ID3D11DepthStencilView) {
        let (mut rtv, mut dsv) = (null_mut::<ID3D11RenderTargetView>(), null_mut::<ID3D11DepthStencilView>());

        unsafe { (*context).OMGetRenderTargets(1, &mut rtv, &mut dsv) };

        (rtv, dsv)
    }

    fn create_rtv(&mut self) {
        let hr = unsafe { (*self.device).CreateRenderTargetView(
            self.backbuffer as *mut ID3D11Resource,
            null_mut(),
            &mut self.rtv
        ) };

        if !SUCCEEDED(hr) { self.rtv = null_mut::<ID3D11RenderTargetView>() };
    }

    fn release_rtv(&mut self) {
        if !self.rtv.is_null() {
            unsafe { (*self.rtv).Release() };
            self.rtv = null_mut::<ID3D11RenderTargetView>();
        }
    }
}
unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}