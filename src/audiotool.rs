use std::{collections::HashMap, sync::RwLock};

use json::JsonValue;
use reqwest::{header::HeaderMap, Client};

const KEEP_ALIVE_URL: &str = "https://www.audiotool.com/";

pub struct AudiotoolHttpClient {
    client: RwLock<Client>,
    cookies: RwLock<HashMap<String, String>>,
}

impl AudiotoolHttpClient {
    pub async fn keep_alive(&self) {
        let request = {
            let cookies = self.cookies.read().expect("failed to unlock cular_cookie");
            let mut cookie_string = String::new();
            for (key, value) in &*cookies {
                if !cookie_string.is_empty() {
                    cookie_string.push_str("; ");
                }
                cookie_string.push_str(key);
                cookie_string.push('=');
                cookie_string.push_str(value);
            }

            let client = self.client.read().expect("failed to unlock http client");
            let request = client
                .get(KEEP_ALIVE_URL)
                //.header("Cookie", cookie_string)
                .build()
                .expect("failed to build keep alive request");
            // println!("keep alive-request: {request:#?}");
            client.execute(request)
        };
        match request.await {
            Ok(response) => {
                // println!("keep alive-response: {response:#?}");
                match Self::extract_cookies(response.headers()) {
                    Ok(cookies) => {
                        // I guess it's ok to not receive a follow-up cookie on a keep-alive-connection
                        if !cookies.is_empty() {
                            let old_cookies =
                                &mut *self.cookies.write().expect("failed to unlock cular_cookie");

                            if cookies != *old_cookies {
                                println!("cookies changed to {cookies:?}");
                            }
                            *old_cookies = cookies;
                            //*old_cookies = new_cookie; //.ok_or_else(|| "failed to retrieve cookie".to_owned())?;
                        }
                    }
                    Err(err) => {
                        eprintln!("keep_alive didn't yield a cular cookie: {err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("keep_alive request failed: {err}");
            }
        }
    }

    pub fn cular_cookie(&self) -> HashMap<String, String> {
        self.cookies
            .write()
            .expect("failed to unlock cular_cookie")
            .clone()
    }

    fn extract_cookies(headers: &HeaderMap) -> Result<HashMap<String, String>, String> {
        let mut cookies = HashMap::new();
        for cookie in headers.get_all("set-cookie") {
            let s = cookie.to_str().expect("defective cookie");
            // TODO replace with regex
            // Set-Cookie: cular-session=497759f4239490854d65e3ee70b55a41?t;Path=/;Domain=.audiotool.com
            //if let Some(c) = s.strip_prefix("cular-session=") {
            let Some((result, _)) = s.split_once(';') else {
                return Err(format!("malformed cookie format: {s}"));
            };

            let Some((key, value)) = result.split_once('=') else {
                return Err(format!("malformed cookie format: {s}"));
            };

            cookies.insert(key.to_owned(), value.to_owned());
        }

        Ok(cookies)
    }

    pub async fn request_track_details(&self, track_key: &str) -> Result<JsonValue, String> {
        let track_details_response = {
            let client = self.client.read().expect("failed to unlock http client");
            let track_details_url =
                format!("https://www.audiotool.com/track/{track_key}/details.json");
            let track_details_request = client.get(track_details_url).build().unwrap();
            client.execute(track_details_request)
        };

        let track_details_str = track_details_response
            .await
            .expect("track details request failed")
            .text()
            .await
            .unwrap();

        json::parse(&track_details_str)
            .map_err(|err| format!("failed to parse track details as json: {err}"))
    }

    pub async fn request_single_charts_details(&self) -> Result<JsonValue, String> {
        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = "https://api.audiotool.com/tracks/charts.json?offset=0&limit=10";
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response
            .await
            .expect("single charts details request failed")
            .text()
            .await
            .unwrap();

        json::parse(&details_str)
            .map_err(|err| format!("failed to parse single charts details as json: {err}"))
    }

    pub async fn request_genre_charts_details(
        &self,
        genre_key: &str,
        date: &str,
    ) -> Result<JsonValue, String> {
        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = format!(
                "https://api.audiotool.com/genre/{genre_key}/charts/{date}.json?offset=0&limit=10",
            );
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response
            .await
            .expect("single charts details request failed")
            .text()
            .await
            .unwrap();

        json::parse(&details_str)
            .map_err(|err| format!("failed to parse single charts details as json: {err}"))
    }

    pub async fn request_album_details(&self, album_key: &str) -> Result<JsonValue, String> {
        let response = {
            let client = self.client.read().expect("failed to unlock http client");
            let url = format!(
                "https://api.audiotool.com/album/{album_key}/tracks.json?offset=0&limit=100"
            );
            let request = client.get(url).build().unwrap();
            client.execute(request)
        };

        let details_str = response
            .await
            .expect("single charts details request failed")
            .text()
            .await
            .unwrap();

        json::parse(&details_str)
            .map_err(|err| format!("failed to parse single charts details as json: {err}"))
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
            cookies: RwLock::new(HashMap::new()),
        }
    }
}
