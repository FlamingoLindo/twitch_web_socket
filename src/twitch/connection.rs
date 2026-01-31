use colored::Colorize;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::twitch::user::load_acc;

pub struct TwitchConnection {
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub oauth_token: String,
    pub nickname: String,
    pub channel: String,
}

pub async fn create_twitch_connection() -> TwitchConnection {
    // Load settings
    let settings = load_acc().await;

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

pub async fn connect_to_twitch(conn: &mut TwitchConnection) {
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
