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
    io::{Read, self},
    rc::Rc
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

pub fn init_battery() -> Rc<Label> {
    let label = Label::new(None);
    label.set_margin_end(7);
    set_label_color(&label, 255, 255, 255);
    let label_rc = Rc::new(label);
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
    let label_clone = label_rc.clone();
    let battery_id_clone = battery_id.clone(); // just use lifetime of battery_id or reintroduce battery Struct ?
    timeout_add(500, move || {
        let current_charge = get_charge(&battery_id_clone, "energy_now").parse::<u32>()
            .unwrap_or_else(|_| 0 as u32);
        let battery_percentage = format!("{:.*}%", 0, current_charge as f32 / full_charge as f32 * 100 as f32);
        label_clone.set_text(&battery_percentage);
        Continue(true)
    });
    label_rc // Why is Rc needed here and not in clock module ?
}
