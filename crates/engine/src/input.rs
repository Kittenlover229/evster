use hashbrown::{HashMap, HashSet};
use nalgebra_glm::{vec2, Vec2};
use winit::event::{KeyboardInput, VirtualKeyCode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyState {
    #[default]
    Idle,
    Pressed,
    Held,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxialInput2D {
    pub normalize: bool,
    pub up: VirtualKeyCode,
    pub down: VirtualKeyCode,
    pub right: VirtualKeyCode,
    pub left: VirtualKeyCode,
}

#[derive(Clone, Debug)]
pub struct InputHandler {
    keys: HashMap<VirtualKeyCode, KeyState>,
    filter: HashSet<VirtualKeyCode>,
    axial_inputs: Vec<AxialInput2D>,
}

impl InputHandler {
    pub fn new_with_filter(
        filters: impl Iterator<Item = VirtualKeyCode>,
        axial_inputs: impl Iterator<Item = AxialInput2D>,
    ) -> Self {
        let mut filter: Vec<VirtualKeyCode> = filters.collect();
        let axial_inputs: Vec<AxialInput2D> = axial_inputs.collect();
        for axial_input in &axial_inputs {
            filter.extend(
                [
                    axial_input.up,
                    axial_input.down,
                    axial_input.right,
                    axial_input.left,
                ]
                .iter(),
            )
        }

        let keys = {
            let mut hs = HashMap::new();
            hs.reserve(filter.len());
            hs
        };

        Self {
            keys,
            filter: HashSet::from_iter(filter),
            axial_inputs,
        }
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

    pub fn get_axial(&self, idx: u32) -> Vec2 {
        let mut vec = vec2(0., 0.);
        let axial_input = self
            .axial_inputs
            .get(idx as usize)
            .expect("No axial input at requested index");

        vec += vec2(0., 1.) * self.is_active(axial_input.up) as u8 as f32;
        vec += vec2(0., -1.) * self.is_active(axial_input.down) as u8 as f32;
        vec += vec2(1., 0.) * self.is_active(axial_input.right) as u8 as f32;
        vec += vec2(-1., 0.) * self.is_active(axial_input.left) as u8 as f32;

        if vec != Vec2::zeros() && axial_input.normalize {
            vec = vec.normalize();
        }

        vec
    }

    pub fn is_active(&self, key: VirtualKeyCode) -> bool {
        self.keys
            .get(&key)
            .map(|x| matches!(x, KeyState::Pressed | KeyState::Held))
            .unwrap_or(false)
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
