/* SPDX-License-Identifier: Zlib */

use piston_window::*;

use log::debug;

pub struct Search {
    pub buffer: String,
    // We are going to implement modal editing in the application launcher
    pub insert_mode: bool,
    // We want to display the placeholder "Search" text until the first keypress,
    // so we have this boolean to check whether or not to replace that placeholder.
    events_run: bool,
    input_shift_enabled: bool,
}

impl Search {
    pub fn new() -> Self {
        Search {
            buffer: "".to_string(),
            insert_mode: false,
            events_run: false,
            input_shift_enabled: false,
        }
    }

    // Function to draw the search bar on the screen
    pub fn draw<G>(&self, coords: [f64; 2], c: &Context, g: &mut G, glyph_cache: &mut Glyphs)
    where
        G: Graphics<Texture = Texture<gfx_device_gl::Resources>>,
    {
        const RECT_HEIGHT: f64 = 40.0;

        Rectangle::new_border([1.0, 1.0, 1.0, 1.0], 1.0)
            .color([0.0, 0.0, 0.0, 0.0])
            .draw(
                [coords[0], coords[1], 700.0, RECT_HEIGHT],
                &Default::default(),
                c.transform,
                g,
            );

        const SEARCH_FONTSIZE: f64 = RECT_HEIGHT / 2.0;
        // TODO we want to actually calculate where the center of the box would be
        text::Text::new_color([1.0, 1.0, 1.0, 1.0], SEARCH_FONTSIZE as u32)
            .draw(
                if self.events_run {
                    &self.buffer
                } else {
                    "Search"
                },
                glyph_cache,
                &DrawState::default(),
                c.transform.trans(
                    coords[0] + 15.0,
                    // Not sure why, but subtracting 6% times this search_fontsize
                    // works consistently
                    coords[1] + (RECT_HEIGHT / 2.0) + (SEARCH_FONTSIZE / 2.0)
                        - ((3.0 / 20.0) * SEARCH_FONTSIZE),
                ),
                g,
            )
            .unwrap();
    }

    pub fn handle_event(&mut self, ev: &Event) {
        // We can get a string from the input very easily—no need to handle shift
        // and all those messy thiings
        if let Event::Input(inp, _) = ev {
            if let Input::Text(add_string) = inp {
                self.buffer += add_string;
                self.events_run = true;
                debug!("self.buffer is now {}", self.buffer);
            }
        }
    }
}
