use gtk::prelude::*;

fn u8_to_u16_ratio(num: u8) -> u16 {
    u16::from(num) * (std::u16::MAX / std::u8::MAX as u16)
}

pub fn set_label_color(label: &gtk::Label, r: u8, g: u8, b: u8) {
    let attr_list = pango::AttrList::new();
    let foreground = pango::Attribute::new_foreground(u8_to_u16_ratio(r), u8_to_u16_ratio(g), u8_to_u16_ratio(b))
        .expect("Couldn't create new foreground");
    let weight = pango::Attribute::new_weight(pango::Weight::Bold)
        .expect("Couldn't create new weight");
    attr_list.insert(foreground);
    attr_list.insert(weight);
    label.set_attributes(&attr_list);
}
