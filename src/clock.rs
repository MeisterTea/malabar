use gtk::{Label, LabelExt};
use std::rc::Rc;

use chrono::Local;
use crate::paint::set_label_color;

fn current_time() -> String {
    return format!("{}", Local::now().format("%H:%M"));
}

fn set_time_widget() -> Label {
    let time = current_time();
    let label_time = Label::new(None);
    label_time.set_text(&time);
    set_label_color(&label_time, 255, 255, 255);
    label_time
}

pub fn init_clock() -> Label {
    let label_time = set_time_widget();
    let label_rc = Rc::new(label_time.clone());
    let label_clone = label_rc.clone();
    let tick = move || {
        let time = current_time();
        label_clone.set_text(&time);
        gtk::Continue(true)
    };
    gtk::timeout_add_seconds(1, tick);
    label_time
}
