use chrono::prelude::*;
use colored::*;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::{
    error::Error,
    fs::File,
    io::{BufReader, Write},
    path::Path,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
mod parser;
use crate::parser::{parse_twitch_message, user_badges};
use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use std::io;

#[tokio::main]
async fn main() {
    // Load twitch badges
    let twitch_badges = read_user_from_file("twitch/json/twitch_badges.json").unwrap();

    let mut conn = set_env().await;

    connect_to_twitch(&mut conn).await;

    while let Some(message) = conn.read.next().await {
        // println!("{:?}", message);
        match message {
            Ok(Message::Text(text)) => {
                for line in text.lines() {
                    if let Some(msg) = parse_twitch_message(line) {
                        // println!("Channel: {}", msg.channel.red());

                        // println!("Prefix: {}", msg.prefix);

                        // println!("Command: {}", msg.command);

                        // Set default values
                        let default_color = "#555555".to_string();
                        let user_color = msg.tags.get("color").unwrap_or(&default_color);
                        let default_name = "Guest".to_string();

                        // Print user and its message
                        println!(
                            "{}: {}",
                            msg.tags
                                .get("display-name")
                                .unwrap_or(&default_name)
                                .color(user_color.as_str()),
                            msg.message
                        );

                        // Prints string array of user's badges
                        let user_badges = user_badges(&msg.tags);
                        // println!("{:?}", badges);
                        for user_badge in &user_badges {
                            if let Some(twitch_badge) =
                                twitch_badges.iter().find(|tb| &tb.name == user_badge)
                            {
                                let badge_url = twitch_badge.url.replace("{SIZE}", "3");
                                println!("Badge URL: {}", badge_url);
                            }
                        }

                        // Prints current time when message was sent (from users system time)
                        let local_time = Local::now();
                        println!("{}", local_time.format("%H:%M"));
                    }
                }

                if text.contains("!hello") {
                    conn.write
                        .send(Message::Text(
                            format!("PRIVMSG #{} :test!", conn.channel).into(),
                        ))
                        .await
                        .expect("Failed to send message");
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
            _ => {}
        }
    }
}

// START-UP
struct TwitchConnection {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    oauth_token: String,
    nickname: String,
    channel: String,
}

async fn set_env() -> TwitchConnection {
    // Load settings
    let settings = load_twitch_acc_settings();

    // Load URL from settings
    let url = settings.url;
    println!("{}", "Connecting to websocket...".yellow());

    // Connect to Twitch WebSocket
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect!");
    println!("{}", "Connected!".green());

    // Split the WebSocket stream into write and read halves
    let (write, read) = ws_stream.split();

    // Create TwitchConnection struct
    let oauth_token = settings.token;
    let nickname = settings.nickname;
    // TODO add into settings
    let channel = "pleaseendmyloniness".to_string();

    TwitchConnection {
        write,
        read,
        oauth_token,
        nickname,
        channel,
    }
}

async fn connect_to_twitch(conn: &mut TwitchConnection) {
    // Request capabilities
    conn.write
        .send(Message::Text(
            "CAP REQ :twitch.tv/tags twitch.tv/commands".into(),
        ))
        .await
        .expect("Failed to request capabilities");

    // Send PASS, NICK, and JOIN commands
    conn.write
        .send(Message::Text(format!("PASS {}", conn.oauth_token).into()))
        .await
        .expect("Failed to send PASS");

    conn.write
        .send(Message::Text(format!("NICK {}", conn.nickname).into()))
        .await
        .expect("Failed to send NICK");

    conn.write
        .send(Message::Text(format!("JOIN #{}", conn.channel).into()))
        .await
        .expect("Failed to join channel");

    println!("{}", format!("Joined #{}", conn.channel).green());
    println!("{}", "Listening for messages...".cyan());
}

// BADGES
#[derive(Deserialize, Debug)]
struct TwitchBadge {
    index: i64,
    name: String,
    url: String,
}

fn read_user_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<TwitchBadge>, Box<dyn Error>> {
    // Open the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

// CONFIGS
#[derive(Serialize, Deserialize)]
struct Settings {
    nickname: String,
    token: String,
    url: String,
}

fn load_twitch_acc_settings() -> Settings {
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
                user_token
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
