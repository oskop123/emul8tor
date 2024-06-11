use std::usize;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;

pub struct InputManager {
    event_pump: EventPump,
    key_state: [bool; 16],
}

// TODO Major refactor related to class API and main loop.

// TODO Handle exit. QUIT/Esc
impl InputManager {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        InputManager {
            event_pump,
            key_state: [false; 16],
        }
    }

    // TODO Define hex hey as enums??
    pub fn is_key_pressed(&mut self, hex_key: u8) -> bool {
        self.event_pump.pump_events();
        self.event_pump
            .keyboard_state()
            .is_scancode_pressed(Self::hex_key_to_scancode(hex_key))
    }

    pub fn next_key_release(&mut self) -> u8 {
        self.event_pump.pump_events();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(hex_key) = Self::scancode_to_hex_key(scancode) {
                        if self.key_state[hex_key as usize] {
                            dbg!(hex_key);
                            return hex_key;
                        }
                    }
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(hex_key) = Self::scancode_to_hex_key(scancode) {
                        self.key_state[hex_key as usize] = true;
                    }
                }
                _ => continue,
            }
        }

        // TODO add oOption
        return 64;
    }

    pub fn reset_key_state(&mut self) {
        self.event_pump.pump_events();
        for _ in self.event_pump.poll_iter() {}
        self.key_state = [false; 16];
    }

    fn hex_key_to_scancode(hex_key: u8) -> sdl2::keyboard::Scancode {
        match hex_key {
            0x1 => Scancode::Num1,
            0x2 => Scancode::Num2,
            0x3 => Scancode::Num3,
            0xC => Scancode::Num4,
            0x4 => Scancode::Q,
            0x5 => Scancode::W,
            0x6 => Scancode::E,
            0xD => Scancode::R,
            0x7 => Scancode::A,
            0x8 => Scancode::S,
            0x9 => Scancode::D,
            0xE => Scancode::F,
            0xA => Scancode::Z,
            0x0 => Scancode::X,
            0xB => Scancode::C,
            0xF => Scancode::V,
            _ => Scancode::Escape,
        }
    }

    fn scancode_to_hex_key(scancode: Scancode) -> Option<u8> {
        match scancode {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }
}
