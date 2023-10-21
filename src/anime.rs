use skyline_web::{Webpage, Visibility};

use rand::seq::SliceRandom;

use crate::Config;

static anime_html: &str = include_str!("anime.html");

fn get_episode() -> String {
    let episodes = std::fs::read_dir("sd:/episodes/").unwrap();
    let names = episodes.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(String::from))
        )
    }).collect::<Vec<String>>();

    let mut file_name = names.choose(&mut rand::thread_rng()).unwrap().to_owned();
    let mut filepath = "sd:/episodes/".to_string() + &file_name;
    while std::fs::metadata(&filepath).unwrap().is_dir() {
        file_name = names.choose(&mut rand::thread_rng()).unwrap().to_owned();
        filepath = "sd:/episodes/".to_string() + &file_name;
    };
    filepath
}

pub fn spawn_webpage(config: &mut Config) -> skyline_web::WebSession {
    std::fs::write("sd:/atmosphere/contents/01006A800016E000/manual_html/html-document/contents.htdocs/index.html", anime_html).unwrap();
    if !config.current_video_path.is_empty() && config.watch_time != -1.0 {
        let current_episode = config.current_video_path.clone();
        let current_time = config.watch_time;

        let episode = std::fs::read(current_episode).unwrap();
        let params = format!("seconds={}", current_time);
        dbg!(&params);
        let page = Webpage::new()
            .htdocs_dir("contents")
            .boot_icon(true)
            .start_page(&format!("index.html?{}", params))
            .file("episode.mp4", &episode)
            .open_session(Visibility::Default)
            .unwrap();

        page.show();
        page
    }
    else {
        let current_episode = get_episode();
        config.current_video_path = current_episode.clone();
        let episode = std::fs::read(current_episode).unwrap();
        let params = format!("seconds={}", 0.0);
        dbg!(&params);
        let page = Webpage::new()
            .htdocs_dir("contents")
            .boot_icon(true)
            .start_page(&format!("index.html?{}", params))
            .file("episode.mp4", &episode)
            .open_session(Visibility::Default)
            .unwrap();

        page.show();
        page
    }
}