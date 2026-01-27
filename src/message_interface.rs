use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TwitchMessage {
    pub tags: HashMap<String, String>,
    // pub prefix: Option<String>,
    pub command: String,
    // pub params: Vec<String>,
    // pub channel: Option<String>,
    pub username: Option<String>,
    pub message: Option<String>,
}

impl TwitchMessage {
    pub fn parse(raw: &str) -> Option<Self> {
        let mut tags = HashMap::new();
        let mut remaining = raw;

        // Parse tags if they exist (start with @)
        if remaining.starts_with('@') {
            if let Some(space_pos) = remaining.find(' ') {
                let tags_str = &remaining[1..space_pos];
                for tag in tags_str.split(';') {
                    if let Some(eq_pos) = tag.find('=') {
                        let key = tag[..eq_pos].to_string();
                        let value = tag[eq_pos + 1..].to_string();
                        tags.insert(key, value);
                    }
                }
                remaining = &remaining[space_pos + 1..];
            }
        }

        // Parse prefix if it exists (start with :)
        let prefix = if remaining.starts_with(':') {
            if let Some(space_pos) = remaining.find(' ') {
                let prefix_str = remaining[1..space_pos].to_string();
                remaining = &remaining[space_pos + 1..];
                Some(prefix_str)
            } else {
                None
            }
        } else {
            None
        };

        // Parse command and params
        let parts: Vec<&str> = remaining.splitn(2, ' ').collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0].to_string();
        let mut params = Vec::new();

        if parts.len() > 1 {
            let params_str = parts[1];
            let mut current = params_str;

            while !current.is_empty() {
                if current.starts_with(':') {
                    params.push(current[1..].to_string());
                    break;
                } else if let Some(space_pos) = current.find(' ') {
                    params.push(current[..space_pos].to_string());
                    current = &current[space_pos + 1..];
                } else {
                    params.push(current.to_string());
                    break;
                }
            }
        }

        // Extract username from prefix
        let username = prefix
            .as_ref()
            .and_then(|p| p.split('!').next().map(|s| s.to_string()));

        // Extract channel and message for PRIVMSG
        let (_channel, message) = if command == "PRIVMSG" && params.len() >= 2 {
            (
                Some(params[0].trim_start_matches('#').to_string()),
                Some(params[1].to_string()),
            )
        } else {
            (None, None)
        };

        Some(TwitchMessage {
            tags,
            // prefix,
            command,
            // params,
            // channel,
            username,
            message,
        })
    }

    pub fn display_name(&self) -> Option<String> {
        self.tags
            .get("display-name")
            .cloned()
            .or_else(|| self.username.clone())
    }

    // pub fn color(&self) -> Option<String> {
    //     self.tags.get("color").cloned()
    // }

    pub fn is_mod(&self) -> bool {
        self.tags.get("mod").map(|v| v == "1").unwrap_or(false)
    }

    pub fn is_subscriber(&self) -> bool {
        self.tags
            .get("subscriber")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    pub fn is_vip(&self) -> bool {
        self.tags.contains_key("vip")
    }

    // pub fn badges(&self) -> Vec<String> {
    //     self.tags
    //         .get("badges")
    //         .map(|b| b.split(',').map(|s| s.to_string()).collect())
    //         .unwrap_or_default()
    // }

    pub fn format_display(&self) -> Option<String> {
        if self.command == "PRIVMSG" {
            let username = self.display_name().or(self.username.clone())?;
            let message = self.message.as_ref()?;

            let mut badges_str = String::new();
            if self.is_mod() {
                badges_str.push_str("[MOD] ");
            }
            if self.is_vip() {
                badges_str.push_str("[VIP] ");
            }
            if self.is_subscriber() {
                badges_str.push_str("[SUB] ");
            }

            Some(format!("{}{}: {}", badges_str, username, message))
        } else {
            None
        }
    }
}
