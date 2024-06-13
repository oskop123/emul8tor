use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;
use std::collections::HashMap;
use std::error::Error;

const KEYS_NUM: usize = 16;

const SCANCODE_TO_HEX_MAP: [(Scancode, u8); KEYS_NUM] = [
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

/// Manages input using SDL2.
pub struct InputManager {
    event_pump: EventPump,

    key_state: [bool; KEYS_NUM],
    released_key: Option<u8>,
    waiting_for_key: bool,
    quit: bool,

    scancode_to_hex_map: HashMap<Scancode, u8>,
}

impl InputManager {
    /// Creates a new `InputManager` instance.
    ///
    /// # Arguments
    ///
    /// * `sdl_context` - A reference to an initialized SDL context.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL2 fails to get the event pump.
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, Box<dyn Error>> {
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| format!("Failed to get SDL2 event pump: {}", e))?;

        let scancode_to_hex_map = SCANCODE_TO_HEX_MAP.iter().cloned().collect();

        Ok(InputManager {
            event_pump,
            key_state: [false; KEYS_NUM],
            released_key: None,
            waiting_for_key: false,
            quit: false,
            scancode_to_hex_map,
        })
    }

    /// Checks if a specific hex key is currently pressed.
    pub fn is_key_pressed(&mut self, hex_key: u8) -> bool {
        self.key_state[hex_key as usize]
    }

    /// Gets the next key that was released.
    pub fn get_next_released_key(&mut self) -> Option<u8> {
        self.waiting_for_key = true;
        self.released_key.take()
    }

    /// Updates the state of the InputManager by processing SDL events.
    pub fn update(&mut self) {
        self.event_pump.pump_events();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.quit = true,
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(&hex_key) = self.scancode_to_hex_map.get(&scancode) {
                        self.key_state[hex_key as usize] = false;
                        if self.waiting_for_key {
                            self.released_key = Some(hex_key);
                            self.waiting_for_key = false
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
                    if scancode == Scancode::Escape {
                        self.quit = true;
                    }
                }
                _ => {}
            }
        }
    }

    /// Checks if a quit event has been received.
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}
