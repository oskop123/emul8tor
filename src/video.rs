use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::error::Error;

const X_DIM_LORES: usize = 64;
const Y_DIM_LORES: usize = 32;

const X_DIM_HIRES: usize = 128;
const Y_DIM_HIRES: usize = 64;

const WINDOW_TITLE: &str = "emul8tor";

/// Resolution modes.
pub enum Resolution {
    Low,
    High,
}

/// Manages display rendering using SDL2.
#[allow(non_snake_case)]
pub struct DisplayManager {
    canvas: Option<Canvas<Window>>,
    VRAM: Vec<Vec<u8>>,
    update_needed: bool,
}

impl DisplayManager {
    /// Creates a new `DisplayManager` instance.
    ///
    /// # Arguments
    ///
    /// * `sdl_context` - A reference to an initialized SDL context.
    /// * `resolution` - A selected resolution mode.
    /// * `scale` - A display scaling factor.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL2 fails to get the video subsystem or create the window or canvas.
    pub fn new(
        sdl_context: &sdl2::Sdl,
        resolution: Resolution,
        scale: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| format!("Failed to get SDL2 video subsystem: {}", e))?;

        let (x_dim, y_dim, window_scale) = match resolution {
            Resolution::Low => (X_DIM_LORES, Y_DIM_LORES, scale),
            Resolution::High => (X_DIM_HIRES, Y_DIM_HIRES, scale / 2),
        };

        let window = video_subsystem
            .window(
                WINDOW_TITLE,
                (x_dim * scale) as u32,
                (y_dim * window_scale) as u32,
            )
            .position_centered()
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| format!("Failed to create canvas: {}", e))?;

        canvas
            .set_scale(scale as f32, scale as f32)
            .map_err(|e| format!("Failed to set scale: {}", e))?;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        #[allow(non_snake_case)]
        let VRAM = match resolution {
            Resolution::Low => vec![vec![0; X_DIM_LORES]; Y_DIM_LORES],
            Resolution::High => vec![vec![0; X_DIM_HIRES]; Y_DIM_HIRES],
        };

        Ok(DisplayManager {
            canvas: Some(canvas),
            VRAM,
            update_needed: false,
        })
    }

    /// Returns the height of the display.
    pub fn height(&self) -> usize {
        self.VRAM.len()
    }

    /// Returns the width of the display.
    pub fn width(&self) -> usize {
        self.VRAM[0].len()
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
        self.draw_pixel(x, y, previous_value ^ value);

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

    /// Scrolls the display down.
    ///
    /// # Arguments
    ///
    /// * `rows` - Number of pixel rows to scroll.
    pub fn scroll_down(&mut self, rows: usize) {
        let width = self.width();
        let height = self.height();

        // Move each row n rows down
        if rows < height {
            for y in (rows..height).rev() {
                for x in 0..width {
                    self.draw_pixel(x, y, self.VRAM[y - rows][x]);
                }
            }
        }

        // Clear the top n rows
        for y in 0..rows {
            for x in 0..width {
                self.draw_pixel(x, y, 0);
            }
        }
    }

    /// Scrolls the display up.
    ///
    /// # Arguments
    ///
    /// * `rows` - Number of pixel rows to scroll.
    pub fn scroll_up(&mut self, rows: usize) {
        let width = self.width();
        let height = self.height();

        // Move each row n rows up
        if rows < height {
            for y in 0..height - rows {
                for x in 0..width {
                    self.draw_pixel(x, y, self.VRAM[y + rows][x]);
                }
            }
        }

        // Clear the bottom n rows
        for y in height - rows..height {
            for x in 0..width {
                self.draw_pixel(x, y, 0);
            }
        }
    }

    /// Scrolls the display to the right by 4 pixels.
    pub fn scroll_right(&mut self) {
        let width = self.width();
        let height = self.height();

        // Move each column 4 pixels to the right
        for y in 0..height {
            for x in (4..width).rev() {
                self.draw_pixel(x, y, self.VRAM[y][x - 4]);
            }
            // Clear the left 4 pixels of each row
            for x in 0..4 {
                self.draw_pixel(x, y, 0);
            }
        }
    }

    /// Scrolls the display to the left by 4 pixels.
    pub fn scroll_left(&mut self) {
        let width = self.width();
        let height = self.height();

        // Move each column 4 pixels to the left
        for y in 0..height {
            for x in 0..width - 4 {
                self.draw_pixel(x, y, self.VRAM[y][x + 4]);
            }
            // Clear the right 4 pixels of each row
            for x in width - 4..width {
                self.draw_pixel(x, y, 0);
            }
        }
    }

    /// Draws a single pixel at the given coordinates based on the VRAM content.
    fn draw_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.VRAM[y][x] = value;
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
            VRAM: vec![vec![0; X_DIM_LORES]; Y_DIM_LORES],
            update_needed: false,
        }
    }

    #[test]
    fn test_get_dimensions() {
        let display_manager = create_test_display_manager();
        assert_eq!(display_manager.height(), Y_DIM_LORES);
        assert_eq!(display_manager.width(), X_DIM_LORES);
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

    #[test]
    fn test_scroll_down() {
        let mut display_manager = create_test_display_manager();

        // Set the first row to all 1s
        for x in 0..display_manager.width() {
            display_manager.set_pixel(x, 0, 1);
        }

        display_manager.scroll_down(1);

        // Ensure the second row is now all 1s and the first row is all 0s
        for x in 0..display_manager.width() {
            assert_eq!(display_manager.VRAM[1][x], 1);
            assert_eq!(display_manager.VRAM[0][x], 0);
        }
    }

    #[test]
    fn test_scroll_up() {
        let mut display_manager = create_test_display_manager();

        // Set the last row to all 1s
        let last_row = display_manager.height() - 1;
        for x in 0..display_manager.width() {
            display_manager.set_pixel(x, last_row, 1);
        }

        display_manager.scroll_up(1);

        // Ensure the second to last row is now all 1s and the last row is all 0s
        for x in 0..display_manager.width() {
            assert_eq!(display_manager.VRAM[last_row - 1][x], 1);
            assert_eq!(display_manager.VRAM[last_row][x], 0);
        }
    }

    #[test]
    fn test_scroll_right() {
        let mut display_manager = create_test_display_manager();

        // Set the first column to all 1s
        for y in 0..display_manager.height() {
            display_manager.set_pixel(0, y, 1);
        }

        display_manager.scroll_right();

        // Ensure the fifth column is now all 1s and the first four columns are all 0s
        for y in 0..display_manager.height() {
            assert_eq!(display_manager.VRAM[y][4], 1);
            for x in 0..4 {
                assert_eq!(display_manager.VRAM[y][x], 0);
            }
        }
    }

    #[test]
    fn test_scroll_left() {
        let mut display_manager = create_test_display_manager();

        // Set the last column to all 1s
        let last_col = display_manager.width() - 1;
        for y in 0..display_manager.height() {
            display_manager.set_pixel(last_col, y, 1);
        }

        display_manager.scroll_left();

        // Ensure the last column minus 4 is now all 1s and the last four columns are all 0s
        for y in 0..display_manager.height() {
            assert_eq!(display_manager.VRAM[y][last_col - 4], 1);
            for x in last_col - 3..=last_col {
                assert_eq!(display_manager.VRAM[y][x], 0);
            }
        }
    }
}
