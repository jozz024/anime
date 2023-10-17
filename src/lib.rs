#![feature(fs_try_exists)]

use std::{time::Duration, io::{BufRead, BufReader}};

use serde::{Serialize, Deserialize};
mod anime;

#[derive(Serialize,Deserialize)]
pub struct Config {
    pub current_video_path: String,
    pub watch_time: f32
}

// should public this stuff in arcrop api
#[repr(C)]
#[derive(Copy, Clone)]
pub enum Event {
    ArcFilesystemMounted,
    ModFilesystemMounted,
}

pub type EventCallbackFn = extern "C" fn(Event);

extern "C" {
    fn arcrop_register_event_callback(ty: Event, callback: EventCallbackFn);
}


static mut mods_mounted: bool = false;

extern "C" fn mod_callback(_event: Event) {
    unsafe { mods_mounted = true };
}

#[skyline::main(name = "anime")]
pub fn main() {
    // Ty for the panic hook ray i appreciate
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let err_msg = format!("anime has panicked at '{}', {}", msg, location);
        skyline::error::show_error(
            69,
            "anime has panicked! Please open the details, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
    // register the mods loaded event callback so we know when to turn off the video
    unsafe {
        arcrop_register_event_callback(Event::ModFilesystemMounted, mod_callback)
    }
    // New thread for video player so loading can continue
    std::thread::Builder::new()
    .stack_size(0x10000)
    .spawn(|| {
        // Check for episodes and panic if there arent any
        if !std::fs::try_exists("sd:/episodes/").unwrap() {
            panic!("No episode folder present!")
        }
        if std::fs::read_dir("sd:/episodes/").unwrap().count() < 1 {
            panic!("No videos present for playing!")
        }

        let mut config: Config;
        if !std::fs::try_exists("sd:/episodes/config/config.json").unwrap() {
            std::fs::create_dir("sd:/episodes/config/");
            config = Config {current_video_path: "".to_string(), watch_time: -1.0};
            std::fs::write("sd:/episodes/config/config.json", serde_json::to_string(&config).unwrap());
        }
        else {
            let file = std::fs::File::open("sd:/episodes/config/config.json").unwrap();
            let rdr = BufReader::new(file);
            config = serde_json::from_reader(rdr).unwrap();
        }
        // Spawn the webpage for the video player
        let session = anime::spawn_webpage(&mut config);
        let dur = Duration::from_secs_f32(0.1);
        let mut current_time: f32 = 0.0;
        unsafe {
            // Loop for when mods gets mounted
            loop {
                if mods_mounted {
                    // Sleep because theres a little bit more time left after the mods mount
                    let final_dur = Duration::from_secs(5);
                    std::thread::sleep(final_dur);
                    break;
                }

                if let Some(msg) = session.try_recv() {
                    let message = msg;
                    if message.starts_with("current_time:") {
                        let time = message.strip_prefix("current_time: ").unwrap();
                        current_time = time.parse().unwrap();
                    }
                    else if message == *"video_finished" {
                        current_time = -1.0;
                        break;
                    }
                }
                std::thread::sleep(dur);
            }
        }
        // Exit the video player
        session.exit();
        config.watch_time = current_time;
        std::fs::write("sd:/episodes/config/config.json", serde_json::to_string(&config).unwrap()).unwrap();
    }).unwrap();
}
