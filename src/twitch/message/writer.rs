use futures_util::{stream::SplitSink, SinkExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub async fn send_message(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    channel: &str,
) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    println!("Send message in the chat (type and press Enter):");

    while let Ok(Some(line)) = reader.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(e) = write
            .send(Message::Text(
                format!("PRIVMSG #{} :{}", channel, trimmed).into(),
            ))
            .await
        {
            eprintln!("Failed to send message: {}", e);
            break;
        }
    }
}
