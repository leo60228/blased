#![feature(backtrace)]

use async_trait::async_trait;
use chrono::prelude::*;
use http_types::cookies::Cookie;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::result::Result as StdResult;
use surf::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {surf}")]
    Http {
        surf: surf::Error,
        backtrace: Backtrace,
    },
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
        backtrace: Backtrace,
    },
    #[error("cookie error: {source}")]
    Cookie {
        #[from]
        source: cookie::ParseError,
        backtrace: Backtrace,
    },
    #[error("url encode error: {source}")]
    UrlEncode {
        #[from]
        source: serde_urlencoded::ser::Error,
        backtrace: Backtrace,
    },
    #[error("bad response")]
    BadResponse { backtrace: Backtrace },
}

impl From<surf::Error> for Error {
    fn from(surf: surf::Error) -> Self {
        Self::Http {
            surf,
            backtrace: Backtrace::capture(),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct UnauthenticatedClient {
    client: Client,
}

impl UnauthenticatedClient {
    pub fn new() -> Self {
        let client = Client::new().with(surf::middleware::Redirect::default());
        Self { client }
    }
}

impl Default for UnauthenticatedClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct AuthenticatedClient {
    client: Client,
    token: Cookie<'static>,
}

impl AuthenticatedClient {
    pub async fn user_pass(username: &str, password: &str) -> Result<Self> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            pub username: &'a str,
            pub password: &'a str,
            pub is_login: bool,
        }

        let client = Client::new().with(surf::middleware::Redirect::default());

        let req = Request {
            username,
            password,
            is_login: true,
        };

        let resp = client
            .post("https://blaseball.com/auth/local")
            .body(serde_json::to_value(req)?)
            .header("Content-Type", "application/json")
            .await?;
        let cookies = resp
            .header("Set-Cookie")
            .ok_or_else(|| Error::BadResponse {
                backtrace: Backtrace::capture(),
            })?;
        let cookie = cookies
            .iter()
            .filter_map(|x| Cookie::parse_encoded(x.as_str()).ok())
            .find(|x| x.name() == "connect.sid")
            .ok_or_else(|| Error::BadResponse {
                backtrace: Backtrace::capture(),
            })?;
        let token = cookie.into_owned();

        Ok(Self { client, token })
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub lineup: [String; 9],
    pub rotation: [String; 5],
    pub bullpen: [String; 8],
    pub bench: [String; 3],
    pub season_attributes: Vec<String>,
    pub permanent_attributes: Vec<String>,
    pub full_name: String,
    pub location: String,
    pub main_color: String,
    pub nickname: String,
    pub secondary_color: String,
    pub shorthand: String,
    pub emoji: String,
    pub slogan: String,
    pub shame_runs: usize,
    pub total_shames: usize,
    pub total_shamings: usize,
    pub season_shames: usize,
    pub season_shamings: usize,
    pub championships: usize,
}

#[async_trait]
pub trait Blaseball {
    fn get_client(&self) -> &Client;

    async fn get_team(&self, team: &str) -> Result<Team> {
        let mut req = self
            .get_client()
            .get("https://blaseball.com/database/team")
            .build();
        req.set_query(&[("id", team)])?;
        Ok(self.get_client().send(req).await?.body_json().await?)
    }
}

impl Blaseball for UnauthenticatedClient {
    fn get_client(&self) -> &Client {
        &self.client
    }
}

impl Blaseball for AuthenticatedClient {
    fn get_client(&self) -> &Client {
        &self.client
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    pub apple_id: Option<String>,
    pub google_id: Option<String>,
    pub facebook_id: Option<String>,
    pub name: Option<String>,
    pub coins: usize,
    pub votes: usize,
    pub created: DateTime<Utc>,
    pub favorite_team: String,
    pub unlocked_shop: bool,
    pub unlocked_election: bool,
    pub daily_coins_tier: usize,
    pub begs: usize,
    pub max_bet_tier: usize,
    pub peanuts: usize,
    pub peanuts_eaten: usize,
    pub squirrels: usize,
}

#[async_trait]
pub trait AuthenticatedBlaseball: Blaseball {
    fn get_token(&self) -> &Cookie;

    async fn get_user(&self) -> Result<User> {
        Ok(self
            .get_client()
            .get("https://www.blaseball.com/api/getUser")
            .header("Cookie", self.get_token().to_string())
            .await?
            .body_json()
            .await?)
    }
}

impl AuthenticatedBlaseball for AuthenticatedClient {
    fn get_token(&self) -> &Cookie {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
