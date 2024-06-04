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
}

impl DisplayManager {
    pub fn new() -> Self {
        // TODO Handle errors + make const u32?
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(WINDOW_TITLE, (X_DIM * SCALE) as u32, (Y_DIM * SCALE) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        DisplayManager {
            canvas,
            VRAM: [[0; X_DIM]; Y_DIM],
        }
    }

    /// Sets the pixel at position (x, y) to the given value.
    /// If the value is 1, the pixel is turned on (white); if 0, the pixel is turned off (black).
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        // TODO wrap around
        if x < X_DIM && y < Y_DIM {
            self.VRAM[y][x] ^= value;
            self.draw_pixel(x, y);
        }
    }

    /// Clears the display and resets the VRAM.
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.VRAM.iter_mut().for_each(|row| row.fill(0));
        // TODO Should update_display be called here?
        //self.update_display();
    }

    /// Updates the display by rendering the VRAM content.
    pub fn update(&mut self) {
        self.canvas.present();
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
            .unwrap();
    }
}
