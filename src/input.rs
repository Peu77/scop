use glfw::{Action, Key, Window, WindowEvent};

#[derive(Clone, Copy, Debug, Default)]
pub struct Movement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default)]
pub struct Keyboard {
    texture_toggle_requested: bool,
    reset_requested: bool,
}

impl Keyboard {
    pub fn handle_event(&mut self, window: &mut Window, event: WindowEvent) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true);
            }
            WindowEvent::Key(Key::T, _, Action::Press, _) => {
                self.texture_toggle_requested = true;
            }
            WindowEvent::Key(Key::R, _, Action::Press, _) => {
                self.reset_requested = true;
            }
            _ => {}
        }
    }

    pub fn movement(&self, window: &Window) -> Movement {
        Movement {
            x: axis(window, Key::A, Key::Left, Key::D, Key::Right),
            y: axis(window, Key::S, Key::Down, Key::W, Key::Up),
            z: axis(window, Key::E, Key::PageDown, Key::Q, Key::PageUp),
        }
    }

    pub fn take_texture_toggle(&mut self) -> bool {
        std::mem::take(&mut self.texture_toggle_requested)
    }

    pub fn take_reset(&mut self) -> bool {
        std::mem::take(&mut self.reset_requested)
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
