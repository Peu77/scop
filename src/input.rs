use glfw::{Action, Key, MouseButton, Window, WindowEvent};

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseDelta {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Movement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default)]
pub struct Input {
    texture_toggle_requested: bool,
    shader_toggle_requested: bool,
    mode_toggle_requested: bool,
    reset_requested: bool,
    dragging: bool,
    cursor_position: Option<(f64, f64)>,
    mouse_delta: MouseDelta,
    scroll_delta: f32,
}

impl Input {
    pub fn handle_event(&mut self, window: &mut Window, event: WindowEvent) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true);
            }
            WindowEvent::Key(Key::T, _, Action::Press, _) => {
                self.texture_toggle_requested = true;
            }
            WindowEvent::Key(Key::P, _, Action::Press, _) => {
                self.shader_toggle_requested = true;
            }
            WindowEvent::Key(Key::F, _, Action::Press, _) => {
                self.mode_toggle_requested = true;
            }
            WindowEvent::Key(Key::R, _, Action::Press, _) => {
                self.reset_requested = true;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                self.dragging = true;
                self.cursor_position = None;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.dragging = false;
                self.cursor_position = None;
            }
            WindowEvent::CursorPos(x, y) if self.dragging => {
                if let Some((previous_x, previous_y)) = self.cursor_position {
                    self.mouse_delta.x += (x - previous_x) as f32;
                    self.mouse_delta.y += (y - previous_y) as f32;
                }
                self.cursor_position = Some((x, y));
            }
            WindowEvent::Scroll(_, y) => {
                self.scroll_delta += y as f32;
            }
            _ => {}
        }
    }

    pub fn take_texture_toggle(&mut self) -> bool {
        std::mem::take(&mut self.texture_toggle_requested)
    }

    pub fn take_shader_toggle(&mut self) -> bool {
        std::mem::take(&mut self.shader_toggle_requested)
    }

    pub fn take_mode_toggle(&mut self) -> bool {
        std::mem::take(&mut self.mode_toggle_requested)
    }

    pub fn take_reset(&mut self) -> bool {
        std::mem::take(&mut self.reset_requested)
    }

    pub fn take_mouse_delta(&mut self) -> MouseDelta {
        std::mem::take(&mut self.mouse_delta)
    }

    pub fn take_scroll_delta(&mut self) -> f32 {
        std::mem::take(&mut self.scroll_delta)
    }

    pub fn movement(&self, window: &Window) -> Movement {
        Movement {
            x: axis(window, Key::D, Key::Right, Key::A, Key::Left),
            y: axis(window, Key::W, Key::Up, Key::S, Key::Down),
            z: axis(window, Key::Q, Key::PageUp, Key::E, Key::PageDown),
        }
    }
}

fn axis(
    window: &Window,
    negative: Key,
    negative_alt: Key,
    positive: Key,
    positive_alt: Key,
) -> f32 {
    let negative = pressed(window, negative) || pressed(window, negative_alt);
    let positive = pressed(window, positive) || pressed(window, positive_alt);
    positive as i8 as f32 - negative as i8 as f32
}

fn pressed(window: &Window, key: Key) -> bool {
    matches!(window.get_key(key), Action::Press | Action::Repeat)
}
