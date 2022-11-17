use std::sync::RwLock;

use json::JsonValue;
use reqwest::{Client, header::HeaderMap};

const KEEP_ALIVE_URL: &str = "https://www.audiotool.com/";

pub struct AudiotoolHttpClient {
    client: RwLock<Client>,
    cular_cookie: RwLock<Option<String>>,
}

impl AudiotoolHttpClient {

    pub async fn keep_alive(&self) {
        let request = {
            let client = self.client.read().expect("failed to unlock http client");
            let request = client.get(KEEP_ALIVE_URL).build().expect("failed to build keep alive request");
            client.execute(request)
        };
        match request.await {
            Ok(response) => {
                if let Some(cookie) = Self::extract_cular(response.headers()) {
                    let new_cookie = Some(cookie.to_owned());
                    let old_cookie = &mut *self.cular_cookie.write().expect("failed to unlock cular_cookie");
                    if new_cookie != *old_cookie {
                        println!("cular cookie changed to {}", cookie);
                    }
                    *old_cookie = new_cookie; //.ok_or_else(|| "failed to retrieve cookie".to_owned())?;
                } else {
                    eprintln!("keep_alive didn't yield a cular cookie");
                }
            }
            Err(err) => {
                eprintln!("keep_alive request failed: {}", err);
            }
        }
    }

    pub fn cular_cookie(&self) -> Option<String> {
        self.cular_cookie.write().expect("failed to unlock cular_cookie").clone()
    }

    fn extract_cular(headers: &HeaderMap) -> Option<&str> {
        for cookie in headers.get_all("set-cookie") {
            let s = cookie.to_str().expect("defective cookie");
            // TODO replace with regex
            if let Some(c) = s.strip_prefix("cular-session=") {
                return c.split(';').next();//.map(|s| &s[0..32]);
            }
        }
        None
    }

    pub async fn request_track_details(&self, track_key: &str) -> Result<JsonValue, String> {
        let track_details_response = {
            let client = self.client.read().expect("failed to unlock http client");
            let track_details_url = format!("https://www.audiotool.com/track/{}/details.json", track_key);
            let track_details_request = client.get(track_details_url).build().unwrap();
            client.execute(track_details_request)
        };

        let track_details_str = track_details_response.await.expect("track details request failed").text().await.unwrap();

        json::parse(&track_details_str).map_err(|err| format!("failed to parse track details as json: {}", err))
    }

    pub async fn request_single_charts_details(&self) -> Result<JsonValue, String> {
        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = "https://api.audiotool.com/tracks/charts.json?offset=0&limit=10";
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response.await.expect("single charts details request failed").text().await.unwrap();

        json::parse(&details_str).map_err(|err| format!("failed to parse single charts details as json: {}", err))
    }

    pub async fn request_genre_charts_details(&self, genre_key: &str, date: &str) -> Result<JsonValue, String> {
        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = format!("https://api.audiotool.com/genre/{}/charts/{}.json?offset=0&limit=10", genre_key, date);
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response.await.expect("single charts details request failed").text().await.unwrap();

        json::parse(&details_str).map_err(|err| format!("failed to parse single charts details as json: {}", err))
    }

    pub async fn request_album_details(&self, album_key: &str) -> Result<JsonValue, String> {

        // https://api.audiotool.com/album/t9xfkexp/tracks.json?cover=256&snapshot=320&orderBy=created

        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = format!("https://api.audiotool.com/album/{}/tracks.json?offset=0&limit=100", album_key);
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response.await.expect("single charts details request failed").text().await.unwrap();

        json::parse(&details_str).map_err(|err| format!("failed to parse single charts details as json: {}", err))
    }

}

impl Default for AudiotoolHttpClient {
    fn default() -> Self {

        let client = reqwest::ClientBuilder::new()
            .user_agent("Discord Bot")
            .cookie_store(true)
            .build()
            .unwrap_or_else(|err| panic!("could not create audiotool http client: {}", err));

        Self {
            client: RwLock::new(client),
            cular_cookie: RwLock::new(None),
        }
    }
}
