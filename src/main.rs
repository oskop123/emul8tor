use std::io;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify the emulation mode (Chip8, SuperChip, XOChip)
    #[arg(short, long, value_name = "MODE", default_value_t = String::from("Chip8"))]
    mode: String,

    /// Set the display scaling factor
    #[arg(long, value_name = "SCALE", default_value_t = 10)]
    scale: u32,

    /// Adjust the execution speed (in Hz)
    #[arg(long, value_name = "SPEED", default_value_t = 700)]
    speed: u32,

    /// Path to the ROM file
    #[arg(value_name = "ROM_PATH")]
    rom_path: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mode = match cli.mode.as_str() {
        "Chip8" => emul8tor::Mode::Chip8,
        "SuperChip" => emul8tor::Mode::SuperChip,
        "XOChip" => emul8tor::Mode::XOChip,
        _ => panic!("Unavailable mode!"),
    };

    match emul8tor::load_program_rom(&cli.rom_path) {
        Ok(bytes) => {
            emul8tor::run(
                emul8tor::Chip8::new(mode, cli.scale as usize, bytes),
                cli.speed,
            );
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }

    Ok(())
}
