use twitch_web_socket::{
    connect_to_twitch, create_twitch_connection, parse_badges_from_str, read_chat, send_message,
    TwitchConnection,
};

const TWITCH_BADGES_JSON: &str = include_str!("assets/json/twitch_badges.json");

#[tokio::main]
async fn main() {
    // Load twitch badges
    let twitch_badges =
        parse_badges_from_str(TWITCH_BADGES_JSON).expect("Failed to parse twitch badges json");

    let mut conn = create_twitch_connection().await;
    connect_to_twitch(&mut conn).await;

    let TwitchConnection {
        mut write,
        mut read,
        channel,
        ..
    } = conn;

    tokio::join!(
        send_message(&mut write, &channel),
        read_chat(&mut read, twitch_badges)
    );
}
