#![feature(backtrace)]

use serde::Deserialize;
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
    #[error("url encode error: {source}")]
    UrlEncode {
        #[from]
        source: serde_urlencoded::ser::Error,
        backtrace: Backtrace,
    },
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
pub struct BlaseballClient {
    client: Client,
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

impl BlaseballClient {
    pub fn new() -> Self {
        let client = Client::new().with(surf::middleware::Redirect::default());
        Self { client }
    }

    pub async fn get_team(&self, team: &str) -> Result<Team> {
        let mut req = self
            .client
            .get("https://blaseball.com/database/team")
            .build();
        req.set_query(&[("id", team)])?;
        Ok(self.client.send(req).await?.body_json().await?)
    }
}

impl Default for BlaseballClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
