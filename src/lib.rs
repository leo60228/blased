use serde::Deserialize;
use std::f64::consts::PI;
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
    #[serde(rename = "weekAttr")]
    pub season_attributes: Vec<String>,
    #[serde(rename = "gameAttr")]
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

pub enum Score {
    Batting,
    Pitching,
    Defense,
    Baserunning,
    Vibes { day: usize },
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn round_to_even(x: f64) -> f64 {
    use std::arch::x86_64::*;

    let x_vec = unsafe { _mm_set1_pd(x) }; // SAFETY: _mm_set1_pd can never violate memory safety
    let y_vec = unsafe { _mm_round_pd(x_vec, _MM_FROUND_TO_NEAREST_INT) }; // SAFETY: valid rounding mode is provided

    unsafe {
        std::mem::transmute_copy(&y_vec) // SAFETY: _mm128d is larger than f64, and the first 8 bytes of _mm128d are always a valid f64
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn round_to_even(x: f64) -> f64 {
    // inefficient fallback algorithm from CPython
    let rounded = x.round();
    if (x - rounded).abs() == 0.5 {
        2.0 * (x / 2.0).round()
    } else {
        rounded
    }
}

impl Player {
    pub fn score(&self, cat: Score) -> f64 {
        match cat {
            Score::Batting => {
                (1.0 - self.tragicness).powf(0.01)
                    * (1.0 - self.patheticism).powf(0.05)
                    * (self.thwackability * self.divinity).powf(0.35)
                    * (self.moxie * self.musclitude).powf(0.075)
                    * self.martyrdom.powf(0.02)
            }
            Score::Pitching => {
                self.unthwackability.powf(0.5)
                    * self.ruthlessness.powf(0.4)
                    * self.overpowerment.powf(0.15)
                    * self.shakespearianism.powf(0.1)
                    * self.coldness.powf(0.025)
            }
            Score::Defense => {
                (self.omniscience * self.tenaciousness).powf(0.2)
                    * (self.watchfulness * self.anticapitalism * self.chasiness).powf(0.1)
            }
            Score::Baserunning => {
                self.laserlikeness.powf(0.5)
                    * (self.base_thirst
                        * self.continuation
                        * self.ground_friction
                        * self.indulgence)
                        .powf(0.1)
            }
            Score::Vibes { day } => {
                0.5 * round_to_even(dbg!(
                    (self.pressurization + self.cinnamon)
                        * ((PI * day as f64) / (5.0 * self.buoyancy + 3.0)).cos()
                        - self.pressurization
                        + self.cinnamon,
                ))
            }
        }
    }

    pub fn rating(&self, cat: Score) -> f64 {
        0.5 * round_to_even(10.0 * self.score(cat))
    }
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
