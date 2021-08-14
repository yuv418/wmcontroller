use crate::{configuration::FOREGROUND_COLOR, search::Search, select::Select, widgets::Widget};
use dirs::home_dir;
use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter, PathSource};
use log::debug;
use piston_window::*;
use regex::Regex;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{collections::HashMap, iter::IntoIterator, path::PathBuf};

pub struct ApplicationLauncher {
    search: Search,
    select: Select,
}

impl ApplicationLauncher {
    pub fn new() -> Self {
        // Basically copied from https://crates.io/crates/freedesktop-desktop-entry
        let mut select_entries: HashMap<String, Box<dyn Fn() -> Result<(), String>>> =
            HashMap::new();
        // We want to iter through the local dirs last so they overwrite/override the system .desktop
        // for users that want to overwrite the system .desktop entries. Using a HashMap helps us do this.
        // The library already iters through the local directories first, so we are fine. Otherwise, we'd have to
        // define our own custom list of directories to iter through.
        //
        // TODO handle this so that if they don't have a home directory, only use the system directories (the library
        // panics if that happens).
        //
        // The Exec line of a .desktop entry has a few "field codes" that we aren't going to use (
        // I think they're used for adding command line arguments for files/whatnot). You can see more
        // details here (https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s07.html),
        // but basically this regex pattern will take all those field codes that start with percent
        // and a letter and replace them.
        //
        // We create the regex struct here so that we don't do it every time in the loop; that would
        // be more inefficient.
        let fieldcode_replace_regex = Regex::new("%(f|F|u|U|d|D|n|N|i|k|v|m)").expect(
            "Programmer error in creating regex object to clean Exec line of desktop entry.",
        );

        for (_, path) in Iter::new(default_paths()) {
            debug!("path {:#?}", path);

            if let Ok(bytes) = std::fs::read_to_string(&path) {
                if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                    // We don't want duplicate desktop entries in here, but
                    // we should override system desktop entries with the local ones.
                    //
                    // The library has a default order in which it loops through the desktop entry dirs,
                    // and it does the local sources first, so we can just make it so that once it's done looping through

                    //
                    // TODO locale-changing? It wouldn't be too hard to implement. The
                    // None that we put in entry.name/description is for specifying a locale.
                    //
                    // This variable chooses the string that you'll see in the select menu.
                    let display_name = if let Some(entry_name) = entry.name(None) {
                        entry_name
                    }
                    // Fall back to the application's appid, which should have enough
                    // info to tell someone what they might be running.
                    else {
                        entry.appid
                    };

                    // If the .desktop file doesn't have an Exec field, we
                    // can't launch it. We skip it.
                    if entry.exec().is_none() {
                        continue;
                    }

                    // We replace the field codes in the Exec field as described above.
                    // TODO handle Exec fields with quotes in them.
                    let exec_string = entry.exec().unwrap().to_owned();
                    let exec_string = fieldcode_replace_regex
                        .replace(&exec_string, "")
                        .into_owned();

                    select_entries.insert(
                        display_name.to_string(),
                        Box::new(move || {
                            debug!("exec is {:?}", exec_string);
                            // We are going to call execvp(3) using the nix crate
                            // to replace this process with the application the user
                            // selected.

                            let mut exec_string = exec_string.split_whitespace();
                            Command::new(
                                // If the expect/panic runs, that means the Exec is malformed (empty string?) We
                                // probably *don't* want to panic on this (later), but I'm lazy.
                                exec_string.next().expect(
                                    ".desktop entry's Exec field is malformed (maybe blank)?",
                                ),
                            )
                            .args(exec_string)
                            .exec();

                            // This will never happen… okay, maybe it will. Haha. That's why we can't panic!() here.
                            Ok(())
                        }),
                    );
                }
            }
        }

        Self {
            search: Search::new(),
            select: Select::new(select_entries.into_iter().collect()),
        }
    }
}

// Technically, this isn't a "widget," but it is a struct that renders other widgets.
// … And some text.
impl Widget for ApplicationLauncher {
    fn draw<G>(&self, coords: [f64; 2], c: &Context, g: &mut G, glyph_cache: &mut Glyphs)
    where
        G: Graphics<Texture = Texture<gfx_device_gl::Resources>>,
    {
        text::Text::new_color(FOREGROUND_COLOR, 60)
            .draw(
                "Applications",
                glyph_cache,
                &DrawState::default(),
                c.transform.trans(coords[0], coords[1]),
                g,
            )
            .unwrap();
        self.search
            .draw([coords[0], coords[1] + 40.0], c, g, glyph_cache);
        self.select
            .draw([coords[0], coords[1] + 100.0], c, g, glyph_cache);
    }
    fn handle_event(&mut self, ev: &Event) {
        self.search.handle_event(ev);
        self.select.handle_event(ev);

        if !self.search.buffer.is_empty() {
            // Ew copy
            self.select
                .update_entry_filter(Some(self.search.buffer.clone()));
        } else {
            self.select.update_entry_filter(None);
        }
    }
}
