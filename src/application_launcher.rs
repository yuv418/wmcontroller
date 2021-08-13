use crate::{configuration::FOREGROUND_COLOR, search::Search, select::Select, widgets::Widget};
use dirs::home_dir;
use freedesktop_desktop_entry::{DesktopEntry, Iter, PathSource};
use log::debug;
use piston_window::*;
use std::path::PathBuf;

pub struct ApplicationLauncher {
    search: Search,
    select: Select,
}

impl ApplicationLauncher {
    pub fn new() -> Self {
        // Basically copied from https://crates.io/crates/freedesktop-desktop-entry
        let mut select_entries: Vec<(String, Box<dyn Fn() -> Result<(), String>>)> = vec![];
        // We want to use a different default_paths so that we iter through the
        // local directories last (the libraries goes through the local directories) first,
        // so we change the order of default_paths from what you see here:
        // https://codeberg.org/mmstick/freedesktop-desktop-entry/src/branch/main/src/lib.rs (
        // although I basically just copied the function).
        //
        // We want to iter through the local dirs last so they overwrite/override the system .desktop
        // for users that want to overwrite the system .desktop entries.

        // TODO handle this so that if they don't have a home directory, only use the system directories
        let home_dir = home_dir().expect("You do not have a home directory");
        let desktop_dirs_iter_order = vec![
            (
                PathSource::SystemSnap,
                PathBuf::from("/var/lib/snapd/desktop/applications"),
            ),
            (
                PathSource::SystemFlatpak,
                PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            ),
            (PathSource::System, PathBuf::from("/usr/share/applications")),
            (PathSource::LocalDesktop, home_dir.join("Desktop")),
            (
                PathSource::LocalFlatpak,
                home_dir.join(".local/share/flatpak/exports/share/applications"),
            ),
            (
                PathSource::Local,
                home_dir.join(".local/share/applications"),
            ),
        ];
        for (_, path) in Iter::new(desktop_dirs_iter_order) {
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

                    select_entries.push((display_name.to_string(), Box::new(|| Ok(()))))
                }
            }
        }

        Self {
            search: Search::new(),
            select: Select::new(select_entries),
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
