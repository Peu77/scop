use std::time::Instant;

use std::path::Path;

use glfw::{Context, OpenGlProfileHint, WindowHint};

use crate::error::{AppError, Result};
use crate::input::{Keyboard, Movement};
use crate::math::{Mat4, Vec3};
use crate::mesh::Mesh;
use crate::ppm;
use crate::renderer::Renderer;

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const MOVEMENT_SPEED: f32 = 1.8;
const ROTATION_SPEED: f32 = 0.65;
const BLEND_SPEED: f32 = 1.6;
const TEXTURE_PATH: &str = "assets/texture.ppm";

pub fn run(mesh: Mesh) -> Result<()> {
    let triangle_count = mesh.triangle_count();
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
    window.set_framebuffer_size_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    gl::load_with(|symbol| window.get_proc_address(symbol));

    let texture = ppm::load(Path::new(TEXTURE_PATH))?;
    let renderer = Renderer::new(&mesh, &texture)?;
    let mut keyboard = Keyboard::default();
    let mut state = State::default();
    let mut previous_frame = Instant::now();

    while !window.should_close() {
        let now = Instant::now();
        let delta_time = (now - previous_frame).as_secs_f32().min(0.1);
        previous_frame = now;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            keyboard.handle_event(&mut window, event);
        }

        state.update(&mut keyboard, &window, delta_time);
        let framebuffer_size = window.get_framebuffer_size();
        if framebuffer_size.0 > 0 && framebuffer_size.1 > 0 {
            let aspect = framebuffer_size.0 as f32 / framebuffer_size.1 as f32;
            let projection = Mat4::perspective(55_f32.to_radians(), aspect, 0.1, 100.0);
            let view = Mat4::translation(Vec3::new(0.0, 0.0, -4.0));
            let model = Mat4::translation(state.position) * Mat4::rotation_y(state.rotation_angle);
            renderer.draw(
                &(projection * view * model),
                state.texture_blend,
                framebuffer_size,
            );
            window.swap_buffers();
        }
    }

    Ok(())
}

#[derive(Debug)]
struct State {
    position: Vec3,
    rotation_angle: f32,
    texture_target: f32,
    texture_blend: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation_angle: 0.0,
            texture_target: 0.0,
            texture_blend: 0.0,
        }
    }
}

impl State {
    fn update(&mut self, keyboard: &mut Keyboard, window: &glfw::Window, delta_time: f32) {
        if keyboard.take_texture_toggle() {
            self.texture_target = if self.texture_target < 0.5 { 1.0 } else { 0.0 };
        }
        if keyboard.take_reset() {
            self.position = Vec3::ZERO;
        }

        self.apply_movement(keyboard.movement(window), MOVEMENT_SPEED * delta_time);
        self.rotation_angle =
            (self.rotation_angle + ROTATION_SPEED * delta_time) % std::f32::consts::TAU;
        self.texture_blend = move_toward(
            self.texture_blend,
            self.texture_target,
            BLEND_SPEED * delta_time,
        );
    }

    fn apply_movement(&mut self, movement: Movement, distance: f32) {
        self.position += Vec3::new(movement.x, movement.y, movement.z) * distance;
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
    use super::move_toward;

    #[test]
    fn blend_motion_does_not_overshoot() {
        assert_eq!(move_toward(0.9, 1.0, 0.2), 1.0);
        assert_eq!(move_toward(0.1, 0.0, 0.2), 0.0);
        assert_eq!(move_toward(0.0, 1.0, 0.2), 0.2);
    }
}
