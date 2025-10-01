use crate::dependencies::{
    CString, null_mut, SUCCEEDED, D3DCompile, ID3DBlob, ID3D11Device, ID3D11DeviceContext, ID3D11VertexShader, ID3D11PixelShader, ID3D11InputLayout, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION, DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT
};

#[derive(Clone)]
pub struct Shader {
    pub vertex: *mut ID3D11VertexShader,
    pub pixel: *mut ID3D11PixelShader,
    pub input_layout: *mut ID3D11InputLayout,
    pub vs_blob: *mut ID3DBlob
}
impl Shader {
    pub fn build(vs_source: &[u8], ps_source: &[u8], device: *mut ID3D11Device) -> Option<Self> {
        let vs_blob = match Self::compile(vs_source, "VSMain", "vs_5_0") {
            Some(vs_blob) => vs_blob,
            None => return None
        };
        let ps_blob = match Self::compile(ps_source, "PSMain", "ps_5_0") {
            Some(ps_blob) => ps_blob,
            None => return None
        };
        let vertex = match Self::create_vertex(device, vs_blob) {
            Some(vertex) => vertex,
            None => return None
        };
        let pixel = match Self::create_pixel(device, ps_blob) {
            Some(pixel) => pixel,
            None => return None
        };
        let input_layout = match Self::create_input_layout(device, vs_blob) {
            Some(input_layout) => input_layout,
            None => return None
        };

        Self::release(ps_blob);

        Some(Self { vertex, pixel, input_layout, vs_blob })
    }

    pub fn setup(&self, context: *mut ID3D11DeviceContext) {
        unsafe {
            (*context).VSSetShader(self.vertex, null_mut(), 0);
            (*context).PSSetShader(self.pixel, null_mut(), 0);
            (*context).IASetInputLayout(self.input_layout);
        }
    }

    fn release(ps_blob: *mut ID3DBlob) {
        unsafe { (*ps_blob).Release() };
    }

    fn compile(source: &[u8], entry_point: &str, target: &str) -> Option<*mut ID3DBlob> {
        let mut blob: *mut ID3DBlob = null_mut();
        let mut error_blob: *mut ID3DBlob = null_mut();
        let entry_point_cstr = CString::new(entry_point).unwrap();
        let target_cstr = CString::new(target).unwrap();

        let hr = unsafe { D3DCompile(
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
        ) };

        if SUCCEEDED(hr) { return Some(blob) };

        None
    }

    fn create_vertex(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> Option<*mut ID3D11VertexShader> {
        let mut shader: *mut ID3D11VertexShader = null_mut();
        let hr = unsafe { (*device).CreateVertexShader(
            (*shader_blob).GetBufferPointer(),
            (*shader_blob).GetBufferSize(),
            null_mut(),
            &mut shader,
        ) };

        if SUCCEEDED(hr) { return Some(shader) };

        None
    }

    fn create_pixel(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> Option<*mut ID3D11PixelShader> {
        let mut shader: *mut ID3D11PixelShader = null_mut();
        let hr = unsafe { (*device).CreatePixelShader(
            (*shader_blob).GetBufferPointer(),
            (*shader_blob).GetBufferSize(),
            null_mut(),
            &mut shader,
        ) };

        if SUCCEEDED(hr) { return Some(shader) };

        None
    }

    fn create_input_layout(device: *mut ID3D11Device, shader_blob: *mut ID3DBlob) -> Option<*mut ID3D11InputLayout> {
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
            }
        ];
            
        let mut input_layout: *mut ID3D11InputLayout = null_mut();
        let hr = unsafe { (*device).CreateInputLayout(
            layout_desc.as_ptr(),
            layout_desc.len() as u32,
            (*shader_blob).GetBufferPointer(),
            (*shader_blob).GetBufferSize(),
            &mut input_layout,
        ) };

        if SUCCEEDED(hr) { return Some(input_layout) };

        None
    }
}
unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}