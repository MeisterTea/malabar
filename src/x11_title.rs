use glib::Continue;
use x11_get_windows::Session;
use std::rc::Rc;
use std::cell::RefCell;
use crate::paint::set_label_color;
use gtk::{
    Label,
    LabelExt,
    timeout_add,
    WidgetExt
};
use crate::REFRESH_INTERVAL;

pub fn init_x11() -> Label {
    let session = Session::open()
        .expect("Could not open a new session.");
    let label = Label::new(None);
    label.set_margin_end(7);
    set_label_color(&label, 255, 255, 255);
    let label_rc = Rc::new(label.clone());
    let label_clone = label_rc.clone();
    let session_rc = Rc::new(RefCell::new(session));
    timeout_add(REFRESH_INTERVAL, move || {
        let session_clone = session_rc.clone();
        let raw_title = get_current_title(session_clone);
        let title = raw_title.as_str();
        label_clone.set_text(title);
        Continue(true)
    });
    label
}

pub fn get_current_title(session: Rc<RefCell<Session>>) -> String {
    let mut session_mut = session.borrow_mut();
    let mut lossy_title = String::from("");
    match session_mut.active_window() {
        Ok(active_window) => {
            match active_window.get_title(&session_mut.display) {
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
