use crate::dependencies::{
    c_int, null_mut, zeroed, WS_OVERLAPPEDWINDOW, CreateWindowExW, DestroyWindow, GetClientRect, DWORD, HINSTANCE, LPVOID, LPCWSTR, NULL, HWND, HMENU, RECT
};

use crate::{ WindowClassHandle};

#[derive(Clone)]
pub struct WindowHandle {
    pub extended_style: DWORD,
    pub class_handle_name: LPCWSTR,
    pub name: LPCWSTR,
    pub style: DWORD,
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub parent: HWND,
    pub menu: HMENU,
    pub instance: HINSTANCE,
    pub param: LPVOID,
    pub hwnd: Option<HWND>
}
impl WindowHandle {
    pub fn build(class_handle: &WindowClassHandle) -> Self {
        Self {
            extended_style: 0,
            class_handle_name: class_handle.name,
            name: class_handle.name,
            style: WS_OVERLAPPEDWINDOW,
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            parent: NULL as HWND,
            menu: NULL as HMENU,
            instance: class_handle.instance,
            param: null_mut(),
            hwnd: None
        }
    }

    pub fn create(&mut self) {
        self.hwnd = Some(unsafe { CreateWindowExW(
            self.extended_style,
            self.class_handle_name,
            self.name,
            self.style,
            self.x,
            self.y,
            self.width,
            self.height,
            self.parent,
            self.menu,
            self.instance,
            self.param
        ) });
    }

    pub fn destroy(&self) {
        if !self.hwnd.is_none() {
            unsafe { DestroyWindow(self.hwnd.unwrap()) };
        }
    }

    pub fn get_rect(&self) -> Option<RECT> {
        if !self.hwnd.is_none() {
            let mut rect =  unsafe { zeroed::<RECT>() };
            unsafe { GetClientRect(self.hwnd.unwrap(), &mut rect) };

            return Some(rect);
        }

        None
    }
}
unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}