use reqwest::StatusCode;

pub async fn validate_token(twitch_token: String) -> Result<String, reqwest::Error> {
    let formatted = twitch_token.trim_start_matches("oauth:");

    let client = reqwest::Client::new();
    let response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .header("Authorization", format!("OAuth {}", formatted))
        .send()
        .await?;

    // Check for 401 Unauthorized
    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(response.error_for_status().unwrap_err());
    }

    let text = response.text().await?;

    Ok(text)
}
