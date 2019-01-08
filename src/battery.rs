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

pub struct BatteryWidget {
    charge: u8,
    name: String,
    state: String
}

impl BatteryWidget {
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

    fn update(&self, label: &Label, force_refresh: bool) {
        match BatteryWidget::get_data(&self.name) {
            Ok((state, charge)) => {
                if self.charge != charge
                    || self.state != state
                        || force_refresh {
                            let mut battery_icon = match charge {
                                c if c < 20 => " ",
                                c if c < 40 => " ",
                                c if c < 60 => " ",
                                c if c < 80 => " ",
                                _ => " "
                            };
                            battery_icon = match state.as_ref() {
                                "Charging" => " ",
                                _ => battery_icon
                            };
                            let charge_string = &charge.to_string();
                            label.set_text(&format!("{}{}%", battery_icon, charge_string));
                        }
            },
            Err(_e) => {}
        }
    }

    pub fn new() -> Rc<Label> {
        let label = Label::new(None);
        label.set_margin_end(7);
        set_label_color(&label, 255, 255, 255);
        let label_rc = Rc::new(label);
        let name = &BatteryWidget::get_battery_names()[0];
        let label_clone = label_rc.clone();
        let (state, charge) = BatteryWidget::get_data(&name).unwrap_or_else(|_| (String::from(""), 0));
        let battery = BatteryWidget {
            charge,
            name: name.to_string(),
            state
        };
        BatteryWidget::update(&battery, &label_clone, true); // TODO refactor to closure ?
        timeout_add(REFRESH_RATE, move || {
            BatteryWidget::update(&battery, &label_clone, false);
            Continue(true)
        });
        label_rc
    }
}
