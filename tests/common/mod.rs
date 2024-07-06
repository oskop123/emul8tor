use std::sync::Mutex;
use std::sync::Once;

static INIT: Once = Once::new();
static mut SDL_CONTEXT: Option<Mutex<sdl2::Sdl>> = None;

pub fn setup() -> &'static Mutex<sdl2::Sdl> {
    unsafe {
        INIT.call_once(|| {
            let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
            SDL_CONTEXT = Some(Mutex::new(sdl_context));
        });
        SDL_CONTEXT.as_ref().expect("SDL_CONTEXT not initialized")
    }
}
