use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;
use sdl2::IntegerOrSdlError;
use std::collections::HashMap;

const KEYS_NUM: usize = 16;

/// Maps specific Scancodes to corresponding hex values.
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
    event_pump: Option<EventPump>,
    key_state: [bool; KEYS_NUM],
    released_key_queue: Option<u8>,
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
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, IntegerOrSdlError> {
        let event_pump = sdl_context
            .event_pump()
            .map_err(IntegerOrSdlError::SdlError)?;
        let scancode_to_hex_map = SCANCODE_TO_HEX_MAP.iter().cloned().collect();

        Ok(InputManager {
            event_pump: Some(event_pump),
            key_state: [false; KEYS_NUM],
            released_key_queue: None,
            waiting_for_key: false,
            quit: false,
            scancode_to_hex_map,
        })
    }

    /// Checks if a specific hex key is currently pressed.
    ///
    /// # Arguments
    ///
    /// * `hex_key` - The hex value of the key to check.
    ///
    /// # Returns
    ///
    /// `true` if the key is pressed, `false` otherwise.
    pub fn is_key_pressed(&self, hex_key: u8) -> bool {
        self.key_state[hex_key as usize]
    }

    /// Gets the next key that was released.
    ///
    /// # Returns
    ///
    /// The hex value of the next released key, or `None` if no key was released.
    pub fn get_next_released_key(&mut self) -> Option<u8> {
        self.waiting_for_key = true;
        self.released_key_queue.take()
    }

    /// Updates the state of the InputManager by processing SDL events.
    pub fn update(&mut self) {
        if let Some(event_pump) = self.event_pump.as_mut() {
            event_pump.pump_events();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => self.quit = true,
                    Event::KeyUp {
                        scancode: Some(scancode),
                        ..
                    } => {
                        if let Some(&hex_key) = self.scancode_to_hex_map.get(&scancode) {
                            self.key_state[hex_key as usize] = false;
                            if self.waiting_for_key {
                                self.released_key_queue = Some(hex_key);
                                self.waiting_for_key = false;
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
    }

    /// Checks if a quit event has been received.
    ///
    /// # Returns
    ///
    /// `true` if a quit event has been received, `false` otherwise.
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input_manager() -> InputManager {
        let scancode_to_hex_map: HashMap<Scancode, u8> =
            SCANCODE_TO_HEX_MAP.iter().cloned().collect();

        InputManager {
            event_pump: None,
            key_state: [false; KEYS_NUM],
            released_key_queue: None,
            waiting_for_key: false,
            quit: false,
            scancode_to_hex_map,
        }
    }

    #[test]
    fn test_is_key_pressed() {
        let mut input_manager = create_test_input_manager();

        // Simulate pressing the '1' key
        input_manager.key_state[0x1 as usize] = true;

        assert!(input_manager.is_key_pressed(0x1));
        assert!(!input_manager.is_key_pressed(0x2));
    }

    #[test]
    fn test_get_next_released_key() {
        let mut input_manager = create_test_input_manager();

        // Simulate releasing the '1' key
        input_manager.released_key_queue = Some(0x1);

        assert_eq!(input_manager.get_next_released_key(), Some(0x1));
        assert_eq!(input_manager.get_next_released_key(), None); // Queue should be empty now
    }

    #[test]
    fn test_should_quit() {
        let mut input_manager = create_test_input_manager();
        assert!(!input_manager.should_quit());

        input_manager.quit = true;
        assert!(input_manager.should_quit());
    }
}

