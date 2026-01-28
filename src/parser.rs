use std::collections::HashMap;

#[derive(Debug)]
pub struct TwitchMessage {
    pub tags: HashMap<String, String>,
    pub prefix: String,
    pub command: String,
    pub channel: String,
    pub message: String,
}

pub fn parse_twitch_message(raw: &str) -> Option<TwitchMessage> {
    let raw = raw.trim();

    if !raw.starts_with('@') {
        return None;
    }

    let mut parts = raw.split_whitespace();

    let tags_str = parts.next()?;
    let tags: HashMap<String, String> = tags_str
        .trim_start_matches('@')
        .split(';')
        .filter_map(|tag| {
            let mut kv = tag.splitn(2, '=');
            Some((kv.next()?.to_string(), kv.next().unwrap_or("").to_string()))
        })
        .collect();

    let prefix = parts.next()?.trim_start_matches(':').to_string();

    let command = parts.next()?.to_string();

    if command != "PRIVMSG" {
        return None;
    }

    // Parse channel
    let channel = parts.next()?.to_string();

    let message_part = parts.collect::<Vec<&str>>().join(" ");
    let message = message_part.trim_start_matches(':').to_string();

    Some(TwitchMessage {
        tags,
        prefix,
        command,
        channel,
        message,
    })
}

pub fn user_badges(tags: &HashMap<String, String>) -> Vec<String> {
    tags.get("badges")
        .unwrap_or(&String::new())
        .split(',')
        .filter_map(|badge| badge.split_once('/').map(|(name, _)| format!("{}", name)))
        .collect()
}
