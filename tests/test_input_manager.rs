use emul8tor::input;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

mod common;

#[test]
fn test_update_key_down() {
    let sdl_context = common::setup().lock().expect("Failed to lock SDL_CONTEXT");
    let mut input_manager = input::InputManager::new(&sdl_context).unwrap();

    // Simulate key down event
    sdl_context
        .event()
        .unwrap()
        .push_event(Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: None,
            scancode: Some(Scancode::Num1),
            repeat: false,
            keymod: sdl2::keyboard::Mod::empty(),
        })
        .unwrap();

    input_manager.update();

    assert!(input_manager.is_key_pressed(0x1));
}

#[test]
fn test_update_key_up() {
    let sdl_context = common::setup().lock().expect("Failed to lock SDL_CONTEXT");
    let mut input_manager = input::InputManager::new(&sdl_context).unwrap();

    input_manager.get_next_released_key();

    // Simulate key down event
    sdl_context
        .event()
        .unwrap()
        .push_event(Event::KeyDown {
            timestamp: 0,
            window_id: 0,
            keycode: None,
            scancode: Some(Scancode::Num1),
            repeat: false,
            keymod: sdl2::keyboard::Mod::empty(),
        })
        .unwrap();
    input_manager.update();

    // Simulate key up event
    sdl_context
        .event()
        .unwrap()
        .push_event(Event::KeyUp {
            timestamp: 0,
            window_id: 0,
            keycode: None,
            scancode: Some(Scancode::Num1),
            repeat: false,
            keymod: sdl2::keyboard::Mod::empty(),
        })
        .unwrap();
    input_manager.update();

    assert!(!input_manager.is_key_pressed(0x1));
    assert_eq!(input_manager.get_next_released_key(), Some(0x1));
}

#[test]
fn test_should_quit() {
    let sdl_context = common::setup().lock().expect("Failed to lock SDL_CONTEXT");
    let mut input_manager = input::InputManager::new(&sdl_context).unwrap();

    // Simulate quit event
    sdl_context
        .event()
        .unwrap()
        .push_event(Event::Quit { timestamp: 0 })
        .unwrap();

    input_manager.update();

    assert!(input_manager.should_quit());
}
