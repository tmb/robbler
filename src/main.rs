use aspotify::model as spotify_model;
use aspotify::{AuthCodeFlow, ClientCredentials};
use core::time::Duration;
use dotenv;
use serde::{Deserialize, Serialize};
use spotify_model::Track;
use std::env;

type Song = (Track, Duration);

#[derive(Serialize, Deserialize)]
struct MyTrack {
    name: String,         // name of the song
    artist: String,       // name of the artist
    listened_to_for: u64, // seconds listened to
    spotify_id: String,   // spotify id of the song
    artist_spotify_id: String
}

#[tokio::main]
async fn main() {
    // Read .env file into environment variables.
    dotenv::dotenv().ok();

    // Auth stuff. Storing credentials and using them + refresh token to get a real token
    let credentials = ClientCredentials::from_env().unwrap();
    let token = env::var("REFRESH_TOKEN").unwrap();
    let flow = AuthCodeFlow::from_refresh(credentials, token);
    let token = flow.send().await.unwrap();

    let mut current_song: Option<Song> = None;

    let mut time = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        time.tick().await;
        let song = check_song(&token, current_song.clone()).await;
        current_song = song;
    }
}

fn fmt_track_info(track: Track, time: Duration) -> String {
    return format!("{} by {} - listened for {}", track.name, track.artists[0].name, time.as_secs());
}

async fn get_current_track(token: &aspotify::authorization::AccessToken) -> Option<Song> {
    if let Some(currently_playing) = aspotify::endpoints::player::get_playing_track(token, None)
        .await
        .unwrap()
    {
        if let Some(track) = currently_playing.item {
            Some((track, currently_playing.progress.unwrap()))
        } else {
            (None)
        }
    } else {
        (None)
    }
}

async fn check_song(
    token: &aspotify::authorization::AccessToken,
    old_song: Option<Song>,
) -> Option<Song> {
    if let Some((track, progress)) = get_current_track(&token).await {
        if let Some((old_track, old_track_progress)) = old_song {
            // if the track changed & the old traz
            if old_track.id != track.id {
                send_play((old_track, old_track_progress)).await;
            } else if old_track_progress.as_secs() > 0 && (progress.as_secs() == 1 || progress.as_secs() == 0) {
                send_play((track.clone(), old_track_progress)).await;
            };
        }
        Some((track.clone(), progress))
    } else {
        None
    }
}

// when the track changes
async fn send_play((track, progress): Song) {
    let serialized = MyTrack {
        name: track.clone().name,
        artist: track.artists[0].clone().name,
        listened_to_for: progress.as_secs(),
        spotify_id: track.clone().id,
        artist_spotify_id: track.artists[0].clone().id
    };

    let client = reqwest::Client::new();

    client.post(&env::var("URL").unwrap())
        .json(&serialized)
        .send()
        .await.unwrap();

    print!("{}\n", fmt_track_info(track.clone(), progress));
}
