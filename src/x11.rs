use gtk::Label;

pub fn init_x11() -> gtk::Label {
    let label = Label::new(None);
    /*
    timeout_add(::REFRESH_INTERVAL, move || {
        let iter = rx.try_iter();
        for query_result in iter {
            set_desktops_style(labels_clone.borrow(), &query_result);
        }
        Continue(true) 
    });
    */
    label
}
