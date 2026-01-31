use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct TwitchBadge {
    // index: i64,
    pub name: String,
    pub url: String,
}

pub fn parse_badges_from_str(json_str: &str) -> Result<Vec<TwitchBadge>, Box<dyn Error>> {
    let badges = serde_json::from_str(json_str)?;
    Ok(badges)
}
