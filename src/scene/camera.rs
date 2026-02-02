use instant::Duration;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye_pos: glm::Vec3,
    pub eye_dir: glm::Vec3,
    pub up: glm::Vec3,
    pub vfov: f32,
    /// Aperture must be between 0..=1.
    pub aperture: f32,
    /// Focus distance must be a positive number.
    pub focus_distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraController {
    updated: bool,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            updated: false,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn clear(&mut self) {
        self.updated = false;
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        self.scroll = 0.0;
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        let s = match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        };
        self.updated = s;
        s
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn handle_input(&mut self, event: &WindowEvent, mouse_pressed: &mut bool) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                self.process_keyboard(*key, *state);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                *mouse_pressed = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Right,
                ..
            } => {
                *mouse_pressed = false;
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, device_event: &DeviceEvent, mouse_pressed: bool) {
        match device_event {
            DeviceEvent::MouseMotion { delta } => {
                if mouse_pressed {
                    self.process_mouse(delta.0, delta.1);
                }
            }
            // DeviceEvent::MouseWheel { delta } => {
            //     // TODO: Not behaving as expected
            //     self.process_scroll(delta);
            // }
            _ => {}
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let forward = self.amount_forward - self.amount_backward;
        let right = self.amount_right - self.amount_left;
        let up = self.amount_up - self.amount_down;
        let rotate_horizontal = self.rotate_horizontal;
        let rotate_vertical = self.rotate_vertical;
        let scroll = self.scroll;

        let dt = dt.as_secs_f32();
        let speed = self.speed;
        let sensitivity = self.sensitivity;

        let forward = forward * speed * dt;
        let right = right * speed * dt;
        let up = up * speed * dt;
        let rotate_horizontal = rotate_horizontal * sensitivity * dt;
        let rotate_vertical = rotate_vertical * sensitivity * dt;
        let scroll = scroll * speed * dt;

        let forward = camera.eye_dir * forward;
        let right = glm::cross(&camera.eye_dir, &camera.up) * right;
        let up = camera.up * up;

        camera.eye_pos += forward + right + up;
        camera.eye_dir = glm::rotate_vec3(&camera.eye_dir, rotate_horizontal, &camera.up);
        camera.eye_dir = glm::rotate_vec3(
            &camera.eye_dir,
            rotate_vertical,
            &glm::cross(&camera.eye_dir, &camera.up),
        );
        camera.eye_dir = glm::normalize(&camera.eye_dir);

        camera.focus_distance -= scroll;
        camera.focus_distance = camera.focus_distance.max(0.1);
        self.clear();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCamera {
    eye: glm::Vec3,
    _padding1: f32,
    horizontal: glm::Vec3,
    _padding2: f32,
    vertical: glm::Vec3,
    _padding3: f32,
    u: glm::Vec3,
    _padding4: f32,
    v: glm::Vec3,
    lens_radius: f32,
    lower_left_corner: glm::Vec3,
    _padding5: f32,
}

impl GpuCamera {
    pub fn new(camera: &Camera, viewport_size: (u32, u32)) -> Self {
        let lens_radius = 0.5_f32 * camera.aperture;
        let aspect = viewport_size.0 as f32 / viewport_size.1 as f32;
        let theta = camera.vfov.to_radians();
        let half_height = camera.focus_distance * (0.5_f32 * theta).tan();
        let half_width = aspect * half_height;

        let w = glm::normalize(&camera.eye_dir);
        let v = glm::normalize(&camera.up);
        let u = glm::cross(&w, &v);

        let lower_left_corner =
            camera.eye_pos + camera.focus_distance * w - half_width * u - half_height * v;
        let horizontal = 2_f32 * half_width * u;
        let vertical = 2_f32 * half_height * v;

        Self {
            eye: camera.eye_pos,
            _padding1: 0_f32,
            horizontal,
            _padding2: 0_f32,
            vertical,
            _padding3: 0_f32,
            u,
            _padding4: 0_f32,
            v,
            lens_radius,
            lower_left_corner,
            _padding5: 0_f32,
        }
    }
}
