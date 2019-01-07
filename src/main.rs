extern crate cairo;
extern crate chrono;
extern crate clap;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate mpris;
extern crate pango;
extern crate x11_get_windows;

use clap::{App, Arg};

use gio::prelude::*;
use crate::window::build_ui;

mod battery;
mod bspwm;
mod clock;
mod paint;
mod player;
mod window;
mod x11_title;

const REFRESH_INTERVAL: u32 = 100;

pub struct Settings {
    debug: bool
}

#[derive(Copy, Clone, Debug)]
pub struct Null;

fn args_to_settings() -> Settings {
    let matches = App::new("malabar")
        .version("alpha")
        .about("A simple rust bar aimed to bspwm")
        .author("MeisterTea")
        .arg(Arg::with_name("debug")
             .short("d")
             .index(1))
        .help("Turn debugging information on")
        .get_matches();
    let mut settings = Settings { debug: false };
    if matches.is_present("debug") { settings.debug = true; };
    settings
}

fn main() {
    let settings = args_to_settings();
    let application = gtk::Application::new("com.meistertea.malabar",
                                            gio::ApplicationFlags::empty())
        .expect("Initialization failed...");
    application.connect_startup(move |app| {
        build_ui(app, &settings);
    });
    application.connect_activate(|_| {});
    let no_arg: [String; 0] = [];
    application.run(&no_arg);
}

