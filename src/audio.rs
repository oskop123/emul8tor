use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::error::Error;

/// Manages audio playback using SDL2.
pub struct AudioManager {
    device: AudioDevice<SquareWave>,
}

impl AudioManager {
    /// Creates a new `AudioManager` instance.
    ///
    /// # Arguments
    ///
    /// * `sdl_context` - A reference to an initialized SDL context.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL2 fails to get the audio subsystem or open the playback device.
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, Box<dyn Error>> {
        let audio_subsystem = sdl_context
            .audio()
            .map_err(|e| format!("Failed to get SDL2 audio subsystem: {}", e))?;

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                SquareWave::new(440.0, 0.25, spec.freq as f32)
            })
            .map_err(|e| format!("Failed to open audio playback device: {}", e))?;

        Ok(AudioManager { device })
    }

    /// Starts the audio playback.
    pub fn start(&self) {
        self.device.resume();
    }

    /// Stops the audio playback.
    pub fn stop(&self) {
        self.device.pause();
    }
}

/// Generates a square wave for audio playback.
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SquareWave {
    /// Creates a new `SquareWave` instance.
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of the square wave.
    /// * `volume` - Volume of the square wave.
    /// * `sample_rate` - Sample rate of the audio playback.
    fn new(freq: f32, volume: f32, sample_rate: f32) -> Self {
        SquareWave {
            phase_inc: freq / sample_rate,
            phase: 0.0,
            volume,
        }
    }
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    /// Fills the output buffer with audio data.
    ///
    /// Generates a square wave and writes it to the output buffer.
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::audio::AudioStatus;
    use std::sync::Mutex;
    use std::sync::Once;

    static INIT: Once = Once::new();
    static mut SDL_CONTEXT: Option<Mutex<sdl2::Sdl>> = None;

    fn setup() -> &'static Mutex<sdl2::Sdl> {
        unsafe {
            INIT.call_once(|| {
                let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
                SDL_CONTEXT = Some(Mutex::new(sdl_context));
            });
            SDL_CONTEXT.as_ref().expect("SDL_CONTEXT not initialized")
        }
    }

    #[test]
    fn test_audio_manager_creation() {
        let sdl_context = setup().lock().expect("Failed to lock SDL_CONTEXT");
        let audio_manager = AudioManager::new(&sdl_context).expect("Failed to create AudioManager");
        assert_eq!(audio_manager.device.status(), AudioStatus::Paused);
    }

    #[test]
    fn test_audio_manager_start_stop() {
        let sdl_context = setup().lock().expect("Failed to lock SDL_CONTEXT");
        let audio_manager = AudioManager::new(&sdl_context).expect("Failed to create AudioManager");

        audio_manager.start();
        assert_eq!(audio_manager.device.status(), AudioStatus::Playing);

        audio_manager.stop();
        assert_eq!(audio_manager.device.status(), AudioStatus::Paused);
    }

    #[test]
    fn test_square_wave_callback() {
        let mut square_wave = SquareWave::new(440.0, 0.25, 44100.0);
        let mut buffer = [0.0; 100];

        square_wave.callback(&mut buffer);

        for x in buffer.iter() {
            assert!(*x == 0.25 || *x == -0.25);
        }
    }
}
