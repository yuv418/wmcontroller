use glutin::dpi::{PhysicalPosition, Position};
use glutin::event_loop::EventLoop;
use glutin_window::GlutinWindow;
use piston_window::*;
use winit::platform::unix::WindowBuilderExtUnix;
use winit::window::WindowBuilder;

fn main() {
    let eventloop = EventLoop::with_user_event();
    let window_builder = WindowBuilder::new()
        // This is the magic setting that lets the window float like how you see in rofi
        .with_override_redirect(true);

    let window_settings = WindowSettings::new("WMController", [300, 300])
        .decorated(false)
        .exit_on_esc(true)
        .resizable(false);

    let gw: GlutinWindow =
        GlutinWindow::from_raw(&window_settings, eventloop, window_builder).unwrap();

    // Center the window

    {
        // The thought was putting this in a separate scope might save a hair of memory
        let window_ref = gw.ctx.window();
        if let Some(monitor) = window_ref.current_monitor() {
            let screen_size = monitor.size();
            let window_size = window_ref.inner_size();

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

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            rectangle(
                [0.0, 0.0, 1.0, 1.0],
                [0.0, 0.0, 100.0, 100.0],
                c.transform,
                g,
            );
        });
    }
}
