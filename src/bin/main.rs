#![feature(destructuring_assignment)]

use ehound::{update::Manager, TEST};
use graphics::clear;
use piston_window::{
    AdvancedWindow,
    Button,
    EventLoop,
    Key,
    MouseCursorEvent,
    MouseScrollEvent,
    OpenGL,
    PistonWindow,
    PressEvent,
    ResizeEvent,
    Size,
    Window,
    WindowSettings,
};
use sdl2::video::FullscreenType;
use sdl2_window::Sdl2Window;

#[allow(unused_variables, unused_assignments)]
#[tokio::main]
async fn main() {
    let gl = OpenGL::V4_5;
    // let gl = Version::vulkan(1, 2);
    let mut window: PistonWindow<Sdl2Window> =
        WindowSettings::new("Downloader", [1., 1.])
            .exit_on_esc(true)
            .samples(16)
            .vsync(false)
            .graphics_api(gl)
            .build()
            .expect("Couldn't create a window");
    window.set_capture_cursor(false);
    window.set_max_fps(60);
    window.set_ups(30);
    let mut ar = 0.;
    let mut width = 0.;
    let mut height = 0.;
    let mut cursor = [0.; 2];
    #[allow(unused_mut)]
    let mut ctx = window.create_texture_context();

    let mut manager = Manager::default();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _device| {
            clear([0.0; 4], g);
            // app.draw(c, g, None);
        });
        if let Some(_) = e.resize_args() {
            Size { width, height } = window.window.draw_size();
            ar = width / height;
        }
        if let Some(pos) = e.mouse_cursor(|xy| xy) {
            cursor = pos;
        };
        e.mouse_scroll(|d| {
            d[1];
        });
        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::R => println!("{}", manager.refresh().await),
                    Key::E => {
                        manager.add_book(None, TEST.to_string().into()).await
                    }
                    Key::Q => break,
                    Key::F | Key::F12 => fullscreen(&mut window),
                    _ => {}
                }
            }
        }
    }
}
fn fullscreen(window: &mut PistonWindow<Sdl2Window>) {
    match window.window.window.fullscreen_state() {
        FullscreenType::Off => {
            &window.window.window.set_fullscreen(FullscreenType::Desktop)
        }
        FullscreenType::True => {
            &window.window.window.set_fullscreen(FullscreenType::Desktop)
        }
        FullscreenType::Desktop => {
            &window.window.window.set_fullscreen(FullscreenType::Off)
        }
    };
}
