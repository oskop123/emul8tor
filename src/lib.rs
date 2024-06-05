mod video;

use std::fs::File;
use std::io::{self, Read};
use std::thread::sleep;
use std::time::Duration;
use std::usize;

// TODO Create enum with PC moves

use video::DisplayManager;

const ROM_SIZE: usize = 4096;
const V_COUNT: usize = 16;
const PROGRAM_START_ADDRESS: usize = 0x200;
const SPRITE_WIDTH: usize = 8;
const MAX_STACK_LEVELS: usize = 16;

pub const CHIP8_FONTSET: [u8; 80] = [
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

#[allow(non_snake_case)]
pub struct Chip8 {
    ROM: [u8; ROM_SIZE],
    V: [u8; V_COUNT],
    I: u16,
    PC: usize,

    stack: [usize; MAX_STACK_LEVELS],
    SP: usize,

    // Move all drivers into seperate drivers structure
    display: DisplayManager,
}

impl Chip8 {
    #[allow(non_snake_case)]
    pub fn new(ROM: [u8; ROM_SIZE]) -> Self {
        Chip8 {
            ROM,
            V: [0; V_COUNT],
            I: 0,
            PC: PROGRAM_START_ADDRESS,
            stack: [0; MAX_STACK_LEVELS],
            SP: 0,
            display: DisplayManager::new(),
        }
    }

    fn next_opcode(&mut self) -> u16 {
        // TODO Ensure we don't read out of bounds???
        // if self.pc + 1 >= MEMORY_SIZE {
        //   return Err("Program counter out of bounds");
        //}
        //

        let opcode = (self.ROM[self.PC] as u16) << 8 | self.ROM[self.PC + 1] as u16;
        self.PC += 2;

        opcode
    }

    fn execute_opcode(&mut self, opcode: u16) {
        // TODO Maybe here some symbols should be cast to usize? Define utility functions and move
        // them to opcode execution methods
        let kk = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => panic!("{}", format!("{:x}", opcode)), //TODO error?
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
                _ => panic!("{}", format!("{:x}", opcode)), //TODO error?
            },
            0x9000 => self.op_9xy0(x, y),
            0xA000 => self.op_annn(nnn),
            0xD000 => self.op_dxyn(x, y, n),
            0xF000 => match opcode & 0x00FF {
                0x001E => self.op_fx1e(x),
                0x0033 => self.op_fx33(x),
                0x0055 => self.op_fx55(x),
                0x0065 => self.op_fx65(x),
                _ => panic!("{}", format!("{:x}", opcode)), //TODO error?
            },
            _ => panic!("{}", format!("{:x}", opcode)), //TODO error?
        }
    }

    // 00E0 - CLS
    // Clear the display.
    fn op_00e0(&mut self) {
        self.display.clear();
    }

    // 00EE - RET
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.SP -= 1;
        self.PC = self.stack[self.SP];
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.PC = nnn as usize;
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    fn op_2nnn(&mut self, nnn: u16) {
        // TODO Define stack overflow error
        self.stack[self.SP] = self.PC;
        self.SP += 1;
        self.PC = nnn as usize;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.V[x] == kk {
            self.PC += 2;
        }
    }

    // 4xkk - SE Vx, byte
    // Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.V[x] != kk {
            self.PC += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.V[x] == self.V[y] {
            self.PC += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.V[x] = kk;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.V[x] = self.V[x].wrapping_add(kk);
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.V[x] = self.V[y];
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.V[x] |= self.V[y];
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.V[x] &= self.V[y];
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.V[x] ^= self.V[y];
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, overflowed) = self.V[x].overflowing_add(self.V[y]);
        self.V[x] = sum;
        self.V[0xF] = if overflowed { 1 } else { 0 }
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        self.V[0xF] = if self.V[x] > self.V[y] { 1 } else { 0 };
        self.V[x] = self.V[x].wrapping_sub(self.V[y]);
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, x: usize, _y: usize) {
        self.V[0xF] = self.V[x] & 0x1;
        self.V[x] >>= 1;
    }

    //8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        self.V[0xF] = if self.V[y] > self.V[x] { 1 } else { 0 };
        self.V[x] = self.V[y].wrapping_sub(self.V[x]);
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    fn op_8xye(&mut self, x: usize, _y: usize) {
        self.V[0xF] = self.V[x] & 0x80;
        self.V[x] <<= 1;
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.V[x] != self.V[y] {
            self.PC += 2;
        }
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        self.I += self.V[x] as u16;
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, x: usize) {
        let mut vx = self.V[x];
        for offset in (0..=2).rev() {
            self.ROM[self.I as usize + offset] = vx % 10;
            vx /= 10;
        }
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) {
        for offset in 0..=x {
            self.ROM[self.I as usize + offset] = self.V[offset];
        }
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) {
        for offset in 0..=x {
            self.V[offset] = self.ROM[self.I as usize + offset];
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.I = nnn;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        // TODO set VF
        for row in 0..n as usize {
            let register_value = self.ROM[self.I as usize + row];

            for offset in 0..SPRITE_WIDTH {
                let x_coord = self.V[x] as usize + offset;
                let y_coord = self.V[y] as usize + row;

                self.display
                    .set_pixel(x_coord, y_coord, (register_value >> (7 - offset)) & 1);
            }
        }

        self.display.update();

        // TODO Remove after refresh timer implementation. Screen tearing visible without delay.
        sleep(Duration::from_millis(200));
    }
}

pub fn run(mut chip8: Chip8) {
    // TODO loop while result of next-opcode is ok
    loop {
        let opcode = chip8.next_opcode();
        chip8.execute_opcode(opcode);
    }
}

// Maybe make non public?
pub fn load_program(file_path: &str) -> io::Result<[u8; ROM_SIZE]> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; ROM_SIZE];
    buffer[..CHIP8_FONTSET.len()].copy_from_slice(&CHIP8_FONTSET);
    file.read(&mut buffer[PROGRAM_START_ADDRESS..])?;
    Ok(buffer)
}
