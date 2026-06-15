use std::time::Instant;

use std::path::Path;

use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowEvent, WindowHint};

use crate::error::{AppError, Result};
use crate::input::{Input, MouseDelta, Movement};
use crate::math::{Mat4, Vec3};
use crate::mesh::Mesh;
use crate::ppm;
use crate::renderer::Renderer;

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const MOVEMENT_SPEED: f32 = 1.8;
const ROTATION_SPEED: f32 = 0.65;
const BLEND_SPEED: f32 = 1.6;
const MOUSE_SENSITIVITY: f32 = 0.008;
const ZOOM_SENSITIVITY: f32 = 0.4;
const MIN_CAMERA_DISTANCE: f32 = 0.25;
const MAX_CAMERA_DISTANCE: f32 = 12.0;
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.05;
const TEXTURE_PATH: &str = "assets/texture.ppm";

type EventReceiver = GlfwReceiver<(f64, WindowEvent)>;
type WindowContext = (Glfw, PWindow, EventReceiver);

pub fn run(mesh: Mesh) -> Result<()> {
    let (mut glfw, mut window, events) = create_window(mesh.triangle_count())?;
    let renderer = create_renderer(&mesh)?;
    let mut input = Input::default();
    let mut state = State::default();
    let mut previous_frame = Instant::now();

    while !window.should_close() {
        let delta_time = frame_delta(&mut previous_frame);
        poll_input(&mut glfw, &mut window, &events, &mut input);
        state.update(&mut input, &window, delta_time);
        draw_frame(&renderer, &mut window, &state);
    }

    Ok(())
}

fn create_window(triangle_count: usize) -> Result<WindowContext> {
    let mut glfw =
        glfw::init(glfw::fail_on_errors).map_err(|error| AppError::Glfw(error.to_string()))?;
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            &format!("scop - {triangle_count} triangles"),
            glfw::WindowMode::Windowed,
        )
        .ok_or_else(|| AppError::Glfw("could not create an OpenGL window".into()))?;

    window.make_current();
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_framebuffer_size_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    gl::load_with(|symbol| window.get_proc_address(symbol));

    Ok((glfw, window, events))
}

fn create_renderer(mesh: &Mesh) -> Result<Renderer> {
    let material_textures = mesh
        .textures
        .iter()
        .map(|path| ppm::load(path))
        .collect::<Result<Vec<_>>>()?;
    let normal_maps = mesh
        .normal_maps
        .iter()
        .map(|path| ppm::load(path))
        .collect::<Result<Vec<_>>>()?;
    let fallback_texture = ppm::load(Path::new(TEXTURE_PATH))?;
    Renderer::new(mesh, &material_textures, &normal_maps, &fallback_texture)
}

fn frame_delta(previous_frame: &mut Instant) -> f32 {
    let now = Instant::now();
    let delta_time = (now - *previous_frame).as_secs_f32().min(0.1);
    *previous_frame = now;
    delta_time
}

fn poll_input(glfw: &mut Glfw, window: &mut PWindow, events: &EventReceiver, input: &mut Input) {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(events) {
        input.handle_event(window, event);
    }
}

fn draw_frame(renderer: &Renderer, window: &mut PWindow, state: &State) {
    let framebuffer_size = window.get_framebuffer_size();
    if framebuffer_size.0 <= 0 || framebuffer_size.1 <= 0 {
        return;
    }

    let aspect = framebuffer_size.0 as f32 / framebuffer_size.1 as f32;
    let projection = Mat4::perspective(55_f32.to_radians(), aspect, 0.1, 100.0);
    let view = Mat4::translation(Vec3::new(0.0, 0.0, -state.camera_distance));
    let model = Mat4::translation(state.position)
        * Mat4::rotation_x(state.pitch)
        * Mat4::rotation_y(state.yaw);

    renderer.draw(
        &(projection * view * model),
        &model,
        state.texture_blend,
        state.effect_enabled,
        framebuffer_size,
    );
    window.swap_buffers();
}

#[derive(Debug)]
struct State {
    automatic_rotation: bool,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    camera_distance: f32,
    texture_target: f32,
    texture_blend: f32,
    effect_enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            automatic_rotation: true,
            position: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            camera_distance: 4.0,
            texture_target: 0.0,
            texture_blend: 0.0,
            effect_enabled: false,
        }
    }
}

impl State {
    fn update(&mut self, input: &mut Input, window: &glfw::Window, delta_time: f32) {
        if input.take_texture_toggle() {
            self.texture_target = if self.texture_target < 0.5 { 1.0 } else { 0.0 };
        }
        if input.take_shader_toggle() {
            self.effect_enabled = !self.effect_enabled;
        }
        if input.take_mode_toggle() {
            self.automatic_rotation = !self.automatic_rotation;
            if !self.automatic_rotation {
                self.position = Vec3::new(0.0, 0.0, 0.0);
            }
        }
        if input.take_reset() {
            self.reset_view();
        }

        let mouse_delta = input.take_mouse_delta();
        let scroll_delta = input.take_scroll_delta();
        if self.automatic_rotation {
            self.apply_movement(input.movement(window), MOVEMENT_SPEED * delta_time);
            self.yaw = (self.yaw + ROTATION_SPEED * delta_time) % std::f32::consts::TAU;
        } else {
            self.rotate(mouse_delta);
        }
        self.zoom(scroll_delta);
        self.texture_blend = move_toward(
            self.texture_blend,
            self.texture_target,
            BLEND_SPEED * delta_time,
        );
    }

    fn rotate(&mut self, delta: MouseDelta) {
        self.yaw = (self.yaw + delta.x * MOUSE_SENSITIVITY) % std::f32::consts::TAU;
        self.pitch = (self.pitch + delta.y * MOUSE_SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH);
    }

    fn apply_movement(&mut self, movement: Movement, distance: f32) {
        self.position += Vec3::new(movement.x, movement.y, movement.z) * distance;
    }

    fn zoom(&mut self, scroll_delta: f32) {
        self.camera_distance = (self.camera_distance - scroll_delta * ZOOM_SENSITIVITY)
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    }

    fn reset_view(&mut self) {
        self.position = Vec3::new(0.0, 0.0, 0.0);
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.camera_distance = 4.0;
    }
}

fn move_toward(current: f32, target: f32, maximum_delta: f32) -> f32 {
    if (target - current).abs() <= maximum_delta {
        target
    } else {
        current + (target - current).signum() * maximum_delta
    }
}

#[cfg(test)]
mod tests {
    use crate::input::{MouseDelta, Movement};
    use crate::math::Vec3;

    use super::{move_toward, State, MAX_CAMERA_DISTANCE, MAX_PITCH, MIN_CAMERA_DISTANCE};

    #[test]
    fn blend_motion_does_not_overshoot() {
        assert_eq!(move_toward(0.9, 1.0, 0.2), 1.0);
        assert_eq!(move_toward(0.1, 0.0, 0.2), 0.0);
        assert_eq!(move_toward(0.0, 1.0, 0.2), 0.2);
    }

    #[test]
    fn mouse_rotation_clamps_vertical_angle() {
        let mut state = State::default();

        state.rotate(MouseDelta {
            x: 0.0,
            y: -10_000.0,
        });

        assert_eq!(state.pitch, -MAX_PITCH);
    }

    #[test]
    fn mouse_drag_right_increases_yaw() {
        let mut state = State::default();

        state.rotate(MouseDelta { x: 10.0, y: 0.0 });

        assert!(state.yaw > 0.0);
    }

    #[test]
    fn movement_changes_position_in_free_floating_mode() {
        let mut state = State::default();

        state.apply_movement(
            Movement {
                x: 1.0,
                y: -1.0,
                z: 1.0,
            },
            2.0,
        );

        assert_eq!(state.position, Vec3::new(2.0, -2.0, 2.0));
    }

    #[test]
    fn zoom_clamps_camera_distance() {
        let mut state = State::default();

        state.zoom(100.0);
        assert_eq!(state.camera_distance, MIN_CAMERA_DISTANCE);
        state.zoom(-100.0);
        assert_eq!(state.camera_distance, MAX_CAMERA_DISTANCE);
    }
}
