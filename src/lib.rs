pub mod audio;
pub mod input;
pub mod video;

use std::fs::File;
use std::io::{self, Read};
use std::time::{Duration, Instant};

use rand::Rng;

use audio::AudioManager;
use input::InputManager;
use video::{DisplayManager, Resolution};

const MEMORY_SIZE: usize = 4096;
const V_COUNT: usize = 16;
const ROM_START_ADDRESS: usize = 0x200;
const SPRITE_WIDTH: usize = 8;
const MAX_STACK_LEVELS: usize = 16;

const FRAME_RATE: u32 = 60;

const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(PartialEq)]
pub enum Mode {
    Chip8,
    SuperChip,
    XOChip,
}

#[allow(non_snake_case)]
pub struct Chip8 {
    mode: Mode,
    scale: usize,

    memory: [u8; MEMORY_SIZE],
    V: [u8; V_COUNT],
    I: u16,
    PC: usize,

    stack: [usize; MAX_STACK_LEVELS],
    SP: usize,

    delay_timer: u8,
    sound_timer: u8,

    display: DisplayManager,
    input: InputManager,
    audio: AudioManager,
    sdl_context: sdl2::Sdl,

    release_key_register: Option<usize>,
}

impl Chip8 {
    #[allow(non_snake_case)]
    pub fn new(mode: Mode, scale: usize, memory: [u8; MEMORY_SIZE]) -> Self {
        let sdl_context = sdl2::init().expect("Failed to initialize SDL2");

        let mut chip8 = Chip8 {
            mode,
            scale,
            memory,
            V: [0; V_COUNT],
            I: 0,
            PC: ROM_START_ADDRESS,
            stack: [0; MAX_STACK_LEVELS],
            SP: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: DisplayManager::new(&sdl_context, Resolution::Low, scale).unwrap(),
            input: InputManager::new(&sdl_context).unwrap(),
            audio: AudioManager::new(&sdl_context).unwrap(),
            sdl_context,
            release_key_register: None,
        };

        // Load fontset into memory
        chip8.memory[..CHIP8_FONTSET.len()].copy_from_slice(&CHIP8_FONTSET);
        chip8
    }

    fn emulate_cycle(&mut self) {
        if let Some(register) = self.release_key_register {
            self.wait_for_next_key(register);
        } else {
            let opcode = self.fetch_opcode();
            self.execute_opcode(opcode);
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.audio.start();
            self.sound_timer -= 1;
        } else {
            self.audio.stop()
        }
    }

    fn wait_for_next_key(&mut self, register: usize) {
        if let Some(val) = self.input.get_next_released_key() {
            self.V[register] = val;
            self.release_key_register = None;
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        let opcode = (self.memory[self.PC] as u16) << 8 | self.memory[self.PC + 1] as u16;
        self.PC += 2;
        opcode
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let kk = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x0F00 {
                0x0000 => match opcode & 0x00F0 {
                    0x00C0 if self.mode == Mode::SuperChip || self.mode == Mode::XOChip => {
                        self.op_00cn(n)
                    }
                    0x00D0 if self.mode == Mode::XOChip => self.op_00dn(n),
                    0x00E0 => match opcode & 0x000F {
                        0x0000 => self.op_00e0(),
                        0x000E => self.op_00ee(),
                        _ => Self::unknown_opcode(opcode),
                    },
                    0x00F0 => match opcode & 0x000F {
                        0x000B => self.op_00fb(),
                        0x000C => self.op_00fc(),
                        // 0x000D => self.op_00fd(),
                        0x000E => self.op_00fe(),
                        0x000F => self.op_00ff(),
                        _ => Self::unknown_opcode(opcode),
                    },
                    _ => Self::unknown_opcode(opcode),
                },
                _ => self.op_0nnn(nnn),
            },
            0x1000 => self.op_1nnn(nnn),
            0x2000 => self.op_2nnn(nnn),
            0x3000 => self.op_3xkk(x, kk),
            0x4000 => self.op_4xkk(x, kk),
            0x5000 => self.op_5xy0(x, y),
            0x6000 => self.op_6xkk(x, kk),
            0x7000 => self.op_7xkk(x, kk),
            0x8000 => match opcode & 0xF00F {
                0x8000 => self.op_8xy0(x, y),
                0x8001 => self.op_8xy1(x, y),
                0x8002 => self.op_8xy2(x, y),
                0x8003 => self.op_8xy3(x, y),
                0x8004 => self.op_8xy4(x, y),
                0x8005 => self.op_8xy5(x, y),
                0x8006 => self.op_8xy6(x, y),
                0x8007 => self.op_8xy7(x, y),
                0x800E => self.op_8xye(x, y),
                _ => Self::unknown_opcode(opcode),
            },
            0x9000 => self.op_9xy0(x, y),
            0xA000 => self.op_annn(nnn),
            0xB000 if self.mode != Mode::SuperChip => self.op_bnnn(nnn),
            0xB000 if self.mode == Mode::SuperChip => self.op_bxnn(x, nnn),
            0xC000 => self.op_cxkk(x, kk),
            0xD000 => self.op_dxyn(x, y, n),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(x),
                0x00A1 => self.op_exa1(x),
                _ => Self::unknown_opcode(opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(x),
                0x000A => self.op_fx0a(x),
                0x0015 => self.op_fx15(x),
                0x0018 => self.op_fx18(x),
                0x001E => self.op_fx1e(x),
                0x0029 => self.op_fx29(x),
                0x0033 => self.op_fx33(x),
                0x0055 => self.op_fx55(x),
                0x0065 => self.op_fx65(x),
                _ => Self::unknown_opcode(opcode),
            },
            _ => Self::unknown_opcode(opcode),
        }
    }

    fn unknown_opcode(opcode: u16) {
        panic!("Unknown opcode: {:X}", opcode);
    }

    // 0nnn - SYS addr: Jump to a machine code routine at nnn.
    fn op_0nnn(&mut self, _nnn: u16) {
        // This instruction is only used on the old computers on which Chip-8
        // was originally implemented. It is ignored by modern interpreters.
    }

    fn op_00cn(&mut self, n: u8) {
        self.display.scroll_down(n as usize)
    }

    fn op_00dn(&mut self, n: u8) {
        self.display.scroll_up(n as usize)
    }

    // 00E0 - CLS: Clear the display.
    fn op_00e0(&mut self) {
        self.display.clear();
    }

    // 00EE - RET: Return from a subroutine.
    fn op_00ee(&mut self) {
        if self.SP == 0 {
            panic!("Stack underflow!");
        }
        self.SP -= 1;
        self.PC = self.stack[self.SP];
    }

    fn op_00fb(&mut self) {
        self.display.scroll_right()
    }

    fn op_00fc(&mut self) {
        self.display.scroll_left()
    }

    // 00FE - LORES: Switch to lores mode.
    fn op_00fe(&mut self) {
        self.display = DisplayManager::new(&self.sdl_context, Resolution::Low, self.scale).unwrap();
    }

    // 00FF - HIRES: Switch to hires mode.
    fn op_00ff(&mut self) {
        self.display =
            DisplayManager::new(&self.sdl_context, Resolution::High, self.scale).unwrap();
    }

    // 1nnn - JP addr: Jump to location nnn.
    fn op_1nnn(&mut self, addr: u16) {
        self.PC = addr as usize;
    }

    // 2nnn - CALL addr: Call subroutine at nnn.
    fn op_2nnn(&mut self, addr: u16) {
        if self.SP >= MAX_STACK_LEVELS {
            panic!("Stack overflow!");
        }
        self.stack[self.SP] = self.PC;
        self.SP += 1;
        self.PC = addr as usize;
    }

    // 3xkk - SE Vx, byte: Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.V[x] == kk {
            self.PC += 2;
        }
    }

    // 4xkk - SNE Vx, byte: Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.V[x] != kk {
            self.PC += 2;
        }
    }

    // 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.V[x] == self.V[y] {
            self.PC += 2;
        }
    }

    // 6xkk - LD Vx, byte: Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.V[x] = kk;
    }

    // 7xkk - ADD Vx, byte: Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.V[x] = self.V[x].wrapping_add(kk);
    }

    // 8xy0 - LD Vx, Vy: Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.V[x] = self.V[y];
    }

    // 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy.
    // SuperChip and XOChip modes don't reset VF.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.V[x] |= self.V[y];
        if self.mode == Mode::Chip8 {
            self.V[0xF] = 0;
        }
    }

    // 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy.
    // SuperChip and XOChip modes don't reset VF.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.V[x] &= self.V[y];
        if self.mode == Mode::Chip8 {
            self.V[0xF] = 0;
        }
    }

    // 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy.
    // SuperChip and XOChip modes don't reset VF.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.V[x] ^= self.V[y];
        if self.mode == Mode::Chip8 {
            self.V[0xF] = 0;
        }
    }

    // 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (result, carry) = self.V[x].overflowing_add(self.V[y]);
        self.V[x] = result;
        self.V[0xF] = if carry { 1 } else { 0 }
    }

    // 8xy5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (result, borrow) = self.V[x].overflowing_sub(self.V[y]);
        self.V[x] = result;
        self.V[0xF] = !borrow as u8;
    }

    // 8xy6 - SHR Vx {, Vy}: Set Vx = Vx SHR 1.
    // SuperChip doesn't set vX to vY.
    fn op_8xy6(&mut self, x: usize, y: usize) {
        if self.mode != Mode::SuperChip {
            self.V[x] = self.V[y];
        }
        let bit = self.V[x] & 0x1;
        self.V[x] >>= 1;
        self.V[0xF] = bit;
    }

    // 8xy7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (result, borrow) = self.V[y].overflowing_sub(self.V[x]);
        self.V[x] = result;
        self.V[0xF] = !borrow as u8;
    }

    // 8xye - SHL Vx {, Vy}: Set Vx = Vx SHL 1.
    // SuperChip doesn't set vX to vY.
    fn op_8xye(&mut self, x: usize, y: usize) {
        if self.mode != Mode::SuperChip {
            self.V[x] = self.V[y];
        }
        let bit = (self.V[x] >> 7) & 0x1;
        self.V[x] <<= 1;
        self.V[0xF] = bit;
    }

    // 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.V[x] != self.V[y] {
            self.PC += 2;
        }
    }

    // Annn - LD I, addr: Set I = nnn.
    fn op_annn(&mut self, addr: u16) {
        self.I = addr;
    }

    // Bnnn - JP V0, addr: Jump to location nnn + V0.
    fn op_bnnn(&mut self, addr: u16) {
        self.PC = (addr + self.V[0] as u16) as usize;
    }

    // Bxnn - JP VX, addr: Jump to location xnn + VX.
    fn op_bxnn(&mut self, x: usize, xnn: u16) {
        self.PC = (xnn + self.V[x] as u16) as usize;
    }

    // Cxkk - RND Vx, byte: Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        self.V[x] = rand::thread_rng().gen::<u8>() & kk;
    }

    // Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // XO-CHIP wraps pixels instead of clipping them.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let x_coord = self.V[x] as usize % self.display.width();
        let y_coord = self.V[y] as usize % self.display.height();

        self.V[0xF] = 0;
        for byte_index in 0..n as usize {
            let y = (y_coord + byte_index) % self.display.height();
            if self.mode != Mode::XOChip && y_coord + byte_index >= self.display.height() {
                break;
            }
            let byte = self.memory[self.I as usize + byte_index];
            for bit_index in 0..SPRITE_WIDTH {
                let x = (x_coord + bit_index) % self.display.width();
                if self.mode != Mode::XOChip && x_coord + bit_index >= self.display.width() {
                    break;
                }
                let bit = (byte >> (7 - bit_index)) & 1;
                self.V[0xF] |= self.display.set_pixel(x, y, bit);
            }
        }
    }

    // Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) {
        if self.input.is_key_pressed(self.V[x]) {
            self.PC += 2;
        }
    }

    // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) {
        if !self.input.is_key_pressed(self.V[x]) {
            self.PC += 2;
        }
    }

    // Fx07 - LD Vx, DT: Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) {
        self.V[x] = self.delay_timer;
    }

    // Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) {
        self.release_key_register = Some(x);
    }

    // Fx15 - LD DT, Vx: Set delay timer = Vx.
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.V[x];
    }

    // Fx18 - LD ST, Vx: Set sound timer = Vx.
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.V[x];
    }

    // Fx1E - ADD I, Vx: Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        self.I += self.V[x] as u16;
    }

    // Fx29 - LD F, Vx: Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, x: usize) {
        self.I = self.V[x] as u16 * 5;
    }

    // Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, x: usize) {
        self.memory[self.I as usize] = self.V[x] / 100;
        self.memory[self.I as usize + 1] = (self.V[x] % 100) / 10;
        self.memory[self.I as usize + 2] = self.V[x] % 10;
    }

    // Fx55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I.
    // SuperChip doesn't increment I.
    fn op_fx55(&mut self, x: usize) {
        for offset in 0..=x {
            self.memory[self.I as usize + offset] = self.V[offset];
        }

        if self.mode != Mode::SuperChip {
            self.I += self.V[x] as u16;
            self.I += 1;
        }
    }

    // Fx65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I.
    // SuperChip doesn't increment I.
    fn op_fx65(&mut self, x: usize) {
        for offset in 0..=x {
            self.V[offset] = self.memory[self.I as usize + offset];
        }

        if self.mode != Mode::SuperChip {
            self.I += self.V[x] as u16;
            self.I += 1;
        }
    }
}

pub fn run(mut chip8: Chip8, speed: u32) {
    let mut last_frame = Instant::now();
    let frame_duration: Duration = Duration::from_secs_f64(1.0 / FRAME_RATE as f64);

    let mut last_cycle = Instant::now();
    let cycle_duration: Duration = Duration::from_secs_f64(1.0 / speed as f64);

    loop {
        if last_cycle.elapsed() >= cycle_duration {
            last_cycle = Instant::now();
            chip8.emulate_cycle();
            chip8.input.update();
        }

        if last_frame.elapsed() >= frame_duration {
            last_frame = Instant::now();
            chip8.display.render();
            chip8.update_timers();
        }

        if chip8.input.should_quit() {
            break;
        }
    }
}

pub fn load_program_rom(rom_path: &str) -> io::Result<[u8; MEMORY_SIZE]> {
    let mut rom = File::open(rom_path)?;
    let mut buffer = [0u8; MEMORY_SIZE];
    rom.read(&mut buffer[ROM_START_ADDRESS..])?;
    Ok(buffer)
}
