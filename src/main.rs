use emul8tor;
use std::io;

fn main() -> io::Result<()> {
    let file_path = "3-corax+.ch8";

    match emul8tor::load_program(file_path) {
        Ok(bytes) => {
            emul8tor::run(emul8tor::Chip8::new(bytes));
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }

    Ok(())
}
