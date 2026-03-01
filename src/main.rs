use std::result::Result as StdResult;
use std::{env, fs::read_to_string, io};

use brawl_api::{Client, Player, traits::PropFetchable};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use thiserror::Error;

#[derive(Error, Debug)]
enum RilistarrError {
    #[error("failed to query the brawl stars API: {0}")]
    BrawlApi(String),
    #[error("failed to load .env: {0}")]
    DotEnvLoad(#[from] dotenvy::Error),
    #[error("failed to read BRAWL_TOKEN: {0}")]
    ReadToken(#[from] env::VarError),
    #[error("failed to read config: {0}")]
    ReadConfig(#[from] io::Error),
    #[error("failed to parse config: {0}")]
    ParseConfig(#[from] serde_json::Error),
}

impl From<brawl_api::Error> for RilistarrError {
    fn from(value: brawl_api::Error) -> Self {
        Self::BrawlApi(value.to_string())
    }
}

type Result<T> = StdResult<T, RilistarrError>;

fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let token = env::var("BRAWL_TOKEN")?;
    let client = Client::new(&token);

    let data = read_to_string("data.json")?;
    let ids = serde_json::from_str::<Box<[&str]>>(&data)?;

    let mut players: Vec<_> = ids
        .par_iter()
        .map(|id| Player::fetch(&client, id))
        .collect::<StdResult<Vec<_>, _>>()?;

    players.sort_by(|p1, p2| p2.trophies.cmp(&p1.trophies));

    let text = players
        .iter()
        .enumerate()
        .map(|(i, p)| format!("({}) {}: {} trophies\n", i + 1, p.name, p.trophies))
        .collect::<String>();

    println!("{}", text);

    Ok(())
}
