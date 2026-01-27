use colored::*;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod message_interface;
use message_interface::TwitchMessage;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let url = env::var("URL").expect("URL not found in environment");
    println!("{}", "Connecting to websocket...".yellow());

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect!");
    println!("{}", "Connected!".green());

    let (mut write, mut read) = ws_stream.split();

    let oauth_token = env::var("TOKEN").expect("TOKEN not found in environment");
    let nickname = env::var("NICKNAME").expect("NICKNAME not found in environment");
    let channel = "pleaseendmyloniness";

    write
        .send(Message::Text(format!("PASS {}", oauth_token).into()))
        .await
        .expect("Failed to send PASS");

    write
        .send(Message::Text(format!("NICK {}", nickname).into()))
        .await
        .expect("Failed to send NICK");

    write
        .send(Message::Text(format!("JOIN #{}", channel).into()))
        .await
        .expect("Failed to join channel");

    println!("{}", format!("Joined #{}", channel).green());
    println!("{}", "Listening for messages...".cyan());

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if text.contains("!hello") {
                    write
                        .send(Message::Text(format!("PRIVMSG #{} :test!", channel).into()))
                        .await
                        .expect("Failed to send message");
                }

                if let Some(parsed) = TwitchMessage::parse(&text) {
                    if let Some(display) = parsed.format_display() {
                        println!("{}", format!("{}", display).bright_white());
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
