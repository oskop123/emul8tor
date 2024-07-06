use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::error::Error;

const X_DIM: usize = 64;
const Y_DIM: usize = 32;
const SCALE: usize = 20;
const WINDOW_TITLE: &str = "emul8tor";

/// Manages display rendering using SDL2.
#[allow(non_snake_case)]
pub struct DisplayManager {
    canvas: Option<Canvas<Window>>,
    VRAM: [[u8; X_DIM]; Y_DIM],
    update_needed: bool,
}

impl DisplayManager {
    /// Creates a new `DisplayManager` instance.
    ///
    /// # Arguments
    ///
    /// * `sdl_context` - A reference to an initialized SDL context.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL2 fails to get the video subsystem or create the window or canvas.
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| format!("Failed to get SDL2 video subsystem: {}", e))?;

        let window = video_subsystem
            .window(WINDOW_TITLE, (X_DIM * SCALE) as u32, (Y_DIM * SCALE) as u32)
            .position_centered()
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| format!("Failed to create canvas: {}", e))?;

        canvas
            .set_scale(SCALE as f32, SCALE as f32)
            .map_err(|e| format!("Failed to set scale: {}", e))?;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Ok(DisplayManager {
            canvas: Some(canvas),
            VRAM: [[0; X_DIM]; Y_DIM],
            update_needed: false,
        })
    }

    /// Returns the height of the display.
    pub fn get_height(&self) -> usize {
        Y_DIM
    }

    /// Returns the width of the display.
    pub fn get_width(&self) -> usize {
        X_DIM
    }

    /// Sets the pixel at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the pixel.
    /// * `y` - Y coordinate of the pixel.
    /// * `value` - Value of the pixel.
    ///
    /// # Returns
    ///
    /// Returns 1 if the pixel was already set to the given value, 0 otherwise.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) -> u8 {
        self.update_needed = true;

        let previous_value = self.VRAM[y][x];
        self.VRAM[y][x] ^= value;
        self.draw_pixel(x, y);

        previous_value & value
    }

    /// Clears the display and resets the VRAM.
    pub fn clear(&mut self) {
        self.update_needed = true;
        self.VRAM.iter_mut().for_each(|row| row.fill(0));
        if let Some(canvas) = self.canvas.as_mut() {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
        }
    }

    /// Renders the display by presenting the canvas if any changes were made.
    pub fn render(&mut self) {
        if self.update_needed {
            self.update_needed = false;

            if let Some(canvas) = self.canvas.as_mut() {
                canvas.present();
            }
        }
    }

    /// Draws a single pixel at the given coordinates based on the VRAM content.
    fn draw_pixel(&mut self, x: usize, y: usize) {
        if let Some(canvas) = self.canvas.as_mut() {
            let color = if self.VRAM[y][x] != 0 {
                Color::WHITE
            } else {
                Color::BLACK
            };
            canvas.set_draw_color(color);
            canvas
                .draw_point(Point::new(x as i32, y as i32))
                .expect("Failed to draw point");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_display_manager() -> DisplayManager {
        DisplayManager {
            canvas: None,
            VRAM: [[0; X_DIM]; Y_DIM],
            update_needed: false,
        }
    }

    #[test]
    fn test_get_dimensions() {
        let display_manager = create_test_display_manager();
        assert_eq!(display_manager.get_height(), Y_DIM);
        assert_eq!(display_manager.get_width(), X_DIM);
    }

    #[test]
    fn test_set_pixel() {
        let mut display_manager = create_test_display_manager();
        let x = 10;
        let y = 10;
        let value = 1;

        assert_eq!(display_manager.set_pixel(x, y, value), 0);
        assert_eq!(display_manager.VRAM[y][x], value);

        // Setting the same pixel to the same value should return 1
        assert_eq!(display_manager.set_pixel(x, y, value), 1);
        assert_eq!(display_manager.VRAM[y][x], 0);
    }

    #[test]
    fn test_clear() {
        let mut display_manager = create_test_display_manager();
        display_manager.set_pixel(10, 10, 1);
        display_manager.clear();
        assert!(display_manager
            .VRAM
            .iter()
            .all(|row| row.iter().all(|&pixel| pixel == 0)));
    }

    #[test]
    fn test_render() {
        let mut display_manager = create_test_display_manager();
        display_manager.set_pixel(10, 10, 1);
        display_manager.render();
        assert!(!display_manager.update_needed);
    }
}

