use std::{env, io};

fn main() -> io::Result<()> {
    let mut args = env::args();
    args.next();

    let file_path = match args.next() {
        Some(arg) => arg,
        None => panic!("Didn't get a file path"),
    };

    let mode = match args.next() {
        Some(arg) => match arg.as_str() {
            "1" => emul8tor::Mode::Chip8,
            "2" => emul8tor::Mode::SuperChip,
            "3" => emul8tor::Mode::XOChip,
            _ => panic!(),
        },
        None => panic!("Didn't get a mode"),
    };

    match emul8tor::load_program_rom(&file_path) {
        Ok(bytes) => {
            emul8tor::run(emul8tor::Chip8::new(mode, bytes));
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }

    Ok(())
}
