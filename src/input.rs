use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;
use std::collections::HashMap;

const KEYS_NUM: usize = 16;

pub struct InputManager {
    event_pump: EventPump,
    key_state: [bool; KEYS_NUM],
    scancode_to_hex_map: HashMap<Scancode, u8>,
    hex_to_scancode_map: HashMap<u8, Scancode>,
}

// TODO Major refactor related to class API and main loop.

// TODO Handle exit. QUIT/Esc
impl InputManager {
    /// Creates a new InputManager instance, initializing SDL2 and setting up the event pump.
    /// Panics if SDL2 initialization or event pump creation fails.
    pub fn new() -> Self {
        let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
        let event_pump = sdl_context
            .event_pump()
            .expect("Failed to get SDL2 event pump");

        let (scancode_to_hex_map, hex_to_scancode_map) = Self::create_key_mappings();

        InputManager {
            event_pump,
            key_state: [false; KEYS_NUM],
            scancode_to_hex_map,
            hex_to_scancode_map,
        }
    }

    /// Creates the mappings between scancodes and hex keys.
    ///
    /// # Returns
    /// * `(HashMap<Scancode, u8>, HashMap<u8, Scancode>)` - The scancode to hex and hex to scancode mappings.
    fn create_key_mappings() -> (HashMap<Scancode, u8>, HashMap<u8, Scancode>) {
        let mut scancode_to_hex_map = HashMap::new();
        let mut hex_to_scancode_map = HashMap::new();

        let mappings = [
            (Scancode::Num1, 0x1),
            (Scancode::Num2, 0x2),
            (Scancode::Num3, 0x3),
            (Scancode::Num4, 0xC),
            (Scancode::Q, 0x4),
            (Scancode::W, 0x5),
            (Scancode::E, 0x6),
            (Scancode::R, 0xD),
            (Scancode::A, 0x7),
            (Scancode::S, 0x8),
            (Scancode::D, 0x9),
            (Scancode::F, 0xE),
            (Scancode::Z, 0xA),
            (Scancode::X, 0x0),
            (Scancode::C, 0xB),
            (Scancode::V, 0xF),
        ];

        for &(scancode, hex_key) in &mappings {
            scancode_to_hex_map.insert(scancode, hex_key);
            hex_to_scancode_map.insert(hex_key, scancode);
        }

        (scancode_to_hex_map, hex_to_scancode_map)
    }

    /// Checks if the specified hex key is pressed.
    ///
    /// # Arguments
    /// * `hex_key` - A u8 representing the hex key to check.
    ///
    /// # Returns
    /// * `bool` - true if the key is pressed, false otherwise.
    pub fn is_key_pressed(&mut self, hex_key: u8) -> bool {
        self.event_pump.pump_events();
        if let Some(&scancode) = self.hex_to_scancode_map.get(&hex_key) {
            self.event_pump
                .keyboard_state()
                .is_scancode_pressed(scancode)
        } else {
            false
        }
    }

    /// Returns the next key release in hex key format.
    ///
    /// # Returns
    /// * `Option<u8>` - The hex key code of the next key release, or None if no key was released.
    pub fn next_key_release(&mut self) -> Option<u8> {
        self.event_pump.pump_events();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(&hex_key) = self.scancode_to_hex_map.get(&scancode) {
                        if self.key_state[hex_key as usize] {
                            self.key_state[hex_key as usize] = false;
                            return Some(hex_key);
                        }
                    }
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(&hex_key) = self.scancode_to_hex_map.get(&scancode) {
                        self.key_state[hex_key as usize] = true;
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Resets the key state by clearing all pressed keys.
    pub fn reset_key_state(&mut self) {
        self.event_pump.pump_events();
        for _ in self.event_pump.poll_iter() {}
        self.key_state = [false; KEYS_NUM];
    }
}
