use raylib::{ffi::KeyboardKey, RaylibHandle};

pub struct Environment {
    scroll: ScrollState,
    scale: ScaleState,
}
struct ScrollState {
    pub scroll: f32,
    pub scroll_speed: f32,
}
struct ScaleState {
    pub scale: f32,
    pub scale_speed: f32,
    pub min_scale: f32,
}
impl Environment {
    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            self.scale.scale = (self.scale.scale
                + rl.get_mouse_wheel_move() * self.scale.scale_speed * self.scale.scale)
                .max(self.scale.min_scale);
        } else {
            self.scroll.scroll += rl.get_mouse_wheel_move() * self.scroll.scroll_speed;
        }
    }

    pub fn get_scroll(&self) -> f32 {
        self.scroll.scroll
    }

    pub fn get_scale(&self) -> f32 {
        self.scale.scale
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            scroll: ScrollState {
                scroll: 0.0,
                scroll_speed: 50.,
            },
            scale: ScaleState {
                scale: 1.,
                scale_speed: 0.1,
                min_scale: 0.5,
            },
        }
    }
}
