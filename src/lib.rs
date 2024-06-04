mod video;

use std::fs::File;
use std::io::{self, Read};
use std::thread::sleep;
use std::time::Duration;

use video::DisplayManager;

const ROM_SIZE: usize = 4096;
const V_COUNT: usize = 16;
const PROGRAM_START_ADDRESS: usize = 0x200;
const SPRITE_WIDTH: usize = 8;

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

        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                _ => (),
            },
            0x1000 => self.op_1nnn(nnn),
            0x6000 => self.op_6xkk(x, kk),
            0xA000 => self.op_annn(nnn),
            0xD000 => self.op_dxyn(x, y, n),
            _ => (), //TODO error?
        }
    }

    // 00E0 - CLS
    // Clear the display.
    fn op_00e0(&mut self) {
        self.display.clear();
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.PC = nnn as usize;
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    fn op_6xkk(&mut self, x: u8, nn: u8) {
        // TODO Raise error if x bigger than v_COUNT

        self.V[x as usize] = nn;
    }

    // Annn - LD I, addr
    // Set I = nnn.
    fn op_annn(&mut self, nnn: u16) {
        self.I = nnn;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        // TODO set VF
        for row in 0..n as usize {
            let register_value = self.ROM[self.I as usize + row];

            for offset in 0..SPRITE_WIDTH {
                let x_coord = self.V[x as usize] as usize + offset;
                let y_coord = self.V[y as usize] as usize + row;

                self.display
                    .set_pixel(x_coord, y_coord, (register_value >> (7 - offset)) & 1);
            }
        }

        self.display.update();

        // TODO Remove after refresh timer implementation. Screen tearing visible without delay.
        sleep(Duration::from_millis(30));
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
