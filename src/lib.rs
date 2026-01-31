use log::info;
use scene::Scene;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

mod render_context;
use render_context::RenderContext;

mod utils;

mod scene;
extern crate nalgebra_glm as glm;

mod object;
struct MyUserEvent;

struct State<'a> {
    window: &'a Window,
    render_context: RenderContext<'a>,
    last_time: instant::Instant,
    mouse_pressed: bool,
    surface_configured: bool,
    counter: i32,
}

impl ApplicationHandler<MyUserEvent> for State<'_> {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _user_eventt: MyUserEvent) {
        // Handle user event.
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Your application got resumed.
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.render_context
            .window_event(&event, &mut self.mouse_pressed);
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();
                if !self.surface_configured {
                    log::info!("Surface not configured yet");
                    return;
                }
                let now = instant::Instant::now();
                let dt = now - self.last_time;
                self.last_time = now;

                self.render_context.fps = 1.0 / dt.as_secs_f64();

                self.render_context.update(dt);
                match self.render_context.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.render_context.resize(self.render_context.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory");
                        event_loop.exit();
                    }
                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.surface_configured = true;
                self.render_context.resize(physical_size);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.render_context.device_event(&event, self.mouse_pressed);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.request_redraw();
        self.counter += 1;
    }
}

fn init(
    width: u32,
    height: u32,
) -> (
    winit::window::Window,
    winit::event_loop::EventLoop<MyUserEvent>,
) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    log::info!("Starting up");

    let event_loop = EventLoop::<MyUserEvent>::with_user_event().build().unwrap();
    #[allow(unused_mut)]
    let mut attributes =
        WindowAttributes::default().with_inner_size(winit::dpi::PhysicalSize::new(width, height));

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;
        let canvas = wgpu::web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("pathracer-canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        attributes = attributes.with_canvas(Some(canvas));
    }

    #[allow(deprecated)]
    let window = event_loop.create_window(attributes).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // center the window
        let monitor = window.current_monitor();
        let monitor = monitor.unwrap();
        let monitor_size = monitor.size();
        let window_size = window.inner_size();
        let x = (monitor_size.width - window_size.width) / 2;
        let y = (monitor_size.height - window_size.height) / 2;
        window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    return (window, event_loop);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    info!("Starting up");
    let scale = 2.2;
    let width = 500 * scale as u32;
    let height = 450 * scale as u32;
    let (window, event_loop) = init(width, height);

    let mut state = State {
        window: &window,
        mouse_pressed: false,
        surface_configured: true,
        last_time: instant::Instant::now(),
        render_context: RenderContext::new(
            &window,
            &Scene::cornell_scene(
                scene::RenderParam {
                    samples_per_pixel: 1,
                    max_depth: 30,
                    samples_max_per_pixel: 1000,
                    total_samples: 0,
                    clear_samples: 0,
                },
                scene::FrameData {
                    width,
                    height,
                    index: 0,
                },
            ),
        )
        .await,
        counter: 0,
    };

    let _ = event_loop.run_app(&mut state);
}
