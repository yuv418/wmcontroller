/* SPDX-License-Identifier: Zlib */

// Technically I can just import FOREGROUND_COLOR from main.rs (root of crate), but I feel better about this.
use crate::configuration::FOREGROUND_COLOR;
use core::iter::FromIterator;
use piston_window::*;

use crate::widgets::Widget;
use log::debug;

pub struct Search {
    pub buffer: String,
    // We are going to implement modal editing in the application launcher
    pub insert_mode: bool,
    // We want to display the placeholder "Search" text until the first keypress,
    // so we have this boolean to check whether or not to replace that placeholder.
    events_run: bool,
    ctrl_pressed: bool,
}
impl Search {
    pub fn new() -> Self {
        Search {
            buffer: "".to_string(),
            insert_mode: false,
            events_run: false,
            ctrl_pressed: false,
        }
    }
}
impl Widget for Search {
    // Function to draw the search bar on the screen
    fn draw<G>(&self, coords: [f64; 2], c: &Context, g: &mut G, glyph_cache: &mut Glyphs)
    where
        G: Graphics<Texture = Texture<gfx_device_gl::Resources>>,
    {
        const RECT_HEIGHT: f64 = 40.0;

        // TODO we want to actually be able to set the width of the text box rectangle on the fly
        // or as an argument to this draw function (we wouldn't want it to be a struct param)
        // since we want people to be able to change details about the drawing independently
        // of the state that is stored in this struct.
        Rectangle::new_border(FOREGROUND_COLOR, 1.0)
            .color([0.0, 0.0, 0.0, 0.0])
            .draw(
                // TODO calculate RECT_WIDTH (700.0) from the window size.
                [coords[0], coords[1], 700.0, RECT_HEIGHT],
                &Default::default(),
                c.transform,
                g,
            );

        const SEARCH_FONTSIZE: f64 = RECT_HEIGHT / 2.0;

        // The text to display/use to calculate cursor position
        let render_text = if self.events_run {
            &self.buffer
        } else {
            "Search"
        };

        // I found that 'C' was the tallest of the characters, but that
        // made the text not look vertically cenetered in a rectangle, so we
        // want a slightly shorter character. We can use that to find the
        // "max" (ish) height that our text will be… although this is still a hack.
        let char_height = glyph_cache
            .character(SEARCH_FONTSIZE as u32, 'A')
            .expect("Failed to get max char height to vertically center text in the window!")
            .top();

        let text_xpos = coords[0] + 15.0;
        // We use our character height from before to calculate where to put our text.
        // The point we need is actually the bottom left of the text, so what we can do is
        let text_ypos = coords[1] + (RECT_HEIGHT / 2.0) + (char_height / 2.0);

        text::Text::new_color(FOREGROUND_COLOR, SEARCH_FONTSIZE as u32 * 2)
            .draw(
                render_text,
                glyph_cache,
                &DrawState::default(),
                c.transform.trans(text_xpos, text_ypos).zoom(0.5),
                g,
            )
            .unwrap();

        // Loop through all the characters to find the width of all the text.
        // This is for determining cursor height.
        let mut cursor_offset = 0.0;
        for ch in render_text.chars() {
            let ch = glyph_cache
                .character(SEARCH_FONTSIZE as u32, ch)
                .expect("Failed to get size of character to display the cursor!");
            cursor_offset += ch.advance_width();
        }

        // Calculate the width of the text we're rendering so we know where to put the cursor.
        // We don't render the cursor until we start populating the buffer
        if self.events_run {
            line(
                FOREGROUND_COLOR,
                1.0,
                [
                    // Add 5 pixels so we can have a nice comfortable gap
                    // and the cursor isn't pressed up against the last character
                    // in the buffer
                    text_xpos + cursor_offset + 5.0,
                    text_ypos - (SEARCH_FONTSIZE * 0.85),
                    text_xpos + cursor_offset + 5.0,
                    text_ypos + 2.0,
                ],
                c.transform,
                g,
            );
        }
    }

    fn handle_event(&mut self, ev: &Event) {
        // We can get a string from the input very easily—no need to handle shift
        // and all those messy thiings
        if let Event::Input(inp, _) = ev {
            match inp {
                // We don't want to add the character to the text box if control is pressed,
                // since we'll use Ctrl + key combinations for manipulating text in the text box.
                Input::Text(add_string) if add_string != "" && !self.ctrl_pressed => {
                    debug!("Add string is {}", add_string);
                    self.buffer += add_string;
                    self.events_run = true;
                }

                // This is for backspace (and ctrl-backspace since I'll implement that as well
                // events.
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(Key::Backspace),
                    state: ButtonState::Press,
                    ..
                }) => {
                    if !self.buffer.is_empty() {
                        // We could probably consolidate this to get rid of the next_back twice
                        // since both of these are double-ended iterators.

                        // Use generics to save code. Both removals use a
                        // DoubleEndedIterator, so we can captialize on that
                        fn remove_last_iter_to_str<D>(mut de_iter: D) -> String
                        where
                            D: DoubleEndedIterator,
                            String: FromIterator<<D as Iterator>::Item>,
                        {
                            de_iter.next_back();
                            de_iter.collect()
                        }
                        self.buffer = if self.ctrl_pressed {
                            // Take all the words except the last.                             let split_buffer =
                            remove_last_iter_to_str(
                                self.buffer
                                    .split_whitespace()
                                    .map(|word| word.to_owned() + " "),
                            )
                        } else {
                            remove_last_iter_to_str(self.buffer.chars())
                        };

                        // We're going to reset the search bar if the string
                        // becomes blank again through backspacing
                        if self.buffer.is_empty() {
                            self.events_run = false;
                        }
                    }
                }
                // We want to be able to manipulate the text in our buffer more easily
                // This would do things like move cursor position and whatnot.
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state: ButtonState::Press,
                    ..
                }) if self.ctrl_pressed => match key {
                    Key::K => self.buffer.clear(),
                    _ => {}
                },
                // For control-backspace, which I absolutely cannot live without.
                // We set ctrl_pressed so we can use it in conjunction with the backspace
                // press to figure out when ctrl-backspace occurs.
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
}
