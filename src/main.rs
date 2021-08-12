/* SPDX-License-Identifier: Zlib */

use glutin::{
    dpi::{PhysicalPosition, Position},
    event_loop::EventLoop,
};
use glutin_window::GlutinWindow;
use piston_window::*;
use winit::{
    dpi::LogicalSize,
    platform::unix::{x11, WindowBuilderExtUnix, WindowExtUnix},
    window::WindowBuilder,
};

use fontconfig::Fontconfig;

use log::{debug, error, info, trace, warn};

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 500;
    flexi_logger::Logger::try_with_env()
        .unwrap()
        .start()
        .unwrap();

    let eventloop = EventLoop::with_user_event();
    let window_builder = WindowBuilder::new()
        // This is the magic setting that lets the window float like how you see in rofi
        .with_override_redirect(true)
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));

    // The WIDTH, HEIGHT here doesn't matter, so we set it above with with_inner_size.
    let window_settings = WindowSettings::new("WMController", [WIDTH, HEIGHT])
        .decorated(false)
        .exit_on_esc(true)
        .resizable(false);

    let gw: GlutinWindow =
        GlutinWindow::from_raw(&window_settings, eventloop, window_builder).unwrap();

    // Center the window

    {
        // The thought was putting this in a separate scope might save a hair of memory

        // This captures keyboard input so we can actually do
        // something useful with the window (eg. text input and whatnot)
        // Idea stolen from https://github.com/seanpringle/simpleswitcher/blob/master/simpleswitcher.c
        let window_ref = gw.ctx.window();
        unsafe {
            let xconn = window_ref.xlib_xconnection().unwrap();
            ((*xconn).xlib.XGrabKeyboard)(
                window_ref.xlib_display().unwrap() as *mut x11::ffi::_XDisplay,
                window_ref.xlib_window().unwrap(),
                x11::ffi::True,
                x11::ffi::GrabModeAsync,
                x11::ffi::GrabModeAsync,
                x11::ffi::CurrentTime,
            );
        }

        if let Some(monitor) = window_ref.current_monitor() {
            let screen_size = monitor.size();
            let window_size = window_ref.inner_size();
            debug!("Size of screen is {:?}", screen_size);
            debug!("Size of window is {:?}", window_size);

            window_ref.set_outer_position(Position::Physical(PhysicalPosition {
                x: (screen_size.width / 2 - window_size.width / 2) as i32,
                y: (screen_size.height / 2 - window_size.height / 2) as i32,
            }));
        }
    }

    // Stolen from https://github.com/PistonDevelopers/piston_window/blob/master/src/lib.rs
    let api = window_settings
        .get_maybe_graphics_api()
        .unwrap_or(Api::opengl(3, 2));
    let samples = window_settings.get_samples();

    let opengl = OpenGL::from_api(api).unwrap();
    let mut window = PistonWindow::new(opengl, samples, gw);

    // Find font for window—panic is fine here
    // TODO let users specify their own font (or me if I want to change it)
    let fc = Fontconfig::new().unwrap();
    let font = fc
        .find("JetBrains Mono", None)
        .expect("Failed to find the JetBrains Mono font!");
    let mut glyph_cache = window
        // TODO We want to end up using some kind of font loader here
        // so we can specify the font family/name and it finds the tttf
        .load_font(font.path)
        .unwrap();

    while let Some(e) = window.next() {
        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g, device| {
                clear([0.0, 72.0 / 255.0, 71.0 / 255.0, 1.0], g);
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 60)
                    .draw(
                        "Applications",
                        &mut glyph_cache,
                        &DrawState::default(),
                        c.transform.trans(40.0, 100.0),
                        g,
                    )
                    .unwrap();
                Rectangle::new_border([1.0, 1.0, 1.0, 1.0], 1.0)
                    .color([0.0, 0.0, 0.0, 0.0])
                    .draw(
                        [40.0, 140.0, 700.0, 40.0],
                        &Default::default(),
                        c.transform,
                        g,
                    );
                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 20)
                    .draw(
                        "Search",
                        &mut glyph_cache,
                        &DrawState::default(),
                        c.transform.trans(55.0, 167.0),
                        g,
                    )
                    .unwrap();

                glyph_cache.factory.encoder.flush(device);
            });
        }
    }
}
