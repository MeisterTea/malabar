use gtk::{
    Label,
    LabelExt,
    WidgetExt
};
use std::{
    fs,
    fs::File,
    io::{Read, self}
};
use crate::paint::set_label_color;

const POWER_SUPPLY_ROOT: &str = "/sys/class/power_supply";

#[derive(Debug)]
struct Battery {
    id: String,
    charging: bool,
    current_charge: u32,
    full_charge: u32
}

fn get_battery_ids() -> Vec<String> {
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

fn get_value(battery_id: &String, state: &str) -> Result<String, io::Error> {
    let state_path = format!("{}/{}/{}", POWER_SUPPLY_ROOT, battery_id, state);
    let mut file = File::open(state_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.trim().to_owned())
}

fn get_charge(battery_id: &String, state: &str) -> String {
    get_value(&battery_id, state)
        .unwrap_or_else(|_| String::from(""))
}

pub fn init_battery() -> Label {
    let label = Label::new(None);
    let battery_id = &get_battery_ids()[0];
    let current_charge = get_charge(battery_id, "energy_now").parse::<u32>()
        .unwrap_or_else(|_| 0 as u32);
    let full_charge = get_charge(battery_id, "energy_full").parse::<u32>()
        .unwrap_or_else(|_| 0 as u32);
    let battery = Battery {
        id: battery_id.to_string(),
        charging: true,
        current_charge,
        full_charge
    };
    let battery_percentage = format!("{:.*}%", 0, current_charge as f32 / full_charge as f32 * 100 as f32);
    label.set_text(&battery_percentage);
    label.set_margin_end(7);
    set_label_color(&label, 255, 255, 255);
    label
}
