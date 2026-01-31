use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub nickname: String,
    pub token: String,
    pub url: String,
}
