#![feature(iter_intersperse)]

mod contributions;
mod draw;
mod window;

use std::{
    num::NonZeroU32,
    rc::Rc,
    sync::mpsc::{Receiver, channel},
    thread,
    time::{Duration, Instant},
};

use time::Weekday;
use window::{ScreenDimensions, keep_window_lowered, setup_x11_window};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::contributions::{ContributionGrid, load_contribution_grid};

const MAX_FPS: u64 = 1;
pub const OPACITY_PERCENT: f32 = 50.0;

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
    contribution_grid: ContributionGrid,
    current_day: Weekday,
    graph_receiver: Option<Receiver<ContributionGrid>>,
}

impl App {
    fn new() -> Self {
        let today = time::OffsetDateTime::now_local().unwrap().weekday();

        Self {
            window: None,
            context: None,
            surface: None,
            x11_window_id: None,
            width: 0,
            height: 0,
            state: RenderState::new(),
            contribution_grid: load_contribution_grid(false),
            current_day: today,
            graph_receiver: None,
        }
    }

    fn check_and_reload_if_new_day(&mut self) {
        let today = time::OffsetDateTime::now_local().unwrap().weekday();

        if today != self.current_day {
            self.current_day = today;

            let (sender, receiver) = channel::<ContributionGrid>();
            self.graph_receiver = Some(receiver);

            thread::spawn(move || {
                let grid = contributions::load_contribution_grid(true);
                let _ = sender.send(grid);
            });
        }

        // try receive and update new grid
        if let Some(receiver) = &self.graph_receiver
            && let Ok(new_grid) = receiver.try_recv()
        {
            self.contribution_grid = new_grid;
            self.graph_receiver = None;
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
        self.check_and_reload_if_new_day();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(surface) = self.surface.as_mut()
                    && let Some(width) = NonZeroU32::new(new_size.width)
                    && let Some(height) = NonZeroU32::new(new_size.height)
                {
                    let _ = surface.resize(width, height);
                    self.width = new_size.width;
                    self.height = new_size.height;
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(surface) = self.surface.as_mut() {
                    self.state.redraw_count += 1;

                    let now = Instant::now();
                    if now.duration_since(self.state.last_fps_check).as_secs() >= 1 {
                        println!("FPS: {}", self.state.redraw_count);
                        self.state.redraw_count = 0;
                        self.state.last_fps_check = now;
                    }

                    if let Ok(mut buffer) = surface.buffer_mut()
                        && self.width > 0
                        && self.height > 0
                    {
                        buffer.fill(0x0000_0000);

                        draw::graph::draw_contribution_graph(
                            buffer.as_mut(),
                            self.width,
                            self.height,
                            &self.contribution_grid,
                        );

                        let font_scale = if self.height > 1080 { 3 } else { 2 };

                        draw::text::draw_text(
                            buffer.as_mut(),
                            self.width,
                            self.height,
                            "the     quick brown fox jumps over the lazy dog",
                            font_scale,
                        );

                        let _ = buffer.present();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(x11_window_id) = self.x11_window_id {
            keep_window_lowered(x11_window_id);
        }

        let now = Instant::now();
        let frame_duration_nanos = Duration::from_secs(1).as_nanos() as u64 / MAX_FPS;

        let elapsed_nanos = now.duration_since(self.state.start_time).as_nanos();
        let expected_frame = (elapsed_nanos / frame_duration_nanos as u128) as u64;

        // If we are more than 5 frames behind, reset start_time to avoid aggressive fast-forwarding
        if expected_frame > self.state.total_frames + 5 {
            self.state.start_time =
                now - Duration::from_nanos(self.state.total_frames * frame_duration_nanos);
        }

        if expected_frame >= self.state.total_frames {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            self.state.total_frames = expected_frame + 1;
        }

        let next_target = self.state.start_time
            + Duration::from_nanos(self.state.total_frames * frame_duration_nanos);
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_target));
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
