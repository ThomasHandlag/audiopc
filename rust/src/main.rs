use std::{thread::sleep, time::Duration};

use audiopc::api::{player::AudioPlayer, source::AudioSource};

fn main() {
    let mut player = AudioPlayer::new();

    player.set_source(AudioSource::Path(
        "/home/thuong/Downloads/oblivion/alwaysbe.mp3".to_string(),
    ));

    player.play();

    sleep(Duration::from_mins(2));
}
