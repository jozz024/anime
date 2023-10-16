use skyline_web::{Webpage, Visibility};

use rand::seq::SliceRandom;

static anime_html: &str = include_str!("anime.html");

fn get_episode() -> String {
    let episodes = std::fs::read_dir("sd:/episodes/").unwrap();
    let names = episodes.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(String::from))
        )
    }).collect::<Vec<String>>();

    let file_name = names.choose(&mut rand::thread_rng()).unwrap().to_owned();
    "sd:/episodes/".to_string() + &file_name
}

pub fn spawn_webpage() -> skyline_web::WebSession{
    println!("{}", std::fs::try_exists(get_episode()).unwrap());
    let episode = std::fs::read(get_episode()).unwrap();
    let page = Webpage::new()
        .htdocs_dir("contents")
        .file("index.html", anime_html)
        .file("episode.mp4", &episode)
        .open_session(Visibility::Default)
        .unwrap();

    page.show();
    page
}