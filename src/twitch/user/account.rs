use colored::*;
use config::{Config, ConfigError};
use std::io;
use std::{fs::File, io::Write, path::Path};

use crate::twitch::user::settings::Settings;
use crate::twitch::user::validate_token;

pub async fn load_acc() -> Settings {
    // Check if settings file exists, create if it doesn't
    if !Path::new("settings.toml").exists() {
        File::create("settings.toml").expect("Failed to create settings.toml");
    }

    // Load settings from file and environment variables
    let settings = Config::builder()
        .add_source(config::File::with_name("settings.toml"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let mut needs_save = false;

    // Get URL, use default if not found
    let url = match settings.get::<String>("url") {
        Ok(url) => {
            println!("URL: {}", url);
            url
        }
        Err(ConfigError::NotFound(..)) => {
            let default_url = "wss://irc-ws.chat.twitch.tv:443".to_string();
            println!(
                "{}",
                format!("URL not found, using default: {}", default_url).yellow()
            );
            needs_save = true;
            default_url
        }
        Err(e) => {
            println!("There has been an error: {}", e);
            "wss://irc-ws.chat.twitch.tv:443".to_string()
        }
    };

    // Get nickname and token, prompt user if not found or empty
    let nickname = match settings.get::<String>("nickname") {
        Ok(nickname) => {
            if nickname.is_empty() {
                println!(
                    "{}",
                    "Nickname is empty. Please enter a valid nickname:".yellow()
                );
                let mut nickname = String::new();
                loop {
                    nickname.clear();
                    io::stdin()
                        .read_line(&mut nickname)
                        .expect("Failed to read line");
                    let trimmed = nickname.trim();
                    if !trimmed.is_empty() {
                        needs_save = true;
                        break trimmed.to_string();
                    }
                    println!("{}", "Nickname cannot be empty. Please try again:".yellow());
                }
            } else {
                nickname
            }
        }
        Err(ConfigError::NotFound(..)) => {
            println!("{}", "Nickname not found:".yellow());
            let mut nickname = String::new();
            loop {
                nickname.clear();
                io::stdin()
                    .read_line(&mut nickname)
                    .expect("Failed to read line");
                let trimmed = nickname.trim();
                if !trimmed.is_empty() {
                    needs_save = true;
                    break trimmed.to_string();
                }
                println!("{}", "Nickname cannot be empty. Please try again:".yellow());
            }
        }
        Err(e) => {
            println!("There has been an error: {}", e);
            String::new()
        }
    };

    let user_token = match settings.get::<String>("token") {
        Ok(user_token) => {
            if user_token.is_empty() {
                println!("{}", "Token is empty. Please enter a valid token:".yellow());
                let mut user_token = String::new();
                loop {
                    user_token.clear();
                    io::stdin()
                        .read_line(&mut user_token)
                        .expect("Failed to read line");
                    let trimmed = user_token.trim();
                    if !trimmed.is_empty() {
                        needs_save = true;
                        break trimmed.to_string();
                    }
                    println!("{}", "Token cannot be empty. Please try again:".yellow());
                }
            } else {
                // Validate the token and prompt for a new one if invalid
                let mut token = user_token;
                loop {
                    match validate_token(token.clone()).await {
                        Ok(_) => {
                            println!("{}", "Token validated successfully!".green());
                            break token;
                        }
                        Err(e) => {
                            eprintln!("{}", format!("Failed to validate token: {}", e).red());
                            println!("{}", "Please enter a valid token:".yellow());
                            token.clear();
                            io::stdin()
                                .read_line(&mut token)
                                .expect("Failed to read line");
                            token = token.trim().to_string();
                            needs_save = true;
                        }
                    }
                }
            }
        }
        Err(ConfigError::NotFound(..)) => {
            println!("{}", "Token not found".yellow());
            let mut user_token = String::new();
            loop {
                user_token.clear();
                io::stdin()
                    .read_line(&mut user_token)
                    .expect("Failed to read line");
                let trimmed = user_token.trim();
                if !trimmed.is_empty() {
                    needs_save = true;
                    break trimmed.to_string();
                }
                println!("{}", "Token cannot be empty. Please try again:".yellow());
            }
        }
        Err(e) => {
            println!("There has been an error: {}", e);
            String::new()
        }
    };

    // Save settings if needed
    if needs_save {
        let settings_data = Settings {
            nickname: nickname.clone(),
            token: user_token.clone(),
            url: url.clone(),
        };

        let toml_string = toml::to_string(&settings_data).expect("Failed to serialize settings");

        let mut file = File::create("settings.toml").expect("Failed to open settings.toml");
        file.write_all(toml_string.as_bytes())
            .expect("Failed to write to settings.toml");

        println!("Settings saved!");
    }

    // Return settings
    Settings {
        nickname,
        token: user_token,
        url,
    }
}
