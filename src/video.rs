use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

const X_DIM: usize = 64;
const Y_DIM: usize = 32;
const SCALE: usize = 20;
const WINDOW_TITLE: &str = "emul8tor";

#[allow(non_snake_case)]
pub struct DisplayManager {
    canvas: Canvas<Window>,
    VRAM: [[u8; X_DIM]; Y_DIM],
    update_needed: bool,
}

impl DisplayManager {
    /// Creates a new DisplayManager with an SDL2 context and a window.
    /// Panics if SDL2 initialization or window creation fails.
    pub fn new() -> Self {
        let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
        let video_subsystem = sdl_context
            .video()
            .expect("Failed to get SDL2 video subsystem");

        let window = video_subsystem
            .window(WINDOW_TITLE, (X_DIM * SCALE) as u32, (Y_DIM * SCALE) as u32)
            .position_centered()
            .build()
            .expect("Failed to create window");

        let mut canvas = window
            .into_canvas()
            .build()
            .expect("Failed to create canvas");

        canvas
            .set_scale(SCALE as f32, SCALE as f32)
            .expect("Failed to set scale");

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        DisplayManager {
            canvas,
            VRAM: [[0; X_DIM]; Y_DIM],
            update_needed: false,
        }
    }

    /// Sets the pixel at position (x, y) to the given value (1 for white, 0 for black).
    /// Returns true if the pixel was already set to the given value, false otherwise.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) -> bool {
        self.update_needed = true;

        let x = x % X_DIM;
        let y = y % Y_DIM;
        let previous_value = self.VRAM[y][x];
        self.VRAM[y][x] ^= value;
        self.draw_pixel(x, y);

        previous_value == value
    }

    /// Clears the display and resets the VRAM.
    pub fn clear(&mut self) {
        self.update_needed = true;
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.VRAM.iter_mut().for_each(|row| row.fill(0));
    }

    /// Updates the display by presenting the canvas if any changes were made.
    pub fn update(&mut self) {
        if self.update_needed {
            self.canvas.present();
        }
        self.update_needed = false;
    }

    /// Draws a single pixel at the given coordinates based on the VRAM content.
    fn draw_pixel(&mut self, x: usize, y: usize) {
        let color = if self.VRAM[y][x] != 0 {
            Color::WHITE
        } else {
            Color::BLACK
        };
        self.canvas.set_draw_color(color);
        self.canvas
            .draw_point(Point::new(x as i32, y as i32))
            .expect("Failed to draw point");
    }
}
