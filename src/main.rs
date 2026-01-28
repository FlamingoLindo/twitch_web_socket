use colored::*;
use dotenv::dotenv;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::env;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

mod parser;
use crate::parser::{parse_twitch_message, user_badges};

#[tokio::main]
async fn main() {
    let mut conn = set_env().await;

    connect_to_twitch(&mut conn).await;

    while let Some(message) = conn.read.next().await {
        println!("{:?}", message);
        match message {
            Ok(Message::Text(text)) => {
                for line in text.lines() {
                    if let Some(msg) = parse_twitch_message(line) {
                        // println!("Channel: {}", msg.channel.red());

                        // println!("Prefix: {}", msg.prefix);

                        // println!("Command: {}", msg.command);

                        let default_color = "#ffffff".to_string();
                        let user_color = msg.tags.get("color").unwrap_or(&default_color);
                        let default_name = "Guest".to_string();
                        println!(
                            "{}: {}",
                            msg.tags
                                .get("display-name")
                                .unwrap_or(&default_name)
                                .color(user_color.as_str()),
                            msg.message
                        );

                        let badges = user_badges(&msg.tags);
                        println!("{:?}", badges);
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
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

struct TwitchConnection {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    oauth_token: String,
    nickname: String,
    channel: String,
}

async fn set_env() -> TwitchConnection {
    dotenv().ok();
    let url = env::var("URL").expect("URL not found in environment");
    println!("{}", "Connecting to websocket...".yellow());

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect!");
    println!("{}", "Connected!".green());

    let (write, read) = ws_stream.split();

    let oauth_token = env::var("TOKEN").expect("TOKEN not found in environment");
    let nickname = env::var("NICKNAME").expect("NICKNAME not found in environment");
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
    conn.write
        .send(Message::Text(
            "CAP REQ :twitch.tv/tags twitch.tv/commands".into(),
        ))
        .await
        .expect("Failed to request capabilities");

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
