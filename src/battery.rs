use gtk::{
    timeout_add,
    Continue,
    Label,
    LabelExt,
    WidgetExt
};
use std::{
    fs,
    fs::File,
    io::{
        BufRead,
        BufReader,
        self
    },
    iter::FromIterator,
    rc::Rc
};
use crate::paint::set_label_color;

const POWER_SUPPLY_ROOT: &str = "/sys/class/power_supply";
const REFRESH_RATE: u32 = 500;

fn get_battery_names() -> Vec<String> {
    let dir_entries = fs::read_dir(POWER_SUPPLY_ROOT).unwrap();
    let mut batteries = Vec::new();
    for dir_entry in dir_entries {
        let path = dir_entry.unwrap().path();
        if let Some(file_stem) = path.file_stem() {
            let file_stem_string = file_stem.to_string_lossy();
            if file_stem_string.starts_with("BAT") {
                batteries.push(file_stem_string.into_owned());
            }
        }
    }
    batteries
}

fn get_data(battery_name: &String) -> Result<(String, u8), io::Error> {
    let file = File::open(format!("{}/{}/uevent", POWER_SUPPLY_ROOT, battery_name))?;
    let content = BufReader::new(&file);
    let mut state = String::from("");
    let mut charge = 0 as u8;
    for line in content.lines() {
        let line = line?;
        let tokens = Vec::from_iter(line.split('=')); 
        let token = tokens.last().unwrap();
        match tokens.first().unwrap() {
            &"POWER_SUPPLY_STATUS" => { state = token.to_string(); },
            &"POWER_SUPPLY_CAPACITY" => { charge = token.parse::<u8>().unwrap(); },
            _ => {}
        }
    }
    Ok((state, charge))
}

fn update(label: &Label, battery_name: &String) {
    match get_data(&battery_name) {
        Ok((_state, charge)) => {
            label.set_text(&charge.to_string());
        },
        Err(_e) => {}
    }
}

pub fn init_battery() -> Rc<Label> {
    let label = Label::new(None);
    label.set_margin_end(7);
    set_label_color(&label, 255, 255, 255);
    let label_rc = Rc::new(label);
    let battery_name = &get_battery_names()[0];
    let label_clone = label_rc.clone();
    let battery_name_clone = battery_name.clone();
    timeout_add(REFRESH_RATE, move || {
        update(&label_clone, &battery_name_clone);
        Continue(true)
    });
    label_rc
}
