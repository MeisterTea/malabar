use gtk::prelude::*;
use gtk::Label;
use pango::{Attribute, AttrList};

fn u8_to_u16_ratio(num: u8) -> u16 {
    u16::from(num) * (std::u16::MAX / std::u8::MAX as u16)
}

pub fn set_label_color(label: &Label, r: u8, g: u8, b: u8) {
    let attr_list: AttrList;
    if let Some(temp_attr_list) = label.get_attributes() {
        attr_list = temp_attr_list;
    } else {
        attr_list = AttrList::new();
    }
    let foreground = Attribute::new_foreground(u8_to_u16_ratio(r), u8_to_u16_ratio(g), u8_to_u16_ratio(b))
        .expect("Couldn't create new foreground");
    let weight = Attribute::new_weight(pango::Weight::Bold)
        .expect("Couldn't create new weight");
    attr_list.insert(foreground);
    attr_list.insert(weight);
    label.set_attributes(&attr_list);
}

pub fn set_label_scale(label: &Label, scale: f64) {
    let attr_list: AttrList;
    if let Some(temp_attr_list) = label.get_attributes() {
        attr_list = temp_attr_list;
    } else {
        attr_list = AttrList::new();
    }
    let scale = Attribute::new_scale(scale)
        .expect("Couldn't create new scale");
    attr_list.insert(scale);
    label.set_attributes(&attr_list);
}
