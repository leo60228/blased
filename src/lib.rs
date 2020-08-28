#![feature(backtrace)]

use http_types::cookies::Cookie;
use serde::Serialize;
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
    token: String,
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
        let token = cookie.value().to_string();

        Ok(Self { client, token })
    }
}

pub trait Blaseball {
    fn get_client(&self) -> &Client;
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

pub trait AuthenticatedBlaseball: Blaseball {}

impl AuthenticatedBlaseball for AuthenticatedClient {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
