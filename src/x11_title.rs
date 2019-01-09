use glib::Continue;
use x11_get_windows::Session;
use std::rc::Rc;
use crate::paint::set_label_color;
use gtk::{
    Label,
    LabelExt,
    timeout_add,
    WidgetExt
};
use crate::REFRESH_INTERVAL;

pub struct X11TitleWidget {
    title: String
}

impl X11TitleWidget {
    pub fn new() -> Label {
        let mut session = Session::open()
            .expect("Could not open a new session.");
        let label = Label::new(None);
        label.set_margin_end(7);
        set_label_color(&label, 255, 255, 255);
        let label_rc = Rc::new(label.clone());
        let label_clone = label_rc.clone();
        let mut x11_title_widget = X11TitleWidget {
            title: X11TitleWidget::get_current_title(&mut session)
        };
        x11_title_widget.update(&mut session, &label_clone, true);
        timeout_add(REFRESH_INTERVAL, move || {
            x11_title_widget.update(&mut session, &label_clone, false);
            Continue(true)
        });
        label
    }

    fn update(&mut self, session: &mut Session, label: &Label, force_refresh: bool) {
        let title = X11TitleWidget::get_current_title(session);
        if self.title != title || force_refresh {
            label.set_text(&title);
            self.title = title;
        }
    }

    pub fn get_current_title(session: &mut Session) -> String {
        let mut lossy_title = String::from("");
        match session.active_window() {
            Ok(active_window) => {
                match active_window.get_title(&session.display) {
                    Ok(title) => {
                        let title = title;
                        lossy_title = title.as_ref().to_string_lossy().into_owned();
                    },
                    Err(_e) => {}
                }
            },
            Err(_e) => {}
        }
        lossy_title
    }
}
