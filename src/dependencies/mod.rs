pub use std::{
    os::{
        windows::{
            ffi::{ OsStrExt }
        }
    },
    iter::{ once },
    ffi::{ c_int, OsStr, CString },
    ptr::{ null, null_mut, copy_nonoverlapping },
    mem::{ zeroed, size_of }
};

pub use winapi::{
    um::{
        winuser::{ WNDPROC, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, WS_OVERLAPPEDWINDOW, GetClientRect, RegisterClassExW, CreateWindowExW, DefWindowProcW, DestroyWindow, UnregisterClassW },
        libloaderapi::{ GetModuleHandleW },
        memoryapi::{ VirtualAlloc },
        winnt::{ MEM_COMMIT, PAGE_READWRITE },
        d3dcommon::{ D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0, ID3DBlob },
        d3d11::{ D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, ID3D11RenderTargetView, ID3D11DepthStencilView, ID3D11Resource, ID3D11Buffer, ID3D11VertexShader, ID3D11PixelShader, ID3D11InputLayout, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_BUFFER_DESC, D3D11_BIND_VERTEX_BUFFER, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_BIND_INDEX_BUFFER },
        d3dcompiler::{ D3DCompile, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION }
    },
    shared::{
        dxgi::{ DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, IDXGIFactory, IDXGIAdapter, IDXGISwapChain, IDXGIDevice, IDXGISurface },
        dxgitype::{ DXGI_RATIONAL, DXGI_MODE_DESC, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT },
        dxgiformat::{ DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_R32_UINT },
        windef::{ RECT, HWND, HICON, HCURSOR, HBRUSH, HMENU },
        minwindef::{ UINT, LPVOID, DWORD, HINSTANCE },
        winerror::{ SUCCEEDED },
        ntdef::{ NULL, LPCWSTR }
    },
    Interface
};