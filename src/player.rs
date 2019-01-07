use std::rc::Rc;
use std::cell::RefCell;
use glib::Continue;
use gtk::Orientation::Horizontal;
use crate::paint::{set_label_color, set_label_scale};
use std::thread;
use std::sync::mpsc::{channel, Sender};
use mpris::{
    PlaybackStatus,
    Player,
    PlayerFinder,
    ProgressTick,
    ProgressTracker
};
use gtk::{
    timeout_add,
    ContainerExt,
    EventBox,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt
};
use crate::Settings;
use crate::REFRESH_INTERVAL;

struct EventTracker<'a> {
    progress_tracker: ProgressTracker<'a>,
    tx: Sender<String>
}

struct Controls {
    title: Rc<RefCell<Label>>,
    play_pause: Rc<RefCell<Label>>
}

const CONTROLS_SCALE: f64 = 1.3;

impl<'a> EventTracker<'a> {
    fn main_loop(&mut self) {
        let should_continue = true;
        let mut should_refresh = true;

        while should_continue {
            self.tick_progress_and_refresh(should_refresh);
            should_refresh = false;
        }
    }

    fn tick_progress_and_refresh(&mut self, should_refresh: bool) {
        let ProgressTick {
            progress,
            progress_changed,
            track_list_changed,
            ..
        } = self.progress_tracker.tick();
        if progress_changed || track_list_changed || should_refresh {
            let _playback_string = match progress.playback_status() {
                PlaybackStatus::Playing => self.tx.send(String::from("status::playing")),
                PlaybackStatus::Paused => self.tx.send(String::from("status::paused")),
                PlaybackStatus::Stopped => self.tx.send(String::from("status::stopped")),
            };
            if let Some(title) = progress.metadata().title() {
                self.tx.send(format!("title::{}", String::from(title))).unwrap();
            }
        }
    }
}

pub fn init_player(settings: &Settings) -> gtk::Box {
    let previous = Label::new(None);
    let play_pause = Label::new(None);
    let next = Label::new(None);
    let hbox = gtk::Box::new(Horizontal, 0);
    match PlayerFinder::new().unwrap().find_active() {
        Ok(player) => {
            let player_rc = Rc::new(player);
            let title = Label::new(None);
            set_label_color(&title, 255, 255, 255);
            title.set_margin_end(10);
            let previous_event_box = set_previous_button(&player_rc, &previous);
            let play_pause_event_box = set_play_pause_button(&player_rc, &play_pause);
            let next_event_box = set_next_button(&player_rc, &next);
            let title_rc = Rc::new(RefCell::new(title.clone()));
            let play_pause_rc = Rc::new(RefCell::new(play_pause.clone()));
            let controls = Controls {
                title: title_rc.clone(),
                play_pause: play_pause_rc.clone(),
            };
            spawn_loop_thread(controls, settings.debug);
            hbox.add(&title);
            hbox.add(&previous_event_box);
            hbox.add(&play_pause_event_box);
            hbox.add(&next_event_box);
        },
        Err(e) => if settings.debug { println!("{}", e); }
    }
    hbox
}

fn spawn_loop_thread(controls: Controls, debug: bool) {
    let (tx, rx) = channel();
    thread::spawn(move || {
        match PlayerFinder::new().unwrap().find_active() {
            Ok(player) => {
                let progress_tracker = player
                    .track_progress(REFRESH_INTERVAL)
                    .expect("Could not determine progress of player");

                let mut event_tracker = EventTracker {
                    progress_tracker: progress_tracker,
                    tx
                };
                event_tracker.main_loop();
            },
            Err(e) => if debug { println!("{}", e); }
        }

    });
    timeout_add(REFRESH_INTERVAL, move || {
        let title = controls.title.borrow();
        let play_pause = controls.play_pause.borrow();
        let iter = rx.try_iter();
        for query_result in iter {
            let data_split: Vec<&str> = query_result.split("::").collect();
            let (name, info) = (data_split[0], data_split[1]);
            match name {
                "title" => title.set_text(info),
                "status" => {
                    match info {
                        "playing" => play_pause.set_text("▮▮"),
                        "paused" => play_pause.set_text("▶"),
                        "stopped" => {},
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Continue(true) 
    });
}

fn set_default_text_style(label: &Label) {
    set_label_color(&label, 255, 255, 255);
    label.set_margin_end(10);
}

fn set_play_pause_button(player: &Rc<Player<'static>>, label: &Label) -> EventBox {
    set_default_text_style(label);
    let event_box = EventBox::new();
    let player_clone = player.clone();
    event_box.connect_button_press_event(move |_, _event_button| {
        player_clone.play_pause().expect("Could not pause");
        Inhibit(false)
    });
    event_box.add(label);
    event_box
}

fn set_previous_button(player: &Rc<Player<'static>>, label: &Label) -> EventBox {
    label.set_text("");
    set_default_text_style(label);
    set_label_scale(&label, CONTROLS_SCALE);
    let event_box = EventBox::new();
    let player_clone = player.clone();
    event_box.connect_button_press_event(move |_, _event_button| {
        player_clone.previous().expect("Could not go to previous song");
        Inhibit(false)
    });
    event_box.add(label);
    event_box
}

fn set_next_button(player: &Rc<Player<'static>>, label: &Label) -> EventBox {
    label.set_text("");
    set_default_text_style(label);
    set_label_scale(&label, CONTROLS_SCALE);
    let event_box = EventBox::new();
    let player_clone = player.clone();
    event_box.connect_button_press_event(move |_, _event_button| {
        player_clone.next().expect("Could not go to next song");
        Inhibit(false)
    });
    event_box.add(label);
    event_box
}
