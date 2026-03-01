use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

mod draw;
mod window;

use window::{ScreenDimensions, keep_window_lowered, setup_x11_window};

struct RenderState {
    last_fps_check: Instant,
    redraw_count: u32,
    start_time: Instant,
    total_frames: u64,
}

impl RenderState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_fps_check: now,
            redraw_count: 0,
            start_time: now,
            total_frames: 0,
        }
    }
}

struct App {
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    x11_window_id: Option<x11_dl::xlib::Window>,
    width: u32,
    height: u32,
    state: RenderState,
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
            state: RenderState::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Desktop Background")
                .with_transparent(true)
                .with_decorations(false)
                .with_resizable(false)
                .with_visible(false);

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            let window_rc = Rc::new(window);
            let ScreenDimensions { width, height } =
                setup_x11_window(&window_rc, &mut self.x11_window_id);
            self.width = width;
            self.height = height;

            let _ = window_rc.request_inner_size(PhysicalSize::new(width, height));
            window_rc.set_visible(true);

            let context = softbuffer::Context::new(window_rc.clone())
                .expect("Failed to create softbuffer context");
            let surface = softbuffer::Surface::new(&context, window_rc.clone())
                .expect("Failed to create softbuffer surface");

            self.window = Some(window_rc);
            self.context = Some(context);
            self.surface = Some(surface);
        }

        if let Some(surface) = self.surface.as_mut() {
            draw::resize_surface(surface, self.width, self.height);
        }
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
                    draw::resize_surface(surface, new_size.width, new_size.height);
                    draw::draw_gradient(surface, new_size.width, new_size.height);
                }
                self.width = new_size.width;
                self.height = new_size.height;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(x11_window_id) = self.x11_window_id {
            keep_window_lowered(x11_window_id);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
