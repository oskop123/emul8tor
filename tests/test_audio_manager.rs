use emul8tor::audio;
use sdl2::audio::AudioStatus;
mod common;

#[test]
fn test_audio_manager_creation() {
    let sdl_context = common::setup().lock().expect("Failed to lock SDL_CONTEXT");
    let audio_manager =
        audio::AudioManager::new(&sdl_context).expect("Failed to create AudioManager");
    assert_eq!(audio_manager.status(), AudioStatus::Paused);
}

#[test]
fn test_audio_manager_start_stop() {
    let sdl_context = common::setup().lock().expect("Failed to lock SDL_CONTEXT");
    let audio_manager =
        audio::AudioManager::new(&sdl_context).expect("Failed to create AudioManager");

    audio_manager.start();
    assert_eq!(audio_manager.status(), AudioStatus::Playing);

    audio_manager.stop();
    assert_eq!(audio_manager.status(), AudioStatus::Paused);
}
