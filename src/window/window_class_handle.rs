use crate::dependencies::{
    c_int, OsStrExt, OsStr, UINT, WNDPROC, HINSTANCE, once, null, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, RegisterClassExW, DefWindowProcW, UnregisterClassW, GetModuleHandleW, LPCWSTR, NULL, HICON, HCURSOR, HBRUSH
};

#[derive(Clone)]
pub struct WindowClassHandle {
    pub size: UINT,
    pub style: UINT,
    pub process: WNDPROC,
    pub class_extra: c_int,
    pub window_extra: c_int,
    pub instance: HINSTANCE,
    pub icon: HICON,
    pub cursor: HCURSOR,
    pub background: HBRUSH,
    pub menu_name: LPCWSTR,
    pub name: LPCWSTR,
    pub small_icon: HICON,
    pub wndclassexw: Option<WNDCLASSEXW>
}
impl WindowClassHandle {
    pub fn build(name_str: &str) -> Self {
        let name: Vec<u16> = OsStr::new(name_str)
            .encode_wide()
            .chain(once(0))
            .collect();

        Self {
            size: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            process: Some(DefWindowProcW),
            class_extra: 0,
            window_extra: 0,
            instance: unsafe { GetModuleHandleW(null()) },
            icon: NULL as HICON,
            cursor: NULL as HCURSOR,
            background: NULL as HBRUSH,
            menu_name: NULL as LPCWSTR,
            name: name.as_ptr() as LPCWSTR,
            small_icon: NULL as HICON,
            wndclassexw: None
        }
    }

    pub fn register(&mut self) {
        self.unregister();

        let wndclassexw = WNDCLASSEXW {
            cbSize: self.size,
            style: self.style,
            lpfnWndProc: self.process,
            cbClsExtra: self.class_extra,
            cbWndExtra: self.window_extra,
            hInstance: self.instance,
            hIcon: self.icon,
            hCursor: self.cursor,
            hbrBackground: self.background,
            lpszMenuName: self.menu_name,
            lpszClassName: self.name,
            hIconSm: self.small_icon
        };

        unsafe { RegisterClassExW(&wndclassexw) };
        self.wndclassexw = Some(wndclassexw);
    }

    pub fn unregister(&self) {
        unsafe { UnregisterClassW(self.name, self.instance) };
    }
}
unsafe impl Send for WindowClassHandle {}
unsafe impl Sync for WindowClassHandle {}