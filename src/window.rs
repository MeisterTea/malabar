use gdk::{ScreenExt, DisplayExt, MonitorExt, Rectangle};
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::Orientation::Horizontal;

use bspwm::{get_desktops_from_display, render_desktops};
use clock::init_clock;
use playerctl::init_playerctl;

#[derive(Debug)]
struct Screen {
    name: String,
    dimensions: Rectangle
}

fn set_window_positions(window: &ApplicationWindow, dimension: Rectangle) {
    const OFFSET: i32 = 8;
    window.stick();
    window.set_type_hint(gdk::WindowTypeHint::Dock);
    window.set_keep_above(true);
    window.show_all();
    let strut_partial_cardinal_atom: gdk::Atom = "_NET_WM_STRUT_PARTIAL".into();
    let strut_cardinal_atom: gdk::Atom = "_NET_WM_STRUT".into();
    let cardinal_atom: gdk::Atom = "CARDINAL".into();
    let prop_replace = gdk::PropMode::Replace;
    let data_partial_cardinal = gdk::ChangeData::ULongs(&[0, 0, 48, 0, 0, 0, 0, 0, 8, 1916, 0, 0]);
    let data_cardinal = gdk::ChangeData::ULongs(&[0, 0, 48, 0]);

    if let Some(my_window) = window.get_window() {
        gdk::property_change(
            &my_window, 
            &strut_partial_cardinal_atom, 
            &cardinal_atom,
            32,
            prop_replace,
            data_partial_cardinal
            );
        gdk::property_change(
            &my_window, 
            &strut_cardinal_atom, 
            &cardinal_atom,
            32,
            prop_replace,
            data_cardinal
            );
    }
    window.move_(dimension.x + OFFSET, dimension.y + OFFSET);
}

fn set_bar(window: &ApplicationWindow, screen: Screen) {
    let screen_width = screen.dimensions.width;

    set_visual(&window, &None);

    window.connect_delete_event(quit);
    window.connect_screen_changed(set_visual);
    window.connect_draw(draw);

    window.set_title("Malabar");
    let width = screen_width * 99 / 100;
    let height = 30;
    window.set_default_size(width, height);
    window.set_app_paintable(true); // crucial for transparency
    let hbox = gtk::Box::new(Horizontal, 0);
    let desktops_list = get_desktops_from_display(&screen.name);
    let desktops_labels = render_labels(desktops_list);
    let desktops_box = render_desktops(desktops_labels);
    hbox.add(&desktops_box);
    let label_artist = init_playerctl();
    hbox.add(&label_artist);
    let label_time = init_clock();
    hbox.add(&label_time);
    window.add(&hbox);
    set_window_positions(&window, screen.dimensions);
}

pub fn build_ui(application: &gtk::Application) {
    let window = ApplicationWindow::new(application);
    let screens = get_displays_geometry(&window);
    for (i, screen) in screens.into_iter().enumerate() {
        if i != 0 {
            set_bar(&window, screen);
            return;
        }
        set_bar(&ApplicationWindow::new(application), screen);
    }
}

fn get_displays_geometry(window: &ApplicationWindow) -> Vec<Screen> {
    let mut screens = Vec::new();
    if let Some(screen) = window.get_screen() {
        let display = screen.get_display();
        let monitors_count = display.get_n_monitors();
        for monitor_index in 0..monitors_count {
            if let Some(monitor) = display.get_monitor(monitor_index) {
                if let Some(name) = monitor.get_model() {
                    let dimensions = monitor.get_geometry();
                    screens.push(Screen{ name, dimensions });
                }
            }
        }
    }
    return screens;
}

fn set_visual(window: &ApplicationWindow, _screen: &Option<gdk::Screen>) {
    if let Some(screen) = window.get_screen() {
        if let Some(visual) = screen.get_rgba_visual() {
            window.set_visual(&visual);
        }
    }
}

fn draw(_window: &ApplicationWindow, ctx: &cairo::Context) -> Inhibit {
    ctx.set_source_rgba(0.0, 0.0, 0.0, 0.8);
    ctx.set_operator(cairo::enums::Operator::Screen);
    ctx.paint();
    Inhibit(false)
}

fn quit(_window: &ApplicationWindow, _event: &gdk::Event) -> Inhibit {
    _window.destroy();
    Inhibit(false)
}

fn render_labels(desktops: Vec<String>) -> Vec<gtk::Label> {
    const MARGINS: i32 = 7;
    let mut desktop_labels: Vec<gtk::Label> = Vec::new();
    for desktop in desktops {
        let label = gtk::Label::new(desktop.as_str());
        label.set_margin_start(MARGINS);
        label.set_margin_end(MARGINS);
        desktop_labels.push(label);
    }
    desktop_labels
}