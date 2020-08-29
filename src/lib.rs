use serde::Deserialize;
use std::result::Result as StdResult;
use surf::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {surf}")]
    Http { surf: surf::Error },
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    #[error("url encode error: {source}")]
    UrlEncode {
        #[from]
        source: serde_urlencoded::ser::Error,
    },
}

impl From<surf::Error> for Error {
    fn from(surf: surf::Error) -> Self {
        Self::Http { surf }
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

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: String,
    pub anticapitalism: f64,
    pub base_thirst: f64,
    pub buoyancy: f64,
    pub chasiness: f64,
    pub coldness: f64,
    pub continuation: f64,
    pub divinity: f64,
    pub ground_friction: f64,
    pub indulgence: f64,
    pub laserlikeness: f64,
    pub martyrdom: f64,
    pub moxie: f64,
    pub musclitude: f64,
    pub name: String,
    pub bat: Option<String>,
    pub omniscience: f64,
    pub overpowerment: f64,
    pub patheticism: f64,
    pub ruthlessness: f64,
    pub shakespearianism: f64,
    pub suppression: f64,
    pub tenaciousness: f64,
    pub thwackability: f64,
    pub tragicness: f64,
    pub unthwackability: f64,
    pub watchfulness: f64,
    pub pressurization: f64,
    pub total_fingers: usize,
    pub soul: usize,
    pub deceased: bool,
    pub peanut_allergy: bool,
    pub cinnamon: f64,
    pub fate: usize,
    pub armor: Option<String>,
    pub ritual: Option<String>,
    pub coffee: Option<usize>,
    pub blood: Option<usize>,
}

impl BlaseballClient {
    pub fn new() -> Self {
        let client = Client::new().with(surf::middleware::Redirect::default());
        Self { client }
    }

    pub async fn team(&self, team: &str) -> Result<Team> {
        let mut req = self
            .client
            .get("https://blaseball.com/database/team")
            .build();
        req.set_query(&[("id", team)])?;
        Ok(self.client.send(req).await?.body_json().await?)
    }

    pub async fn all_teams(&self) -> Result<Vec<Team>> {
        Ok(self
            .client
            .get("https://blaseball.com/database/allTeams")
            .await?
            .body_json()
            .await?)
    }

    pub async fn players(&self, players: &[&str]) -> Result<Vec<Player>> {
        let mut req = self
            .client
            .get("https://blaseball.com/database/players")
            .build();
        req.set_query(&[("ids", players.join(","))])?;
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
