use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::io::{BufReader, BufRead};
use std::str;
use std::thread;
use std::rc::Rc;
use std::sync::mpsc::{
    channel,
    Receiver
};
use glib::Continue;
use gtk::{
    ContainerExt,
    Inhibit,
    Label,
    LabelExt,
    Orientation::Horizontal,
    WidgetExt
};
use crate::REFRESH_INTERVAL;

use crate::paint::set_label_color;

struct Desktop {
    name: String,
    status: String
}

pub struct BspwmDesktopsWidget {
    desktops: HashMap<String, Desktop>,
    first_render: bool
}

impl BspwmDesktopsWidget {

    fn set_desktops(rx: &Receiver<String>) -> Result<Vec<Desktop>, &'static str> {
        fn get_desktops_info(my_string: &str) -> Desktop {
            let mut name = String::from("");
            let mut status = String::from("");
            for (i, my_char) in my_string.chars().enumerate() {
                if i == 0 {
                    status = my_char.to_string();
                    continue;
                }
                name.push(my_char);
            }
            Desktop {
                name,
                status
            }
        }
        let query_results = rx.try_iter();
        let mut desktops = vec!(Desktop { name: "".to_string(), status: "f".to_string() });
        let mut query_count = 0;
        for query_result in query_results {
            query_count = query_count + 1;
            let split = query_result.split(':').collect::<Vec<_>>();
            desktops = (&split).iter().map(|part| get_desktops_info(part)).collect::<Vec<_>>();
        }
        if query_count == 0 { return Err("No bspwm desktops data"); }
        Ok(desktops)
    }

    fn bspc_subscribe(tx: Sender<String>) {
        let mut command = Command::new("bspc")
            .arg("subscribe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process");
        match command.stdout.as_mut() {
            Some(out) => {
                let buf_reader = BufReader::new(out);
                for line in buf_reader.lines() {
                    match line {
                        Ok(l) => {
                            tx.send(l).unwrap();
                        },
                        Err(_) => return
                    };
                }
            },
            None => return
        }
    }

    fn render_desktops(desktops_labels: Vec<Label>, desktops_names: &Vec<String>) -> gtk::Box {
        let desktops_box = gtk::Box::new(Horizontal, 0);
        let desktops_labels_rc = Rc::new(desktops_labels.clone());
        let labels_clone = desktops_labels_rc.clone();
        BspwmDesktopsWidget::spawn_bspc_subscribe(&labels_clone, &desktops_names);
        for desktop_label in labels_clone.iter() {
            let event_box = gtk::EventBox::new();
            if let Some(desktop_string) = desktop_label.get_label() {
                event_box.connect_button_press_event(move |_, _event_button| {
                    Command::new("bspc")
                        .args(&[
                              "desktop",
                              "-f",
                              &desktop_string
                        ])
                        .output()
                        .expect("failed to execute process");
                    Inhibit(false)
                });
            }
            event_box.add(desktop_label);
            desktops_box.add(&event_box);
        }
        return desktops_box;
    }

    pub fn new(screen_name: &String) -> gtk::Box {
        let desktops_list = BspwmDesktopsWidget::get_desktops_from_display(screen_name);
        let desktops_labels = BspwmDesktopsWidget::render_labels(&desktops_list);
        BspwmDesktopsWidget::render_desktops(desktops_labels, &desktops_list)
    }

    fn get_desktops_from_display(monitor: &str) -> Vec<String> {
        let command_res = Command::new("bspc")
            .args(&[
                  "query",
                  "-D",
                  "--names",
                  "-m",
                  monitor
            ])
            .output()
            .expect("failed to execute process");
        let stdout: &[u8] = &command_res.stdout;
        let stdout_as_string = str::from_utf8(stdout).unwrap();
        stdout_as_string.split("\n").filter(|c| c != &"").map(|c| String::from(c)).collect()
    }

    fn update(&mut self, rx: &Receiver<String>, labels: &Rc<Vec<Label>>) {
        let first_render = self.first_render;
        match BspwmDesktopsWidget::set_desktops(&rx) {
            Ok(desktops) => {
                let set_default_color = |label| set_label_color(label, 255, 255, 255);
                let set_occupied_color = |label| set_label_color(label, 255, 51, 42);
                let set_focused_color = |label| set_label_color(label, 250, 189, 47);
                let set_urgent_color = |label| set_label_color(label, 152, 151, 26);
                for desktop in desktops {
                    for label in labels.iter() {
                        if let Some(gtk_label) = label.get_label() {
                            let gtk_label_clone = gtk_label.clone();
                            let status_clone = desktop.status.clone();
                            if gtk_label == desktop.name
                                && (&self.desktops[&gtk_label].status != &desktop.status
                                    || first_render) {
                                    match desktop.status.as_str() {
                                        "M" => set_focused_color(label), // Focused monitor
                                        "m" => set_default_color(label), // Unfocused monitor
                                        "O" => set_focused_color(label), // Focused occupied
                                        "o" => set_occupied_color(label), // Unfocused occupied
                                        "F" => set_focused_color(label), // Focused empty
                                        "f" => set_default_color(label), // Unfocused empty
                                        "U" => set_focused_color(label), // Focused Urgent
                                        "u" => set_urgent_color(label), // Unfocused Urgent
                                        _ => {},
                                    }
                                    self.desktops.insert(gtk_label, Desktop {
                                        name: gtk_label_clone,
                                        status: status_clone
                                    });
                                }
                        }
                    }
                }
                if first_render { self.first_render = false; }
            },
            Err(_e) => {}
        }
    }

    fn spawn_bspc_subscribe(desktop_labels: &Rc<Vec<Label>>, desktop_names: &Vec<String>) {
        let (tx, rx) = channel();
        thread::spawn(move || {
            BspwmDesktopsWidget::bspc_subscribe(tx);

        });
        let desktops: HashMap<String, Desktop> = desktop_names.into_iter()
            .map(|name| 
                 (
                     name.to_string(),
                     Desktop {
                         name: name.to_string(),
                         status: "f".to_string()
                     })
                ).rev().collect();
        let mut bspwm_desktops_widget = BspwmDesktopsWidget {
            desktops,
            first_render: true
        };
        let labels_clone = desktop_labels.clone();
        bspwm_desktops_widget.update(&rx, &labels_clone);
        gtk::timeout_add(REFRESH_INTERVAL, move || {
            bspwm_desktops_widget.update(&rx, &labels_clone);
            Continue(true) 
        });
    }

    fn render_labels(desktops: &Vec<String>) -> Vec<Label> {
        const MARGINS: i32 = 7;
        let mut desktop_labels: Vec<Label> = Vec::new();
        for desktop in desktops {
            let label = Label::new(desktop.as_str());
            label.set_margin_start(MARGINS);
            label.set_margin_end(MARGINS);
            desktop_labels.push(label);
        }
        desktop_labels
    }
}
