# em8lator

**em8lator** is a high-performance, lightweight emulator for the classic CHIP-8, SuperChip, and XO-Chip interpreted programming languages. Written in Rust, this project brings the simplicity and nostalgia of these early systems to modern machines.

## Table of Contents

- [Features](#features)
- [Supported Modes](#supported-modes)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)

## Features

- **Multi-Mode Support**: Emulates CHIP-8, SuperChip, and XO-Chip.
- **High Performance**: Written in Rust, optimized for speed and efficiency.
- **Customizable**: Easily adjustable settings to tweak the emulation experience.
- **Open Source**: Contributions are welcome!

## Supported Modes

1. **CHIP-8**: The original interpreter from the 1970s.
2. **SuperChip**: An extension of CHIP-8 with a higher resolution and additional features.
3. **XO-Chip**: A modern enhancement of CHIP-8, introducing new instructions and increased capabilities.

## Installation

### Prerequisites

- **Rust**: Make sure you have Rust installed. If not, you can install it from [rust-lang.org](https://www.rust-lang.org/).

### Build from Source

1. Clone the repository:

    ```bash
    git clone https://github.com/yourusername/em8lator.git
    cd em8lator
    ```

2. Build the project:

    ```bash
    cargo build --release
    ```

3. Run the emulator:

    ```bash
    ./target/release/em8lator
    ```

## Usage

### Running a ROM

To start the emulator with a CHIP-8 ROM, use the following command:

```bash
./em8lator /path/to/your/rom.ch8
```

### Command-Line Options
```bash
Usage: emul8tor [OPTIONS] <ROM_PATH>

Arguments:
  <ROM_PATH>  Path to the ROM file

Options:
  -m, --mode <MODE>    Specify the emulation mode (Chip8, SuperChip, XOChip) [default: Chip8]
      --scale <SCALE>  Set the display scaling factor [default: 10]
      --speed <SPEED>  Adjust the execution speed (in Hz) [default: 700]
  -h, --help           Print help
  -V, --version        Print version
```

Example:

```bash
./em8lator --mode XOChip --scale 10 --speed 500 /path/to/your/rom.ch8
```

## Contributing

Contributions are welcome! If you'd like to contribute, please fork the repository and submit a pull request. Make sure to follow the Rust API guidelines and write tests for your code.

- [ ] Add support for save states.
- [ ] Add support of other modes.
- [ ] Add display wait feature for original CHIP-8 mode.
