
use gtk::prelude::*;
use std::process::{Command, Stdio};
use std::cell::Ref;
use std::sync::mpsc::Sender;
use std::io::{BufReader, BufRead};
use std::str;
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::channel;
use gtk::Orientation::Horizontal;

use paint::set_label_color;

#[derive(Debug)]
pub struct Desktop {
    pub name: String,
    pub status: String
}

fn set_desktops_style(labels: Ref<Vec<gtk::Label>>, query_result: &str) {
    let desktops = set_desktops(query_result);
    let set_default_color = |label| set_label_color(label, 255, 255, 255);
    let set_occupied_color = |label| set_label_color(label, 255, 51, 42);
    let set_focused_color = |label| set_label_color(label, 250, 189, 47);
    let set_urgent_color = |label| set_label_color(label, 152, 151, 26);
    for desktop in desktops {
        for label in labels.iter() {
            if let Some(gtk_label) = label.get_label() {
                if gtk_label == desktop.name {
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
                }
            }
        }
    }
}

pub fn set_desktops(query_result: &str) -> Vec<Desktop> {
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
    let split = query_result.split(':').collect::<Vec<_>>();
    (&split[1..split.len()-3]).iter().map(|part| get_desktops_info(part)).collect::<Vec<_>>()
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
                    Err(_) => return,
                };
            }
        },
        None => return,
    }
}

pub fn render_desktops(desktops_labels: Vec<gtk::Label>) -> gtk::Box {
    let desktops_box = gtk::Box::new(Horizontal, 0);
    let desktops_labels_rc = Rc::new(RefCell::new(desktops_labels.clone()));
    let labels_clone = desktops_labels_rc.clone();
    spawn_bspc_subscribe(labels_clone);
    let desktoplab_rc = desktops_labels_rc.borrow();
    for desktop_label in desktoplab_rc.iter() {
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

pub fn get_desktops_from_display(monitor: &str) -> Vec<String> {
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

fn spawn_bspc_subscribe(desktop_labels: Rc<RefCell<Vec<gtk::Label>>>) {
    let (tx, rx) = channel();
    thread::spawn(move || {
        bspc_subscribe(tx);

    });
    let labels_clone = desktop_labels.clone();
    gtk::timeout_add(::REFRESH_INTERVAL, move || {
        let iter = rx.try_iter();
        for query_result in iter {
            set_desktops_style(labels_clone.borrow(), &query_result);
        }
        Continue(true) 
    });
}
