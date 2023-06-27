use hashbrown::HashMap;
use winit::event::{KeyboardInput, VirtualKeyCode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyState {
    #[default]
    Idle,
    Pressed,
    Held,
    Released,
}

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<VirtualKeyCode, KeyState>,
    filter: Vec<VirtualKeyCode>,
}

impl InputHandler {
    pub fn new_with_filter(filters: impl Iterator<Item = VirtualKeyCode>) -> Self {
        let filter: Vec<VirtualKeyCode> = filters.collect();
        let keys = {
            let mut hs = HashMap::new();
            hs.reserve(filter.len());
            hs
        };

        Self { keys, filter }
    }

    pub fn handle_input(&mut self, input: &KeyboardInput) {
        let keycode = match &input.virtual_keycode {
            Some(keycode) => keycode,
            None => return,
        };

        if !self.filter.contains(keycode) {
            return;
        }

        match (input.state, self.keys.get_mut(keycode)) {
            (winit::event::ElementState::Pressed, None) => {
                self.keys.insert(*keycode, KeyState::Pressed);
            }

            (winit::event::ElementState::Pressed, Some(key)) => {
                *key = KeyState::Pressed;
            }
            (winit::event::ElementState::Released, None) => {
                self.keys.insert(*keycode, KeyState::Idle);
            }
            (winit::event::ElementState::Released, Some(key)) => {
                if let KeyState::Held = *key {
                    *key = KeyState::Released;
                };
            }
        }
    }

    // Move all the keys into their next phase
    // Idle -> Idle
    // Pressed -> Held
    // Held -> Held
    // Released -> Idle
    pub fn flush(&mut self) {
        for state in self.keys.values_mut() {
            *state = match *state {
                KeyState::Pressed => KeyState::Held,
                KeyState::Released => KeyState::Idle,
                _ => *state,
            }
        }
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys
            .get(&key)
            .map(|x| matches!(x, KeyState::Pressed))
            .unwrap_or(false)
    }

    pub fn is_held(&self, key: VirtualKeyCode) -> bool {
        self.keys
            .get(&key)
            .map(|x| matches!(x, KeyState::Held))
            .unwrap_or(false)
    }

    pub fn is_released(&self, key: VirtualKeyCode) -> bool {
        self.keys
            .get(&key)
            .map(|x| matches!(x, KeyState::Released))
            .unwrap_or(false)
    }
}
