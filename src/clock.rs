use gtk::{Label, LabelExt};
use std::rc::Rc;

use chrono::Local;
use crate::paint::set_label_color;

pub struct ClockWidget {
    hours: String,
    minutes: String
}

impl ClockWidget {
    fn update(&mut self, label: &Label, force_refresh: bool) {
        let (current_hours, current_minutes) = ClockWidget::get_time();
        let ClockWidget { hours, minutes } = &self;
        if hours != &current_hours
            || minutes != &current_minutes
                || force_refresh {
                    label.set_text(&format!("{}:{}", current_hours, current_minutes));
                    self.hours = current_hours;
                    self.minutes = current_minutes;
                }
    }

    fn get_time() -> (String, String) {
        let hours = format!("{}", Local::now().format("%H"));
        let minutes = format!("{}", Local::now().format("%M"));
        (hours.to_string(), minutes.to_string())
    }

    pub fn new() -> Label {
        let label = Label::new(None);
        set_label_color(&label, 255, 255, 255);
        let label_rc = Rc::new(label.clone());
        let label_clone = label_rc.clone();
        let (hours, minutes) = ClockWidget::get_time();
        let mut clock = ClockWidget {
            hours,
            minutes
        };
        ClockWidget::update(&mut clock, &label_clone, true);
        let tick = move || {
            ClockWidget::update(&mut clock, &label_clone, false);
            gtk::Continue(true)
        };
        gtk::timeout_add_seconds(1, tick);
        label
    }
}
