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
    // This field filters the entries using String::contains to only have
    // entries whose strings contiain this String. An empty entry_filter
    // means that the entries won't be filtered.
    //
    // We want to force the selected entries to restart from the first entry
    // if the filter is updated, so we do that by making this field private.
    entry_filter: Option<String>,
    // List of the indices that match the entry_filter.
    // We use this list when looping through and rendering each
    // entry in the select list in the draw function.
    filtered_entry_indices: Vec<usize>,
    // This is actually the selected entry in the *filtered* indices, since
    // it's easier to do the rendering when this is tied to what we're rendering from.
    // When we do an up/down arrow keypress, we can just increment/decrement this
    // instead of having to find where selected_entry's value is in filtered_entry_indices (eg.
    // if you selected the second entry in entries you'd have to find where the index '1' is
    // in filtered_entry_indices, which is annoying *and* inefficient).
    pub selected_entry: usize,
    ctrl_pressed: bool,
}

impl Select {
    pub fn new(entries: Vec<(String, Box<dyn Fn() -> Result<(), String>>)>) -> Self {
        let filtered_entry_indices = (0..entries.len()).collect();
        Select {
            entries,
            filtered_entry_indices,
            entry_filter: None,
            // The first element will always be the one that's selected by default.
            selected_entry: 0,
            ctrl_pressed: false,
        }
    }

    // We use this function to update the list of indices that are rendered.
    // We have a whole self.entries with the big list of entries, but when you
    // filter you need to know what to render. To do this, we use this function
    // to create a list of indices that can be rendered (if there is no filter, the list
    // of indices is a list from zero to 1 minus the length of the entires—the entire list
    // of entries).
    pub fn update_entry_filter(&mut self, entry_filter: Option<String>) {
        // We want to first check if the filter is the Some variant of Option—so we're checking if the
        // value of entry_filter *is* a filter or not. If it's Some, there is a filter, and if it's
        // None, there is no filter.
        if let Some(ref filter) = entry_filter {
            // Restart the selected entries from zero when we start filtering and the old and the
            // new filter are different. We don't want to do this in the future, though.
            if self.entry_filter != entry_filter {
                // TODO enhancement would be to keep the selection on the current
                // item if the item is in the new filtered array
                self.selected_entry = 0;
                // Clear the indices since we're going to "calculate" a new list of filtered indices.
                self.filtered_entry_indices.clear();
                for (i, (entry, _)) in self.entries.iter().enumerate() {
                    // We're checking if the string we search for is within any of the entries' strings.
                    // It's quite basic, but for a simple application launcher, this is probably all we'll
                    // need.
                    if entry.contains(filter.as_str()) {
                        self.filtered_entry_indices.push(i);
                    }
                }
            }
        }
        // We want to make the list of indices to render in the list the full thing
        // if there is no filter. In other words, show everything.
        else if let None = entry_filter {
            self.filtered_entry_indices = (0..self.entries.len()).collect();
        }
        // Finally, we
        self.entry_filter = entry_filter;
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
                        .get(self.filtered_entry_indices[self.selected_entry])
                        .expect("Couldn't fine call back for selected entry!");

                    if let Err(msg) = callback() {
                        warn!(
                            "Callback for selected entry returned error message '{}'",
                            msg
                        );
                    }
                }
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
                // Handle up/down arrow keys (and we'll also add ctrl-n and ctrl-p for
                // Emacs-esque handling here). We can use these to change
                // the entry that's currently selected.
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state: ButtonState::Press,
                    ..
                }) => {
                    // We match two keycodes to go up an entry: one is an up arrow key, as normal,
                    // and the other is Ctrl-P, which is how you go up a line in Emacs (or even
                    // Bash, but that's more "go up to the previous command").
                    if !self.filtered_entry_indices.is_empty() {
                        if ((*key == Key::P && self.ctrl_pressed)
                        || *key == Key::Up) &&
                    // Obviously, we don't want to let the user go up
                    // beyond the first entry, so we have this condition.
                        self.selected_entry > 0
                        {
                            self.selected_entry -= 1;
                        }
                        // We again do the same kind of keyboard-matching for the
                        // obvious down-arrow keypress. The Emacs/Bash equivalent is Ctrl-N, so
                        // we match that as well.
                        else if ((*key == Key::N && self.ctrl_pressed)
                        || *key == Key::Down) &&
                    // Same thing here—don't let the user go past the
                    // last entry.
                        self.selected_entry < self.filtered_entry_indices.len() - 1
                        {
                            self.selected_entry += 1;
                        }
                    }
                }

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

        // Handle overflow. When the selected index goes past the page, we have to get rid of the
        // first 7 items and replace those with the next seven, and move the cursor to the top
        // of the page. At least that's what Rofi does.
        //
        // We'll figure out how many entries to skip.
        //
        // By doing some integer division here, we only get skip indices every 7 entries,
        // which is exactly what we want.
        let start_entries = (self.selected_entry / MAX_ENTRIES as usize) * MAX_ENTRIES as usize;

        for (index, entry_index) in self
            .filtered_entry_indices
            .iter()
            .skip(start_entries)
            .take(MAX_ENTRIES as usize)
            // .filter(|entry| {
            //     if let Some(filter) = &self.entry_filter {
            //         entry.0.contains(filter.as_str())
            //     } else {
            //         true
            //     }
            // })
            .enumerate()
        {
            // We get the entry from the index that's stored in the filtered_indices.
            let entry = &self.entries[*entry_index].0;
            // We add start_entries to index since the index from the iterator isn't
            // actually the index in the entry array; the iterator cuts out parts
            // of the entry when iterating, so its zero-index is different.
            //
            // It's easier to store this in a variable since we reuse this value multiple times.
            let selected_entry = start_entries + index == self.selected_entry;

            // Move the index so that we actually start at the right index
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
            if selected_entry {
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
                if selected_entry {
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
                &entry,
                glyph_cache,
                &DrawState::default(),
                c.transform.trans(entry_text_xpos, entry_text_ypos),
                g,
            )
            .unwrap();
        }
    }
}
