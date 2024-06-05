use emul8tor;
use std::{env, io};

fn main() -> io::Result<()> {
    let mut args = env::args();
    args.next();

    let file_path = match args.next() {
        Some(arg) => arg,
        None => return panic!("Didn't get a file path"),
    };

    match emul8tor::load_program(&file_path) {
        Ok(bytes) => {
            emul8tor::run(emul8tor::Chip8::new(bytes));
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }

    Ok(())
}
