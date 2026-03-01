use std::rc::Rc;

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

struct App {
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    x11_window_id: Option<x11_dl::xlib::Window>,
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
        let ScreenDimensions { width, height } =
            setup_x11_window(&window_rc, &mut self.x11_window_id);
        self.width = width;
        self.height = height;

        let _ = window_rc.request_inner_size(PhysicalSize::new(width, height));
        window_rc.set_visible(true);

        let context = softbuffer::Context::new(window_rc.clone())
            .expect("Failed to create softbuffer context");
        let mut surface = softbuffer::Surface::new(&context, window_rc.clone())
            .expect("Failed to create softbuffer surface");

        draw::resize_surface(&mut surface, self.width, self.height);
        draw::draw_gradient(&mut surface, self.width, self.height);

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
