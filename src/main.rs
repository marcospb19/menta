use std::{num::NonZeroU32, rc::Rc};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes, WindowId},
};
use x11_dl::xlib::{CWOverrideRedirect, Window as XWindow, Xlib};

struct App {
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    x11_window_id: Option<XWindow>,
    width: u32,
    height: u32,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            x11_window_id: None,
            width: 0,
            height: 0,
        }
    }

    fn setup_x11_window(&mut self, window: &Window) -> (u32, u32) {
        let x11_window_id = {
            let window_handle = window.window_handle().expect("Failed to get window handle");
            match window_handle.as_raw() {
                RawWindowHandle::Xlib(xlib_handle) => xlib_handle.window as XWindow,
                RawWindowHandle::Xcb(_) => panic!("XCB not supported, please use X11"),
                _ => panic!("Only X11 is supported"),
            }
        };

        unsafe {
            let xlib = Xlib::open().expect("Failed to open Xlib");

            let display = (xlib.XOpenDisplay)(std::ptr::null());
            if display.is_null() {
                panic!("Failed to open X display");
            }

            let desktop = (xlib.XDefaultRootWindow)(display);
            let screen = (xlib.XDefaultScreen)(display);
            let screen_width = (xlib.XDisplayWidth)(display, screen) as u32;
            let screen_height = (xlib.XDisplayHeight)(display, screen) as u32;

            let mut swa: x11_dl::xlib::XSetWindowAttributes = std::mem::zeroed();
            swa.override_redirect = 1;
            (xlib.XChangeWindowAttributes)(
                display,
                x11_window_id,
                CWOverrideRedirect as u64,
                &mut swa,
            );
            (xlib.XSync)(display, 0);

            (xlib.XReparentWindow)(display, x11_window_id, desktop, 0, 0);
            (xlib.XSync)(display, 0);

            (xlib.XResizeWindow)(display, x11_window_id, screen_width, screen_height);

            let net_wm_window_type =
                (xlib.XInternAtom)(display, b"_NET_WM_WINDOW_TYPE\0".as_ptr() as *const i8, 0);
            let wm_window_type_desktop = (xlib.XInternAtom)(
                display,
                b"_NET_WM_WINDOW_TYPE_DESKTOP\0".as_ptr() as *const i8,
                0,
            );
            (xlib.XChangeProperty)(
                display,
                x11_window_id,
                net_wm_window_type,
                4,
                32,
                0,
                &wm_window_type_desktop as *const _ as *const u8,
                1,
            );

            let win_layer = (xlib.XInternAtom)(display, b"_WIN_LAYER\0".as_ptr() as *const i8, 0);
            let layer: u32 = 0;
            (xlib.XChangeProperty)(
                display,
                x11_window_id,
                win_layer,
                6,
                32,
                0,
                &layer as *const _ as *const u8,
                1,
            );

            let net_wm_state =
                (xlib.XInternAtom)(display, b"_NET_WM_STATE\0".as_ptr() as *const i8, 0);
            let net_wm_state_below =
                (xlib.XInternAtom)(display, b"_NET_WM_STATE_BELOW\0".as_ptr() as *const i8, 0);
            let net_wm_state_skip_taskbar = (xlib.XInternAtom)(
                display,
                b"_NET_WM_STATE_SKIP_TASKBAR\0".as_ptr() as *const i8,
                0,
            );
            let net_wm_state_skip_pager = (xlib.XInternAtom)(
                display,
                b"_NET_WM_STATE_SKIP_PAGER\0".as_ptr() as *const i8,
                0,
            );
            let net_wm_state_sticky =
                (xlib.XInternAtom)(display, b"_NET_WM_STATE_STICKY\0".as_ptr() as *const i8, 0);

            let states = [
                net_wm_state_below,
                net_wm_state_skip_taskbar,
                net_wm_state_skip_pager,
                net_wm_state_sticky,
            ];
            (xlib.XChangeProperty)(
                display,
                x11_window_id,
                net_wm_state,
                4,
                32,
                0,
                states.as_ptr() as *const u8,
                4,
            );

            let wm_class = (xlib.XInternAtom)(display, b"WM_CLASS\0".as_ptr() as *const i8, 0);
            let class_hint = b"desktop\0Desktop\0";
            (xlib.XChangeProperty)(
                display,
                x11_window_id,
                wm_class,
                31,
                8,
                0,
                class_hint.as_ptr() as *const u8,
                class_hint.len() as i32,
            );

            (xlib.XLowerWindow)(display, x11_window_id);
            (xlib.XMapWindow)(display, x11_window_id);
            (xlib.XSync)(display, 0);

            (xlib.XFlush)(display);
            (xlib.XCloseDisplay)(display);

            self.x11_window_id = Some(x11_window_id);
            (screen_width, screen_height)
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Desktop Background")
            .with_transparent(false)
            .with_decorations(false)
            .with_resizable(false)
            .with_visible(false);

        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        let window_rc = Rc::new(window);
        let (screen_width, screen_height) = self.setup_x11_window(&window_rc);
        self.width = screen_width;
        self.height = screen_height;

        let _ = window_rc.request_inner_size(PhysicalSize::new(screen_width, screen_height));
        window_rc.set_visible(true);

        let context = softbuffer::Context::new(window_rc.clone())
            .expect("Failed to create softbuffer context");
        let mut surface = softbuffer::Surface::new(&context, window_rc.clone())
            .expect("Failed to create softbuffer surface");

        if self.width > 0 && self.height > 0 {
            surface
                .resize(
                    NonZeroU32::new(self.width).unwrap(),
                    NonZeroU32::new(self.height).unwrap(),
                )
                .expect("Failed to resize surface");
        }

        draw_frame(&mut surface, self.width, self.height);

        self.window = Some(window_rc);
        self.context = Some(context);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(surface) = self.surface.as_mut() {
                    if let (Some(w), Some(h)) = (
                        NonZeroU32::new(new_size.width),
                        NonZeroU32::new(new_size.height),
                    ) {
                        let _ = surface.resize(w, h);
                        draw_frame(surface, new_size.width, new_size.height);
                    }
                }
                self.width = new_size.width;
                self.height = new_size.height;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(x11_window_id) = self.x11_window_id {
            unsafe {
                let xlib = Xlib::open().expect("Failed to open Xlib");
                let display = (xlib.XOpenDisplay)(std::ptr::null());
                if !display.is_null() {
                    (xlib.XLowerWindow)(display, x11_window_id);
                    (xlib.XFlush)(display);
                    (xlib.XCloseDisplay)(display);
                }
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}

fn draw_frame(surface: &mut softbuffer::Surface<Rc<Window>, Rc<Window>>, width: u32, height: u32) {
    if width == 0 || height == 0 {
        return;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(_) => return,
    };

    for y in 0..height {
        for x in 0..width {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = 128u8;
            let b = ((y as f32 / height as f32) * 255.0) as u8;
            let color = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
            let index = (y * width + x) as usize;
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }

    let _ = buffer.present();
}
