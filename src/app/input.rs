use bitvec::vec::BitVec;
use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use winit::{
    event::{DeviceEvent, DeviceId},
    keyboard::PhysicalKey, window::WindowId,
};

#[derive(Debug, Default)]
pub struct Input {
    input_map: Mutex<InputMap>,
    player_actions: Mutex<PlayerActions>,
    window_input: Mutex<WindowInput>,
}

#[derive(Debug, Default, Clone)]
pub struct WindowInput {
    pub close: BitVec,
}

#[derive(Debug, Default, Clone)]
pub struct InputMap {
    pub key_map: FxHashMap<PhysicalKey, usize>,
    pub button_map: FxHashMap<u32, usize>,
    pub axis_map: FxHashMap<u32, usize>,
    pub device_map: FxHashMap<DeviceId, usize>,
    pub window_map: FxHashMap<WindowId, usize>
}

#[derive(Debug, Default, Clone)]
pub struct PlayerActions {
    pub actions: Vec<Actions>,
}

#[derive(Debug, Default, Clone)]
pub struct Actions {
    pub digital: BitVec,
    pub analog: Vec<f64>,
}

impl Actions {
    pub fn set_digital(&mut self, action: usize, pressed: bool) {
        self.digital.set(action, pressed);
    }
    pub fn set_analog(&mut self, action: usize, value: f64) {
        self.analog[action] = value;
    }
    pub fn get_digital(&self, action: usize) -> bool {
        self.digital[action]
    }
    pub fn get_analog(&self, action: usize) -> f64 {
        self.analog[action]
    }
}

impl Input {
    pub fn get_input(&self) -> PlayerActions {
        self.player_actions.lock().clone()
    }
    pub fn set_input_map(&self, input_map: InputMap) {
        *self.input_map.lock() = input_map;
    }
    pub fn process_device_event(&self, device_id: &DeviceId, event: DeviceEvent) {
        let input_map = self.input_map.lock();
        let Some(&player) = input_map.device_map.get(device_id) else {
            return;
        };
        let mut player_actions = self.player_actions.lock();
        let actions = &mut player_actions.actions[player];
        match event {
            DeviceEvent::Motion { axis, value } => {
                let Some(&action) = input_map.axis_map.get(&axis) else {
                    return;
                };
                actions.analog[action] = value;
            }
            DeviceEvent::Button { button, state } => {
                let Some(&action) = input_map.button_map.get(&button) else {
                    return;
                };
                actions.digital.set(action, state.is_pressed());
            }
            DeviceEvent::Key(raw_key_event) => {
                let Some(&action) = input_map.key_map.get(&raw_key_event.physical_key) else {
                    return;
                };
                actions
                    .digital
                    .set(action, raw_key_event.state.is_pressed());
            }
            _ => {}
        }
    }
}