extern crate cairo;
extern crate pango;
extern crate gdk;
extern crate gio;
extern crate gtk;
extern crate glib;
extern crate chrono;
extern crate mpris;

use gio::prelude::*;
use std::env::args;
use window::build_ui;

mod paint;
mod bspwm;
mod clock;
mod window;
mod playerctl;

const REFRESH_INTERVAL: u32 = 100;

fn main() {
    let application = gtk::Application::new("com.meistertea.malabar",
                                            gio::ApplicationFlags::empty())
        .expect("Initialization failed...");
    application.connect_startup(move |app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}

