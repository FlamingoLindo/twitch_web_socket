use chrono::Local;
use colored::Colorize;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{
    assets::badges::TwitchBadge,
    message_parser::{parse_twitch_message, user_badges},
};

pub async fn read_chat(
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    twitch_badges: Vec<TwitchBadge>,
) {
    while let Some(message) = read.next().await {
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
                        // println!("{:?}", user_badges);
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
