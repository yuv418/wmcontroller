pub mod search;
pub mod select;

use piston_window::*;
use std::cell::RefCell;

pub trait Widget {
    fn draw<G>(&self, coords: [f64; 2], c: &Context, g: &mut G, glyph_cache: &mut Glyphs)
    where
        G: Graphics<Texture = Texture<gfx_device_gl::Resources>>;
    fn handle_event(&mut self, ev: &Event);
}
