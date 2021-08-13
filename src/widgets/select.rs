/* SPDX-License-Identifier: Zlib */

use crate::configuration::{BACKGROUND_COLOR, FOREGROUND_COLOR};
use crate::widgets::Widget;
use gfx_device_gl::Resources;
use log::{debug, warn};
use piston_window::*;

pub struct Select {
    // These are the entries. The select box will
    // render these entry strings. When you press "Enter"
    // the callback function (closure) here will be called.
    pub entries: Vec<(String, Box<dyn Fn() -> Result<(), String>>)>,
    pub selected_entry: usize,
    ctrl_pressed: bool,
}

impl Select {
    pub fn new() -> Self {
        Select {
            entries: vec![],
            // The first element will always be the one that's selected by default.
            selected_entry: 0,
            ctrl_pressed: false,
        }
    }
}

impl Widget for Select {
    fn handle_event(&mut self, ev: &piston_window::Event) {
        if let Event::Input(input, _) = ev {
            match input {
                // Handle enter---execute the callback for the selected entry
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(Key::Return),
                    state: ButtonState::Press,
                    ..
                }) => {
                    // We use get/expect to deliver our custom error message. Anyway, we can panic
                    // here since the expect should never happen.
                    let (_, callback) = self
                        .entries
                        .get(self.selected_entry)
                        .expect("Couldn't fine call back for selected entry!");

                    if let Err(msg) = callback() {
                        warn!(
                            "Callback for selected entry returned error message '{}'",
                            msg
                        );
                    }
                }
                // Handle up/down arrow keys (and we'll also add ctrl-n and ctrl-p for
                // Emacs-esque handling here). We can use these to change
                // the entry that's currently selected.
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state: ButtonState::Press,
                    ..
                }) => match *key {
                    // Obviously, we don't want to let the user go up
                    // beyond the first entry!
                    Key::Up if self.selected_entry > 0 => {
                        self.selected_entry -= 1;
                    }
                    // Same thing here—don't let the user go past the
                    // last entry.
                    Key::Down if self.selected_entry != self.entries.len() - 1 => {
                        self.selected_entry += 1;
                    }
                    _ => {}
                },

                // Okay, here, I'm not actually too keen on coyping this code
                // to keep things dry. I suppose I might write some kind of
                // universal control handler in the future.
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state,
                    ..
                }) if key == &Key::RCtrl || key == &Key::LCtrl => match state {
                    ButtonState::Press => {
                        self.ctrl_pressed = true;
                    }
                    ButtonState::Release => {
                        self.ctrl_pressed = false;
                    }
                },
                _ => {}
            }
        }
    }
    fn draw<G>(&self, coords: [f64; 2], c: &Context, g: &mut G, glyph_cache: &mut Glyphs)
    where
        G: Graphics<Texture = Texture<Resources>>,
    {
        const RECT_HEIGHT: f64 = 260.0;
        const RECT_WIDTH: f64 = 700.0;

        Rectangle::new_border([1.0, 1.0, 1.0, 1.0], 1.0)
            .color([0.0, 0.0, 0.0, 0.0])
            .draw(
                [coords[0], coords[1], RECT_WIDTH, RECT_HEIGHT],
                &Default::default(),
                c.transform,
                g,
            );

        // TODO we will make the maximum number of entries configurable.
        const MAX_ENTRIES: f64 = 7.0; // Use an f64 to do simpler arithmetic later.
        const ENTRY_HEIGHT: f64 = RECT_HEIGHT / MAX_ENTRIES;

        // See my reasoning in search.rs to understanding why I used the letter 'A.'
        // I was going to use lazy_static to reuse this value between here and search.rs, but
        // it's not a great idea in my opinion, since the font sizes between here and
        // the search widget ~~may~~ differ, so it's easier to do just
        // calculate the character width again.
        const LISTING_FONTSIZE: u32 = (RECT_HEIGHT / MAX_ENTRIES / 2.0) as u32;
        let char_height = glyph_cache
            .character(LISTING_FONTSIZE, 'A')
            .expect("Failed to get max char height to vertically center text in the window!")
            .top();

        // We loop through each entry here to render the list.
        // TODO handle an overflowing number of entries (pagination) using the
        // self.selected_entry value to figure out where to start.
        // That also means we don't have to iter through all of the entries
        // on each draw.
        for (index, (entry, callback)) in self.entries.iter().enumerate() {
            // We'll calculate the y-coordinates of the line since we'll use that to
            // calculate where to position text.
            let entry_line_ypos = coords[1] + (ENTRY_HEIGHT * ((index + 1) as f64));

            // Draw line
            line(
                FOREGROUND_COLOR,
                1.0,
                [
                    // The line should span the entire stretch of the window.
                    coords[0],
                    // Y-position of the line is 1/8th of the height of the window.
                    entry_line_ypos,
                    coords[0] + RECT_WIDTH,
                    entry_line_ypos,
                ],
                c.transform,
                g,
            );

            // We're going to invert the color of the text and the entry if this is the
            // selected entry. To invert the color of the entry, we use a rectangle.
            if index == self.selected_entry {
                rectangle(
                    FOREGROUND_COLOR,
                    [
                        coords[0],
                        // We have to subtract here since the line is the bottom
                        // right of the "rectangle," but the rectangle here is
                        // drawn from the top-left.
                        entry_line_ypos - ENTRY_HEIGHT,
                        RECT_WIDTH,
                        ENTRY_HEIGHT,
                    ],
                    c.transform,
                    g,
                );
            }

            // Just like the search bar, we want to be 15 pixels from the left edge
            // of the box we're drawing in.
            // The y-position is calculated also like with the search bar, but
            // we're going off of different coordinates (bottom-left as reference),
            // so we'll change up the calculation for that.
            // (Haha, trying to appear like I'm following DRY
            // by changing text_xpos to entry_text_xpos but this code is not
            // very DRY-esque here)
            let entry_text_xpos = coords[0] + 15.0;
            let entry_text_ypos = entry_line_ypos - (ENTRY_HEIGHT / 2.0) + (char_height / 2.0);

            text::Text::new_color(
                if index == self.selected_entry {
                    // We make the text the colour of the background when it's selected,
                    // since when it's selected, the entry will be white and white text
                    // on a white background won't be visible.
                    BACKGROUND_COLOR
                } else {
                    FOREGROUND_COLOR
                },
                LISTING_FONTSIZE as u32,
            )
            .draw(
                entry,
                glyph_cache,
                &DrawState::default(),
                c.transform.trans(entry_text_xpos, entry_text_ypos),
                g,
            )
            .unwrap();
        }
    }
}
