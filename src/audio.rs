use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub struct AudioManager {
    device: AudioDevice<SquareWave>,
}

impl AudioManager {
    /// Creates a new AudioManager instance, initializing SDL2 and setting up the audio device.
    /// Panics if SDL2 initialization or audio device creation fails.
    pub fn new() -> Self {
        let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
        let audio_subsystem = sdl_context
            .audio()
            .expect("Failed to get SDL2 audio subsystem");

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                SquareWave::new(440.0, 0.25, spec.freq as f32)
            })
            .expect("Failed to open audio playback device");

        AudioManager { device }
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

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SquareWave {
    /// Creates a new SquareWave instance with the specified frequency, volume, and sample rate.
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

    /// Generates a square wave and fills the output buffer with audio data.
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
