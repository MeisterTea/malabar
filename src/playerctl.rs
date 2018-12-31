use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;
use gtk::{timeout_add, Label, EventBox};
use gtk::Orientation::Horizontal;
use paint::set_label_color;

use std::thread;
use std::sync::mpsc::{channel, Sender};

use mpris::{
    LoopStatus, Metadata, Player, PlayerFinder, ProgressTick,
    ProgressTracker, PlaybackStatus, TrackID, TrackList,
};

struct App<'a> {
    player: &'a Player<'a>,
    progress_tracker: ProgressTracker<'a>,
    tx: Sender<String>
}

struct Controls {
    title: Rc<RefCell<Label>>,
    play_pause: Rc<RefCell<Label>>
}

impl<'a> App<'a> {
    fn main_loop(&mut self) {
        let should_continue = true;
        let mut should_refresh = true;

        while should_continue {
            self.tick_progress_and_refresh(should_refresh);
            should_refresh = false;
        }
    }

    fn tick_progress_and_refresh(&mut self, should_refresh: bool) {
        let supports_position = self.supports_position();
        let ProgressTick {
            progress,
            progress_changed,
            track_list,
            track_list_changed,
            ..
        } = self.progress_tracker.tick();

        // Dirty tracking to keep CPU usage lower. In case nothing happened since the last refresh,
        // only update the progress bar.
        //
        // If player doesn't support position handling, don't even try to refresh the progress bar
        // if no event took place.
        if progress_changed || track_list_changed || should_refresh {
            let current_track_id = progress.metadata().track_id();
            let _playback_string = match progress.playback_status() {
                PlaybackStatus::Playing => self.tx.send(String::from("status::playing")),
                PlaybackStatus::Paused => self.tx.send(String::from("status::paused")),
                PlaybackStatus::Stopped => self.tx.send(String::from("status::stopped")),
            };


            if let Some(title) = progress.metadata().title() {
                self.tx.send(format!("title::{}", String::from(title))).unwrap();
            }
            // clear_screen(&mut self.screen);
            // print_instructions(&mut self.screen, self.player);
            // print_playback_info(&mut self.screen, progress);
            if let Some(tracks) = track_list {
                let next_track = find_next_track(current_track_id, tracks, &self.player);
                // print_track_list(&mut self.screen, tracks, next_track);
            }
            // print_progress_bar(&mut self.screen, progress, supports_position);
        } else if supports_position {
            // clear_progress_bar(&mut self.screen);
            // print_progress_bar(&mut self.screen, progress, supports_position);
        }

        // self.screen.flush().unwrap();
    }

    fn supports_position(&self) -> bool {
        self.player.identity() != "Spotify"
    }
}

fn control_player(result: Result<(), mpris::DBusError>) {
    result.expect("Could not control player");
}

fn toggle_shuffle(player: &Player) -> Result<(), mpris::DBusError> {
    player.set_shuffle(!player.get_shuffle()?)
}

fn cycle_loop_status(player: &Player) -> Result<(), mpris::DBusError> {
    let current_status = player.get_loop_status()?;
    let next_status = match current_status {
        LoopStatus::None => LoopStatus::Playlist,
        LoopStatus::Playlist => LoopStatus::Track,
        LoopStatus::Track => LoopStatus::None,
    };
    player.set_loop_status(next_status)
}

fn change_volume(player: &Player, diff: f64) -> Result<(), mpris::DBusError> {
    let current_volume = player.get_volume()?;
    let new_volume = (current_volume + diff).max(0.0).min(1.0);
    player.set_volume(new_volume)
}

fn find_next_track(
    current_track_id: Option<TrackID>,
    track_list: &TrackList,
    player: &Player,
    ) -> Option<Metadata> {
    if let Some(current_id) = current_track_id {
        track_list
            .metadata_iter(player)
            .ok()?
            .skip_while(|track| match track.track_id() {
                // Stops on current track
                Some(id) => id != current_id,
                None => false,
            }).skip(1) // Skip one more to get the next one
        .next()
    } else {
        None
    }
}

pub fn init_playerctl() -> gtk::Box {
    let title = Label::new(None);
    set_label_color(&title, 255, 255, 255);
    title.set_margin_end(10);
    let play_pause = Label::new(None);
    set_label_color(&play_pause, 255, 255, 255);
    play_pause.set_margin_end(10);
    /*
       let event_box = EventBox::new();
       event_box.connect_button_press_event(move |_, _event_button| {
    //player_clone.pause().expect("Could not pause");
    Inhibit(false)
    });
    event_box.add(&play_pause);
    */
    let previous = Label::new(None);
    set_label_color(&previous, 255, 255, 255);
    previous.set_margin_end(10);
    previous.set_text("←");
    let next = Label::new(None);
    set_label_color(&next, 255, 255, 255);
    next.set_margin_end(10);
    next.set_text("→");
    let title_rc = Rc::new(RefCell::new(title.clone()));
    let play_pause_rc = Rc::new(RefCell::new(play_pause.clone()));
    let controls = Controls {
        title: title_rc.clone(),
        play_pause: play_pause_rc.clone(),
    };
    spawn_loop_thread(controls);
    let hbox = gtk::Box::new(Horizontal, 0);
    hbox.add(&title);
    hbox.add(&previous);
    hbox.add(&play_pause);
    hbox.add(&next);
    hbox
}

fn spawn_loop_thread(controls: Controls) {
    let (tx, rx) = channel();
    thread::spawn(move || {
        let player = PlayerFinder::new()
            .unwrap()
            .find_active()
            .expect("Could not find a running player");
        let progress_tracker = player
            .track_progress(::REFRESH_INTERVAL)
            .expect("Could not determine progress of player");

        let mut app = App {
            player: &player,
            progress_tracker: progress_tracker,
            tx
        };
        app.main_loop();

    });
    timeout_add(::REFRESH_INTERVAL, move || {
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

/*
   fn set_play_pause_button(player: Rc<RefCell<Player>>, label: &Label) -> EventBox {
   let event_box = EventBox::new();
   let label = Label::new(None);
   label.set_text("ll");
   set_label_color(&label, 255, 255, 255);
   label.set_margin_end(10);
   let player_clone = player.clone().borrow();
   event_box.connect_button_press_event(move |_, _event_button| {
// player_clone.pause().expect("Could not pause");
Inhibit(false)
});
event_box.add(&label);
event_box
}
*/
