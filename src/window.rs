use gtk::ApplicationWindow;
use gtk::Orientation::Horizontal;
use std::rc::Rc;
use crate::bspwm::BspwmDesktopsWidget;
use crate::clock::ClockWidget;
use crate::player::PlayerWidget;
use crate::x11_title::X11TitleWidget;
use crate::battery::BatteryWidget;
use crate::Settings;
use gdk::{
    DisplayExt,
    MonitorExt,
    Rectangle,
    Screen,
    ScreenExt
};
use gtk::{
    Inhibit,
    ContainerExt,
    GtkWindowExt,
    WidgetExt
};

struct ScreenWrapper {
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
    //TODO replace sizes by screen
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

fn set_bar(window: &ApplicationWindow, screen_wrapper: ScreenWrapper, settings: &Settings) {
    let screen_width = screen_wrapper.dimensions.width;

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
    let desktops_labels = BspwmDesktopsWidget::new(&screen_wrapper.name);
    hbox.add(&desktops_labels);
    let window_label = X11TitleWidget::new();
    hbox.add(&window_label);
    let artist_label = PlayerWidget::new(settings);
    hbox.add(&artist_label);
    let battery_label = BatteryWidget::new();
    hbox.add(&battery_label);
    let time_label = ClockWidget::new();
    hbox.add(&time_label);
    window.add(&hbox);
    set_window_positions(&window, screen_wrapper.dimensions);
}

pub fn build_ui(application: &gtk::Application, settings: &Settings) {
    let window = ApplicationWindow::new(application);
    let screens = get_displays_geometry(&window);
    for (i, screen) in screens.into_iter().enumerate() {
        if i != 0 {
            set_bar(&window, screen, settings);
            return;
        }
        set_bar(&ApplicationWindow::new(application), screen, settings);
    }
}

fn get_displays_geometry(window: &ApplicationWindow) -> Vec<ScreenWrapper> {
    let mut screens = Vec::new();
    if let Some(screen) = window.get_screen() {
        let screen_rc = Rc::new(screen);
        let screen_clone = screen_rc.clone();
        let display = screen_clone.get_display();
        let monitors_count = display.get_n_monitors();
        for monitor_index in 0..monitors_count {
            if let Some(monitor) = display.get_monitor(monitor_index) {
                if let Some(name) = monitor.get_model() {
                    let dimensions = monitor.get_geometry();
                    screens.push(ScreenWrapper{
                        name,
                        dimensions
                    });
                }
            }
        }
    }
    return screens;
}

fn set_visual(window: &ApplicationWindow, _screen: &Option<Screen>) {
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
