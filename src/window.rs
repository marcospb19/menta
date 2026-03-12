use std::rc::Rc;

use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};
use x11_dl::xlib::{CWOverrideRedirect, Window as XWindow, Xlib};

pub struct ScreenDimensions {
    pub width: u32,
    pub height: u32,
}

pub fn get_x11_window_id(window: &Window) -> XWindow {
    let window_handle = window.window_handle().expect("Failed to get window handle");
    match window_handle.as_raw() {
        RawWindowHandle::Xlib(xlib_handle) => xlib_handle.window as XWindow,
        RawWindowHandle::Xcb(_) => panic!("XCB not supported, please use X11"),
        _ => panic!("Only X11 is supported"),
    }
}

pub unsafe fn open_x11_connection() -> (*mut x11_dl::xlib::Display, Xlib) {
    let xlib = Xlib::open().expect("Failed to open Xlib");
    let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
    assert!(!display.is_null(), "Failed to open X display");
    (display, xlib)
}

pub unsafe fn get_screen_dimensions(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
) -> ScreenDimensions {
    let screen = unsafe { (xlib.XDefaultScreen)(display) };
    let screen_width = unsafe { (xlib.XDisplayWidth)(display, screen) } as u32;
    let screen_height = unsafe { (xlib.XDisplayHeight)(display, screen) } as u32;
    ScreenDimensions {
        width: screen_width,
        height: screen_height,
    }
}

pub unsafe fn set_override_redirect(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    let mut swa: x11_dl::xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
    swa.override_redirect = 1;
    unsafe {
        (xlib.XChangeWindowAttributes)(display, window, CWOverrideRedirect, &raw mut swa);
        (xlib.XSync)(display, 0);
    }
}

pub unsafe fn reparent_to_desktop(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    unsafe {
        let desktop = (xlib.XDefaultRootWindow)(display);
        (xlib.XReparentWindow)(display, window, desktop, 0, 0);
        (xlib.XSync)(display, 0);
    }
}

pub unsafe fn resize_to_fullscreen(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
    width: u32,
    height: u32,
) {
    unsafe {
        (xlib.XResizeWindow)(display, window, width, height);
    }
}

pub unsafe fn set_window_type_desktop(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    unsafe {
        let net_wm_window_type = (xlib.XInternAtom)(display, c"_NET_WM_WINDOW_TYPE".as_ptr(), 0);
        let wm_window_type_desktop =
            (xlib.XInternAtom)(display, c"_NET_WM_WINDOW_TYPE_DESKTOP".as_ptr(), 0);
        (xlib.XChangeProperty)(
            display,
            window,
            net_wm_window_type,
            4,
            32,
            0,
            (&raw const wm_window_type_desktop).cast::<u8>(),
            1,
        );
    }
}

pub unsafe fn set_win_layer_zero(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    unsafe {
        let win_layer = (xlib.XInternAtom)(display, c"_WIN_LAYER".as_ptr(), 0);
        let layer: u32 = 0;
        (xlib.XChangeProperty)(
            display,
            window,
            win_layer,
            6,
            32,
            0,
            (&raw const layer).cast::<u8>(),
            1,
        );
    }
}

pub unsafe fn set_net_wm_states(display: *mut x11_dl::xlib::Display, xlib: &Xlib, window: XWindow) {
    unsafe {
        let net_wm_state = (xlib.XInternAtom)(display, c"_NET_WM_STATE".as_ptr(), 0);
        let net_wm_state_below = (xlib.XInternAtom)(display, c"_NET_WM_STATE_BELOW".as_ptr(), 0);
        let net_wm_state_skip_taskbar =
            (xlib.XInternAtom)(display, c"_NET_WM_STATE_SKIP_TASKBAR".as_ptr(), 0);
        let net_wm_state_skip_pager =
            (xlib.XInternAtom)(display, c"_NET_WM_STATE_SKIP_PAGER".as_ptr(), 0);
        let net_wm_state_sticky = (xlib.XInternAtom)(display, c"_NET_WM_STATE_STICKY".as_ptr(), 0);

        let states = [
            net_wm_state_below,
            net_wm_state_skip_taskbar,
            net_wm_state_skip_pager,
            net_wm_state_sticky,
        ];
        (xlib.XChangeProperty)(
            display,
            window,
            net_wm_state,
            4,
            32,
            0,
            states.as_ptr().cast::<u8>(),
            4,
        );
    }
}

pub unsafe fn set_wm_class_desktop(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    unsafe {
        let wm_class = (xlib.XInternAtom)(display, c"WM_CLASS".as_ptr(), 0);
        let class_hint = b"desktop\0Desktop\0";
        (xlib.XChangeProperty)(
            display,
            window,
            wm_class,
            31,
            8,
            0,
            class_hint.as_ptr(),
            class_hint.len() as i32,
        );
    }
}

pub unsafe fn lower_and_map_window(
    display: *mut x11_dl::xlib::Display,
    xlib: &Xlib,
    window: XWindow,
) {
    unsafe {
        (xlib.XLowerWindow)(display, window);
        (xlib.XMapWindow)(display, window);
        (xlib.XSync)(display, 0);
    }
}

pub unsafe fn flush_and_close(display: *mut x11_dl::xlib::Display, xlib: &Xlib) {
    unsafe {
        (xlib.XFlush)(display);
        (xlib.XCloseDisplay)(display);
    }
}

pub fn setup_x11_window(
    window: &Rc<Window>,
    x11_window_id: &mut Option<XWindow>,
) -> ScreenDimensions {
    let window_id = get_x11_window_id(window);

    unsafe {
        let (display, xlib) = open_x11_connection();
        let dims = get_screen_dimensions(display, &xlib);

        set_override_redirect(display, &xlib, window_id);
        reparent_to_desktop(display, &xlib, window_id);
        resize_to_fullscreen(display, &xlib, window_id, dims.width, dims.height);
        set_window_type_desktop(display, &xlib, window_id);
        set_win_layer_zero(display, &xlib, window_id);
        set_net_wm_states(display, &xlib, window_id);
        set_wm_class_desktop(display, &xlib, window_id);
        lower_and_map_window(display, &xlib, window_id);
        flush_and_close(display, &xlib);

        *x11_window_id = Some(window_id);
        dims
    }
}

pub fn keep_window_lowered(x11_window_id: XWindow) {
    unsafe {
        let (display, xlib) = open_x11_connection();
        (xlib.XLowerWindow)(display, x11_window_id);
        (xlib.XFlush)(display);
        (xlib.XCloseDisplay)(display);
    }
}
